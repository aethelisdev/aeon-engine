// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Core 3D camera transform state and orthonormal basis vector helpers.

use super::projection::ProjectionMode;
use cgmath::*;

/// Core camera state managing position, orientation, and derived direction vectors.
///
/// Holds both spatial transform data (`position`, `yaw`, `pitch`, `target`) and projection
/// parameters (`aspect`, `fovy`, `znear`, `zfar`, `ortho_scale`). Serves as the shared camera
/// representation across simulation, rendering, picking, and editor viewports.
#[derive(Clone, Debug)]
pub struct Camera {
    /// World-space eye position.
    pub position: Point3<f32>,
    /// Horizontal heading angle in radians.
    pub yaw: Rad<f32>,
    /// Vertical elevation angle in radians.
    pub pitch: Rad<f32>,
    /// Viewport width / height aspect ratio.
    pub aspect: f32,
    /// Perspective vertical field of view in degrees.
    pub fovy: f32,
    /// Near clipping plane distance.
    pub znear: f32,
    /// Far clipping plane distance.
    pub zfar: f32,
    /// Active projection mode (`Perspective` vs `Orthographic`).
    pub mode: ProjectionMode,
    /// Orthographic vertical world span at unit zoom.
    pub ortho_scale: f32,
    /// Look-at orbit/pan pivot target in world space.
    pub target: Point3<f32>,
}

impl Camera {
    /// Constructs the right-handed view matrix from position and yaw/pitch orientation.
    #[must_use]
    pub fn build_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(self.position, self.get_forward(), Vector3::unit_y())
    }

    /// Returns the camera position as a `cgmath::Vector3<f32>`.
    #[must_use]
    pub fn position_vec3(&self) -> Vector3<f32> {
        Vector3::new(self.position.x, self.position.y, self.position.z)
    }

    /// Combines projection and view matrices into a single view-projection matrix.
    #[must_use]
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        self.build_projection_matrix() * self.build_view_matrix()
    }

    /// Computes the normalized forward direction vector from yaw and pitch angles.
    #[must_use]
    pub fn get_forward(&self) -> Vector3<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
        Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize()
    }

    /// Computes the horizontal right vector orthogonal to forward and world up.
    #[must_use]
    pub fn get_right(&self) -> Vector3<f32> {
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
        Vector3::new(-sin_yaw, 0.0, cos_yaw).normalize()
    }

    /// Computes the camera's true orthonormal up vector (`right × forward`).
    #[must_use]
    pub fn get_up(&self) -> Vector3<f32> {
        self.get_right().cross(self.get_forward()).normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_basis_vectors_are_orthonormal() {
        let cam = Camera {
            position: Point3::new(1.0, 2.0, 3.0),
            yaw: Rad(0.45),
            pitch: Rad(-0.35),
            aspect: 16.0 / 9.0,
            fovy: 60.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 10.0,
            target: Point3::new(0.0, 0.0, 0.0),
        };
        let f = cam.get_forward();
        let r = cam.get_right();
        let u = cam.get_up();

        assert!((f.magnitude() - 1.0).abs() < 1e-5);
        assert!((r.magnitude() - 1.0).abs() < 1e-5);
        assert!((u.magnitude() - 1.0).abs() < 1e-5);
        assert!(f.dot(r).abs() < 1e-5);
        assert!(f.dot(u).abs() < 1e-5);
        assert!(r.dot(u).abs() < 1e-5);
    }
}