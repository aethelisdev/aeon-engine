// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Binary World Archive Serialization & Asynchronous Asset Loading subsystem.
//!
//! Persists and restores ECS worlds in `.aeworld` binary format via `ae_world` and
//! `ComponentRegistry`, linking entity hierarchies strictly through persistent `EntityGuid`s.

use crate::engine::AeEngine;
pub use ae_assets::{PendingSceneData, SceneAssetParseRequest};
use ae_ecs::World;
use ae_ecs::*;
use ae_plugin_api::*;
use ae_world::{
    AssetKind, ComponentRecord, EntityRecord, GuidGenerator, SCENE_ASSET_BINDING_TYPE,
    SavedLodPaths, SceneAssetBinding, SpatialCellCoord, WorldDocument,
    asset_guid_from_virtual_path, resolve_partitioned_world_paths,
};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Default cubic cell size (world units) for newly authored `.aeworld` partition archives.
pub const DEFAULT_WORLD_CELL_SIZE: f32 = 128.0;

/// Maximum binary world archive size accepted by `load_scene` (256 MiB).
const MAX_SCENE_FILE_SIZE: u64 = 256 * 1024 * 1024;

/// Maximum total entity count accepted when loading a scene archive.
const MAX_SCENE_ENTITIES: usize = 500_000;

/// Ensures every ECS entity in `world` holds a non-nil `EntityGuid`, returning a lookup map.
fn ensure_world_entity_guids(world: &mut World) -> HashMap<ae_ecs::Entity, EntityGuid> {
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

/// Extracts model, sprite, and LOD asset paths for `entity` and registers their GUIDs in `world_doc`.
fn extract_and_register_asset_binding(
    engine: &AeEngine,
    entity: ae_ecs::Entity,
    world_doc: &mut WorldDocument,
) -> std::io::Result<Option<SceneAssetBinding>> {
    let w = &engine.world;
    let model_path = w
        .get_copied::<ModelId>(entity)
        .ok()
        .and_then(|r| engine.asset_manager.models.get(r.0))
        .map(|asset| asset.source_path.clone());
    let sprite_path = w
        .get_copied::<SpriteId>(entity)
        .ok()
        .and_then(|r| engine.asset_manager.textures.get(r.0))
        .map(|asset| asset.source_path.clone());
    let lod_group = w
        .inspect::<LodGroup, _>(entity, |lod_ref| {
            let lod_0_path = engine
                .asset_manager
                .models
                .get(lod_ref.lod_0)
                .map(|a| a.source_path.clone())?;
            let lod_1_path = lod_ref
                .lod_1
                .and_then(|h| engine.asset_manager.models.get(h))
                .map(|a| a.source_path.clone());
            let lod_2_path = lod_ref
                .lod_2
                .and_then(|h| engine.asset_manager.models.get(h))
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

/// Captures a single ECS entity into an `EntityRecord` using binary `ComponentRegistry` payloads.
fn capture_entity_record(
    engine: &AeEngine,
    entity: ae_ecs::Entity,
    entity_to_guid: &HashMap<ae_ecs::Entity, EntityGuid>,
    registry: &ComponentRegistry,
    world_doc: &mut WorldDocument,
) -> std::io::Result<(EntityRecord, Option<[f32; 3]>, bool)> {
    let w = &engine.world;
    let guid = entity_to_guid
        .get(&entity)
        .copied()
        .unwrap_or(EntityGuid::NIL);
    let parent_guid = w
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
        if let Some(payload) = handler.capture(w, entity) {
            record.components.push(ComponentRecord {
                type_hash: handler.type_hash(),
                schema_ver: handler.schema_version(),
                flags: 0,
                payload,
            });
        }
    }

    if let Some(binding) = extract_and_register_asset_binding(engine, entity, world_doc)? {
        let binding_record = ComponentRecord::from_value(SCENE_ASSET_BINDING_TYPE, 1, &binding)
            .map_err(std::io::Error::other)?;
        record.components.push(binding_record);
    }

    let world_pos = w
        .get_copied::<Position>(entity)
        .ok()
        .map(|p| [p.x, p.y, p.z]);
    let is_global = w.has::<Light>(entity) || w.has::<PlayerTag>(entity) || world_pos.is_none();

    Ok((record, world_pos, is_global))
}

/// Captures all live ECS entities, components, and asset bindings into an in-memory [`WorldDocument`].
///
/// Reuses `engine.active_world_guid` so unchanged worlds yield deterministic checksums across
/// explicit saves and background auto-saves.
pub(crate) fn capture_world_document(engine: &mut AeEngine) -> std::io::Result<WorldDocument> {
    let entity_to_guid = ensure_world_entity_guids(&mut engine.world);
    let registry = ae_plugin_api::engine_component_registry();
    let world_guid = if engine.active_world_guid.is_nil() {
        let fresh = GuidGenerator::new().next_entity_guid();
        engine.active_world_guid = fresh;
        fresh
    } else {
        engine.active_world_guid
    };
    let mut world_doc = WorldDocument::new(
        world_guid,
        engine.dimension_mode.is_2d(),
        DEFAULT_WORLD_CELL_SIZE,
    );

    let entities: Vec<ae_ecs::Entity> = engine
        .world
        .iter()
        .map(|entity_ref| entity_ref.entity())
        .collect();

    for entity in entities {
        let (record, world_pos, is_global) =
            capture_entity_record(engine, entity, &entity_to_guid, registry, &mut world_doc)?;
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
/// Compares each spatial cell's encoded CRC32 against the previously saved snapshot so
/// unmodified cells on disk are left untouched while only dirty/new chunks are written
/// atomically in parallel across `rayon` threads.
pub(crate) fn save_scene(engine: &mut AeEngine, filepath: &str) -> std::io::Result<()> {
    let world_doc = capture_world_document(engine)?;

    let (world_dir, manifest_path) = resolve_partitioned_world_paths(Path::new(filepath));
    let prev_crcs = if engine.saved_world_dir.as_ref() == Some(&world_dir) {
        Some(&engine.last_saved_cell_crcs)
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

    engine.saved_world_dir = Some(world_dir);
    engine.last_saved_cell_crcs = stats.cell_crcs;
    engine.editor.active_scene_path = Some(manifest_path);
    engine.autosave.reset_timer();
    Ok(())
}

/// Asynchronously loads a binary `.aeworld` archive from disk.
///
/// Validates header dimensionality (`is_2d`) before decompressing chunks, decodes partitions
/// in parallel, and parses referenced models/textures on Rayon background workers.
pub(crate) fn load_scene(engine: &mut AeEngine, filepath: &str) -> std::io::Result<()> {
    let target_path = Path::new(filepath);
    let manifest_to_check = if target_path.is_dir() {
        target_path.join(ae_world::WORLD_MANIFEST_FILENAME)
    } else {
        target_path.to_path_buf()
    };
    let metadata = std::fs::metadata(&manifest_to_check)?;
    if metadata.len() > MAX_SCENE_FILE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "World archive too large ({} bytes, max {} bytes)",
                metadata.len(),
                MAX_SCENE_FILE_SIZE
            ),
        ));
    }

    let file = File::open(&manifest_to_check)?;
    let mut reader = BufReader::new(file);
    let index = WorldDocument::read_index(&mut reader, None)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    if engine.dimension_mode.is_2d() && !index.is_2d {
        let msg = format!(
            "Blocked loading 3D world '{}' while running in 2D mode. Switch to 3D mode to open this world.",
            filepath
        );
        log::error!("{msg}");
        engine.ui.set_status_message(
            "Cannot load 3D world in 2D mode! Switch to 3D mode.",
            irisui::prelude::Color::RED,
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            msg,
        ));
    }

    if engine.dimension_mode.is_3d() && index.is_2d {
        let msg = format!(
            "Blocked loading 2D world '{}' while running in 3D mode. Switch to 2D mode to open this world.",
            filepath
        );
        log::error!("{msg}");
        engine.ui.set_status_message(
            "Cannot load 2D world in 3D mode! Switch to 2D mode.",
            irisui::prelude::Color::RED,
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            msg,
        ));
    }

    let registry = ae_plugin_api::engine_component_registry();
    let mut allowed_hashes = registry.known_type_hashes();
    allowed_hashes.insert(component_type_hash(SCENE_ASSET_BINDING_TYPE));

    let (world_doc, cell_crcs, is_partitioned_dir) =
        WorldDocument::load_auto(target_path, Some(&allowed_hashes))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    let resolved_world_dir = if is_partitioned_dir {
        if target_path.is_dir() {
            Some(target_path.to_path_buf())
        } else {
            Some(resolve_partitioned_world_paths(&manifest_to_check).0)
        }
    } else {
        None
    };

    let total_entities = world_doc.total_entity_count();
    if total_entities > MAX_SCENE_ENTITIES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("World has too many entities ({total_entities}, max {MAX_SCENE_ENTITIES})"),
        ));
    }

    let world_guid = world_doc.world_guid;
    let all_entities = world_doc.into_all_entities();

    let (models_list, textures_list) = ae_assets::collect_scene_asset_paths(&all_entities)?;

    engine.editor.is_loading_assets = true;
    engine.ui.set_status_message(
        "Loading world: parsing assets in background...",
        irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
    );

    engine.scene_rx = Some(ae_assets::spawn_scene_asset_parse(SceneAssetParseRequest {
        world_guid,
        entities: all_entities,
        saved_world_dir: resolved_world_dir,
        cell_crcs,
        models_list,
        textures_list,
        world_label: filepath.to_string(),
    }));

    Ok(())
}

/// Applies `SceneAssetBinding` onto a newly spawned entity on the main thread.
fn apply_asset_binding(engine: &mut AeEngine, entity: ae_ecs::Entity, binding: &SceneAssetBinding) {
    let vfs = engine.active_vfs.as_deref();
    ae_runtime::streaming::bind_entity_assets_from_binding(
        &mut engine.world,
        &engine.render_state,
        &mut engine.asset_manager,
        vfs,
        entity,
        binding,
    );
}

/// Checks the asynchronous scene load channel, uploads completed asset data to GPU,
/// reconstructs entities & `EntityGuid` parent-child links, and syncs physics.
pub fn process_async_scene_load(engine: &mut AeEngine) {
    let Some(ref rx) = engine.scene_rx else {
        return;
    };
    let Ok(result) = rx.try_recv() else {
        return;
    };
    engine.scene_rx = None;

    let scene_data = match result {
        Ok(data) => data,
        Err(e) => {
            engine.editor.is_loading_assets = false;
            engine.ui.set_status_message(
                format!("Async world load failed: {e}"),
                irisui::prelude::Color::RED,
            );
            return;
        }
    };

    for (orig_path, model_res) in scene_data.parsed_models {
        match model_res {
            Ok(parsed_data) => {
                engine
                    .render_state
                    .upload_model_data(&mut engine.asset_manager, parsed_data);
            }
            Err(e) => log::error!("Failed to parse model '{orig_path}' during world load: {e}"),
        }
    }

    for (orig_path, tex_res) in scene_data.parsed_textures {
        match tex_res {
            Ok((c_path, rgba)) => {
                engine.render_state.upload_texture_data(
                    &mut engine.asset_manager,
                    c_path,
                    rgba,
                    &orig_path,
                );
            }
            Err(e) => log::error!("Failed to parse texture '{orig_path}' during world load: {e}"),
        }
    }

    engine.world = World::new();
    engine.physics_world.clear();
    engine.editor.selected_entities.clear();
    engine.editor.undo_stack.clear();
    engine.editor.redo_stack.clear();
    engine.saved_world_dir = scene_data.saved_world_dir;
    engine.active_world_guid = scene_data.world_guid;
    engine.last_saved_cell_crcs = scene_data.cell_crcs;
    engine.autosave.reset_timer();

    let registry = ae_plugin_api::engine_component_registry();
    let binding_hash = component_type_hash(SCENE_ASSET_BINDING_TYPE);
    let mut guid_to_entity: HashMap<EntityGuid, ae_ecs::Entity> =
        HashMap::with_capacity(scene_data.entities.len());
    let mut child_to_parent_guid: Vec<(ae_ecs::Entity, EntityGuid)> = Vec::new();
    let mut guid_gen = GuidGenerator::new();

    for rec in scene_data.entities {
        let new_ent = engine.world.spawn(());
        let entity_guid = if rec.guid.is_nil() {
            guid_gen.next_entity_guid()
        } else {
            rec.guid
        };
        let _ = engine.world.insert_one(new_ent, entity_guid);
        guid_to_entity.insert(entity_guid, new_ent);

        let mut has_physics = false;
        for comp in &rec.components {
            if comp.type_hash == binding_hash {
                if let Ok(binding) = comp.decode_as::<SceneAssetBinding>() {
                    apply_asset_binding(engine, new_ent, &binding);
                }
                continue;
            }
            if let Some(handler) = registry.get_by_type_hash(comp.type_hash)
                && handler
                    .apply_versioned(&mut engine.world, new_ent, comp.schema_ver, &comp.payload)
                    .is_ok()
            {
                let name = handler.type_name();
                if matches!(name, "RigidBody" | "Collider" | "CharacterController") {
                    has_physics = true;
                }
            }
        }

        if has_physics {
            let _ = engine.world.insert_one(new_ent, TransformDirty);
        }
        if let Some(p_guid) = rec.parent_guid {
            child_to_parent_guid.push((new_ent, p_guid));
        }
    }

    for (child_ent, parent_guid) in child_to_parent_guid {
        if let Some(&parent_ent) = guid_to_entity.get(&parent_guid) {
            let _ = engine.world.insert_one(child_ent, Parent(parent_ent));
            if let Ok(children) = engine.world.get_mut::<Children>(parent_ent) {
                if !children.0.contains(&child_ent) {
                    children.0.push(child_ent);
                }
            } else {
                let _ = engine
                    .world
                    .insert_one(parent_ent, Children(vec![child_ent]));
            }
        }
    }

    ae_core::transform::update_hierarchy_transforms(&mut engine.world);
    engine
        .physics_world
        .sync_ecs_to_physics(&mut engine.world, |handle| {
            engine
                .asset_manager
                .get_physics_mesh_data(handle)
                .map(|(v, i)| (v.as_slice(), i.as_slice()))
        });

    engine.streaming = engine.saved_world_dir.as_ref().map(|world_dir| {
        let cells: HashSet<SpatialCellCoord> =
            engine.last_saved_cell_crcs.keys().copied().collect();
        ae_runtime::streaming::RuntimeStreamingManager::from_world_dir(
            world_dir,
            DEFAULT_WORLD_CELL_SIZE,
            2,
            3,
            cells.clone(),
            cells,
            &engine.world,
        )
    });

    engine.editor.is_loading_assets = false;
    engine.ui.set_status_message(
        "World loaded successfully!",
        irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
    );
    log::info!("Async binary world load processed cleanly.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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

        let entity_to_guid = ensure_world_entity_guids(&mut world);
        let registry = ae_plugin_api::engine_component_registry();

        let mut doc = WorldDocument::new(ids.next_entity_guid(), false, DEFAULT_WORLD_CELL_SIZE);
        for ent in [parent_ent, child_ent] {
            let p_guid = world
                .get_copied::<Parent>(ent)
                .ok()
                .and_then(|p| entity_to_guid.get(&p.0).copied());
            let mut rec = EntityRecord::new(entity_to_guid[&ent], p_guid);
            for handler in registry.handlers() {
                if matches!(handler.type_name(), "Parent" | "Children" | "EntityGuid") {
                    continue;
                }
                if let Some(payload) = handler.capture(&world, ent) {
                    rec.components.push(ComponentRecord {
                        type_hash: handler.type_hash(),
                        schema_ver: handler.schema_version(),
                        flags: 0,
                        payload,
                    });
                }
            }
            let pos = world.get_copied::<Position>(ent).expect("position");
            doc.insert_entity_at([pos.x, pos.y, pos.z], rec);
        }

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