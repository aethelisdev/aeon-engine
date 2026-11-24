// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Binary world archive serialization and partitioned dirty-chunk persistence.
//!
//! Captures live [`ae_ecs::World`] entities into [`ae_world::WorldDocument`] snapshots
//! using [`ComponentRegistry`] payloads and saves only modified spatial chunks via CRC32 diffing.

use ae_ecs::{ComponentRegistry, Entity, World};
use ae_light::Light;
use ae_plugin_api::{EntityGuid, LodGroup, ModelId, Parent, PlayerTag, Position, SpriteId};
use ae_renderer::asset::AssetManager;
use ae_world::{
    AssetKind, ComponentRecord, EntityRecord, GuidGenerator, SCENE_ASSET_BINDING_TYPE,
    SavedLodPaths, SceneAssetBinding, SpatialCellCoord, WorldDocument,
    asset_guid_from_virtual_path, resolve_partitioned_world_paths,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Default cubic cell size (world units) for newly authored `.aeworld` partition archives.
pub const DEFAULT_WORLD_CELL_SIZE: f32 = 128.0;

/// Parameters required to serialize a partitioned `.aeworld` archive with CRC chunk reuse.
pub struct WorldSaveRequest<'a> {
    /// Destination `.aeworld` manifest or world directory path.
    pub filepath: &'a Path,
    /// Active dimension flag (`true` for 2D scenes, `false` for 3D scenes).
    pub is_2d: bool,
    /// Previously saved world directory for dirty-chunk CRC diffing.
    pub prev_saved_world_dir: Option<&'a Path>,
    /// Per-cell CRC32 checksums recorded at the previous save/load.
    pub prev_cell_crcs: &'a HashMap<SpatialCellCoord, u32>,
}

/// Summary and updated partition tracking metadata produced by [`save_partitioned_world`].
pub struct SavedWorldOutcome {
    /// Partitioned world root directory written on disk.
    pub saved_world_dir: PathBuf,
    /// Full path to the `<WorldDir>/world.aeworld` manifest file.
    pub manifest_path: PathBuf,
    /// Updated CRC32 checksums indexed by cell coordinate.
    pub cell_crcs: HashMap<SpatialCellCoord, u32>,
    /// Number of dirty/new spatial chunks written during this save.
    pub written_cells: usize,
    /// Number of unchanged spatial chunks skipped via CRC match.
    pub skipped_clean_cells: usize,
    /// Number of empty/stale chunk files removed from disk.
    pub removed_cells: usize,
}

/// Ensures every ECS entity in `world` holds a non-nil [`EntityGuid`], minting and attaching
/// persistent identifiers where missing and returning an entity-to-GUID lookup map.
pub fn ensure_world_entity_guids(world: &mut World) -> HashMap<Entity, EntityGuid> {
    let mut guid_gen = GuidGenerator::new();
    let mut entity_to_guid = HashMap::with_capacity(world.len() as usize);
    let mut missing_guids = Vec::new();

    for entity_ref in world.iter() {
        let entity = entity_ref.entity();
        if let Some(existing) = entity_ref.get_copied::<EntityGuid>()
            && !existing.is_nil()
        {
            entity_to_guid.insert(entity, existing);
            continue;
        }
        let minted = guid_gen.next_entity_guid();
        entity_to_guid.insert(entity, minted);
        missing_guids.push((entity, minted));
    }

    for (entity, guid) in missing_guids {
        let _ = world.insert_one(entity, guid);
    }

    entity_to_guid
}

fn extract_and_register_asset_binding(
    world: &World,
    asset_manager: &AssetManager,
    entity: Entity,
    world_doc: &mut WorldDocument,
) -> std::io::Result<Option<SceneAssetBinding>> {
    let model_path = world
        .get_copied::<ModelId>(entity)
        .ok()
        .and_then(|r| asset_manager.models.get(r.0))
        .map(|asset| asset.source_path.clone());
    let sprite_path = world
        .get_copied::<SpriteId>(entity)
        .ok()
        .and_then(|r| asset_manager.textures.get(r.0))
        .map(|asset| asset.source_path.clone());
    let lod_group = world
        .inspect::<LodGroup, _>(entity, |lod_ref| {
            let lod_0_path = asset_manager
                .models
                .get(lod_ref.lod_0)
                .map(|a| a.source_path.clone())?;
            let lod_1_path = lod_ref
                .lod_1
                .and_then(|h| asset_manager.models.get(h))
                .map(|a| a.source_path.clone());
            let lod_2_path = lod_ref
                .lod_2
                .and_then(|h| asset_manager.models.get(h))
                .map(|a| a.source_path.clone());
            Some(SavedLodPaths {
                lod_0_path,
                lod_1_path,
                lod_2_path,
                threshold_1: lod_ref.threshold_1,
                threshold_2: lod_ref.threshold_2,
            })
        })
        .ok()
        .flatten();

    if let Some(ref mp) = model_path {
        world_doc
            .register_asset_ref(asset_guid_from_virtual_path(mp), AssetKind::Model, mp)
            .map_err(std::io::Error::other)?;
    }
    if let Some(ref sp) = sprite_path {
        world_doc
            .register_asset_ref(asset_guid_from_virtual_path(sp), AssetKind::Texture, sp)
            .map_err(std::io::Error::other)?;
    }
    if let Some(ref lg) = lod_group {
        world_doc
            .register_asset_ref(
                asset_guid_from_virtual_path(&lg.lod_0_path),
                AssetKind::Model,
                &lg.lod_0_path,
            )
            .map_err(std::io::Error::other)?;
        if let Some(ref l1) = lg.lod_1_path {
            world_doc
                .register_asset_ref(asset_guid_from_virtual_path(l1), AssetKind::Model, l1)
                .map_err(std::io::Error::other)?;
        }
        if let Some(ref l2) = lg.lod_2_path {
            world_doc
                .register_asset_ref(asset_guid_from_virtual_path(l2), AssetKind::Model, l2)
                .map_err(std::io::Error::other)?;
        }
    }

    if model_path.is_none() && sprite_path.is_none() && lod_group.is_none() {
        return Ok(None);
    }

    Ok(Some(SceneAssetBinding {
        model_path,
        sprite_path,
        lod_group,
    }))
}

fn capture_entity_record(
    world: &World,
    asset_manager: &AssetManager,
    entity: Entity,
    entity_to_guid: &HashMap<Entity, EntityGuid>,
    registry: &ComponentRegistry,
    world_doc: &mut WorldDocument,
) -> std::io::Result<(EntityRecord, Option<[f32; 3]>, bool)> {
    let guid = entity_to_guid
        .get(&entity)
        .copied()
        .unwrap_or(EntityGuid::NIL);
    let parent_guid = world
        .get_copied::<Parent>(entity)
        .ok()
        .and_then(|parent_ref| entity_to_guid.get(&parent_ref.0).copied());

    let mut record = EntityRecord::new(guid, parent_guid);
    for handler in registry.handlers() {
        let name = handler.type_name();
        if matches!(
            name,
            "ModelId"
                | "SpriteId"
                | "Parent"
                | "Children"
                | "TransformDirty"
                | "GlobalTransform"
                | "LodGroup"
                | "EntityGuid"
        ) {
            continue;
        }
        if let Some(payload) = handler.capture(world, entity) {
            record.components.push(ComponentRecord {
                type_hash: handler.type_hash(),
                schema_ver: handler.schema_version(),
                flags: 0,
                payload,
            });
        }
    }

    if let Some(binding) =
        extract_and_register_asset_binding(world, asset_manager, entity, world_doc)?
    {
        let binding_record = ComponentRecord::from_value(SCENE_ASSET_BINDING_TYPE, 1, &binding)
            .map_err(std::io::Error::other)?;
        record.components.push(binding_record);
    }

    let world_pos = world
        .get_copied::<Position>(entity)
        .ok()
        .map(|p| [p.x, p.y, p.z]);
    let is_global =
        world.has::<Light>(entity) || world.has::<PlayerTag>(entity) || world_pos.is_none();

    Ok((record, world_pos, is_global))
}

/// Captures all live ECS entities, components, and asset bindings into an in-memory [`WorldDocument`].
///
/// Reuses `*active_world_guid` (minting a fresh GUID only when nil) so unchanged worlds yield
/// deterministic checksums across explicit saves and background auto-saves.
///
/// # Errors
/// Returns an `std::io::Error` if asset reference or component binding encoding fails.
pub fn capture_world_document(
    world: &mut World,
    asset_manager: &AssetManager,
    active_world_guid: &mut EntityGuid,
    is_2d: bool,
) -> std::io::Result<WorldDocument> {
    let entity_to_guid = ensure_world_entity_guids(world);
    let registry = ae_plugin_api::engine_component_registry();
    let world_guid = if active_world_guid.is_nil() {
        let fresh = GuidGenerator::new().next_entity_guid();
        *active_world_guid = fresh;
        fresh
    } else {
        *active_world_guid
    };
    let mut world_doc = WorldDocument::new(world_guid, is_2d, DEFAULT_WORLD_CELL_SIZE);

    let entities: Vec<Entity> = world.iter().map(|entity_ref| entity_ref.entity()).collect();

    for entity in entities {
        let (record, world_pos, is_global) = capture_entity_record(
            world,
            asset_manager,
            entity,
            &entity_to_guid,
            registry,
            &mut world_doc,
        )?;
        match (is_global, world_pos) {
            (false, Some(pos)) => world_doc.insert_entity_at(pos, record),
            _ => world_doc.persistent_entities.push(record),
        }
    }

    Ok(world_doc)
}

/// Saves all ECS entities dynamically into a partitioned binary world directory
/// (`<WorldDir>/world.aeworld` + `<WorldDir>/cells/*.aechunk`).
///
/// Compares each spatial cell's encoded CRC32 against `req.prev_cell_crcs` so unmodified chunks
/// remain untouched on disk while modified/new chunks are written atomically in parallel.
///
/// # Errors
/// Returns an `std::io::Error` if serialization or partitioned chunk disk I/O fails.
pub fn save_partitioned_world(
    world: &mut World,
    asset_manager: &AssetManager,
    active_world_guid: &mut EntityGuid,
    req: WorldSaveRequest<'_>,
) -> std::io::Result<SavedWorldOutcome> {
    let world_doc = capture_world_document(world, asset_manager, active_world_guid, req.is_2d)?;

    let (world_dir, manifest_path) = resolve_partitioned_world_paths(req.filepath);
    let prev_crcs = if req.prev_saved_world_dir == Some(world_dir.as_path()) {
        Some(req.prev_cell_crcs)
    } else {
        None
    };

    let stats = world_doc
        .save_partitioned_dir(&world_dir, prev_crcs)
        .map_err(std::io::Error::other)?;
    log::info!(
        "Saved partitioned world {:?}: {} dirty chunk(s) written, {} clean skipped, {} removed.",
        manifest_path,
        stats.written_cells,
        stats.skipped_clean_cells,
        stats.removed_cells
    );

    Ok(SavedWorldOutcome {
        saved_world_dir: world_dir,
        manifest_path,
        cell_crcs: stats.cell_crcs,
        written_cells: stats.written_cells,
        skipped_clean_cells: stats.skipped_clean_cells,
        removed_cells: stats.removed_cells,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_plugin_api::{Children, Name};

    #[test]
    fn test_binary_world_entity_record_guid_hierarchy_roundtrip() {
        let mut ids = GuidGenerator::from_seed(2026);
        let parent_guid = ids.next_entity_guid();
        let child_guid = ids.next_entity_guid();

        let mut world = World::new();
        let parent_ent = world.spawn((
            parent_guid,
            Name("ParentGroup".to_string()),
            Position::new(10.0, 0.0, 5.0),
        ));
        let child_ent = world.spawn((
            child_guid,
            Name("ChildMesh".to_string()),
            Position::new(1.0, 2.0, 3.0),
            Parent(parent_ent),
        ));
        let _ = world.insert_one(parent_ent, Children(vec![child_ent]));

        let mut active_guid = ids.next_entity_guid();
        let assets = AssetManager::new();
        let doc = capture_world_document(&mut world, &assets, &mut active_guid, false)
            .expect("capture world document");

        let encoded = doc.encode_full().expect("encode binary world");
        let decoded = WorldDocument::decode_full(&encoded, None).expect("decode binary world");
        assert!(!decoded.is_2d);
        let all_entities: Vec<EntityRecord> = decoded
            .cells
            .into_values()
            .flat_map(|c| c.entities.into_iter())
            .collect();
        assert_eq!(all_entities.len(), 2);
        let restored_child = all_entities
            .iter()
            .find(|r| r.guid == child_guid)
            .expect("child record present");
        assert_eq!(restored_child.parent_guid, Some(parent_guid));
    }

    #[test]
    fn test_scene_asset_binding_binary_roundtrip() {
        let binding = SceneAssetBinding {
            model_path: Some("assets/models/hero.glb".to_string()),
            sprite_path: None,
            lod_group: Some(SavedLodPaths {
                lod_0_path: "assets/models/hero_lod0.glb".to_string(),
                lod_1_path: Some("assets/models/hero_lod1.glb".to_string()),
                lod_2_path: None,
                threshold_1: 25.0,
                threshold_2: 75.0,
            }),
        };

        let rec = ComponentRecord::from_value(SCENE_ASSET_BINDING_TYPE, 1, &binding)
            .expect("encode asset binding");
        let decoded: SceneAssetBinding = rec.decode_as().expect("decode asset binding");
        assert_eq!(decoded, binding);
    }

    #[test]
    fn test_load_migrated_test_suite_aeworld_archives() {
        for candidate in [
            "../../test_suite.aeworld",
            "../../assets/scenes/physics_test_suite.aeworld",
            "../../assets/scenes/texture_test_suite.aeworld",
            "../../assets/scenes/scene.aeworld",
        ] {
            let path = Path::new(candidate);
            if path.is_file() {
                let doc = WorldDocument::load_from_file(path, None)
                    .expect("migrated .aeworld archive must decode cleanly");
                let count = doc.persistent_entities.len()
                    + doc.cells.values().map(|c| c.entities.len()).sum::<usize>();
                assert!(count > 0, "{candidate} must contain entities");
            }
        }
    }
}