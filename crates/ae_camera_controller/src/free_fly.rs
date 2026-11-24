// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Free-fly, orthographic pan/zoom, mouse-look, and entity-focus camera controllers.

use ae_camera::{Camera, ProjectionMode};
use cgmath::{EuclideanSpace, InnerSpace, Point3, Rad, Vector3, Zero};

/// Tunable speed and sensitivity settings for editor viewport camera navigation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorCameraConfig {
    /// Base linear movement speed in world units per second.
    pub base_speed: f32,
    /// Speed multiplier applied while the boost key is held.
    pub shift_multiplier: f32,
    /// Linear scroll dolly speed in world units per scroll notch.
    pub scroll_speed: f32,
    /// Angular mouse look sensitivity in radians per pixel.
    pub mouse_sensitivity: f32,
}

impl Default for EditorCameraConfig {
    fn default() -> Self {
        Self {
            base_speed: 5.0,
            shift_multiplier: 5.0,
            scroll_speed: 1.5,
            mouse_sensitivity: 0.003,
        }
    }
}

/// Windowing-agnostic digital movement state for the free-fly camera controller.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FreeCameraInput {
    /// Move forward (`W`) or pan up in Orthographic mode.
    pub forward: bool,
    /// Move backward (`S`) or pan down in Orthographic mode.
    pub backward: bool,
    /// Strafe left (`A`).
    pub left: bool,
    /// Strafe right (`D`).
    pub right: bool,
    /// Ascend vertically (`E`) in Perspective mode.
    pub up: bool,
    /// Descend vertically (`Q`) in Perspective mode.
    pub down: bool,
    /// Apply speed boost multiplier (`Shift`).
    pub boost: bool,
    /// Navigation requires right mouse button held in the editor viewport.
    pub right_mouse_pressed: bool,
}

/// Advances editor camera position and look-at target using digital flight/pan inputs.
pub fn step_free_camera(
    camera: &mut Camera,
    config: EditorCameraConfig,
    input: FreeCameraInput,
    delta_time: f32,
) {
    if !input.right_mouse_pressed || delta_time <= 0.0 || !delta_time.is_finite() {
        return;
    }

    let mut cam_vel = Vector3::zero();
    let speed = if input.boost {
        config.base_speed * config.shift_multiplier
    } else {
        config.base_speed
    };

    let forward = camera.get_forward();
    let right = camera.get_right();

    if camera.mode == ProjectionMode::Orthographic {
        let local_up = camera.get_up();
        if input.forward {
            cam_vel += local_up;
        }
        if input.backward {
            cam_vel -= local_up;
        }
        if input.left {
            cam_vel -= right;
        }
        if input.right {
            cam_vel += right;
        }
    } else {
        let up = Vector3::unit_y();
        if input.forward {
            cam_vel += forward;
        }
        if input.backward {
            cam_vel -= forward;
        }
        if input.left {
            cam_vel -= right;
        }
        if input.right {
            cam_vel += right;
        }
        if input.down {
            cam_vel -= up;
        }
        if input.up {
            cam_vel += up;
        }
    }

    let displacement = cam_vel * speed * delta_time;
    camera.position += displacement;
    camera.target += displacement;
}

/// Pans an orthographic camera in pixel-proportional world units while preserving depth.
pub fn pan_orthographic_camera(
    camera: &mut Camera,
    dx_px: f64,
    dy_px: f64,
    viewport_height_px: f32,
) {
    if !dx_px.is_finite() || !dy_px.is_finite() {
        return;
    }
    let vp_h = viewport_height_px.max(1.0);
    let world_per_pixel = camera.ortho_scale / vp_h;
    camera.position.x -= (dx_px as f32) * world_per_pixel;
    camera.position.y += (dy_px as f32) * world_per_pixel;
    camera.target.x = camera.position.x;
    camera.target.y = camera.position.y;
}

/// Applies pointer motion deltas to camera yaw/pitch with vertical gimbal clamping.
pub fn apply_mouse_look(camera: &mut Camera, dx: f64, dy: f64, mouse_sensitivity: f32) {
    if !dx.is_finite() || !dy.is_finite() || !mouse_sensitivity.is_finite() {
        return;
    }
    camera.yaw += Rad(dx as f32 * mouse_sensitivity);
    camera.pitch -= Rad(dy as f32 * mouse_sensitivity);

    let max_pitch = Rad(std::f32::consts::FRAC_PI_2 - 0.01);
    if camera.pitch > max_pitch {
        camera.pitch = max_pitch;
    } else if camera.pitch < -max_pitch {
        camera.pitch = -max_pitch;
    }
}

/// Applies scroll zoom to either orthographic scale (`>= 1.0`) or perspective forward dolly.
pub fn apply_scroll_zoom(
    camera: &mut Camera,
    scroll_amount: f32,
    boost: bool,
    config: EditorCameraConfig,
) {
    if !scroll_amount.is_finite() || scroll_amount == 0.0 {
        return;
    }

    if camera.mode == ProjectionMode::Orthographic {
        let zoom_speed = 2.0;
        camera.ortho_scale = (camera.ortho_scale - scroll_amount * zoom_speed).max(1.0);
    } else {
        let forward = camera.get_forward();
        let speed = if boost {
            config.scroll_speed * config.shift_multiplier
        } else {
            config.scroll_speed
        };
        camera.position += forward * scroll_amount * speed;
    }
}

/// Frames `camera` onto an explicit world-space focus point at `distance`.
///
/// In `Orthographic` mode, preserves camera orientation and snaps XY coordinates only.
pub fn focus_camera_on_point(camera: &mut Camera, point: Point3<f32>, distance: f32) {
    let target = point.to_vec();
    if camera.mode == ProjectionMode::Orthographic {
        camera.position.x = target.x;
        camera.position.y = target.y;
        camera.target.x = target.x;
        camera.target.y = target.y;
        return;
    }

    let safe_distance = distance.max(0.1);
    let forward = camera.get_forward();
    let new_pos = target - forward * safe_distance;
    camera.position = Point3::from_vec(new_pos);
    camera.target = Point3::from_vec(target);

    let dir = target - new_pos;
    if dir.magnitude2() > 1e-8 {
        let dir = dir.normalize();
        camera.pitch = Rad(dir.y.clamp(-1.0, 1.0).asin());
        camera.yaw = Rad(dir.z.atan2(dir.x));
    }
}

/// Looks up `entity`'s [`ae_plugin_api::Position`] and frames `camera` onto it.
///
/// Returns `false` if the entity is missing a `Position` component.
pub fn focus_camera_on_entity(
    camera: &mut Camera,
    world: &ae_ecs::World,
    entity: ae_ecs::Entity,
    distance: f32,
) -> bool {
    let Ok(pos) = world.get_copied::<ae_plugin_api::Position>(entity) else {
        return false;
    };
    focus_camera_on_point(camera, Point3::new(pos.x, pos.y, pos.z), distance);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_plugin_api::Position;

    fn make_camera(mode: ProjectionMode) -> Camera {
        Camera {
            position: Point3::new(0.0, 0.0, 10.0),
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            aspect: 1.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode,
            ortho_scale: 10.0,
            target: Point3::new(0.0, 0.0, 0.0),
        }
    }

    #[test]
    fn test_focus_camera_on_entity_perspective_and_missing_component_guard() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((Position::new(10.0, 0.0, 0.0),));
        let empty = world.spawn(());

        let mut camera = make_camera(ProjectionMode::Perspective);
        camera.position = Point3::new(0.0, 0.0, 0.0);

        assert!(!focus_camera_on_entity(&mut camera, &world, empty, 5.0));
        assert!(focus_camera_on_entity(&mut camera, &world, entity, 5.0));

        let fwd = camera.get_forward();
        assert!((fwd.x - 1.0).abs() < 1e-4);
        assert!(fwd.z.abs() < 1e-4);
    }

    #[test]
    fn test_focus_camera_orthographic_preserves_orientation() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((Position::new(7.0, -4.0, 0.0),));

        let mut camera = make_camera(ProjectionMode::Orthographic);
        camera.yaw = Rad(-std::f32::consts::FRAC_PI_2);

        assert!(focus_camera_on_entity(&mut camera, &world, entity, 5.0));
        assert_eq!(camera.position.x, 7.0);
        assert_eq!(camera.position.y, -4.0);
        assert_eq!(camera.position.z, 10.0);
        assert_eq!(camera.target.x, 7.0);
        assert_eq!(camera.target.y, -4.0);
        assert_eq!(camera.yaw, Rad(-std::f32::consts::FRAC_PI_2));
    }

    #[test]
    fn test_ortho_zoom_clamps_and_mouse_look_rejects_nan() {
        let mut camera = make_camera(ProjectionMode::Orthographic);
        apply_scroll_zoom(&mut camera, 100.0, false, EditorCameraConfig::default());
        assert_eq!(camera.ortho_scale, 1.0);

        let before_yaw = camera.yaw;
        apply_mouse_look(&mut camera, f64::NAN, 10.0, 0.01);
        assert_eq!(camera.yaw, before_yaw);
    }
}