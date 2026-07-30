// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub use ae_core::camera::{Camera, OrthographicProjection, PerspectiveProjection, ProjectionMode};
/// AE Renderer — GPU-Side Camera Uniform Buffer Representation
use cgmath::SquareMatrix;

/// Uniform structure holding the camera's matrices and position on the GPU side.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub view_inv: [[f32; 4]; 4],
    pub proj_inv: [[f32; 4]; 4],
    pub camera_pos: [f32; 4],
}

impl CameraUniform {
    /// Creates a CameraUniform initialized with identity matrices.
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            view_inv: cgmath::Matrix4::identity().into(),
            proj_inv: cgmath::Matrix4::identity().into(),
            camera_pos: [0.0; 4],
        }
    }

    /// Updates uniform fields based on the core camera state.
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

    /// Tests that CameraUniform gracefully handles zero aspect ratio without panic.
    #[test]
    fn test_camera_uniform_degenerate_matrix_fallback() {
        let camera = Camera {
            position: cgmath::Point3::new(0.0, 0.0, 5.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
            aspect: 0.0, // Degenerate aspect ratio when window is minimized
            fovy: 45.0,
            znear: 0.1,
            zfar: 2000.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 15.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        };

        let mut uniform = CameraUniform::new();
        // Should not panic, should construct valid inverse matrices for zero aspect ratio
        uniform.update_view_proj(&camera);

        for row in &uniform.view_inv {
            for val in row {
                assert!(!val.is_nan(), "Camera view_inv element is NaN!");
            }
        }
        for row in &uniform.proj_inv {
            for val in row {
                assert!(!val.is_nan(), "Camera proj_inv element is NaN!");
            }
        }
    }
}