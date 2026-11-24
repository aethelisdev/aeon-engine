// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Runtime world partition streaming coordinator linking `ae_world` chunks, cell-indexed
//! `.aesave` deltas, cross-cell [`PendingGuidResolver`], `AssetManager` reference counting,
//! and floating-origin camera-relative stability.

use ae_ecs::{Entity, EntityCommandBuffer, World, component_type_hash};
use ae_plugin_api::{
    BoundingRadius, Children, EntityGuid, LodGroup, ModelId, Parent, Position, SpriteId,
    TransformDirty,
};
use ae_renderer::asset::AssetManager;
use ae_renderer::camera::FloatingOrigin;
use ae_renderer::render::RenderState;
use ae_world::{
    CellDelta, CellStreamingPlanner, DiskVfsProvider, EntityRecord, GuidGenerator, PartitionCell,
    PendingGuidLink, PendingGuidResolver, ResolvedGuidLink, SCENE_ASSET_BINDING_TYPE,
    SceneAssetBinding, SpatialCellCoord, VfsProvider, WorldDeltaSnapshot,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

/// Summary metrics emitted after advancing [`RuntimeStreamingManager::step`] for one frame.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamingStepReport {
    /// Spatial chunks streamed into the live ECS world during this tick.
    pub loaded_cells: Vec<SpatialCellCoord>,
    /// Spatial chunks removed from the live ECS world during this tick.
    pub unloaded_cells: Vec<SpatialCellCoord>,
    /// Total entities spawned across all newly loaded chunks (including save deltas).
    pub spawned_entities: usize,
    /// Total entities despawned across unloaded chunks (excluding relocated entities).
    pub despawned_entities: usize,
    /// Cross-cell [`EntityGuid`] links resolved by chunks loaded during this tick.
    pub resolved_links: Vec<ResolvedGuidLink>,
}

/// Orchestrates runtime open-world spatial chunk streaming backed by any [`VfsProvider`].
///
/// Handles:
/// - Hysteresis chunk load/unload scheduling via [`CellStreamingPlanner`],
/// - `O(log C)` per-cell `.aesave` [`CellDelta`] filtering and asymmetric `relocate_entity` replay,
/// - Deferred cross-cell parent and custom [`PendingGuidResolver`] link binding,
/// - Runtime boundary migration tracking so moving entities are never despawned with old cells,
/// - `AssetManager` reference-counted GPU resource retention across neighboring chunks.
pub struct RuntimeStreamingManager {
    planner: CellStreamingPlanner,
    guid_resolver: PendingGuidResolver,
    available_cells: HashSet<SpatialCellCoord>,
    loaded_cells: HashSet<SpatialCellCoord>,
    guid_to_entity: HashMap<EntityGuid, Entity>,
    entity_to_guid: HashMap<Entity, EntityGuid>,
    floating_origin: FloatingOrigin,
    vfs: Arc<dyn VfsProvider>,
    save_delta: Option<WorldDeltaSnapshot>,
    known_hashes: HashSet<u64>,
    pending_parent_links: HashMap<EntityGuid, Vec<Entity>>,
}

impl RuntimeStreamingManager {
    /// Creates a streaming manager backed by any thread-safe [`VfsProvider`] (`.aepak` or disk).
    #[must_use]
    pub fn new(
        vfs: Arc<dyn VfsProvider>,
        cell_size: f32,
        load_radius: i32,
        unload_radius: i32,
        available_cells: HashSet<SpatialCellCoord>,
    ) -> Self {
        let mut known_hashes = ae_plugin_api::engine_component_registry().known_type_hashes();
        known_hashes.insert(component_type_hash(SCENE_ASSET_BINDING_TYPE));
        Self {
            planner: CellStreamingPlanner::new(cell_size, load_radius, unload_radius),
            guid_resolver: PendingGuidResolver::new(),
            available_cells,
            loaded_cells: HashSet::new(),
            guid_to_entity: HashMap::new(),
            entity_to_guid: HashMap::new(),
            floating_origin: FloatingOrigin::new(SpatialCellCoord::ZERO, cell_size),
            vfs,
            save_delta: None,
            known_hashes,
            pending_parent_links: HashMap::new(),
        }
    }

    /// Creates a streaming manager rooted at a partitioned world directory (`Worlds/<Name>/`)
    /// and seeds it with already-loaded cells and live ECS entities.
    #[must_use]
    pub fn from_world_dir(
        world_dir: &Path,
        cell_size: f32,
        load_radius: i32,
        unload_radius: i32,
        available_cells: HashSet<SpatialCellCoord>,
        initially_loaded_cells: HashSet<SpatialCellCoord>,
        world: &World,
    ) -> Self {
        let vfs: Arc<dyn VfsProvider> = Arc::new(DiskVfsProvider::new(world_dir));
        let mut mgr = Self::new(vfs, cell_size, load_radius, unload_radius, available_cells);
        mgr.seed_loaded_world(initially_loaded_cells, world);
        mgr
    }

    /// Registers already-spawned partitioned entities from an initial full-world load so
    /// subsequent Play-mode streaming can unload/reload chunks cleanly.
    pub fn seed_loaded_world(
        &mut self,
        initially_loaded_cells: HashSet<SpatialCellCoord>,
        world: &World,
    ) {
        self.loaded_cells = initially_loaded_cells;
        for entity_ref in world.iter() {
            let Some(guid) = entity_ref.get_copied::<EntityGuid>() else {
                continue;
            };
            if guid.is_nil() {
                continue;
            }
            let Some(pos) = entity_ref.get_copied::<Position>() else {
                continue;
            };
            let cell =
                SpatialCellCoord::from_world_pos([pos.x, pos.y, pos.z], self.planner.cell_size);
            if !self.loaded_cells.contains(&cell) {
                continue;
            }
            self.guid_to_entity.insert(guid, entity_ref.entity());
            self.entity_to_guid.insert(entity_ref.entity(), guid);
            self.guid_resolver.on_entity_spawned(cell, guid);
        }
    }

    /// Replaces the active cell-indexed runtime [`WorldDeltaSnapshot`] applied when loading chunks.
    pub fn set_save_delta(&mut self, delta: Option<WorldDeltaSnapshot>) {
        self.save_delta = delta;
    }

    /// Returns an immutable reference to the active [`VfsProvider`].
    #[must_use]
    pub fn vfs(&self) -> &Arc<dyn VfsProvider> {
        &self.vfs
    }

    /// Returns the active [`FloatingOrigin`] anchor updated from the camera's containing cell.
    #[must_use]
    pub const fn floating_origin(&self) -> FloatingOrigin {
        self.floating_origin
    }

    /// Returns the number of spatial cells currently resident in memory.
    #[must_use]
    pub fn loaded_cell_count(&self) -> usize {
        self.loaded_cells.len()
    }

    /// Returns `true` when `coord` is currently loaded.
    #[must_use]
    pub fn is_cell_loaded(&self, coord: SpatialCellCoord) -> bool {
        self.loaded_cells.contains(&coord)
    }

    /// Resolves a live [`Entity`] handle by persistent [`EntityGuid`], if currently streamed in.
    #[must_use]
    pub fn resolve_entity(&self, guid: EntityGuid) -> Option<Entity> {
        self.guid_to_entity.get(&guid).copied()
    }

    /// Registers a cross-cell [`EntityGuid`] dependency; resolves immediately when present or queues
    /// until the target entity's chunk streams into memory.
    pub fn register_cross_cell_link(&mut self, link: PendingGuidLink) -> Option<ResolvedGuidLink> {
        self.guid_resolver.register_or_queue(link)
    }

    /// Advances runtime chunk streaming for the current `camera_pos`:
    /// 1. Updates floating origin and runtime entity cell-boundary ownership.
    /// 2. Unloads distant cells outside `unload_radius`, dropping GPU assets via ref-counting.
    /// 3. Streams missing cells inside `load_radius` (bounded by `max_loads_per_tick`), applies
    ///    `O(log C)` per-cell save deltas, and resolves deferred cross-cell links.
    pub fn step<F>(
        &mut self,
        world: &mut World,
        asset_manager: &mut AssetManager,
        camera_pos: [f32; 3],
        max_loads_per_tick: usize,
        mut on_asset_binding: F,
    ) -> StreamingStepReport
    where
        F: FnMut(&mut World, &mut AssetManager, Entity, &SceneAssetBinding),
    {
        self.floating_origin.cell =
            SpatialCellCoord::from_world_pos(camera_pos, self.planner.cell_size);
        self.sync_entity_cell_migrations(world);

        let plan = self
            .planner
            .plan(camera_pos, &self.available_cells, &self.loaded_cells);

        let mut report = StreamingStepReport::default();
        self.unload_cells(&plan.to_unload, world, asset_manager, &mut report);

        let load_budget = if max_loads_per_tick == 0 {
            plan.to_load.len()
        } else {
            max_loads_per_tick.min(plan.to_load.len())
        };
        for &cell_coord in plan.to_load.iter().take(load_budget) {
            if let Err(err) = self.load_single_cell(
                cell_coord,
                world,
                asset_manager,
                &mut on_asset_binding,
                &mut report,
            ) {
                log::warn!("Streaming load failed for cell {cell_coord:?}: {err}");
            }
        }

        report
    }

    fn sync_entity_cell_migrations(&mut self, world: &World) {
        let mut moved: Vec<(EntityGuid, SpatialCellCoord)> = Vec::new();
        for (&guid, &ent) in &self.guid_to_entity {
            if let Ok(pos) = world.get_copied::<Position>(ent) {
                let current_cell =
                    SpatialCellCoord::from_world_pos([pos.x, pos.y, pos.z], self.planner.cell_size);
                if self.guid_resolver.entity_cell(guid) != Some(current_cell) {
                    moved.push((guid, current_cell));
                }
            }
        }
        for (guid, new_cell) in moved {
            self.guid_resolver.on_entity_moved_cell(guid, new_cell);
        }
    }

    fn unload_cells(
        &mut self,
        cells: &[SpatialCellCoord],
        world: &mut World,
        asset_manager: &mut AssetManager,
        report: &mut StreamingStepReport,
    ) {
        if cells.is_empty() {
            return;
        }
        let mut cmd = EntityCommandBuffer::new();
        for &cell in cells {
            if !self.loaded_cells.remove(&cell) {
                continue;
            }
            let despawn_guids = self.guid_resolver.on_cell_unloaded(cell);
            for guid in despawn_guids {
                self.pending_parent_links.remove(&guid);
                if let Some(entity) = self.guid_to_entity.remove(&guid) {
                    self.entity_to_guid.remove(&entity);
                    release_entity_assets(world, asset_manager, entity);
                    cmd.despawn(entity);
                    report.despawned_entities += 1;
                }
            }
            report.unloaded_cells.push(cell);
        }
        cmd.apply(world);
    }

    fn load_single_cell<F>(
        &mut self,
        cell_coord: SpatialCellCoord,
        world: &mut World,
        asset_manager: &mut AssetManager,
        on_asset_binding: &mut F,
        report: &mut StreamingStepReport,
    ) -> Result<(), String>
    where
        F: FnMut(&mut World, &mut AssetManager, Entity, &SceneAssetBinding),
    {
        let rel_path = cell_coord.relative_chunk_path();
        let bytes = self.vfs.read_bytes(&rel_path).map_err(|e| e.to_string())?;
        let chunk = PartitionCell::decode_chunk(&bytes, Some(&self.known_hashes))
            .map_err(|e| e.to_string())?;
        let cell_delta = self
            .save_delta
            .as_ref()
            .and_then(|snap| snap.delta_for_cell(cell_coord));

        let records = collect_effective_cell_records(chunk.entities, cell_delta);
        let spawned_guids =
            self.spawn_cell_records(records, world, asset_manager, on_asset_binding);

        report.spawned_entities += spawned_guids.len();
        self.loaded_cells.insert(cell_coord);
        let resolved = self.guid_resolver.on_cell_loaded(cell_coord, spawned_guids);
        report.resolved_links.extend(resolved);
        report.loaded_cells.push(cell_coord);
        Ok(())
    }

    fn spawn_cell_records<F>(
        &mut self,
        records: Vec<EntityRecord>,
        world: &mut World,
        asset_manager: &mut AssetManager,
        on_asset_binding: &mut F,
    ) -> Vec<EntityGuid>
    where
        F: FnMut(&mut World, &mut AssetManager, Entity, &SceneAssetBinding),
    {
        let registry = ae_plugin_api::engine_component_registry();
        let binding_hash = component_type_hash(SCENE_ASSET_BINDING_TYPE);
        let mut guid_gen = GuidGenerator::new();
        let mut spawned_guids = Vec::with_capacity(records.len());

        for rec in records {
            if !rec.guid.is_nil() && self.guid_to_entity.contains_key(&rec.guid) {
                continue;
            }
            let entity_guid = if rec.guid.is_nil() {
                guid_gen.next_entity_guid()
            } else {
                rec.guid
            };
            let entity = world.spawn((entity_guid,));
            self.guid_to_entity.insert(entity_guid, entity);
            self.entity_to_guid.insert(entity, entity_guid);
            spawned_guids.push(entity_guid);

            let mut has_physics = false;
            for comp in &rec.components {
                if comp.type_hash == binding_hash {
                    if let Ok(binding) = comp.decode_as::<SceneAssetBinding>() {
                        on_asset_binding(world, asset_manager, entity, &binding);
                    }
                    continue;
                }
                if let Some(handler) = registry.get_by_type_hash(comp.type_hash)
                    && handler
                        .apply_versioned(world, entity, comp.schema_ver, &comp.payload)
                        .is_ok()
                    && matches!(
                        handler.type_name(),
                        "RigidBody" | "Collider" | "CharacterController"
                    )
                {
                    has_physics = true;
                }
            }
            if has_physics {
                let _ = world.insert_one(entity, TransformDirty);
            }
            if let Some(parent_guid) = rec.parent_guid {
                self.attach_or_defer_parent(world, entity, parent_guid);
            }
            self.resolve_pending_children_for(world, entity_guid, entity);
        }

        spawned_guids
    }

    fn attach_or_defer_parent(
        &mut self,
        world: &mut World,
        child: Entity,
        parent_guid: EntityGuid,
    ) {
        if let Some(&parent_ent) = self.guid_to_entity.get(&parent_guid) {
            link_parent_child(world, child, parent_ent);
        } else {
            self.pending_parent_links
                .entry(parent_guid)
                .or_default()
                .push(child);
        }
    }

    fn resolve_pending_children_for(
        &mut self,
        world: &mut World,
        parent_guid: EntityGuid,
        parent_ent: Entity,
    ) {
        if let Some(children) = self.pending_parent_links.remove(&parent_guid) {
            for child in children {
                if world.contains(child) {
                    link_parent_child(world, child, parent_ent);
                }
            }
        }
    }
}

fn link_parent_child(world: &mut World, child: Entity, parent: Entity) {
    let _ = world.insert_one(child, Parent(parent));
    if let Ok(list) = world.get_mut::<Children>(parent) {
        if !list.0.contains(&child) {
            list.0.push(child);
        }
    } else {
        let _ = world.insert_one(parent, Children(vec![child]));
    }
}

fn release_entity_assets(world: &World, asset_manager: &mut AssetManager, entity: Entity) {
    if let Ok(model_id) = world.get_copied::<ModelId>(entity) {
        asset_manager.release_model(model_id.0);
    }
    if let Ok(sprite_id) = world.get_copied::<SpriteId>(entity) {
        asset_manager.release_texture(sprite_id.0);
    }
}

fn collect_effective_cell_records(
    base_entities: Vec<EntityRecord>,
    cell_delta: Option<&CellDelta>,
) -> Vec<EntityRecord> {
    let Some(delta) = cell_delta else {
        return base_entities;
    };
    let destroyed: HashSet<EntityGuid> = delta.destroyed_entities.iter().copied().collect();
    let modified_map: HashMap<EntityGuid, &EntityRecord> = delta
        .modified_entities
        .iter()
        .map(|rec| (rec.guid, rec))
        .collect();

    let mut effective = Vec::with_capacity(base_entities.len() + delta.spawned_entities.len());
    for rec in base_entities {
        if destroyed.contains(&rec.guid) {
            continue;
        }
        if let Some(&override_rec) = modified_map.get(&rec.guid) {
            effective.push(override_rec.clone());
        } else {
            effective.push(rec);
        }
    }
    for spawned in &delta.spawned_entities {
        effective.push(spawned.clone());
    }
    effective
}

/// Applies `SceneAssetBinding` onto a newly spawned or streamed entity, resolving handles
/// via canonical cache or `VfsProvider` and incrementing asset reference counts.
pub fn bind_entity_assets_from_binding(
    world: &mut World,
    render_state: &RenderState,
    asset_manager: &mut AssetManager,
    vfs: Option<&dyn VfsProvider>,
    entity: Entity,
    binding: &SceneAssetBinding,
) {
    let resolve_model = |am: &mut AssetManager, p: &str| {
        if let Ok(canon) = std::fs::canonicalize(p)
            && let Some(&mid) = am.model_path_map.get(&canon)
        {
            return Some(mid);
        }
        if let Some(provider) = vfs {
            let (mid, _, _) = render_state.load_model_from_vfs(am, provider, p);
            if am.models.get(mid).is_some() {
                return Some(mid);
            }
        }
        None
    };

    if let Some(ref lg) = binding.lod_group {
        let l0_h = resolve_model(asset_manager, &lg.lod_0_path);
        let l1_h = lg
            .lod_1_path
            .as_ref()
            .and_then(|p| resolve_model(asset_manager, p));
        let l2_h = lg
            .lod_2_path
            .as_ref()
            .and_then(|p| resolve_model(asset_manager, p));

        if let Some(lod_0) = l0_h {
            asset_manager.retain_model(lod_0);
            if let Some(h1) = l1_h {
                asset_manager.retain_model(h1);
            }
            if let Some(h2) = l2_h {
                asset_manager.retain_model(h2);
            }
            let _ = world.insert_one(
                entity,
                LodGroup {
                    lod_0,
                    lod_1: l1_h,
                    lod_2: l2_h,
                    threshold_1: lg.threshold_1,
                    threshold_2: lg.threshold_2,
                },
            );
        }
    }

    if let Some(ref mp) = binding.model_path
        && let Some(mid) = resolve_model(asset_manager, mp)
    {
        asset_manager.retain_model(mid);
        let _ = world.insert_one(entity, ModelId(mid));
        if !world.has::<BoundingRadius>(entity)
            && let Some(m) = asset_manager.models.get(mid)
        {
            let max_dim = (m.max[0] - m.min[0])
                .max(m.max[1] - m.min[1])
                .max(m.max[2] - m.min[2]);
            let _ = world.insert_one(entity, BoundingRadius(max_dim * 0.5));
        }
    }

    if let Some(ref sp) = binding.sprite_path {
        let tid_opt = std::fs::canonicalize(sp)
            .ok()
            .and_then(|c| asset_manager.texture_path_map.get(&c).copied())
            .or_else(|| {
                vfs.map(|provider| render_state.load_texture_from_vfs(asset_manager, provider, sp))
                    .filter(|&h| asset_manager.textures.get(h).is_some())
            });
        if let Some(tid) = tid_opt {
            asset_manager.retain_texture(tid);
            let _ = world.insert_one(entity, SpriteId(tid));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_world::{
        AssetKind, ComponentRecord, PakBuilder, PakCodec, PakMemoryVfsProvider, SaveSlotMetadata,
    };

    fn make_pos_record(
        guid: EntityGuid,
        x: f32,
        y: f32,
        z: f32,
        parent: Option<EntityGuid>,
    ) -> EntityRecord {
        let mut rec = EntityRecord::new(guid, parent);
        rec.components.push(
            ComponentRecord::from_value("Position", 1, &Position::new(x, y, z))
                .expect("encode position"),
        );
        rec
    }

    #[test]
    fn test_runtime_streaming_hysteresis_boundary_crossing_and_asymmetric_save() {
        let mut ids = GuidGenerator::from_seed(808);
        let world_guid = ids.next_entity_guid();
        let door_guid = ids.next_entity_guid();
        let switch_guid = ids.next_entity_guid();
        let walker_guid = ids.next_entity_guid();

        let cell_0 = SpatialCellCoord::new(0, 0, 0);
        let cell_1 = SpatialCellCoord::new(1, 0, 0);
        let cell_2 = SpatialCellCoord::new(3, 0, 0);

        let chunk_0 = PartitionCell {
            coord: cell_0,
            revision: 1,
            entities: vec![
                make_pos_record(walker_guid, 10.0, 0.0, 0.0, None),
                make_pos_record(switch_guid, 20.0, 0.0, 0.0, None),
            ],
        };
        let chunk_1 = PartitionCell {
            coord: cell_1,
            revision: 1,
            entities: vec![make_pos_record(
                door_guid,
                135.0,
                0.0,
                0.0,
                Some(switch_guid),
            )],
        };
        let chunk_2 = PartitionCell::new(cell_2);

        let mut pak_builder = PakBuilder::new();
        for chunk in [&chunk_0, &chunk_1, &chunk_2] {
            pak_builder.add_asset(
                &chunk.coord.relative_chunk_path(),
                AssetKind::Unknown,
                PakCodec::Lz4Block,
                chunk.encode_chunk().expect("encode chunk"),
            );
        }
        let vfs: Arc<dyn VfsProvider> = Arc::new(
            PakMemoryVfsProvider::from_bytes(pak_builder.build_to_vec().expect("pak"))
                .expect("mount vfs"),
        );

        let available: HashSet<_> = [cell_0, cell_1, cell_2].into_iter().collect();
        let mut mgr = RuntimeStreamingManager::new(vfs, 128.0, 0, 1, available);

        let mut save = WorldDeltaSnapshot::new(SaveSlotMetadata {
            slot_index: 1,
            world_guid,
            timestamp_unix: 1_700_000_000,
            play_time_secs: 60,
            label: "Border Test".to_string(),
        });
        save.relocate_entity(
            cell_0,
            cell_2,
            make_pos_record(walker_guid, 390.0, 0.0, 0.0, None),
        );
        mgr.set_save_delta(Some(save));

        let mut world = World::new();
        let mut assets = AssetManager::new();

        let step_2 = mgr.step(
            &mut world,
            &mut assets,
            [390.0, 0.0, 0.0],
            8,
            |_, _, _, _| {},
        );
        assert_eq!(step_2.loaded_cells, vec![cell_2]);
        let walker_ent = mgr
            .resolve_entity(walker_guid)
            .expect("walker present in cell_2");

        let step_1 = mgr.step(
            &mut world,
            &mut assets,
            [135.0, 0.0, 0.0],
            8,
            |_, _, _, _| {},
        );
        assert_eq!(step_1.loaded_cells, vec![cell_1]);
        assert_eq!(step_1.unloaded_cells, vec![cell_2]);
        assert!(!world.contains(walker_ent));
        let door_ent = mgr
            .resolve_entity(door_guid)
            .expect("door present in cell_1");

        let link = PendingGuidLink {
            owner_cell: cell_1,
            source_entity: door_guid,
            component_type_hash: component_type_hash("Door"),
            field_hash: component_type_hash("switch"),
            target_guid: switch_guid,
        };
        assert!(mgr.register_cross_cell_link(link).is_none());

        let step_0 = mgr.step(
            &mut world,
            &mut assets,
            [15.0, 0.0, 0.0],
            8,
            |_, _, _, _| {},
        );
        assert_eq!(step_0.loaded_cells, vec![cell_0]);
        assert!(step_0.unloaded_cells.is_empty());
        assert_eq!(step_0.resolved_links.len(), 1);
        assert_eq!(step_0.resolved_links[0].link, link);
        assert_eq!(step_0.resolved_links[0].resolved_cell, cell_0);
        let switch_ent = mgr.resolve_entity(switch_guid).expect("switch present");
        assert_eq!(
            world
                .get_copied::<Parent>(door_ent)
                .expect("parent resolved")
                .0,
            switch_ent
        );

        if let Ok(pos) = world.get_mut::<Position>(door_ent) {
            pos.x = 12.0;
        }
        let step_unload = mgr.step(
            &mut world,
            &mut assets,
            [-15.0, 0.0, 0.0],
            8,
            |_, _, _, _| {},
        );
        assert_eq!(step_unload.unloaded_cells, vec![cell_1]);
        assert!(world.contains(door_ent));
        assert!(mgr.guid_resolver.is_entity_loaded(door_guid));
        assert_eq!(mgr.guid_resolver.entity_cell(door_guid), Some(cell_0));
    }
}