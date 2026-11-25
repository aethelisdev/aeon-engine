// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Thin editor adapters wrapping [`ae_scene`] entity spawning with [`ae_editor::actions::record_spawn`].

use crate::session::EditorSession;
use ae_editor::actions::record_spawn;
use ae_renderer::asset::AssetHandle;
use ae_scene::{
    ModelSpawnDesc, SpriteSpawnDesc, spawn_default_sprite_entity, spawn_empty_2d_entity,
    spawn_model_entity, spawn_model_from_path, spawn_phase1_sandbox, spawn_player_sprite_entity,
    spawn_shape_entity, spawn_sprite_entity, spawn_sprite_from_path, spawn_stress_test_cubes,
};
use std::path::Path;

/// Spawns a cached 3D model asset entity at `pos` and records an editor undo entry.
pub fn handle_spawn_model_at(engine: &mut EditorSession, mid: AssetHandle, pos: [f32; 3]) {
    let entity = spawn_model_entity(
        &mut engine.core.world,
        &engine.render.asset_manager,
        ModelSpawnDesc {
            model_id: mid,
            name_override: None,
            position: pos,
            source_path_override: None,
            apply_fbx_axis_fix: false,
        },
    );
    record_spawn(&engine.core.world, &mut engine.editor, entity);
}

/// Spawns a cached 2D sprite entity at `pos` and records an editor undo entry.
pub fn handle_spawn_sprite_at(engine: &mut EditorSession, tid: AssetHandle, pos: [f32; 3]) {
    let entity = spawn_sprite_entity(
        &mut engine.core.world,
        &engine.render.asset_manager,
        SpriteSpawnDesc {
            texture_id: tid,
            name_override: None,
            position: pos,
        },
    );
    record_spawn(&engine.core.world, &mut engine.editor, entity);
}

/// Loads (or reuses cached) GLTF/GLB asset at `path`, spawns it at `pos`, and records undo.
///
/// # Errors
/// Returns an error string when GLTF/GLB parsing fails.
pub fn handle_spawn_model_path_at(
    engine: &mut EditorSession,
    path: &Path,
    pos: [f32; 3],
) -> Result<(), String> {
    let entity = spawn_model_from_path(
        &mut engine.core.world,
        &engine.render.render_state,
        &mut engine.render.asset_manager,
        path,
        pos,
    )?;
    record_spawn(&engine.core.world, &mut engine.editor, entity);
    Ok(())
}

/// Loads a texture image at `path`, spawns a sprite entity at `pos`, and records undo.
pub fn handle_spawn_sprite_path_at(engine: &mut EditorSession, path: &Path, pos: [f32; 3]) {
    let entity = spawn_sprite_from_path(
        &mut engine.core.world,
        &engine.render.render_state,
        &mut engine.render.asset_manager,
        path,
        pos,
    );
    record_spawn(&engine.core.world, &mut engine.editor, entity);
}

/// Spawns a 3D primitive [`ae_plugin_api::Shape`] entity and records undo.
pub fn handle_spawn_shape(engine: &mut EditorSession, shape: ae_plugin_api::Shape) {
    let entity = spawn_shape_entity(&mut engine.core.world, shape);
    record_spawn(&engine.core.world, &mut engine.editor, entity);
}

/// Spawns `count` cubes for stress benchmarking.
pub fn handle_stress_test(engine: &mut EditorSession, count: usize) {
    let spawned = spawn_stress_test_cubes(&mut engine.core.world, count);
    log::info!("🚀 Spawned stress test batch of {} entities!", spawned);
}

/// Spawns the interactive gameplay test sandbox and selects the player entity.
pub fn handle_spawn_phase1_test_sandbox(engine: &mut EditorSession) {
    let player_ent = spawn_phase1_sandbox(&mut engine.core.world);
    engine.editor.select_single(Some(player_ent));
    log::info!("🎮 Spawned Phase 1 Interactive Test Sandbox into editor!");
}

/// Spawns a UI design canvas element and selects it.
pub fn handle_spawn_ui_element(engine: &mut EditorSession, ui_type: ae_uidesign::UiElementType) {
    let ent = ae_uidesign::spawn_ui_element(&mut engine.core.world, ui_type);
    engine.editor.select_single(Some(ent));
}

/// Spawns a default untextured 2D sprite entity and registers an undo entry.
pub fn handle_spawn_default_sprite(engine: &mut EditorSession) {
    let entity = spawn_default_sprite_entity(&mut engine.core.world);
    record_spawn(&engine.core.world, &mut engine.editor, entity);
}

/// Spawns a 2D player sprite entity and registers an undo entry.
pub fn handle_spawn_player_sprite(engine: &mut EditorSession) {
    let entity = spawn_player_sprite_entity(&mut engine.core.world);
    record_spawn(&engine.core.world, &mut engine.editor, entity);
}

/// Spawns an empty spatial 2D entity and registers an undo entry.
pub fn handle_spawn_empty_2d(engine: &mut EditorSession) {
    let entity = spawn_empty_2d_entity(&mut engine.core.world);
    record_spawn(&engine.core.world, &mut engine.editor, entity);
}