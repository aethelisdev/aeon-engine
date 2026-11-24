// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Runtime third-person orbit and 2D orthographic planar follow camera controller.

use ae_camera::{Camera, ProjectionMode};
use ae_plugin_api::{CharacterController, PlayerTag, Position};
use cgmath::{Point3, Rad};

/// Configuration for Play-mode camera tracking and third-person spring-arm orbit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FollowCameraConfig {
    /// Angular look sensitivity applied to incoming pointer deltas.
    pub mouse_sensitivity: f32,
    /// Third-person spring-arm standoff distance behind the target in Perspective mode.
    pub arm_distance: f32,
    /// Vertical eye-height offset added to the player entity's origin in Perspective mode.
    pub eye_height_offset: f32,
    /// Fixed camera Z coordinate maintained in Orthographic 2D follow mode.
    pub ortho_camera_z: f32,
}

impl Default for FollowCameraConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.003,
            arm_distance: 6.0,
            eye_height_offset: 1.5,
            ortho_camera_z: 10.0,
        }
    }
}

/// Updates Play-mode camera tracking (orthographic 2D follow or 3D orbit rig).
///
/// Consumes `mouse_delta` and resets it to `(0.0, 0.0)` after applying orientation.
pub fn update_follow_camera(
    world: &mut ae_ecs::World,
    camera: &mut Camera,
    mouse_delta: &mut (f32, f32),
    config: FollowCameraConfig,
) {
    if camera.mode == ProjectionMode::Orthographic {
        camera.yaw = Rad(-std::f32::consts::FRAC_PI_2);
        camera.pitch = Rad(0.0);
        *mouse_delta = (0.0, 0.0);

        if let Some(target_pos) = find_play_target_pos(world, false) {
            camera.position.x = target_pos.x;
            camera.position.y = target_pos.y;
            camera.position.z = config.ortho_camera_z;
            camera.target.x = target_pos.x;
            camera.target.y = target_pos.y;
            camera.target.z = 0.0;
        } else {
            camera.target = camera.position + camera.get_forward();
        }
        return;
    }

    let (dx, dy) = *mouse_delta;
    if dx.is_finite() && dy.is_finite() && (dx.abs() > 0.001 || dy.abs() > 0.001) {
        let sensitivity = config.mouse_sensitivity * 0.7;
        camera.yaw += Rad(dx * sensitivity);
        camera.pitch -= Rad(dy * sensitivity);

        let max_pitch = Rad(std::f32::consts::FRAC_PI_2 * 0.83);
        if camera.pitch > max_pitch {
            camera.pitch = max_pitch;
        } else if camera.pitch < -max_pitch {
            camera.pitch = -max_pitch;
        }
    }
    *mouse_delta = (0.0, 0.0);

    if let Some(mut target_pos) = find_play_target_pos(world, true) {
        target_pos.y += config.eye_height_offset;
        let forward = camera.get_forward();
        camera.position = target_pos - forward * config.arm_distance.max(0.1);
        camera.target = target_pos;
    } else {
        camera.target = camera.position + camera.get_forward();
    }
}

fn find_play_target_pos(world: &mut ae_ecs::World, prefer_controller: bool) -> Option<Point3<f32>> {
    if prefer_controller {
        if let Some(pos) = query_controller_pos(world) {
            return Some(pos);
        }
        return query_player_tag_pos(world);
    }
    if let Some(pos) = query_player_tag_pos(world) {
        return Some(pos);
    }
    query_controller_pos(world)
}

fn query_player_tag_pos(world: &mut ae_ecs::World) -> Option<Point3<f32>> {
    world
        .query_mut::<(&PlayerTag, &Position)>()
        .into_iter()
        .next()
        .map(|(_, pos)| Point3::new(pos.x, pos.y, pos.z))
}

fn query_controller_pos(world: &mut ae_ecs::World) -> Option<Point3<f32>> {
    world
        .query_mut::<(&CharacterController, &Position)>()
        .into_iter()
        .next()
        .map(|(_, pos)| Point3::new(pos.x, pos.y, pos.z))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_camera(mode: ProjectionMode) -> Camera {
        Camera {
            position: Point3::new(0.0, 0.0, 0.0),
            yaw: Rad(1.23),
            pitch: Rad(-0.25),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            mode,
            ortho_scale: 10.0,
            target: Point3::new(0.0, 0.0, 0.0),
        }
    }

    #[test]
    fn test_follow_camera_zero_delta_and_nan_delta_preserves_orientation() {
        let mut world = ae_ecs::World::new();
        let _player = world.spawn((PlayerTag, Position::new(10.0, 0.0, 5.0)));

        let mut camera = make_test_camera(ProjectionMode::Perspective);
        let mut delta = (0.0, 0.0);
        let cfg = FollowCameraConfig::default();

        update_follow_camera(&mut world, &mut camera, &mut delta, cfg);
        let init_pos = camera.position;
        let init_target = camera.target;
        let init_yaw = camera.yaw;
        let init_pitch = camera.pitch;

        // NaN deltas must safely zero out without corrupting yaw/pitch
        delta = (f32::NAN, f32::INFINITY);
        update_follow_camera(&mut world, &mut camera, &mut delta, cfg);

        assert_eq!(camera.yaw, init_yaw);
        assert_eq!(camera.pitch, init_pitch);
        assert_eq!(camera.position, init_pos);
        assert_eq!(camera.target, init_target);
        assert_eq!(delta, (0.0, 0.0));
    }

    #[test]
    fn test_follow_camera_orthographic_locks_planar() {
        let mut world = ae_ecs::World::new();
        let _player = world.spawn((PlayerTag, Position::new(5.0, 3.0, 0.0)));

        let mut camera = make_test_camera(ProjectionMode::Orthographic);
        camera.yaw = Rad(0.0);
        camera.pitch = Rad(0.5);
        let mut delta = (25.0, -15.0);

        update_follow_camera(
            &mut world,
            &mut camera,
            &mut delta,
            FollowCameraConfig::default(),
        );

        assert_eq!(camera.yaw, Rad(-std::f32::consts::FRAC_PI_2));
        assert_eq!(camera.pitch, Rad(0.0));
        assert_eq!(camera.position.x, 5.0);
        assert_eq!(camera.position.y, 3.0);
        assert_eq!(camera.position.z, 10.0);
        assert_eq!(delta, (0.0, 0.0));
    }
}