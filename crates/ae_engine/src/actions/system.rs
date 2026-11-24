// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! System-level editor action handlers (mode switching, Undo/Redo, settings, camera, and asset GC).

use crate::engine::AeEngine;
use ae_core::modules::{EngineMode, EngineModule};

/// Switches engine execution mode (Edit / Play / Pause) with scene backup and restore.
pub fn handle_change_mode(engine: &mut AeEngine, mode: EngineMode) {
    if engine.mode == mode {
        return;
    }
    if mode == EngineMode::Play {
        engine.editor.backup_scene(&engine.world);
        engine.editor.camera_backup = Some(engine.camera.clone());
        engine.ui.prepare_for_play_mode();
    } else if engine.mode == EngineMode::Play && mode == EngineMode::Edit {
        engine.editor.restore_scene(&mut engine.world);
        if let Some(cam_backup) = engine.editor.camera_backup.take() {
            engine.camera = cam_backup;
        }
    }
    engine.mode = mode;
    log::info!("🎮 Engine mode changed to {:?}", mode);
}

/// Undoes the latest command in editor history.
pub fn handle_undo(engine: &mut AeEngine) {
    ae_editor::history::undo(&mut engine.editor, &mut engine.world);
}

/// Redoes the latest undone command in editor history.
pub fn handle_redo(engine: &mut AeEngine) {
    ae_editor::history::redo(&mut engine.editor, &mut engine.world);
}

/// Pushes a command batch and immediately undoes it as one atomic editor action.
pub fn handle_undo_batch(engine: &mut AeEngine, batch: Vec<ae_editor::undo_redo::Command>) {
    ae_editor::history::push_undo(
        &mut engine.editor,
        ae_editor::undo_redo::Command::Batch(batch),
    );
    ae_editor::history::undo(&mut engine.editor, &mut engine.world);
}

/// Toggles enable state of a runtime subsystem module on [`ae_core::events::DynamicEventBus`].
pub fn handle_toggle_module(engine: &mut AeEngine, module: EngineModule) {
    let enabled = engine.event_bus.is_module_enabled(module);
    engine.event_bus.set_module_enabled(module, !enabled);
    log::info!("Module {:?} toggled to {}", module, !enabled);
}

/// Updates renderer graphics settings.
pub fn handle_update_graphics_settings(
    engine: &mut AeEngine,
    settings: ae_renderer::graphics_settings::GraphicsSettings,
) {
    engine.render_state.graphics_settings = settings;
    log::info!("🎨 Updated graphics settings");
}

/// Updates editor grid and snapping settings.
pub fn handle_update_snap_settings(engine: &mut AeEngine, snap: ae_editor::snapping::SnapSettings) {
    engine.editor.snapping = snap;
    log::info!(
        "🧲 Updated snapping settings: mode={:?}, grid={:.2}",
        snap.mode,
        snap.grid_size
    );
}

/// Updates editor general configuration preferences.
pub fn handle_update_editor_config(
    engine: &mut AeEngine,
    cfg: ae_editor::editor_state::EditorConfig,
) {
    engine.editor.config = cfg;
    log::info!(
        "⚙ Updated editor config: undo_limit={}, physics_hz={:.0} Hz",
        engine.editor.config.max_undo_history,
        engine.editor.config.physics_hz
    );
}

/// Sets plugin live hot-reload updates flag and surfaces status feedback.
pub fn handle_set_live_editor_updates(engine: &mut AeEngine, val: bool) {
    engine.editor.enable_live_editor_updates = val;
    let (msg, color) = if val {
        (
            "Live Editor Updates (Hot Reload) Enabled",
            irisui::prelude::Color::rgb(70.0 / 255.0, 190.0 / 255.0, 120.0 / 255.0),
        )
    } else {
        (
            "Live Editor Updates (Hot Reload) Disabled",
            irisui::prelude::Color::rgb(220.0 / 255.0, 140.0 / 255.0, 60.0 / 255.0),
        )
    };
    engine.ui.set_status_message(msg, color);
}

/// Updates editor viewport camera projection mode.
pub fn handle_set_camera_mode(engine: &mut AeEngine, mode: ae_camera::ProjectionMode) {
    engine.camera.mode = mode;
}

/// Toggles viewport camera projection mode between perspective and orthographic.
pub fn handle_toggle_camera_projection(engine: &mut AeEngine) {
    match engine.camera.mode {
        ae_camera::ProjectionMode::Perspective => {
            engine.camera.mode = ae_camera::ProjectionMode::Orthographic;
            engine.camera.pitch = cgmath::Rad(0.0);
            engine.camera.yaw = cgmath::Rad(-std::f32::consts::FRAC_PI_2);
        }
        ae_camera::ProjectionMode::Orthographic => {
            engine.camera.mode = ae_camera::ProjectionMode::Perspective;
            if engine.camera.pitch.0.abs() < 0.05 {
                engine.camera.pitch = cgmath::Rad(-0.35);
                engine.camera.yaw = cgmath::Rad(-0.60);
            }
        }
    }
}

/// Applies explicit orientation, position, and optional projection mode to the viewport camera.
pub fn handle_set_camera_transform(
    engine: &mut AeEngine,
    pitch: cgmath::Rad<f32>,
    yaw: cgmath::Rad<f32>,
    pos: cgmath::Point3<f32>,
    mode: Option<ae_camera::ProjectionMode>,
) {
    engine.camera.pitch = pitch;
    engine.camera.yaw = yaw;
    engine.camera.position = pos;
    if let Some(m) = mode {
        engine.camera.mode = m;
    }
}

/// Triggers asset garbage collection sweep via [`ae_scene::garbage_collect_unused_assets`].
pub fn handle_garbage_collect(engine: &mut AeEngine) {
    let (removed_models, removed_textures) =
        ae_scene::garbage_collect_unused_assets(&engine.world, &mut engine.asset_manager);
    log::info!(
        "🧹 Asset GC swept {removed_models} unused model(s) and {removed_textures} texture(s)."
    );
}