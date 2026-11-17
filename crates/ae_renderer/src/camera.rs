// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub use ae_core::camera::{Camera, OrthographicProjection, PerspectiveProjection, ProjectionMode};
use ae_world::SpatialCellCoord;
/// AE Renderer — GPU-Side Camera Uniform Buffer Representation
use cgmath::SquareMatrix;

/// Floating-origin reference cell shifting multi-kilometer world coordinates into
/// origin-centered single-precision space prior to GPU matrix upload.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingOrigin {
    /// Spatial cell currently anchored at `(0, 0, 0)` in render space.
    pub cell: SpatialCellCoord,
    /// Size of each cubic cell in world units.
    pub cell_size: f32,
}

impl FloatingOrigin {
    /// Creates a floating-origin anchor from a cell coordinate and grid cell size.
    #[must_use]
    pub const fn new(cell: SpatialCellCoord, cell_size: f32) -> Self {
        Self { cell, cell_size }
    }

    /// Returns the `f64` world-space shift vector subtracted from absolute entity coordinates.
    #[must_use]
    pub fn world_shift_f64(self) -> [f64; 3] {
        self.cell.world_origin_f64(self.cell_size)
    }

    /// Converts a double-precision world coordinate into camera-origin-relative `f32` coordinates
    /// without single-precision jitter at multi-kilometer distances.
    #[must_use]
    pub fn world_f64_to_render_f32(self, world_pos: [f64; 3]) -> [f32; 3] {
        let shift = self.world_shift_f64();
        [
            (world_pos[0] - shift[0]) as f32,
            (world_pos[1] - shift[1]) as f32,
            (world_pos[2] - shift[2]) as f32,
        ]
    }
}

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

    /// Updates uniform fields relative to a [`FloatingOrigin`] using double-precision subtraction
    /// so multi-kilometer camera positions retain full single-precision GPU accuracy.
    pub fn update_view_proj_floating_origin(
        &mut self,
        camera: &Camera,
        camera_world_pos_f64: [f64; 3],
        origin: FloatingOrigin,
    ) {
        let local_cam_pos = origin.world_f64_to_render_f32(camera_world_pos_f64);
        let shift_target = origin.world_f64_to_render_f32([
            f64::from(camera.target.x),
            f64::from(camera.target.y),
            f64::from(camera.target.z),
        ]);
        let mut local_camera = camera.clone();
        local_camera.position =
            cgmath::Point3::new(local_cam_pos[0], local_cam_pos[1], local_cam_pos[2]);
        local_camera.target =
            cgmath::Point3::new(shift_target[0], shift_target[1], shift_target[2]);
        self.update_view_proj(&local_camera);
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

    #[test]
    fn test_floating_origin_eliminates_far_world_precision_loss() {
        let cell_size = 128.0_f32;
        let far_pos = [10_000_000.25_f64, 42.5_f64, -8_000_000.75_f64];
        let (cell, expected_local) = SpatialCellCoord::split_world_pos_f64(far_pos, cell_size);
        let origin = FloatingOrigin::new(cell, cell_size);

        let cam = Camera {
            position: cgmath::Point3::new(0.0, 0.0, 0.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
            aspect: 16.0 / 9.0,
            fovy: 60.0,
            znear: 0.1,
            zfar: 2000.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 15.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        };
        let mut uniform = CameraUniform::new();
        uniform.update_view_proj_floating_origin(&cam, far_pos, origin);
        assert!((uniform.camera_pos[0] - expected_local[0]).abs() < 1e-5);
        assert!((uniform.camera_pos[1] - expected_local[1]).abs() < 1e-5);
        assert!((uniform.camera_pos[2] - expected_local[2]).abs() < 1e-5);
    }

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