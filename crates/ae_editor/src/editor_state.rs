// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::undo_redo::{Command, EntitySnapshot};
use hecs::Entity;
use std::collections::{HashMap, HashSet};

/// Configurable editor parameters exposed in the Preferences panel.
/// Controls camera movement speeds, mouse sensitivity, undo history depth,
/// and the physics fixed update frequency (physics_hz).
/// These values can be modified at runtime.
#[derive(Clone, Debug)]
pub struct EditorConfig {
    pub camera_base_speed: f32,
    pub camera_shift_multiplier: f32,
    pub camera_scroll_speed: f32,
    pub mouse_sensitivity: f32,
    pub max_undo_history: usize,
    /// The physics simulation fixed update step frequency in Hertz (Hz).
    pub physics_hz: f32,
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
        }
    }
}

/// Aeon Engine — Editor State
/// Isolates all logic and state specific to the level editor interface:
/// selection, dragging state, interaction flags, multi-selection snapshots,
/// scene backup for Play→Edit restore, and configurable preferences.
pub struct EditorState {
    /// Accumulated mouse delta for this frame (used for dragging).
    pub mouse_delta: (f32, f32),
    /// Whether the right mouse button is currently held down (Camera Look).
    pub right_mouse_pressed: bool,
    /// Whether the left mouse button is currently held down (Interaction).
    pub left_mouse_pressed: bool,
    /// Whether the user is currently interacting with a transform gizmo.
    pub gizmo_dragging: bool,
    /// Last recorded cursor position (used for delta calculation).
    pub last_cursor_pos: (f64, f64),
    /// Snapshots of entity properties captured before a manipulation begins for multi-select.
    pub current_edit_snapshots: HashMap<Entity, EntitySnapshot>,
    /// O(1) Quick lookup for selected entities to avoid O(N*M) during extraction.
    pub selected_entities_set: HashSet<Entity>,
    /// A full scene backup used to restore state when exiting Play Mode.
    pub scene_backup: HashMap<Entity, EntitySnapshot>,
    /// Camera backup to restore editor camera transform when exiting Play Mode.
    pub camera_backup: Option<ae_core::camera::Camera>,
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
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            mouse_delta: (0.0, 0.0),
            right_mouse_pressed: false,
            left_mouse_pressed: false,
            gizmo_dragging: false,
            last_cursor_pos: (0.0, 0.0),
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
        }
    }
}

impl EditorState {
    /// Backs up all entity snapshots in the ECS World before entering Play Mode.
    pub fn backup_scene(&mut self, world: &hecs::World) {
        self.scene_backup.clear();
        for entity in world.iter().map(|e| e.entity()) {
            let snapshot = EntitySnapshot::capture(world, entity);
            self.scene_backup.insert(entity, snapshot);
        }
    }

    /// Restores all backed-up entity snapshots when returning from Play Mode to Edit Mode.
    pub fn restore_scene(&self, world: &mut hecs::World) {
        // Despawn temporary entities spawned during gameplay that were not in the scene backup
        let current_entities: Vec<hecs::Entity> = world.iter().map(|e| e.entity()).collect();
        for entity in current_entities {
            if !self.scene_backup.contains_key(&entity) {
                let _ = world.despawn(entity);
            }
        }

        // Restore existing entities and zero out any lingering gameplay physics velocities
        for (entity, snapshot) in &self.scene_backup {
            if world.contains(*entity) {
                snapshot.apply(world, *entity);
                if let Ok(mut vel) = world.get::<&mut ae_core::ecs::Velocity>(*entity) {
                    vel.x = 0.0;
                    vel.y = 0.0;
                    vel.z = 0.0;
                }
                let _ = world.insert_one(*entity, ae_core::ecs::TransformDirty);
                let _ = world.remove_one::<ae_core::ecs::GlobalTransform>(*entity);
            } else {
                // Re-spawn entities that were despawned during play mode
                let new_ent = snapshot.spawn(world);
                let _ = world.insert_one(new_ent, ae_core::ecs::TransformDirty);
                let _ = world.remove_one::<ae_core::ecs::GlobalTransform>(new_ent);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::ecs::{Position, Velocity};

    #[test]
    fn test_editor_scene_backup_and_restore() {
        let mut world = hecs::World::new();
        let e1 = world.spawn((
            Position {
                x: 10.0,
                y: 5.0,
                z: -2.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
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
        if let Ok(mut pos) = world.get::<&mut Position>(e1) {
            pos.x = 999.0;
            pos.y = -500.0;
        }
        if let Ok(mut vel) = world.get::<&mut Velocity>(e1) {
            vel.y = -50.0;
        }
        // Despawn e2 in gameplay
        world.despawn(e2).unwrap();
        // Spawn temporary gameplay bullet
        let _temp = world.spawn((Position {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },));

        // Restore scene
        editor.restore_scene(&mut world);

        // e1 position restored
        let pos1 = world.get::<&Position>(e1).unwrap();
        assert_eq!(pos1.x, 10.0);
        assert_eq!(pos1.y, 5.0);
        assert_eq!(pos1.z, -2.0);

        // e1 velocity zeroed
        let vel1 = world.get::<&Velocity>(e1).unwrap();
        assert_eq!(vel1.x, 0.0);
        assert_eq!(vel1.y, 0.0);
        assert_eq!(vel1.z, 0.0);

        // Temporary bullet despawned and e2 re-spawned (total 2 entities)
        assert_eq!(world.iter().count(), 2);
    }
}