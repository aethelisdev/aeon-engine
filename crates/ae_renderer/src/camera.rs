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
            .expect("View matrix must be invertible — degenerate camera state detected")
            .into();
        self.proj_inv = proj
            .invert()
            .expect("Projection matrix must be invertible — degenerate camera state detected")
            .into();
        self.camera_pos = [camera.position.x, camera.position.y, camera.position.z, 1.0];
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}