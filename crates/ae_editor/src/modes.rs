// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::editor_state::EditorState;
use crate::input::{InputManager, KeyCode};
use ae_core::ecs::PlayerTag;

/// Fixed-timestep update loop for Play Mode.
/// Advances the ECS transform hierarchy and velocity integration by one fixed time step.
/// Player character movement is exclusively driven by `CharacterController` in the physics system.
pub fn fixed_update_play_mode(
    _input: &InputManager,
    ecs: &mut ae_core::ecs::EcsManager,
    fixed_time_step: f32,
) {
    ecs.update(fixed_time_step);
}

/// Per-frame update for Play Mode (render-rate logic).
/// Automatically queries the active player entity (or entity with `CharacterController`),
/// processes mouse look orbit rotation, and maintains a stable spring-arm camera behind the target.
pub fn update_play_mode(
    ecs: &mut ae_core::ecs::EcsManager,
    camera: &mut ae_core::camera::Camera,
    editor: &mut EditorState,
) {
    // Process mouse look orbit angles if mouse moved while right button held or in Play mode
    let (dx, dy) = editor.mouse_delta;
    if dx.abs() > 0.001 || dy.abs() > 0.001 {
        let sensitivity = editor.config.mouse_sensitivity * 0.7; // Comfortable, smooth sensitivity
        camera.yaw += cgmath::Rad(dx * sensitivity);
        camera.pitch -= cgmath::Rad(dy * sensitivity);

        // Clamp pitch between -75 deg and +75 deg to prevent nauseating camera flips
        let max_pitch = cgmath::Rad(std::f32::consts::FRAC_PI_2 * 0.83);
        if camera.pitch > max_pitch {
            camera.pitch = max_pitch;
        } else if camera.pitch < -max_pitch {
            camera.pitch = -max_pitch;
        }
    }
    editor.mouse_delta = (0.0, 0.0);

    // Locate active player target position (check CharacterController first, then PlayerTag)
    let mut player_pos = None;
    if let Some((_ctrl, pos)) = ecs
        .world
        .query_mut::<(&ae_core::ecs::CharacterController, &ae_core::ecs::Position)>()
        .into_iter()
        .next()
    {
        player_pos = Some(cgmath::Point3::new(pos.x, pos.y, pos.z));
    }
    if player_pos.is_none()
        && let Some((_tag, pos)) = ecs
            .world
            .query_mut::<(&PlayerTag, &ae_core::ecs::Position)>()
            .into_iter()
            .next()
    {
        player_pos = Some(cgmath::Point3::new(pos.x, pos.y, pos.z));
    }

    if let Some(mut target_pos) = player_pos {
        target_pos.y += 1.5; // Eye-level offset

        // Compute direct rigid 3D orbit position to eliminate position-target desync and camera wobble
        let forward = camera.get_forward();
        let arm_distance = 6.0_f32;
        camera.position = target_pos - forward * arm_distance;
        camera.target = target_pos;
    } else {
        camera.target = camera.position + camera.get_forward();
    }
}

/// Per-frame update for Edit Mode — handles editor camera movement.
pub fn update_edit_mode(
    editor: &mut EditorState,
    camera: &mut ae_core::camera::Camera,
    input: &InputManager,
    delta_time: f32,
) {
    use cgmath::Zero;
    let mut cam_vel = cgmath::Vector3::zero();
    let cam_base = editor.config.camera_base_speed;
    let speed = if input.is_key_pressed(KeyCode::ShiftLeft) {
        cam_base * editor.config.camera_shift_multiplier
    } else {
        cam_base
    };

    if editor.right_mouse_pressed {
        let forward = camera.get_forward();
        let right = camera.get_right();
        let up = cgmath::Vector3::unit_y();

        if camera.mode == ae_core::camera::ProjectionMode::Orthographic {
            let local_up = camera.get_up();
            if input.is_key_pressed(KeyCode::KeyW) {
                cam_vel += local_up;
            }
            if input.is_key_pressed(KeyCode::KeyS) {
                cam_vel -= local_up;
            }
            if input.is_key_pressed(KeyCode::KeyA) {
                cam_vel -= right;
            }
            if input.is_key_pressed(KeyCode::KeyD) {
                cam_vel += right;
            }
        } else {
            if input.is_key_pressed(KeyCode::KeyW) {
                cam_vel += forward;
            }
            if input.is_key_pressed(KeyCode::KeyS) {
                cam_vel -= forward;
            }
            if input.is_key_pressed(KeyCode::KeyA) {
                cam_vel -= right;
            }
            if input.is_key_pressed(KeyCode::KeyD) {
                cam_vel += right;
            }
            if input.is_key_pressed(KeyCode::KeyQ) {
                cam_vel -= up;
            }
            if input.is_key_pressed(KeyCode::KeyE) {
                cam_vel += up;
            }
        }
    }
    let displacement = cam_vel * speed * delta_time;
    camera.position += displacement;
    camera.target += displacement;

    editor.mouse_delta = (0.0, 0.0);
}

/// Applies mouse delta to camera yaw/pitch for free-look rotation.
pub fn handle_mouse_look(
    camera: &mut ae_core::camera::Camera,
    editor: &EditorState,
    dx: f64,
    dy: f64,
) {
    let sensitivity = editor.config.mouse_sensitivity;
    camera.yaw += cgmath::Rad(dx as f32 * sensitivity);
    camera.pitch -= cgmath::Rad(dy as f32 * sensitivity);

    let max_pitch = cgmath::Rad(std::f32::consts::FRAC_PI_2 - 0.01);
    if camera.pitch > max_pitch {
        camera.pitch = max_pitch;
    } else if camera.pitch < -max_pitch {
        camera.pitch = -max_pitch;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::camera::Camera;
    use ae_core::ecs::{EcsManager, Position};

    fn make_test_camera() -> Camera {
        Camera {
            position: cgmath::Point3::new(0.0, 0.0, 0.0),
            yaw: cgmath::Rad(1.23),
            pitch: cgmath::Rad(-0.25),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            mode: ae_core::camera::ProjectionMode::Perspective,
            ortho_scale: 10.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        }
    }

    /// Verifies that update_play_mode preserves camera orientation and orbit position exactly when mouse_delta is zero.
    #[test]
    fn test_update_play_mode_zero_delta_preserves_camera() {
        let mut ecs = EcsManager::new();
        let _player = ecs.world.spawn((PlayerTag, Position::new(10.0, 0.0, 5.0)));

        let mut camera = make_test_camera();
        let mut editor = EditorState {
            mouse_delta: (0.0, 0.0),
            ..Default::default()
        };

        // Run play mode update
        update_play_mode(&mut ecs, &mut camera, &mut editor);

        let initial_pos = camera.position;
        let initial_target = camera.target;
        let initial_yaw = camera.yaw;
        let initial_pitch = camera.pitch;

        // Run second frame with zero mouse delta
        update_play_mode(&mut ecs, &mut camera, &mut editor);

        assert_eq!(camera.yaw, initial_yaw);
        assert_eq!(camera.pitch, initial_pitch);
        assert_eq!(camera.position, initial_pos);
        assert_eq!(camera.target, initial_target);
        assert_eq!(editor.mouse_delta, (0.0, 0.0));
    }
}