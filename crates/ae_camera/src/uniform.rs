// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU-ready camera uniform buffer payload (`Pod` / `Zeroable`) free of backend state.

use super::transform::Camera;
use cgmath::SquareMatrix;

/// Uniform structure holding view-projection matrices, inverse matrices, and eye position.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    /// Combined camera view-projection matrix (`proj * view`).
    pub view_proj: [[f32; 4]; 4],
    /// Inverse view matrix for world-space ray reconstruction.
    pub view_inv: [[f32; 4]; 4],
    /// Inverse projection matrix.
    pub proj_inv: [[f32; 4]; 4],
    /// Camera eye position in world/render space (`[x, y, z, 1.0]`).
    pub camera_pos: [f32; 4],
}

impl CameraUniform {
    /// Creates a `CameraUniform` initialized with identity matrices.
    #[must_use]
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            view_inv: cgmath::Matrix4::identity().into(),
            proj_inv: cgmath::Matrix4::identity().into(),
            camera_pos: [0.0; 4],
        }
    }

    /// Updates uniform fields from the given [`Camera`].
    ///
    /// Falls back to identity inverses if either matrix is singular.
    pub fn update_view_proj(&mut self, camera: &Camera) {
        let view = camera.build_view_matrix();
        let proj = camera.build_projection_matrix();
        self.view_proj = (proj * view).into();
        self.view_inv = view
            .invert()
            .unwrap_or_else(cgmath::Matrix4::identity)
            .into();
        self.proj_inv = proj
            .invert()
            .unwrap_or_else(cgmath::Matrix4::identity)
            .into();
        self.camera_pos = [camera.position.x, camera.position.y, camera.position.z, 1.0];
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProjectionMode;

    #[test]
    fn test_camera_uniform_alignment_and_zero_aspect_fallback() {
        assert_eq!(std::mem::size_of::<CameraUniform>(), 208);
        assert_eq!(std::mem::size_of::<CameraUniform>() % 16, 0);

        let camera = Camera {
            position: cgmath::Point3::new(0.0, 0.0, 5.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
            aspect: 0.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 2000.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 15.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        };

        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(&camera);
        for row in uniform
            .view_inv
            .iter()
            .chain(uniform.proj_inv.iter())
            .chain(uniform.view_proj.iter())
        {
            for val in row {
                assert!(val.is_finite());
            }
        }
    }
}