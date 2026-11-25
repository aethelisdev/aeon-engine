// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
#![deny(clippy::unwrap_used)]

use crate::undo_redo::{Command, EntitySnapshot};
use ae_ecs::Entity;
use std::collections::{HashMap, HashSet};

/// Action payload sent from async native file dialog threads to the main editor loop.
pub enum SceneDialogAction {
    /// Save the active scene to the selected file path.
    SaveTo(std::path::PathBuf),
    /// Load a scene from the selected file path.
    LoadFrom(std::path::PathBuf),
}

/// Configurable editor parameters exposed in the Preferences panel.
///
/// Controls camera movement speeds, mouse sensitivity, undo history depth,
/// and the physics fixed update frequency (physics_hz).
/// These values can be modified at runtime.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EditorConfig {
    pub camera_base_speed: f32,
    pub camera_shift_multiplier: f32,
    pub camera_scroll_speed: f32,
    pub mouse_sensitivity: f32,
    pub max_undo_history: usize,
    /// The physics simulation fixed update step frequency in Hertz (Hz).
    pub physics_hz: f32,
    /// Whether periodic background Edit-mode auto-save snapshots are enabled.
    pub autosave_enabled: bool,
    /// Seconds of Edit-mode activity between periodic auto-save snapshots (`>= 10.0`).
    pub autosave_interval_secs: f32,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            camera_base_speed: 5.0,
            camera_shift_multiplier: 5.0,
            camera_scroll_speed: 1.5,
            mouse_sensitivity: 0.005,
            max_undo_history: 100,
            physics_hz: 120.0,
            autosave_enabled: true,
            autosave_interval_secs: 300.0,
        }
    }
}

impl EditorConfig {
    /// Converts editor camera preferences into an [`ae_camera_controller::EditorCameraConfig`].
    #[must_use]
    pub fn camera_config(&self) -> ae_camera_controller::EditorCameraConfig {
        ae_camera_controller::EditorCameraConfig {
            base_speed: self.camera_base_speed,
            shift_multiplier: self.camera_shift_multiplier,
            scroll_speed: self.camera_scroll_speed,
            mouse_sensitivity: self.mouse_sensitivity,
        }
    }
}

/// Aeon Engine — Editor State
///
/// Isolates all logic and state specific to the level editor interface:
/// selection, dragging state, interaction flags, multi-selection snapshots,
/// scene backup for Play→Edit restore, and configurable preferences.
pub struct EditorState {
    /// Whether the 3D viewport owns an active right-mouse camera look or pan gesture.
    pub viewport_camera_look_active: bool,
    /// Whether the user is currently interacting with a transform gizmo.
    pub gizmo_dragging: bool,
    /// Snapshots of entity properties captured before a manipulation begins for multi-select.
    pub current_edit_snapshots: HashMap<Entity, EntitySnapshot>,
    /// O(1) Quick lookup for selected entities to avoid O(N*M) during extraction.
    pub selected_entities_set: HashSet<Entity>,
    /// A full scene backup used to restore state when exiting Play Mode.
    pub scene_backup: HashMap<Entity, EntitySnapshot>,
    /// Camera backup to restore editor camera transform when exiting Play Mode.
    pub camera_backup: Option<ae_camera::Camera>,
    /// Multi-selection: list of selected entities.
    pub selected_entities: Vec<Entity>,
    /// Store original positions for multi-selection drag.
    pub multi_start_positions: HashMap<Entity, cgmath::Vector3<f32>>,
    /// Store original scales for multi-selection drag.
    pub multi_start_scales: HashMap<Entity, cgmath::Vector3<f32>>,
    /// Store original rotations for multi-selection drag.
    pub multi_start_rotations: HashMap<Entity, cgmath::Quaternion<f32>>,
    /// Whether hot reloading / live editor updates are enabled.
    pub enable_live_editor_updates: bool,
    /// Configurable settings for the editor (e.g. camera speeds).
    pub config: EditorConfig,
    /// The undo command history stack.
    pub undo_stack: Vec<Command>,
    /// The redo command history stack.
    pub redo_stack: Vec<Command>,
    /// Snapping settings.
    pub snapping: crate::snapping::SnapSettings,
    /// Clipboard buffer for Ctrl+C / Ctrl+V entity copy & paste.
    pub clipboard: Vec<EntitySnapshot>,
    /// Signal flag indicating F2 was pressed to focus inline entity name renaming.
    pub focus_rename: bool,
    /// Currently loaded/saved scene file path for quick Ctrl+S saving.
    pub active_scene_path: Option<std::path::PathBuf>,
    /// Whether the editor is currently running in 2D mode.
    pub is_2d: bool,
    /// Monotonic transaction revision counter incremented on every undo/redo or command execution.
    pub history_revision: u64,
    /// Whether viewport wireframe overlay mode is enabled.
    pub wireframe_enabled: bool,
    /// Whether the infinite world grid overlay is enabled in 3D Edit mode.
    pub grid_enabled: bool,
    /// Active viewport transform gizmo manipulation tool.
    pub gizmo_mode: ae_gizmos::GizmoMode,
    /// Active transform gizmo coordinate space (`Local` vs `World`).
    pub gizmo_space: ae_gizmos::GizmoSpace,
    /// Whether background asset/scene loading progress feedback is active.
    pub is_loading_assets: bool,
    /// Pending save-scene trigger flag consumed by the main application loop.
    pub should_save_scene: bool,
    /// Pending load-scene trigger flag consumed by the main application loop.
    pub should_load_scene: bool,
    /// Explicit destination path queued by Save As file dialog.
    pub pending_save_path: Option<std::path::PathBuf>,
    /// Explicit source path queued by Open Scene file dialog.
    pub pending_load_path: Option<std::path::PathBuf>,
    /// Application exit trigger raised by File -> Exit or titlebar close actions.
    pub should_exit: bool,
    /// Non-blocking receivers polling async native scene open/save file dialog completions.
    pub scene_dialog_receivers: Vec<std::sync::mpsc::Receiver<SceneDialogAction>>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            viewport_camera_look_active: false,
            gizmo_dragging: false,
            current_edit_snapshots: HashMap::new(),
            selected_entities_set: HashSet::new(),
            scene_backup: HashMap::new(),
            camera_backup: None,
            selected_entities: Vec::new(),
            multi_start_positions: HashMap::new(),
            multi_start_scales: HashMap::new(),
            multi_start_rotations: HashMap::new(),
            enable_live_editor_updates: false,
            config: EditorConfig::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            snapping: crate::snapping::SnapSettings::default(),
            clipboard: Vec::new(),
            focus_rename: false,
            active_scene_path: None,
            is_2d: false,
            history_revision: 0,
            wireframe_enabled: false,
            grid_enabled: true,
            gizmo_mode: ae_gizmos::GizmoMode::Select,
            gizmo_space: ae_gizmos::GizmoSpace::Local,
            is_loading_assets: false,
            should_save_scene: false,
            should_load_scene: false,
            pending_save_path: None,
            pending_load_path: None,
            should_exit: false,
            scene_dialog_receivers: Vec::new(),
        }
    }
}

impl EditorState {
    /// Polls asynchronous native scene open/save dialog receivers and applies pending triggers.
    pub fn poll_scene_dialogs(&mut self) {
        if self.scene_dialog_receivers.is_empty() {
            return;
        }
        std::mem::take(&mut self.scene_dialog_receivers)
            .into_iter()
            .for_each(|rx| match rx.try_recv() {
                Ok(SceneDialogAction::SaveTo(path)) => {
                    self.pending_save_path = Some(path);
                    self.should_save_scene = true;
                }
                Ok(SceneDialogAction::LoadFrom(path)) => {
                    self.pending_load_path = Some(path);
                    self.should_load_scene = true;
                    self.is_loading_assets = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => self.scene_dialog_receivers.push(rx),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
            });
    }

    /// Returns the primary selected entity, if any.
    #[inline]
    #[must_use]
    pub fn primary_selected(&self) -> Option<Entity> {
        self.selected_entities.first().copied()
    }

    /// Replaces the active editor selection with a single optional entity.
    pub fn select_single(&mut self, entity: Option<Entity>) {
        self.selected_entities.clear();
        self.selected_entities_set.clear();
        if let Some(ent) = entity {
            self.selected_entities.push(ent);
            self.selected_entities_set.insert(ent);
        }
    }
    /// Backs up all entity snapshots in the ECS World before entering Play Mode.
    pub fn backup_scene(&mut self, world: &ae_ecs::World) {
        self.scene_backup.clear();
        for entity in world.iter().map(|e| e.entity()) {
            let snapshot = EntitySnapshot::capture(world, entity);
            self.scene_backup.insert(entity, snapshot);
        }
    }

    /// Restores all backed-up entity snapshots when returning from Play Mode to Edit Mode.
    ///
    /// Preserves original scene entity states, including user-configured initial velocities,
    /// while cleanly removing transient components or entities created during gameplay.
    pub fn restore_scene(&self, world: &mut ae_ecs::World) {
        // Despawn temporary entities spawned during gameplay that were not in the scene backup
        let current_entities: Vec<ae_ecs::Entity> = world.iter().map(|e| e.entity()).collect();
        for entity in current_entities {
            if !self.scene_backup.contains_key(&entity) {
                let _ = world.despawn(entity);
            }
        }

        // Restore existing entities to pre-play snapshot state
        for (entity, snapshot) in &self.scene_backup {
            if world.contains(*entity) {
                snapshot.apply(world, *entity);
                // If velocity was not present in the pre-play snapshot, clean up any transient gameplay velocity
                if !snapshot.has::<ae_plugin_api::Velocity>() {
                    let _ = world.remove_one::<ae_plugin_api::Velocity>(*entity);
                }
                let _ = world.insert_one(*entity, ae_plugin_api::TransformDirty);
                let _ = world.remove_one::<ae_plugin_api::GlobalTransform>(*entity);
            } else {
                // Re-spawn entities that were despawned during play mode
                let new_ent = snapshot.spawn(world);
                let _ = world.insert_one(new_ent, ae_plugin_api::TransformDirty);
                let _ = world.remove_one::<ae_plugin_api::GlobalTransform>(new_ent);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_plugin_api::{Position, Velocity};

    #[test]
    fn test_editor_scene_backup_and_restore() {
        let mut world = ae_ecs::World::new();
        let e1 = world.spawn((
            Position {
                x: 10.0,
                y: 5.0,
                z: -2.0,
            },
            Velocity {
                x: 1.5,
                y: 2.0,
                z: -3.0,
            },
        ));
        let e2 = world.spawn((Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },));

        let mut editor = EditorState::default();
        editor.backup_scene(&world);

        // Simulate gameplay mutations
        if let Ok(pos) = world.get_mut::<Position>(e1) {
            pos.x = 999.0;
            pos.y = -500.0;
        }
        if let Ok(vel) = world.get_mut::<Velocity>(e1) {
            vel.x = 100.0;
            vel.y = -50.0;
            vel.z = 200.0;
        }
        // Despawn e2 in gameplay
        world
            .despawn(e2)
            .expect("Despawn e2 entity must succeed in test");
        // Spawn temporary gameplay bullet
        let _temp = world.spawn((Position {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },));

        // Restore scene
        editor.restore_scene(&mut world);

        // e1 position restored
        let pos1 = world
            .get_copied::<Position>(e1)
            .expect("e1 Position must exist after scene restore");
        assert_eq!(pos1.x, 10.0);
        assert_eq!(pos1.y, 5.0);
        assert_eq!(pos1.z, -2.0);

        // e1 velocity restored to initial backup value (not zeroed out)
        let vel1 = world
            .get_copied::<Velocity>(e1)
            .expect("e1 Velocity must exist after scene restore");
        assert_eq!(vel1.x, 1.5);
        assert_eq!(vel1.y, 2.0);
        assert_eq!(vel1.z, -3.0);

        // Temporary bullet despawned and e2 re-spawned (total 2 entities)
        assert_eq!(world.iter().count(), 2);
    }
}