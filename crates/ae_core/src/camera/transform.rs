// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::projection::ProjectionMode;
use cgmath::*;

/// Core camera state managing position, orientation, and derived direction vectors.
/// Holds both spatial transform data (position, yaw, pitch, target) and projection
/// parameters (aspect, fovy, znear, zfar, ortho_scale). The Camera struct serves as
/// the unified state container referenced by the engine, renderer, UI, and editor systems.
/// Projection matrix construction is delegated to `PerspectiveProjection` and
/// `OrthographicProjection` structs via an `impl Camera` block in `projection.rs`.
#[derive(Clone, Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    // View Modes
    pub mode: ProjectionMode,
    pub ortho_scale: f32,

    /// The camera's look-at target point. Used by orthographic view buttons
    /// (Front/Right/Top) and updated during panning to keep the scene centered.
    pub target: Point3<f32>,
}

impl Camera {
    /// Constructs the view matrix from the camera's position and orientation.
    /// Uses `get_forward()` internally to derive the look direction from yaw/pitch,
    /// then builds a right-handed look-to matrix with Y-up convention.
    /// This eliminates the duplicate trigonometric calculation that previously
    /// existed between this method and `get_forward()`.
    pub fn build_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(self.position, self.get_forward(), Vector3::unit_y())
    }

    /// Returns the camera position as a `cgmath::Vector3<f32>` for vector arithmetic.
    pub fn position_vec3(&self) -> Vector3<f32> {
        Vector3::new(self.position.x, self.position.y, self.position.z)
    }

    /// Combines projection and view matrices into a single view-projection matrix.
    /// Used by the render pipeline for standard scene rendering. The multiplication
    /// order is `projection * view` following the right-handed convention.
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        self.build_projection_matrix() * self.build_view_matrix()
    }

    /// Computes the camera's normalized forward direction vector from yaw and pitch angles.
    /// This is the **single source of truth** for forward direction computation.
    /// Both `build_view_matrix()` and external systems (movement, raycasting)
    /// rely on this method to avoid duplicating the trigonometric calculation.
    pub fn get_forward(&self) -> Vector3<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
        Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize()
    }

    /// Computes the camera's right direction vector (orthogonal to forward and world up).
    /// Derived purely from yaw angle since the right vector lies in the XZ plane.
    /// Used for strafing movement and as a basis for computing the true up vector.
    pub fn get_right(&self) -> Vector3<f32> {
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
        Vector3::new(-sin_yaw, 0.0, cos_yaw).normalize()
    }

    /// Computes the camera's true up direction vector via cross product.
    /// Calculated as `right × forward` to produce an orthonormal basis.
    /// Unlike world-up (0,1,0), this vector accounts for the camera's pitch.
    pub fn get_up(&self) -> Vector3<f32> {
        self.get_right().cross(self.get_forward()).normalize()
    }
}