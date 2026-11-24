// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor spawning action wrappers connecting [`ae_scene`] entity creation with Undo/Redo and selection.

use crate::engine::AeEngine;
use ae_scene::{
    ModelSpawnDesc, SpriteSpawnDesc, resolve_asset_display_name, spawn_default_sprite_entity,
    spawn_empty_2d_entity, spawn_model_entity, spawn_phase1_sandbox, spawn_player_sprite_entity,
    spawn_shape_entity, spawn_sprite_entity, spawn_stress_test_cubes,
};

fn record_spawn_undo_and_select(engine: &mut AeEngine, entity: ae_ecs::Entity) {
    let snap = ae_editor::undo_redo::EntitySnapshot::capture(&engine.world, entity);
    ae_editor::history::push_undo(
        &mut engine.editor,
        ae_editor::undo_redo::Command::Spawn(entity, snap),
    );
    engine.editor.redo_stack.clear();
    engine.editor.select_single(Some(entity));
}

/// Spawns a 3D model asset entity at the origin and records an undo entry.
pub fn handle_spawn_model(engine: &mut AeEngine, mid: ae_renderer::asset::AssetHandle) {
    handle_spawn_model_at(engine, mid, [0.0, 0.0, 0.0]);
}

/// Spawns a 3D model asset entity at `pos` and selects it.
pub fn handle_spawn_model_at(
    engine: &mut AeEngine,
    mid: ae_renderer::asset::AssetHandle,
    pos: [f32; 3],
) {
    let new_entity = spawn_model_entity(
        &mut engine.world,
        &engine.asset_manager,
        ModelSpawnDesc {
            model_id: mid,
            name_override: None,
            position: pos,
            source_path_override: None,
            apply_fbx_axis_fix: false,
        },
    );
    record_spawn_undo_and_select(engine, new_entity);
}

/// Spawns a 2D sprite texture entity at the origin and records an undo entry.
pub fn handle_spawn_sprite(engine: &mut AeEngine, tid: ae_renderer::asset::AssetHandle) {
    handle_spawn_sprite_at(engine, tid, [0.0, 0.0, 0.0]);
}

/// Spawns a 2D sprite texture entity at `pos` and selects it.
pub fn handle_spawn_sprite_at(
    engine: &mut AeEngine,
    tid: ae_renderer::asset::AssetHandle,
    pos: [f32; 3],
) {
    let new_entity = spawn_sprite_entity(
        &mut engine.world,
        &engine.asset_manager,
        SpriteSpawnDesc {
            texture_id: tid,
            name_override: None,
            position: pos,
        },
    );
    record_spawn_undo_and_select(engine, new_entity);
}

/// Loads (or reuses cached) GLTF/GLB asset at `path` and spawns it at `pos`.
///
/// # Errors
/// Returns an error string when GLTF/GLB parsing fails.
pub fn handle_spawn_model_path_at(
    engine: &mut AeEngine,
    path: std::path::PathBuf,
    pos: [f32; 3],
) -> Result<(), String> {
    let path_str = path.to_string_lossy().to_string();
    let final_name = resolve_asset_display_name(&path, "Model");

    if let Ok(canonical) = std::fs::canonicalize(&path)
        && let Some(&handle) = engine.asset_manager.model_path_map.get(&canonical)
    {
        handle_spawn_model_at(engine, handle, pos);
        return Ok(());
    }

    match ae_mesh::parse_gltf_file(&path_str, final_name) {
        Ok(parsed_data) => {
            let (model_id, _min, _max) = engine
                .render_state
                .upload_model_data(&mut engine.asset_manager, parsed_data);
            handle_spawn_model_at(engine, model_id, pos);
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to load 3D model from '{}': {}", path_str, e);
            Err(e)
        }
    }
}

/// Loads a texture image at `path` and spawns a sprite entity at `pos`.
pub fn handle_spawn_sprite_path_at(engine: &mut AeEngine, path: std::path::PathBuf, pos: [f32; 3]) {
    let path_str = path.to_string_lossy().to_string();
    let texture_handle = engine
        .render_state
        .load_texture(&mut engine.asset_manager, &path_str);
    handle_spawn_sprite_at(engine, texture_handle, pos);
}

/// Spawns a 3D primitive [`ae_plugin_api::Shape`] entity and records an undo entry.
pub fn handle_spawn_shape(engine: &mut AeEngine, shape: ae_plugin_api::Shape) {
    let new_entity = spawn_shape_entity(&mut engine.world, shape);
    record_spawn_undo_and_select(engine, new_entity);
}

/// Spawns `count` pseudo-random cubes for stress benchmarking.
pub fn handle_stress_test(engine: &mut AeEngine, count: usize) {
    let spawned = spawn_stress_test_cubes(&mut engine.world, count);
    log::info!("🚀 Spawned stress test batch of {} entities!", spawned);
}

/// Resets the scene and spawns the interactive Phase 1 gameplay test sandbox.
pub fn handle_spawn_phase1_test_sandbox(engine: &mut AeEngine) {
    let player_ent = spawn_phase1_sandbox(&mut engine.world);
    engine.editor.select_single(Some(player_ent));
    log::info!("🎮 Spawned Phase 1 Interactive Test Sandbox into editor!");
}

/// Spawns a UI design canvas element and selects it.
pub fn handle_spawn_ui_element(engine: &mut AeEngine, ui_type: ae_editor_ui::ui::UiElementType) {
    let ent = ae_uidesign::spawn_ui_element(&mut engine.world, ui_type);
    engine.editor.select_single(Some(ent));
}

/// Spawns a default untextured 2D sprite entity and registers an undo entry.
pub fn handle_spawn_default_sprite(engine: &mut AeEngine) {
    let new_entity = spawn_default_sprite_entity(&mut engine.world);
    record_spawn_undo_and_select(engine, new_entity);
}

/// Spawns a 2D player sprite entity and registers an undo entry.
pub fn handle_spawn_player_sprite(engine: &mut AeEngine) {
    let new_entity = spawn_player_sprite_entity(&mut engine.world);
    record_spawn_undo_and_select(engine, new_entity);
}

/// Spawns an empty spatial 2D entity and registers an undo entry.
pub fn handle_spawn_empty_2d(engine: &mut AeEngine) {
    let new_entity = spawn_empty_2d_entity(&mut engine.world);
    record_spawn_undo_and_select(engine, new_entity);
}