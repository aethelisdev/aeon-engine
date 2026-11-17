// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Runtime cross-cell [`EntityGuid`] deferred binding and cell-ownership tracker for open-world streaming.

use crate::guid::EntityGuid;
use crate::partition::SpatialCellCoord;
use std::collections::{HashMap, HashSet};

/// Cross-cell reference awaiting target entity materialization when a neighbor chunk streams in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PendingGuidLink {
    /// Streaming cell that owns the referencing component; unloading this cell drops pending links.
    pub owner_cell: SpatialCellCoord,
    /// Persistent entity holding the reference field.
    pub source_entity: EntityGuid,
    /// Deterministic 64-bit component type hash identifying which component stores the link.
    pub component_type_hash: u64,
    /// Stable field or slot hash inside the component descriptor.
    pub field_hash: u64,
    /// Referenced target entity expected to arrive with another cell.
    pub target_guid: EntityGuid,
}

/// Completed cross-cell binding emitted when a waited [`EntityGuid`] becomes present in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResolvedGuidLink {
    /// Original pending binding request.
    pub link: PendingGuidLink,
    /// Cell in which `link.target_guid` is currently materialized.
    pub resolved_cell: SpatialCellCoord,
}

/// Tracks active entity-to-cell ownership across runtime chunk streaming and resolves deferred
/// cross-cell [`EntityGuid`] links as neighbor chunks load or unload.
#[derive(Debug, Default)]
pub struct PendingGuidResolver {
    loaded_entities: HashMap<EntityGuid, SpatialCellCoord>,
    entities_in_cell: HashMap<SpatialCellCoord, HashSet<EntityGuid>>,
    pending_by_target: HashMap<EntityGuid, Vec<PendingGuidLink>>,
}

impl PendingGuidResolver {
    /// Creates an empty cross-cell GUID resolver.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a cross-cell link request.
    ///
    /// If `link.target_guid` is already loaded in the world, returns `Some(ResolvedGuidLink)`
    /// immediately without queuing; otherwise stores it until the target arrives.
    /// Ignores nil target GUIDs (`target_guid.is_nil()`).
    pub fn register_or_queue(&mut self, link: PendingGuidLink) -> Option<ResolvedGuidLink> {
        if link.target_guid.is_nil() {
            return None;
        }
        if let Some(&resolved_cell) = self.loaded_entities.get(&link.target_guid) {
            return Some(ResolvedGuidLink {
                link,
                resolved_cell,
            });
        }
        let entries = self.pending_by_target.entry(link.target_guid).or_default();
        if !entries.contains(&link) {
            entries.push(link);
        }
        None
    }

    /// Marks `guid` as materialized inside `cell` and drains any queued cross-cell links targeting it.
    pub fn on_entity_spawned(
        &mut self,
        cell: SpatialCellCoord,
        guid: EntityGuid,
    ) -> Vec<ResolvedGuidLink> {
        if guid.is_nil() {
            return Vec::new();
        }
        self.assign_entity_cell(guid, cell);
        self.drain_pending_for(guid, cell)
    }

    /// Marks all `entity_guids` belonging to `cell` as loaded and resolves waiting cross-cell links.
    pub fn on_cell_loaded(
        &mut self,
        cell: SpatialCellCoord,
        entity_guids: impl IntoIterator<Item = EntityGuid>,
    ) -> Vec<ResolvedGuidLink> {
        let mut resolved = Vec::new();
        for guid in entity_guids {
            if guid.is_nil() {
                continue;
            }
            self.assign_entity_cell(guid, cell);
            resolved.extend(self.drain_pending_for(guid, cell));
        }
        resolved
    }

    /// Transfers runtime cell ownership when an entity crosses into `new_cell`, returning the prior
    /// owning cell if the entity was tracked.
    ///
    /// Subsequent [`Self::on_cell_unloaded`] on the prior cell will not return this entity for despawn.
    pub fn on_entity_moved_cell(
        &mut self,
        guid: EntityGuid,
        new_cell: SpatialCellCoord,
    ) -> Option<SpatialCellCoord> {
        if guid.is_nil() {
            return None;
        }
        self.assign_entity_cell(guid, new_cell)
    }

    /// Unregisters all entities currently owned by `cell`, purges unresolved pending links authored
    /// by `cell`, and returns the entity GUIDs that should be despawned from the world.
    ///
    /// Entities that previously migrated out of `cell` via [`Self::on_entity_moved_cell`] remain owned
    /// by their new active cell and are intentionally excluded from the returned list.
    pub fn on_cell_unloaded(&mut self, cell: SpatialCellCoord) -> Vec<EntityGuid> {
        self.purge_pending_owned_by(cell);
        let Some(owned) = self.entities_in_cell.remove(&cell) else {
            return Vec::new();
        };
        let mut despawn_guids = Vec::with_capacity(owned.len());
        for guid in owned {
            if self.loaded_entities.get(&guid) == Some(&cell) {
                self.loaded_entities.remove(&guid);
                despawn_guids.push(guid);
            }
        }
        despawn_guids
    }

    /// Unregisters a single entity destroyed in runtime without unloading its cell.
    pub fn on_entity_destroyed(&mut self, guid: EntityGuid) -> Option<SpatialCellCoord> {
        let cell = self.loaded_entities.remove(&guid)?;
        if let Some(set) = self.entities_in_cell.get_mut(&cell) {
            set.remove(&guid);
            if set.is_empty() {
                self.entities_in_cell.remove(&cell);
            }
        }
        self.purge_pending_authored_by_entity(guid);
        Some(cell)
    }

    /// Returns the total number of unresolved cross-cell links currently queued.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending_by_target.values().map(Vec::len).sum()
    }

    /// Returns the total number of tracked live entities across all loaded cells.
    #[must_use]
    pub fn loaded_count(&self) -> usize {
        self.loaded_entities.len()
    }

    /// Returns `true` when `guid` is currently registered in any loaded cell.
    #[must_use]
    pub fn is_entity_loaded(&self, guid: EntityGuid) -> bool {
        self.loaded_entities.contains_key(&guid)
    }

    /// Returns the active [`SpatialCellCoord`] owning `guid`, if loaded.
    #[must_use]
    pub fn entity_cell(&self, guid: EntityGuid) -> Option<SpatialCellCoord> {
        self.loaded_entities.get(&guid).copied()
    }

    /// Returns all pending links currently waiting on `target_guid`.
    #[must_use]
    pub fn pending_links_for_target(&self, target_guid: EntityGuid) -> &[PendingGuidLink] {
        self.pending_by_target
            .get(&target_guid)
            .map_or(&[], Vec::as_slice)
    }

    fn assign_entity_cell(
        &mut self,
        guid: EntityGuid,
        cell: SpatialCellCoord,
    ) -> Option<SpatialCellCoord> {
        let previous = self.loaded_entities.insert(guid, cell);
        if let Some(prev_cell) = previous
            && prev_cell != cell
            && let Some(prev_set) = self.entities_in_cell.get_mut(&prev_cell)
        {
            prev_set.remove(&guid);
            if prev_set.is_empty() {
                self.entities_in_cell.remove(&prev_cell);
            }
        }
        self.entities_in_cell.entry(cell).or_default().insert(guid);
        previous
    }

    fn drain_pending_for(
        &mut self,
        guid: EntityGuid,
        resolved_cell: SpatialCellCoord,
    ) -> Vec<ResolvedGuidLink> {
        let Some(waiting) = self.pending_by_target.remove(&guid) else {
            return Vec::new();
        };
        waiting
            .into_iter()
            .map(|link| ResolvedGuidLink {
                link,
                resolved_cell,
            })
            .collect()
    }

    fn purge_pending_owned_by(&mut self, cell: SpatialCellCoord) {
        self.pending_by_target.retain(|_, list| {
            list.retain(|link| link.owner_cell != cell);
            !list.is_empty()
        });
    }

    fn purge_pending_authored_by_entity(&mut self, source_entity: EntityGuid) {
        self.pending_by_target.retain(|_, list| {
            list.retain(|link| link.source_entity != source_entity);
            !list.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guid::{GuidGenerator, component_type_hash};

    #[test]
    fn test_cross_cell_pending_guid_resolves_when_target_cell_loads() {
        let mut ids = GuidGenerator::from_seed(303);
        let switch_guid = ids.next_entity_guid();
        let door_guid = ids.next_entity_guid();
        let switch_cell = SpatialCellCoord::new(0, 0, 0);
        let door_cell = SpatialCellCoord::new(1, 0, 0);

        let mut resolver = PendingGuidResolver::new();
        assert!(
            resolver
                .on_cell_loaded(switch_cell, [switch_guid])
                .is_empty()
        );

        let link = PendingGuidLink {
            owner_cell: switch_cell,
            source_entity: switch_guid,
            component_type_hash: component_type_hash("SwitchTarget"),
            field_hash: component_type_hash("target_door"),
            target_guid: door_guid,
        };

        // Target cell not loaded yet -> queued cleanly without panic
        assert!(resolver.register_or_queue(link).is_none());
        assert_eq!(resolver.pending_count(), 1);
        assert_eq!(resolver.pending_links_for_target(door_guid), &[link]);

        // Duplicate registration does not bloat pending list
        assert!(resolver.register_or_queue(link).is_none());
        assert_eq!(resolver.pending_count(), 1);

        // Stream in neighbor cell -> deferred cross-cell link resolves automatically
        let resolved = resolver.on_cell_loaded(door_cell, [door_guid]);
        assert_eq!(
            resolved,
            vec![ResolvedGuidLink {
                link,
                resolved_cell: door_cell,
            }]
        );
        assert_eq!(resolver.pending_count(), 0);
        assert_eq!(resolver.entity_cell(door_guid), Some(door_cell));

        // Subsequent link to already-loaded entity resolves immediately
        let immediate = resolver.register_or_queue(link);
        assert_eq!(
            immediate,
            Some(ResolvedGuidLink {
                link,
                resolved_cell: door_cell,
            })
        );
    }

    #[test]
    fn test_boundary_crossing_transfers_ownership_and_unloaded_owner_purges_pending() {
        let mut ids = GuidGenerator::from_seed(77);
        let npc_guid = ids.next_entity_guid();
        let missing_door = ids.next_entity_guid();
        let cell_a = SpatialCellCoord::new(0, 0, 0);
        let cell_b = SpatialCellCoord::new(1, 0, 0);

        let mut resolver = PendingGuidResolver::new();
        resolver.on_cell_loaded(cell_a, [npc_guid]);
        resolver.register_or_queue(PendingGuidLink {
            owner_cell: cell_a,
            source_entity: npc_guid,
            component_type_hash: 10,
            field_hash: 20,
            target_guid: missing_door,
        });
        assert_eq!(resolver.pending_count(), 1);

        // NPC physically walks from cell_a into cell_b during runtime
        assert_eq!(
            resolver.on_entity_moved_cell(npc_guid, cell_b),
            Some(cell_a)
        );
        assert_eq!(resolver.entity_cell(npc_guid), Some(cell_b));

        // Unloading cell_a purges cell_a's pending links, and MUST NOT despawn moved npc_guid
        let despawned_from_a = resolver.on_cell_unloaded(cell_a);
        assert!(despawned_from_a.is_empty());
        assert_eq!(resolver.pending_count(), 0);
        assert!(resolver.is_entity_loaded(npc_guid));

        // Unloading cell_b returns npc_guid cleanly
        let despawned_from_b = resolver.on_cell_unloaded(cell_b);
        assert_eq!(despawned_from_b, vec![npc_guid]);
        assert!(!resolver.is_entity_loaded(npc_guid));
        assert_eq!(resolver.loaded_count(), 0);
    }
}