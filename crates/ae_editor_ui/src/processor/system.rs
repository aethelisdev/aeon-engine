// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::UiContext;

/// Handles switching active engine execution mode (Edit, Play, Pause).
pub fn handle_change_mode(ctx: &mut UiContext, mode: ae_core::modules::EngineMode) {
    *ctx.mode = mode;
}

/// Handles undoing the last command in editor history.
pub fn handle_undo(ctx: &mut UiContext) {
    ae_editor::history::undo(ctx.editor, ctx.world);
}

/// Handles redoing the last undone command in editor history.
pub fn handle_redo(ctx: &mut UiContext) {
    ae_editor::history::redo(ctx.editor, ctx.world);
}

/// Handles undoing a batch command list.
pub fn handle_undo_batch(ctx: &mut UiContext, batch: Vec<ae_editor::undo_redo::Command>) {
    ae_editor::history::push_undo(ctx.editor, ae_editor::undo_redo::Command::Batch(batch));
    ae_editor::history::undo(ctx.editor, ctx.world);
}

/// Handles toggling engine subsystem module (Physics, Audio, Render).
pub fn handle_toggle_module(ctx: &mut UiContext, module: ae_core::modules::EngineModule) {
    let enabled = ctx.event_bus.is_module_enabled(module);
    ctx.event_bus.set_module_enabled(module, !enabled);
    log::info!("Module {:?} toggled to {}", module, !enabled);
}

/// Handles updating graphics settings (MSAA, Shadow resolution, Bloom, VSync).
pub fn handle_update_graphics_settings(
    ctx: &mut UiContext,
    settings: ae_renderer::graphics_settings::GraphicsSettings,
) {
    ctx.render_state.graphics_settings = settings;
    log::info!("🎨 Updated graphics settings");
}

/// Handles updating snapping parameters.
pub fn handle_update_snap_settings(ctx: &mut UiContext, snap: ae_editor::snapping::SnapSettings) {
    ctx.editor.snapping = snap;
}

/// Handles updating editor config settings.
pub fn handle_update_editor_config(
    ctx: &mut UiContext,
    cfg: ae_editor::editor_state::EditorConfig,
) {
    ctx.editor.config = cfg;
}

/// Handles setting live editor updates toggle.
pub fn handle_set_live_editor_updates(ctx: &mut UiContext, val: bool) {
    ctx.editor.enable_live_editor_updates = val;
}

/// Handles setting 3D viewport camera projection mode (Perspective vs Orthographic).
pub fn handle_set_camera_mode(ctx: &mut UiContext, mode: ae_renderer::camera::ProjectionMode) {
    ctx.camera.mode = mode;
}

/// Handles snapping 3D viewport camera orientation to standard axes.
pub fn handle_set_camera_transform(
    ctx: &mut UiContext,
    pitch: cgmath::Rad<f32>,
    yaw: cgmath::Rad<f32>,
    pos: cgmath::Point3<f32>,
) {
    ctx.camera.pitch = pitch;
    ctx.camera.yaw = yaw;
    ctx.camera.position = pos;
}

/// Handles triggering ECS asset garbage collection sweep.
pub fn handle_garbage_collect(ctx: &mut UiContext) {
    let mut used_models = std::collections::HashSet::new();
    let mut used_textures = std::collections::HashSet::new();

    for (_ent, model_id) in ctx
        .world
        .query::<(hecs::Entity, &ae_core::ecs::ModelId)>()
        .iter()
    {
        used_models.insert(model_id.0);
    }
    for (_ent, sprite_id) in ctx
        .world
        .query::<(hecs::Entity, &ae_core::ecs::SpriteId)>()
        .iter()
    {
        used_textures.insert(sprite_id.0);
    }
    for (_ent, lod) in ctx
        .world
        .query::<(hecs::Entity, &ae_core::ecs::LodGroup)>()
        .iter()
    {
        used_models.insert(lod.lod_0);
        if let Some(h) = lod.lod_1 {
            used_models.insert(h);
        }
        if let Some(h) = lod.lod_2 {
            used_models.insert(h);
        }
    }

    let model_keys: Vec<_> = ctx
        .asset_manager
        .models
        .iter()
        .map(|(handle, _)| handle)
        .collect();
    for handle in model_keys {
        if !used_models.contains(&handle) {
            ctx.asset_manager.models.remove(handle);
        }
    }

    let texture_keys: Vec<_> = ctx
        .asset_manager
        .textures
        .iter()
        .map(|(handle, _)| handle)
        .collect();
    for handle in texture_keys {
        if !used_textures.contains(&handle) {
            ctx.asset_manager.textures.remove(handle);
        }
    }
}