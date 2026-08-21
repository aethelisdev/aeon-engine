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
    camera.yaw = Rad(dir.z.atan2(dir.x));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that focus_selected points the camera forward vector towards the selected entity.
    #[test]
    fn test_focus_selected_forward_alignment() {
        let mut world = hecs::World::new();
        let entity = world.spawn((Position::new(10.0, 0.0, 0.0),));

        let mut editor = EditorState::default();
        editor.selected_entities.push(entity);

        let mut camera = ae_core::camera::Camera {
            position: Point3::new(0.0, 0.0, 0.0),
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            aspect: 1.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ae_core::camera::ProjectionMode::Perspective,
            ortho_scale: 1.0,
            target: Point3::new(0.0, 0.0, 0.0),
        };

        focus_selected(&mut camera, &editor, &world);

        let fwd = camera.get_forward();
        assert!(
            (fwd.x - 1.0).abs() < 1e-4,
            "Forward X should be ~1.0 towards entity, got {}",
            fwd.x
        );
        assert!(
            fwd.z.abs() < 1e-4,
            "Forward Z should be ~0.0 towards entity, got {}",
            fwd.z
        );
    }
}