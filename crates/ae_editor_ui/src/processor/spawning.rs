// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor action handlers for spawning models, sprites, primitives, and test scenes with
//! automatic Undo/Redo history and active editor selection registration.

#![deny(clippy::allow_attributes)]
#![deny(clippy::unwrap_used)]

use super::UiContext;
use ae_scene::{
    ModelSpawnDesc, SpriteSpawnDesc, resolve_asset_display_name, spawn_default_sprite_entity,
    spawn_empty_2d_entity, spawn_model_entity, spawn_phase1_sandbox, spawn_player_sprite_entity,
    spawn_shape_entity, spawn_sprite_entity, spawn_stress_test_cubes,
};

fn record_spawn_undo_and_select(ctx: &mut UiContext, entity: ae_ecs::Entity) {
    let snap = ae_editor::undo_redo::EntitySnapshot::capture(ctx.world, entity);
    ae_editor::history::push_undo(
        ctx.editor,
        ae_editor::undo_redo::Command::Spawn(entity, snap),
    );
    ctx.editor.redo_stack.clear();
    ctx.editor.select_single(Some(entity));
}

/// Handles spawning a 3D model asset entity into the ECS world at the default origin.
pub fn handle_spawn_model(ctx: &mut UiContext, mid: ae_renderer::asset::AssetHandle) {
    handle_spawn_model_at(ctx, mid, [0.0, 0.0, 0.0]);
}

/// Handles spawning a 3D model asset entity into the ECS world at a specified 3D world position.
pub fn handle_spawn_model_at(
    ctx: &mut UiContext,
    mid: ae_renderer::asset::AssetHandle,
    pos: [f32; 3],
) {
    let new_entity = spawn_model_entity(
        ctx.world,
        ctx.asset_manager,
        ModelSpawnDesc {
            model_id: mid,
            name_override: None,
            position: pos,
            source_path_override: None,
            apply_fbx_axis_fix: false,
        },
    );
    log::info!(
        "[SPAWN] ae_ecs::World::spawn succeeded! Entity={:?}, ModelId={:?}, Pos={:?}",
        new_entity,
        mid,
        pos
    );
    record_spawn_undo_and_select(ctx, new_entity);
}

/// Handles spawning a 2D sprite texture asset entity into the ECS world at default origin.
pub fn handle_spawn_sprite(ctx: &mut UiContext, tid: ae_renderer::asset::AssetHandle) {
    handle_spawn_sprite_at(ctx, tid, [0.0, 0.0, 0.0]);
}

/// Handles spawning a 2D sprite texture asset entity into the ECS world at a specified 3D world position.
pub fn handle_spawn_sprite_at(
    ctx: &mut UiContext,
    tid: ae_renderer::asset::AssetHandle,
    pos: [f32; 3],
) {
    let new_entity = spawn_sprite_entity(
        ctx.world,
        ctx.asset_manager,
        SpriteSpawnDesc {
            texture_id: tid,
            name_override: None,
            position: pos,
        },
    );
    log::info!(
        "[SPAWN] ae_ecs::World::spawn succeeded! Entity={:?}, SpriteId={:?}, Pos={:?}",
        new_entity,
        tid,
        pos
    );
    record_spawn_undo_and_select(ctx, new_entity);
}

/// Handles spawning a 3D model entity into the ECS world from an arbitrary file path.
///
/// Resolves cached model handles if already uploaded, or dynamically parses the GLTF/GLB file,
/// uploads vertex and index buffers to GPU memory, and spawns the entity at the target coordinates.
/// Returns `Err(message)` if model parsing fails so callers can surface errors rather than reporting
/// a false spawn confirmation.
pub fn handle_spawn_model_path_at(
    ctx: &mut UiContext,
    path: std::path::PathBuf,
    pos: [f32; 3],
) -> Result<(), String> {
    let path_str = path.to_string_lossy().to_string();
    let final_name = resolve_asset_display_name(&path, "Model");

    let mut existing_handle = None;
    if let Ok(canonical) = std::fs::canonicalize(&path)
        && let Some(&handle) = ctx.asset_manager.model_path_map.get(&canonical)
    {
        existing_handle = Some(handle);
    }

    if let Some(handle) = existing_handle {
        handle_spawn_model_at(ctx, handle, pos);
        return Ok(());
    }

    match ae_renderer::render::resources::parse_gltf_file(&path_str, final_name) {
        Ok(parsed_data) => {
            let (model_id, _min, _max) = ctx
                .render_state
                .upload_model_data(ctx.asset_manager, parsed_data);
            handle_spawn_model_at(ctx, model_id, pos);
            log::info!(
                "Model dynamically loaded from '{}' and spawned at {:?}",
                path_str,
                pos
            );
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to load 3D model from '{}': {}", path_str, e);
            Err(e)
        }
    }
}

/// Handles spawning a 2D sprite texture asset entity into the ECS world from a file path.
///
/// Dynamically loads the texture image from disk into GPU memory if not already cached
/// and spawns a new sprite entity at the designated 3D world location.
pub fn handle_spawn_sprite_path_at(ctx: &mut UiContext, path: std::path::PathBuf, pos: [f32; 3]) {
    let path_str = path.to_string_lossy().to_string();
    let texture_handle = ctx.render_state.load_texture(ctx.asset_manager, &path_str);
    handle_spawn_sprite_at(ctx, texture_handle, pos);
    log::info!(
        "Texture dynamically loaded from '{}' and spawned sprite at {:?}",
        path_str,
        pos
    );
}

/// Handles spawning a parametric 3D primitive shape entity.
pub fn handle_spawn_shape(ctx: &mut UiContext, shape: ae_plugin_api::Shape) {
    let new_entity = spawn_shape_entity(ctx.world, shape);
    record_spawn_undo_and_select(ctx, new_entity);
}

/// Handles selecting an entity in the editor hierarchy or viewport.
pub fn handle_select_entity(ctx: &mut UiContext, entity_opt: Option<ae_ecs::Entity>) {
    ctx.editor.select_single(entity_opt);
}

/// Handles deleting selected entities from the ECS world and editor state.
pub fn handle_delete_selected(ctx: &mut UiContext) {
    ae_editor::actions::delete_selected(ctx.world, ctx.editor);
}

/// Handles running the multi-object stress test spawning benchmark.
pub fn handle_stress_test(ctx: &mut UiContext, count: usize) {
    let spawned = spawn_stress_test_cubes(ctx.world, count);
    log::info!("🚀 Spawned stress test batch of {} entities!", spawned);
}

/// Handles spawning the Phase 1 interactive test sandbox.
pub fn handle_spawn_phase1_test_sandbox(ctx: &mut UiContext) {
    let player_ent = spawn_phase1_sandbox(ctx.world);
    ctx.editor.select_single(Some(player_ent));
    log::info!("🎮 Spawned Phase 1 Interactive Test Sandbox into editor!");
}

/// Handles spawning standard UI primitive entities into the ECS world.
pub fn handle_spawn_ui_element(ctx: &mut UiContext, ui_type: crate::ui::UiElementType) {
    let ent = ae_uidesign::spawn_ui_element(ctx.world, ui_type);
    ctx.editor.select_single(Some(ent));
}

/// Spawns a standard 2D Sprite entity at the origin and registers it in undo/redo history.
pub fn handle_spawn_default_sprite(ctx: &mut UiContext) {
    let new_entity = spawn_default_sprite_entity(ctx.world);
    record_spawn_undo_and_select(ctx, new_entity);
}

/// Spawns a 2D Player Sprite entity with `PlayerTag` component at the origin.
pub fn handle_spawn_player_sprite(ctx: &mut UiContext) {
    let new_entity = spawn_player_sprite_entity(ctx.world);
    record_spawn_undo_and_select(ctx, new_entity);
}

/// Spawns an empty 2D entity with spatial transform components at the origin.
pub fn handle_spawn_empty_2d(ctx: &mut UiContext) {
    let new_entity = spawn_empty_2d_entity(ctx.world);
    record_spawn_undo_and_select(ctx, new_entity);
}