// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::engine::AeEngine;
use ae_core::ecs::*;
use hecs::World;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

/// An intermediate schema for serializing LOD (Level of Detail) groups using asset disk paths.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedLodGroup {
    pub lod_0_path: String,
    pub lod_1_path: Option<String>,
    pub lod_2_path: Option<String>,
    pub threshold_1: f32,
    pub threshold_2: f32,
}

/// An intermediate schema for serializing an ECS Entity with partial components.
#[derive(Serialize, Deserialize, Clone)]
pub struct SavedEntity {
    pub name: Option<Name>,
    pub position: Option<Position>,
    pub rotation: Option<Rotation>,
    pub scale: Option<Scale>,
    pub color: Option<Color>,
    pub light: Option<Light>,
    pub velocity: Option<Velocity>,
    pub shape: Option<Shape>,
    pub bounding_radius: Option<BoundingRadius>,
    pub bounding_box: Option<BoundingBox>,
    pub lod_group: Option<SavedLodGroup>,
    pub is_player: bool,
    pub rigid_body: Option<RigidBody>,
    pub collider: Option<Collider>,
    #[serde(default)]
    pub character_controller: Option<CharacterController>,

    // Instead of raw model_id or sprite_id integers which change across restarts,
    // we store the actual disk path of the asset!
    pub model_path: Option<String>,
    pub sprite_path: Option<String>,

    /// Uniquely links parent-child relationships after scene reload.
    pub parent_name: Option<String>,
}

/// A structured container representing fully parsed but not yet GPU-uploaded model and texture data.
/// Sent from the background parallel parsing thread to the main thread for instant GPU registration.
pub struct PendingSceneData {
    /// Saved entities blueprint deserialized from the JSON scene file.
    pub entities: Vec<SavedEntity>,
    /// Thread-safe pre-parsed GLTF model data ready for GPU upload.
    pub parsed_models: Vec<(String, Result<ae_renderer::asset::ParsedModelData, String>)>,
    /// Thread-safe pre-parsed RGBA textures and their canonical paths.
    pub parsed_textures: Vec<(
        String,
        Result<(std::path::PathBuf, image::RgbaImage), String>,
    )>,
}

/// Serialize the current engine ECS state to a JSON file on disk.
pub(crate) fn save_scene(engine: &AeEngine, filepath: &str) -> std::io::Result<()> {
    let mut saved_entities = Vec::new();

    for entity_ref in engine.ecs.world.iter() {
        let entity = entity_ref.entity();
        let w = &engine.ecs.world;

        let mut model_p = None;
        if let Ok(model_ref) = w.get::<&ModelId>(entity) {
            if let Some(asset) = engine.asset_manager.models.get(model_ref.0) {
                model_p = Some(asset.source_path.clone());
            }
        }

        let mut sprite_p = None;
        if let Ok(sprite_ref) = w.get::<&SpriteId>(entity) {
            if let Some(asset) = engine.asset_manager.textures.get(sprite_ref.0) {
                sprite_p = Some(asset.source_path.clone());
            }
        }

        let mut parent_n = None;
        if let Ok(parent_ref) = w.get::<&Parent>(entity) {
            let parent_ent = parent_ref.0;
            if let Ok(p_name) = w.get::<&Name>(parent_ent) {
                parent_n = Some(p_name.0.clone());
            }
        }

        let mut lod_group_saved = None;
        if let Ok(lod_ref) = w.get::<&LodGroup>(entity) {
            let l0_p = engine
                .asset_manager
                .models
                .get(lod_ref.lod_0)
                .map(|a| a.source_path.clone());
            let l1_p = lod_ref.lod_1.and_then(|h| {
                engine
                    .asset_manager
                    .models
                    .get(h)
                    .map(|a| a.source_path.clone())
            });
            let l2_p = lod_ref.lod_2.and_then(|h| {
                engine
                    .asset_manager
                    .models
                    .get(h)
                    .map(|a| a.source_path.clone())
            });
            if let Some(l0_path) = l0_p {
                lod_group_saved = Some(SavedLodGroup {
                    lod_0_path: l0_path,
                    lod_1_path: l1_p,
                    lod_2_path: l2_p,
                    threshold_1: lod_ref.threshold_1,
                    threshold_2: lod_ref.threshold_2,
                });
            }
        }

        let se = SavedEntity {
            name: w.get::<&Name>(entity).map(|c| (*c).clone()).ok(),
            position: w.get::<&Position>(entity).map(|c| *c).ok(),
            rotation: w.get::<&Rotation>(entity).map(|c| *c).ok(),
            scale: w.get::<&Scale>(entity).map(|c| *c).ok(),
            color: w.get::<&Color>(entity).map(|c| *c).ok(),
            light: w.get::<&Light>(entity).map(|c| *c).ok(),
            velocity: w.get::<&Velocity>(entity).map(|c| *c).ok(),
            shape: w.get::<&Shape>(entity).map(|c| *c).ok(),
            bounding_radius: w.get::<&BoundingRadius>(entity).map(|c| *c).ok(),
            bounding_box: w.get::<&BoundingBox>(entity).map(|c| *c).ok(),
            lod_group: lod_group_saved,
            is_player: w.get::<&PlayerTag>(entity).is_ok(),
            rigid_body: w.get::<&RigidBody>(entity).map(|c| *c).ok(),
            collider: w.get::<&Collider>(entity).map(|c| *c).ok(),
            character_controller: w.get::<&CharacterController>(entity).map(|c| *c).ok(),
            model_path: model_p,
            sprite_path: sprite_p,
            parent_name: parent_n,
        };

        saved_entities.push(se);
    }

    let file = File::create(filepath)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &saved_entities)?;
    writer.flush()?;
    Ok(())
}

/// Asynchronously loads a scene JSON from disk.
/// Spawns a background thread that parallel-parses all unique GLTF meshes and texture files
/// using Rayon, totally avoiding main-thread frames freeze. Updates the UI overlay status.
pub(crate) fn load_scene(engine: &mut AeEngine, filepath: &str) -> std::io::Result<()> {
    // Security: Reject scene files larger than 256 MB to prevent memory exhaustion DoS
    // via crafted deserialization bombs (multi-gigabyte JSON arrays of SavedEntity).
    const MAX_SCENE_FILE_SIZE: u64 = 256 * 1024 * 1024;
    let metadata = std::fs::metadata(filepath)?;
    if metadata.len() > MAX_SCENE_FILE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Scene file too large ({} bytes, max {} bytes)",
                metadata.len(),
                MAX_SCENE_FILE_SIZE
            ),
        ));
    }

    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let saved_entities: Vec<SavedEntity> = serde_json::from_reader(reader)?;

    // Security: Reject scenes with an unreasonable number of entities to prevent
    // RAM exhaustion during entity instantiation and GPU buffer allocation.
    const MAX_SCENE_ENTITIES: usize = 500_000;
    if saved_entities.len() > MAX_SCENE_ENTITIES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Scene has too many entities ({}, max {})",
                saved_entities.len(),
                MAX_SCENE_ENTITIES
            ),
        ));
    }

    // Gather unique asset paths to avoid redundant background parsing work
    let mut unique_models = std::collections::HashSet::new();
    let mut unique_textures = std::collections::HashSet::new();

    for se in &saved_entities {
        if let Some(ref mp) = se.model_path {
            if !ae_renderer::asset::is_safe_path(mp) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("Security Error: Unsafe model path detected: {}", mp),
                ));
            }
            unique_models.insert(mp.clone());
        }
        if let Some(ref sp) = se.sprite_path {
            if !ae_renderer::asset::is_safe_path(sp) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("Security Error: Unsafe sprite path detected: {}", sp),
                ));
            }
            unique_textures.insert(sp.clone());
        }
        if let Some(ref lg) = se.lod_group {
            if ae_renderer::asset::is_safe_path(&lg.lod_0_path) {
                unique_models.insert(lg.lod_0_path.clone());
            }
            if let Some(ref p1) = lg.lod_1_path {
                if ae_renderer::asset::is_safe_path(p1) {
                    unique_models.insert(p1.clone());
                }
            }
            if let Some(ref p2) = lg.lod_2_path {
                if ae_renderer::asset::is_safe_path(p2) {
                    unique_models.insert(p2.clone());
                }
            }
        }
    }

    engine.ui.is_loading_assets = true;
    engine.ui.status_message = Some((
        vec![(
            "Loading scene: parsing assets in background...".to_string(),
            egui::Color32::LIGHT_BLUE,
        )],
        std::time::Instant::now(),
    ));

    let (tx, rx) = std::sync::mpsc::channel();
    engine.scene_rx = Some(rx);

    let models_list: Vec<String> = unique_models.into_iter().collect();
    let textures_list: Vec<String> = unique_textures.into_iter().collect();
    let filepath_str = filepath.to_string();

    rayon::spawn(move || {
        // 1. Parallel parse GLTF/GLB models using Rayon
        let parsed_models: Vec<(String, Result<ae_renderer::asset::ParsedModelData, String>)> =
            models_list
                .par_iter()
                .map(|path| {
                    let res = ae_renderer::render::resources::parse_gltf_file(path, String::new());
                    (path.clone(), res)
                })
                .collect();

        // 2. Parallel parse texture files using Rayon
        let parsed_textures: Vec<(
            String,
            Result<(std::path::PathBuf, image::RgbaImage), String>,
        )> = textures_list
            .par_iter()
            .map(|path| {
                let canonical = std::fs::canonicalize(path)
                    .map_err(|e| format!("Failed to canonicalize texture path '{}': {}", path, e));
                let res = canonical.and_then(|c_path| {
                    ae_renderer::render::resources::parse_texture_file(path)
                        .map(|rgba| (c_path, rgba))
                });
                (path.clone(), res)
            })
            .collect();

        let _ = tx.send(Ok(PendingSceneData {
            entities: saved_entities,
            parsed_models,
            parsed_textures,
        }));
        log::info!(
            "Finished parallel background parsing for scene loaded from: {}",
            filepath_str
        );
    });

    Ok(())
}

/// Checks the asynchronous scene load channel and uploads completed asset data to the GPU.
/// Reconstructs the ECS hierarchy and finishes the scene swap on the main thread.
pub fn process_async_scene_load(engine: &mut AeEngine) {
    if let Some(ref rx) = engine.scene_rx {
        if let Ok(result) = rx.try_recv() {
            engine.scene_rx = None; // Reset the receiver immediately

            match result {
                Ok(scene_data) => {
                    log::info!("Scene parsing finished, starting GPU upload of scene assets.");

                    // 1. Upload Models to GPU
                    for (orig_path, model_res) in scene_data.parsed_models {
                        match model_res {
                            Ok(parsed_data) => {
                                engine
                                    .render_state
                                    .upload_model_data(&mut engine.asset_manager, parsed_data);
                            }
                            Err(e) => {
                                log::error!(
                                    "Failed to parse model '{}' during scene load: {}",
                                    orig_path,
                                    e
                                );
                            }
                        }
                    }

                    // 2. Upload Textures to GPU
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
                            Err(e) => {
                                log::error!(
                                    "Failed to parse texture '{}' during scene load: {}",
                                    orig_path,
                                    e
                                );
                            }
                        }
                    }

                    // 3. Destruct and Clear old world resources completely
                    engine.ecs.world = World::new();
                    engine.physics_world.clear();
                    engine.editor.selected_entities.clear();
                    engine.editor.undo_stack.clear();
                    engine.editor.redo_stack.clear();

                    let mut name_to_entity = std::collections::HashMap::new();
                    let mut entity_parent_links = Vec::new();

                    // 4. Instantiation of new entities
                    for se in scene_data.entities {
                        let new_ent = engine.ecs.world.spawn(());

                        // Track parenting link for hierarchy reconstruction
                        let parent_name_opt = se.parent_name.clone();
                        if let Some(ref n) = se.name {
                            name_to_entity.insert(n.0.clone(), new_ent);
                        }
                        if let Some(pn) = parent_name_opt {
                            entity_parent_links.push((new_ent, pn));
                        }

                        if let Some(n) = se.name {
                            let _ = engine.ecs.world.insert_one(new_ent, n);
                        }
                        if let Some(p) = se.position {
                            let _ = engine.ecs.world.insert_one(new_ent, p);
                        }
                        if let Some(r) = se.rotation {
                            let _ = engine.ecs.world.insert_one(new_ent, r);
                        }
                        if let Some(s) = se.scale {
                            let _ = engine.ecs.world.insert_one(new_ent, s);
                        }
                        if let Some(c) = se.color {
                            let _ = engine.ecs.world.insert_one(new_ent, c);
                        }
                        if let Some(l) = se.light {
                            let _ = engine.ecs.world.insert_one(new_ent, l);
                        }
                        if let Some(v) = se.velocity {
                            let _ = engine.ecs.world.insert_one(new_ent, v);
                        }
                        if let Some(sh) = se.shape {
                            let _ = engine.ecs.world.insert_one(new_ent, sh);
                        }
                        if let Some(b) = se.bounding_radius {
                            let _ = engine.ecs.world.insert_one(new_ent, b);
                        }
                        if let Some(bbox) = se.bounding_box {
                            let _ = engine.ecs.world.insert_one(new_ent, bbox);
                        }
                        if se.is_player {
                            let _ = engine.ecs.world.insert_one(new_ent, PlayerTag);
                        }
                        if let Some(rb) = se.rigid_body {
                            let _ = engine.ecs.world.insert_one(new_ent, rb);
                        }
                        if let Some(col) = se.collider {
                            let _ = engine.ecs.world.insert_one(new_ent, col);
                        }
                        if let Some(ctrl) = se.character_controller {
                            let _ = engine.ecs.world.insert_one(new_ent, ctrl);
                        }

                        if se.rigid_body.is_some()
                            || se.collider.is_some()
                            || se.character_controller.is_some()
                        {
                            let _ = engine.ecs.world.insert_one(new_ent, TransformDirty);
                        }

                        if let Some(lg) = se.lod_group {
                            let l0_h = std::fs::canonicalize(&lg.lod_0_path)
                                .ok()
                                .and_then(|c| engine.asset_manager.model_path_map.get(&c).copied());
                            let l1_h = lg
                                .lod_1_path
                                .as_ref()
                                .and_then(|p| std::fs::canonicalize(p).ok())
                                .and_then(|c| engine.asset_manager.model_path_map.get(&c).copied());
                            let l2_h = lg
                                .lod_2_path
                                .as_ref()
                                .and_then(|p| std::fs::canonicalize(p).ok())
                                .and_then(|c| engine.asset_manager.model_path_map.get(&c).copied());

                            if let Some(l0) = l0_h {
                                let lod_comp = LodGroup {
                                    lod_0: l0,
                                    lod_1: l1_h,
                                    lod_2: l2_h,
                                    threshold_1: lg.threshold_1,
                                    threshold_2: lg.threshold_2,
                                };
                                let _ = engine.ecs.world.insert_one(new_ent, lod_comp);
                            }
                        }

                        // Assign GPU assets
                        if let Some(mp) = se.model_path {
                            let canonical_path = std::fs::canonicalize(&mp).ok();
                            let handle = canonical_path
                                .and_then(|c| engine.asset_manager.model_path_map.get(&c).copied());
                            if let Some(mid) = handle {
                                let _ = engine.ecs.world.insert_one(new_ent, ModelId(mid));

                                // Reconstruct bounding sphere if missing
                                if se.bounding_radius.is_none() {
                                    if let Some(m) = engine.asset_manager.models.get(mid) {
                                        let size_x = m.max[0] - m.min[0];
                                        let size_y = m.max[1] - m.min[1];
                                        let size_z = m.max[2] - m.min[2];
                                        let max_dim = size_x.max(size_y).max(size_z);
                                        let radius = max_dim / 2.0;
                                        let _ = engine
                                            .ecs
                                            .world
                                            .insert_one(new_ent, BoundingRadius(radius));
                                    }
                                }
                            }
                        }

                        if let Some(sp) = se.sprite_path {
                            let canonical_path = std::fs::canonicalize(&sp).ok();
                            let handle = canonical_path.and_then(|c| {
                                engine.asset_manager.texture_path_map.get(&c).copied()
                            });
                            if let Some(tid) = handle {
                                let _ = engine.ecs.world.insert_one(new_ent, SpriteId(tid));
                            }
                        }
                    }

                    // Reconstruct parent-child hierarchy links
                    for (child_ent, parent_name) in entity_parent_links {
                        if let Some(&parent_ent) = name_to_entity.get(&parent_name) {
                            let _ = engine.ecs.world.insert_one(child_ent, Parent(parent_ent));

                            if let Ok(mut children) =
                                engine.ecs.world.get::<&mut Children>(parent_ent)
                            {
                                if !children.0.contains(&child_ent) {
                                    children.0.push(child_ent);
                                }
                            } else {
                                let _ = engine
                                    .ecs
                                    .world
                                    .insert_one(parent_ent, Children(vec![child_ent]));
                            }
                        }
                    }

                    // Force calculate transforms after loaded scene setup
                    ae_core::ecs::update_hierarchy_transforms(&mut engine.ecs.world);

                    // Sync newly loaded scene colliders and rigidbodies into Rapier3D physics world
                    engine
                        .physics_world
                        .sync_ecs_to_physics(&mut engine.ecs.world, |handle| {
                            engine
                                .asset_manager
                                .get_physics_mesh_data(handle)
                                .map(|(v, i)| (v.as_slice(), i.as_slice()))
                        });

                    engine.ui.is_loading_assets = false;
                    engine.ui.status_message = Some((
                        vec![(
                            "Scene loaded successfully!".to_string(),
                            egui::Color32::LIGHT_BLUE,
                        )],
                        std::time::Instant::now(),
                    ));
                    log::info!("Async scene load fully processed. Active world rebuilt.");
                }
                Err(e) => {
                    engine.ui.is_loading_assets = false;
                    engine.ui.status_message = Some((
                        vec![(
                            format!("Async scene load failed: {}", e),
                            egui::Color32::RED,
                        )],
                        std::time::Instant::now(),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_test_suite_scene() {
        if let Ok(file) = File::open("../../test_suite.aee") {
            let reader = BufReader::new(file);
            let entities: Result<Vec<SavedEntity>, _> = serde_json::from_reader(reader);
            assert!(
                entities.is_ok(),
                "test_suite.aee must deserialize cleanly into Vec<SavedEntity>: {:?}",
                entities.err()
            );
            let entities = entities.unwrap();
            assert!(!entities.is_empty(), "test_suite.aee must contain entities");
        }
    }

    #[test]
    fn test_deserialize_texture_test_suite_scene() {
        if let Ok(file) = File::open("assets/scenes/texture_test_suite.aee") {
            let reader = BufReader::new(file);
            let entities: Result<Vec<SavedEntity>, _> = serde_json::from_reader(reader);
            assert!(
                entities.is_ok(),
                "texture_test_suite.aee must deserialize cleanly into Vec<SavedEntity>: {:?}",
                entities.err()
            );
            let entities = entities.unwrap();
            assert!(
                !entities.is_empty(),
                "texture_test_suite.aee must contain entities"
            );
        }
    }
}