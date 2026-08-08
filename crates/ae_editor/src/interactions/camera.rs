// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::editor_state::EditorState;
use crate::input::{InputManager, KeyCode};
use ae_core::ecs::Position;
use cgmath::{EuclideanSpace, InnerSpace, Point3, Rad, Vector3};

/// Handles mouse scroll wheel input for camera zoom in Edit Mode.
pub fn handle_mouse_scroll(
    camera: &mut ae_core::camera::Camera,
    editor: &EditorState,
    input: &InputManager,
    is_edit_mode: bool,
    delta: &winit::event::MouseScrollDelta,
) {
    if !is_edit_mode {
        return;
    }

    let scroll_amount = match delta {
        winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.02,
    };

    if camera.mode == ae_core::camera::ProjectionMode::Orthographic {
        let zoom_speed = 2.0;
        camera.ortho_scale -= scroll_amount * zoom_speed;
        if camera.ortho_scale < 1.0 {
            camera.ortho_scale = 1.0;
        }
    } else {
        let forward = camera.get_forward();
        let scroll_base = editor.config.camera_scroll_speed;
        let speed = if input.is_key_pressed(KeyCode::ShiftLeft) {
            scroll_base * editor.config.camera_shift_multiplier
        } else {
            scroll_base
        };

        camera.position += forward * scroll_amount * speed;
    }
}

/// Focuses the camera on the first selected entity.
pub fn focus_selected(
    camera: &mut ae_core::camera::Camera,
    editor: &EditorState,
    world: &hecs::World,
) {
    let entity = match editor.selected_entities.first() {
        Some(&e) => e,
        None => return,
    };

    let pos = match world.get::<&Position>(entity) {
        Ok(p) => Vector3::new(p.x, p.y, p.z),
        Err(_) => return,
    };

    let forward = camera.get_forward();
    let distance = 5.0;
    let new_pos = pos - forward * distance;
    camera.position = Point3::from_vec(new_pos);
    camera.target = Point3::from_vec(pos);

    let dir = (pos - new_pos).normalize();
    camera.pitch = Rad(dir.y.asin());
    camera.yaw = Rad(dir.x.atan2(dir.z));
}