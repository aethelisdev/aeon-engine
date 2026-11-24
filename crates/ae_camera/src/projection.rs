// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Perspective and orthographic projection builders with WGPU `[0, 1]` NDC correction.

use super::transform::Camera;
use cgmath::*;

/// Coordinate system correction matrix converting OpenGL clip-space to WGPU clip-space.
///
/// OpenGL uses NDC Z range `[-1, 1]` while WGPU (Vulkan/DX12/Metal) uses `[0, 1]`.
/// This matrix remaps the Z coordinate accordingly. Applied as a prefix to all
/// projection matrices in the engine.
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

/// Determines which projection model the camera uses for 3D-to-2D mapping.
///
/// `Perspective` provides realistic depth foreshortening for 3D navigation.
/// `Orthographic` provides uniform scaling without depth distortion, used
/// for technical views (Front, Top, Right) and 2D-style editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionMode {
    /// Perspective view with field-of-view depth foreshortening.
    Perspective,
    /// Parallel orthographic view without depth distortion.
    Orthographic,
}

/// Stateless utility for building perspective projection matrices.
pub struct PerspectiveProjection;

impl PerspectiveProjection {
    /// Builds a perspective projection matrix with the given parameters.
    ///
    /// Applies the OpenGL-to-WGPU coordinate correction automatically.
    /// Clamps `aspect` to `>= 0.0001` to prevent panics when viewport aspect is zero.
    #[must_use]
    pub fn build_matrix(fovy: f32, aspect: f32, znear: f32, zfar: f32) -> Matrix4<f32> {
        let safe_aspect = aspect.max(0.0001);
        OPENGL_TO_WGPU_MATRIX * cgmath::perspective(Deg(fovy), safe_aspect, znear, zfar)
    }
}

/// Stateless utility for building orthographic projection matrices.
pub struct OrthographicProjection;

impl OrthographicProjection {
    /// Builds an orthographic projection matrix from scale, aspect ratio, and depth range.
    ///
    /// Clamps `aspect` and `ortho_scale` to `>= 0.0001` to avoid degenerate projections.
    #[must_use]
    pub fn build_matrix(ortho_scale: f32, aspect: f32, znear: f32, zfar: f32) -> Matrix4<f32> {
        let safe_aspect = aspect.max(0.0001);
        let safe_ortho_scale = ortho_scale.max(0.0001);
        let half_height = safe_ortho_scale * 0.5;
        let half_width = half_height * safe_aspect;
        OPENGL_TO_WGPU_MATRIX
            * cgmath::ortho(
                -half_width,
                half_width,
                -half_height,
                half_height,
                znear,
                zfar,
            )
    }
}

impl Camera {
    /// Builds the appropriate projection matrix based on the camera's current mode.
    #[must_use]
    pub fn build_projection_matrix(&self) -> Matrix4<f32> {
        match self.mode {
            ProjectionMode::Perspective => {
                PerspectiveProjection::build_matrix(self.fovy, self.aspect, self.znear, self.zfar)
            }
            ProjectionMode::Orthographic => OrthographicProjection::build_matrix(
                self.ortho_scale,
                self.aspect,
                -10000.0,
                10000.0,
            ),
        }
    }

    /// Builds a view-projection matrix with a capped far-plane, used for CPU frustum culling.
    ///
    /// Decouples visual render depth from CPU culling range to eliminate distant objects early.
    #[must_use]
    pub fn build_culling_matrix(&self) -> Matrix4<f32> {
        let culling_zfar = 400.0_f32.min(self.zfar);
        let proj = match self.mode {
            ProjectionMode::Perspective => PerspectiveProjection::build_matrix(
                self.fovy,
                self.aspect,
                self.znear,
                culling_zfar,
            ),
            ProjectionMode::Orthographic => OrthographicProjection::build_matrix(
                self.ortho_scale,
                self.aspect,
                -culling_zfar,
                culling_zfar,
            ),
        };
        proj * self.build_view_matrix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degenerate_aspect_and_ortho_scale_remain_finite() {
        let persp = PerspectiveProjection::build_matrix(60.0, 0.0, 0.1, 100.0);
        let ortho = OrthographicProjection::build_matrix(-5.0, -2.0, -10.0, 10.0);
        for col in [persp.x, persp.y, persp.z, persp.w] {
            assert!(
                col.x.is_finite() && col.y.is_finite() && col.z.is_finite() && col.w.is_finite()
            );
        }
        for col in [ortho.x, ortho.y, ortho.z, ortho.w] {
            assert!(
                col.x.is_finite() && col.y.is_finite() && col.z.is_finite() && col.w.is_finite()
            );
        }
    }

    #[test]
    fn test_orthographic_projection_center_depth_in_wgpu_range() {
        let proj = OrthographicProjection::build_matrix(10.0, 1.0, 0.0, 100.0);
        let mid_clip = proj * Vector4::new(0.0, 0.0, -50.0, 1.0);
        assert!(mid_clip.z >= 0.0 && mid_clip.z <= 1.0);
    }
}