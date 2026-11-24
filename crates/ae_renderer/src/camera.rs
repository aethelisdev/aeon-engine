// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Render-space floating-origin shifting helper for multi-kilometer worlds.

use ae_camera::{Camera, CameraUniform};
use ae_world::SpatialCellCoord;

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

    /// Updates a [`CameraUniform`] relative to this floating origin using double-precision subtraction.
    pub fn update_camera_uniform(
        self,
        uniform: &mut CameraUniform,
        camera: &Camera,
        camera_world_pos_f64: [f64; 3],
    ) {
        let local_cam_pos = self.world_f64_to_render_f32(camera_world_pos_f64);
        let shift_target = self.world_f64_to_render_f32([
            f64::from(camera.target.x),
            f64::from(camera.target.y),
            f64::from(camera.target.z),
        ]);
        let mut local_camera = camera.clone();
        local_camera.position =
            cgmath::Point3::new(local_cam_pos[0], local_cam_pos[1], local_cam_pos[2]);
        local_camera.target =
            cgmath::Point3::new(shift_target[0], shift_target[1], shift_target[2]);
        uniform.update_view_proj(&local_camera);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_camera::ProjectionMode;

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
        origin.update_camera_uniform(&mut uniform, &cam, far_pos);
        assert!((uniform.camera_pos[0] - expected_local[0]).abs() < 1e-5);
        assert!((uniform.camera_pos[1] - expected_local[1]).abs() < 1e-5);
        assert!((uniform.camera_pos[2] - expected_local[2]).abs() < 1e-5);
    }
}