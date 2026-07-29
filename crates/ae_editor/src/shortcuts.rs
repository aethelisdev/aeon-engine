// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::editor_state::EditorState;
use crate::input::InputManager;
use ae_core::modules::EngineMode;
use winit::keyboard::KeyCode;

/// Output signals produced by editor keyboard shortcut evaluation.
#[derive(Debug, Default)]
pub struct ShortcutResult {
    /// Signal to trigger undo.
    pub trigger_undo: bool,
    /// Signal to trigger redo.
    pub trigger_redo: bool,
    /// Signal to open the scene load file dialog (Ctrl+O).
    pub trigger_open_scene_dialog: bool,
    /// Signal to save the scene (Ctrl+S).
    pub trigger_save_scene: bool,
    /// Signal to save the scene as a new file (Ctrl+Shift+S).
    pub trigger_save_scene_as: bool,
    /// Signal to focus the camera on the selected entity (F).
    pub trigger_focus_selected: bool,
    /// Updated gizmo mode if W/E/R keys were pressed in Edit mode.
    pub new_gizmo_mode: Option<crate::gizmo::GizmoMode>,
}

/// Evaluates all keyboard shortcuts (Ctrl+C, Ctrl+V, Ctrl+O, Ctrl+S, Ctrl+Shift+S, F2, Ctrl+Z, Ctrl+Y, Ctrl+D, Delete, W/E/R/F).
/// Directly dispatches ECS & editor actions (copy/paste, delete, duplicate, rename)
/// and returns a `ShortcutResult` for engine-level signals (undo/redo, open scene, save scene, camera focus).
pub fn process_shortcuts(
    input: &InputManager,
    world: &mut hecs::World,
    editor: &mut EditorState,
    ui_selected_entity: &mut Option<hecs::Entity>,
    engine_mode: EngineMode,
) -> ShortcutResult {
    let mut result = ShortcutResult::default();
    editor.focus_rename = false;

    let ctrl =
        input.is_key_pressed(KeyCode::ControlLeft) || input.is_key_pressed(KeyCode::ControlRight);
    let shift =
        input.is_key_pressed(KeyCode::ShiftLeft) || input.is_key_pressed(KeyCode::ShiftRight);

    // --- CTRL SHORTCUTS ---
    if ctrl {
        // Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y
        if input.is_key_just_pressed(KeyCode::KeyZ) {
            if shift {
                result.trigger_redo = true;
            } else {
                result.trigger_undo = true;
            }
        }
        if input.is_key_just_pressed(KeyCode::KeyY) {
            result.trigger_redo = true;
        }

        // Ctrl+S (Save Scene) / Ctrl+Shift+S (Save Scene As)
        if input.is_key_just_pressed(KeyCode::KeyS) {
            if shift {
                result.trigger_save_scene_as = true;
            } else {
                result.trigger_save_scene = true;
            }
        }

        // Ctrl+C (Copy Selected)
        if input.is_key_just_pressed(KeyCode::KeyC) {
            crate::clipboard::copy_selected(world, editor);
        }

        // Ctrl+V (Paste Clipboard)
        if input.is_key_just_pressed(KeyCode::KeyV) {
            crate::clipboard::paste_clipboard(world, editor, ui_selected_entity);
        }

        // Ctrl+D (Duplicate Selected)
        if input.is_key_just_pressed(KeyCode::KeyD) {
            crate::actions::duplicate_selected(world, editor, ui_selected_entity);
        }

        // Ctrl+O (Open Scene Dialog)
        if input.is_key_just_pressed(KeyCode::KeyO) {
            result.trigger_open_scene_dialog = true;
        }
    }

    // --- F2 SHORTCUT (Rename Selected) ---
    if input.is_key_just_pressed(KeyCode::F2) {
        crate::clipboard::rename_selected(editor);
    }

    // --- DELETE SHORTCUT ---
    if input.is_key_just_pressed(KeyCode::Delete) {
        crate::actions::delete_selected(world, editor, ui_selected_entity);
    }

    // --- EDIT MODE TRANSFORM SHORTCUTS (W, E, R, F) ---
    if engine_mode == EngineMode::Edit {
        if input.is_key_just_pressed(KeyCode::KeyW) {
            result.new_gizmo_mode = Some(crate::gizmo::GizmoMode::Translate);
        }
        if input.is_key_just_pressed(KeyCode::KeyE) {
            result.new_gizmo_mode = Some(crate::gizmo::GizmoMode::Rotate);
        }
        if input.is_key_just_pressed(KeyCode::KeyR) {
            result.new_gizmo_mode = Some(crate::gizmo::GizmoMode::Scale);
        }
        if input.is_key_just_pressed(KeyCode::KeyF) {
            result.trigger_focus_selected = true;
        }
    }

    // Snapping hold mode sync
    if editor.snapping.mode == crate::snapping::SnapMode::Hold {
        editor.snapping.current_enabled = ctrl;
    }

    result
}