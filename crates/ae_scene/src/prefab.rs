// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Binary hierarchical prefab template system (`.aeprefab`) backed by `ae_world`.

use ae_ecs::{Entity, World};
use ae_plugin_api::{Children, EntityGuid, Name, Parent, Position, TransformDirty};
use ae_world::{
    ComponentRecord, EntityRecord, GuidGenerator, decode_checksummed_envelope,
    encode_checksummed_envelope,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

/// Magic header signature identifying binary `.aeprefab` templates.
pub const PREFAB_MAGIC: [u8; 8] = *b"AEPRB001";
/// Current `.aeprefab` binary format version.
pub const PREFAB_FORMAT_VERSION: u32 = 1;

/// Reusable binary prefab template preserving an entity subtree and its components.
///
/// Serialized to compressed CRC32-verified `.aeprefab` files without JSON and instantiated
/// with fresh runtime [`EntityGuid`]s and reconstructed `Parent`/`Children` links.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prefab {
    /// Human-readable prefab display name.
    pub name: String,
    /// Persistent GUID of the root entity inside [`Self::entities`].
    pub root_guid: EntityGuid,
    /// Root and descendant entity records stored in breadth-first hierarchy order.
    pub entities: Vec<EntityRecord>,
}

impl Prefab {
    /// Creates a hierarchical [`Prefab`] starting at `root_entity` and collecting all
    /// reachable descendants via `Children` components.
    #[must_use]
    pub fn create_from_entity(world: &World, root_entity: Entity) -> Self {
        let registry = crate::scene_component_registry();
        let mut guid_gen = GuidGenerator::new();
        let mut entity_to_guid: HashMap<Entity, EntityGuid> = HashMap::new();
        let mut ordered: Vec<(Entity, Option<Entity>)> = Vec::new();
        let mut visited: HashSet<Entity> = HashSet::new();
        let mut queue: VecDeque<(Entity, Option<Entity>)> = VecDeque::new();

        queue.push_back((root_entity, None));
        while let Some((ent, parent_ent)) = queue.pop_front() {
            if !world.contains(ent) || !visited.insert(ent) {
                continue;
            }
            let assigned = world
                .get_copied::<EntityGuid>(ent)
                .ok()
                .filter(|g| !g.is_nil())
                .unwrap_or_else(|| guid_gen.next_entity_guid());
            entity_to_guid.insert(ent, assigned);
            ordered.push((ent, parent_ent));

            let _ = world.inspect::<Children, _>(ent, |children| {
                for &child in &children.0 {
                    queue.push_back((child, Some(ent)));
                }
            });
        }

        let root_guid = entity_to_guid
            .get(&root_entity)
            .copied()
            .unwrap_or(EntityGuid::NIL);
        let name = world
            .inspect::<Name, _>(root_entity, |n| n.0.clone())
            .unwrap_or_else(|_| "PrefabEntity".to_string());

        let mut records = Vec::with_capacity(ordered.len());
        for (ent, parent_ent) in ordered {
            let guid = entity_to_guid
                .get(&ent)
                .copied()
                .unwrap_or_else(|| guid_gen.next_entity_guid());
            let parent_guid = parent_ent.and_then(|p| entity_to_guid.get(&p).copied());

            let mut record = EntityRecord::new(guid, parent_guid);
            for handler in registry.handlers() {
                let comp_name = handler.type_name();
                if matches!(
                    comp_name,
                    "Parent"
                        | "Children"
                        | "TransformDirty"
                        | "GlobalTransform"
                        | "EntityGuid"
                        | "ModelId"
                        | "SpriteId"
                        | "LodGroup"
                ) {
                    continue;
                }
                if let Some(payload) = handler.capture(world, ent) {
                    record.components.push(ComponentRecord {
                        type_hash: handler.type_hash(),
                        schema_ver: handler.schema_version(),
                        flags: 0,
                        payload,
                    });
                }
            }
            records.push(record);
        }

        Self {
            name,
            root_guid,
            entities: records,
        }
    }

    /// Spawns all entities of this prefab into `world` with fresh [`EntityGuid`]s,
    /// reconstructing `Parent`/`Children` links and optionally overriding the root's `Position`.
    pub fn instantiate(&self, world: &mut World, target_pos: Option<Position>) -> Entity {
        let registry = crate::scene_component_registry();
        let mut guid_gen = GuidGenerator::new();
        let mut old_to_new_ent: HashMap<EntityGuid, Entity> =
            HashMap::with_capacity(self.entities.len());
        let mut spawned_order: Vec<(Entity, EntityRecord)> =
            Vec::with_capacity(self.entities.len());

        for rec in &self.entities {
            let ent = world.spawn(());
            let fresh_guid = guid_gen.next_entity_guid();
            let _ = world.insert_one(ent, fresh_guid);

            for comp in &rec.components {
                if let Some(handler) = registry.get_by_type_hash(comp.type_hash) {
                    let _ = handler.apply_versioned(world, ent, comp.schema_ver, &comp.payload);
                }
            }

            let _ = world.insert_one(ent, TransformDirty);
            let _ = world.remove_one::<ae_plugin_api::GlobalTransform>(ent);
            old_to_new_ent.insert(rec.guid, ent);
            spawned_order.push((ent, rec.clone()));
        }

        for (ent, rec) in &spawned_order {
            if let Some(p_guid) = rec.parent_guid
                && let Some(&parent_ent) = old_to_new_ent.get(&p_guid)
            {
                let _ = world.insert_one(*ent, Parent(parent_ent));
                if let Ok(children) = world.get_mut::<Children>(parent_ent) {
                    if !children.0.contains(ent) {
                        children.0.push(*ent);
                    }
                } else {
                    let _ = world.insert_one(parent_ent, Children(vec![*ent]));
                }
            }
        }

        let root_ent = old_to_new_ent
            .get(&self.root_guid)
            .copied()
            .or_else(|| spawned_order.first().map(|(e, _)| *e))
            .unwrap_or_else(|| {
                world.spawn((
                    self.name_component(),
                    GuidGenerator::new().next_entity_guid(),
                ))
            });

        if let Some(pos) = target_pos {
            let _ = world.insert_one(root_ent, pos);
        }

        root_ent
    }

    fn name_component(&self) -> Name {
        Name(self.name.clone())
    }

    /// Encodes the prefab into an LZ4-compressed, CRC32-verified binary buffer (`.aeprefab`).
    ///
    /// # Errors
    /// Returns an error string if binary `postcard` serialization fails.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let raw = postcard::to_stdvec(self).map_err(|e| e.to_string())?;
        let compressed = lz4_flex::compress_prepend_size(&raw);
        Ok(encode_checksummed_envelope(
            PREFAB_MAGIC,
            PREFAB_FORMAT_VERSION,
            0,
            &compressed,
        ))
    }

    /// Decodes a binary `.aeprefab` byte slice, validating magic header, version, and CRC32.
    ///
    /// # Errors
    /// Returns an error string when the buffer is corrupted, truncated, or uses an invalid format.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let env = decode_checksummed_envelope(bytes, PREFAB_MAGIC, PREFAB_FORMAT_VERSION)
            .map_err(|e| e.to_string())?;
        let decompressed =
            lz4_flex::decompress_size_prepended(env.payload).map_err(|e| e.to_string())?;
        postcard::from_bytes(&decompressed).map_err(|e| e.to_string())
    }

    /// Saves the binary prefab atomically to `path`, creating missing parent directories.
    ///
    /// # Errors
    /// Returns an error string on serialization or filesystem write failure.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let bytes = self.to_bytes()?;
        let tmp = path.with_extension("aeprefab.tmp");
        std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, path).map_err(|e| e.to_string())
    }

    /// Loads and validates a binary `.aeprefab` file from disk.
    ///
    /// # Errors
    /// Returns an error string if the file cannot be read or fails binary validation.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        Self::from_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_plugin_api::{Color, Name, Rotation, Scale, Shape};

    #[test]
    fn test_prefab_binary_roundtrip_and_hierarchy_instantiation() {
        let mut world = World::new();

        let original_root = world.spawn((
            Name("TestBarrel".to_string()),
            Shape::Cylinder,
            Position::new(10.0, 2.0, -5.0),
            Rotation::identity(),
            Scale::new(1.0, 2.0, 1.0),
            Color::soft_blue(),
        ));
        let original_child = world.spawn((
            Name("BarrelLid".to_string()),
            Shape::Cube,
            Position::new(0.0, 1.0, 0.0),
            Parent(original_root),
        ));
        world
            .insert_one(original_root, Children(vec![original_child]))
            .expect("attach child");

        let prefab = Prefab::create_from_entity(&world, original_root);
        assert_eq!(prefab.name, "TestBarrel");
        assert_eq!(prefab.entities.len(), 2);

        let encoded = prefab.to_bytes().expect("encode binary prefab");
        let loaded = Prefab::from_bytes(&encoded).expect("decode binary prefab");
        assert_eq!(loaded, prefab);

        let inst_root = loaded.instantiate(&mut world, Some(Position::new(50.0, 0.0, 50.0)));
        assert_ne!(inst_root, original_root);
        assert_eq!(
            world
                .inspect::<Name, _>(inst_root, |n| n.0.clone())
                .unwrap(),
            "TestBarrel"
        );
        assert_eq!(
            world.get_copied::<Position>(inst_root).unwrap(),
            Position::new(50.0, 0.0, 50.0)
        );

        let child_ids = world
            .inspect::<Children, _>(inst_root, |c| c.0.clone())
            .expect("children restored");
        assert_eq!(child_ids.len(), 1);
        let inst_child = child_ids[0];
        assert_ne!(inst_child, original_child);
        assert_eq!(world.get_copied::<Parent>(inst_child).unwrap().0, inst_root);
        assert_ne!(
            world.get_copied::<EntityGuid>(inst_root).unwrap(),
            world.get_copied::<EntityGuid>(inst_child).unwrap()
        );

        // Corrupted payload must error cleanly
        let mut corrupted = encoded;
        if let Some(last) = corrupted.last_mut() {
            *last ^= 0xff;
        }
        assert!(Prefab::from_bytes(&corrupted).is_err());
        assert!(Prefab::from_bytes(&[]).is_err());
    }

    #[test]
    fn test_prefab_save_creates_parent_directories() {
        let mut world = World::new();
        let entity = world.spawn((Name("TreePrefab".to_string()),));
        let prefab = Prefab::create_from_entity(&world, entity);

        let temp_dir = std::env::temp_dir().join("ae_scene_test_prefabs_deterministic");
        let _ = std::fs::remove_dir_all(&temp_dir);
        let nested_file = temp_dir.join("subfolder").join("tree.aeprefab");

        assert!(!nested_file.exists());
        prefab
            .save_to_file(&nested_file)
            .expect("Saving to nested directory should succeed");
        assert!(nested_file.exists());

        let loaded = Prefab::load_from_file(&nested_file).expect("Should load back");
        assert_eq!(loaded.name, "TreePrefab");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}