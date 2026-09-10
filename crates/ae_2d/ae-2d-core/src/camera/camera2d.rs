// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use ae_core::camera::OrthographicProjection;
use cgmath::{Deg, Matrix4, Vector3};
use glam::Vec2;
use hecs::Entity;
use serde::{Deserialize, Serialize};

/// 2D Camera component providing 2D world-to-screen projection, zoom, and smooth target tracking.
/// Wraps `ae_core::camera::OrthographicProjection` directly, ensuring zero duplication of projection
/// matrix math while maintaining standard 2D camera controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera2D {
    /// Vertical world units visible on screen at zoom = 1.0.
    /// For pixel-art or tile games, this might be 10.0 or 14.0 world units.
    pub ortho_size: f32,

    /// Magnification factor. Greater than 1.0 zooms in, less than 1.0 zooms out.
    /// Clamped to positive non-zero values (`>= 0.0001`).
    pub zoom: f32,

    /// Near clipping plane distance for the orthographic frustum.
    pub znear: f32,

    /// Far clipping plane distance for the orthographic frustum.
    pub zfar: f32,

    /// Optional target entity for the camera to follow smoothly in world space.
    #[serde(skip)]
    pub target: Option<Entity>,

    /// Smoothing lerp factor used when tracking a target entity (`0.0` = instant, `> 0.0` = smooth lag).
    pub smooth_speed: f32,

    /// Deadzone half-extents `[x, y]` around the camera center where target movement does not trigger camera motion.
    pub deadzone: [f32; 2],

    /// Z-axis rotation angle in degrees for screen roll/tilt effects.
    pub rotation_degrees: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            ortho_size: 10.0,
            zoom: 1.0,
            znear: -1000.0,
            zfar: 1000.0,
            target: None,
            smooth_speed: 10.0,
            deadzone: [0.0, 0.0],
            rotation_degrees: 0.0,
        }
    }
}

impl Camera2D {
    /// Constructs a 2D camera with an explicit vertical orthographic world size.
    pub fn new(ortho_size: f32) -> Self {
        Self {
            ortho_size: ortho_size.max(0.0001),
            ..Default::default()
        }
    }

    /// Sets the camera zoom level. Clamped to safe minimum `0.0001`.
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom.max(0.0001);
        self
    }

    /// Sets the target entity for continuous following.
    pub fn with_target(mut self, target: Entity, smooth_speed: f32) -> Self {
        self.target = Some(target);
        self.smooth_speed = smooth_speed.max(0.0);
        self
    }

    /// Calculates the view-projection matrix combining `ae_core::camera::OrthographicProjection`
    /// with the camera's 2D world position, roll angle, and zoom.
    pub fn build_view_projection(&self, position: Vec2, aspect: f32) -> Matrix4<f32> {
        let safe_zoom = self.zoom.max(0.0001);
        let effective_ortho_scale = self.ortho_size / safe_zoom;

        // Directly reuse ae_core::camera::OrthographicProjection
        let proj = OrthographicProjection::build_matrix(
            effective_ortho_scale,
            aspect,
            self.znear,
            self.zfar,
        );

        // Compute 2D view transformation: Translate to -position, rotate around Z
        let translation = Matrix4::from_translation(Vector3::new(-position.x, -position.y, 0.0));
        let rotation = Matrix4::from_angle_z(Deg(self.rotation_degrees));
        let view = rotation * translation;

        proj * view
    }

    /// Updates the camera position to follow a target position, accounting for deadzone and smoothing.
    pub fn update_follow(&self, current_pos: Vec2, target_pos: Vec2, delta_time: f32) -> Vec2 {
        let delta = target_pos - current_pos;
        let deadzone = Vec2::new(self.deadzone[0], self.deadzone[1]);

        let mut offset = Vec2::ZERO;
        if delta.x.abs() > deadzone.x {
            offset.x = delta.x - delta.x.signum() * deadzone.x;
        }
        if delta.y.abs() > deadzone.y {
            offset.y = delta.y - delta.y.signum() * deadzone.y;
        }

        if self.smooth_speed <= 0.0 {
            current_pos + offset
        } else {
            let t = (self.smooth_speed * delta_time).clamp(0.0, 1.0);
            current_pos + offset * t
        }
    }
}