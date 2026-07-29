// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use cgmath::*;

/// Coordinate system correction matrix converting OpenGL clip-space to WGPU clip-space.
/// OpenGL uses NDC Z range [-1, 1] while WGPU (Vulkan/DX12/Metal) uses [0, 1].
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
/// `Perspective` provides realistic depth foreshortening for 3D navigation.
/// `Orthographic` provides uniform scaling without depth distortion, used
/// for technical views (Front, Top, Right) and 2D-style editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionMode {
    Perspective,
    Orthographic,
}

/// Stateless utility for building perspective projection matrices.
/// Encapsulates the perspective projection math: field-of-view based frustum
/// with depth foreshortening. All methods are pure functions operating on
/// parameters directly — no internal state is stored.
pub struct PerspectiveProjection;

impl PerspectiveProjection {
    /// Builds a perspective projection matrix with the given parameters.
    /// Applies the OpenGL-to-WGPU coordinate correction automatically.
    /// `fovy` is in degrees, `aspect` is width/height ratio.
    pub fn build_matrix(fovy: f32, aspect: f32, znear: f32, zfar: f32) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * cgmath::perspective(Deg(fovy), aspect, znear, zfar)
    }
}

/// Stateless utility for building orthographic projection matrices.
/// Encapsulates orthographic projection math: uniform-scale parallel projection
/// without depth foreshortening. Used for technical/engineering views where
/// accurate relative sizing is more important than depth perception.
pub struct OrthographicProjection;

impl OrthographicProjection {
    /// Builds an orthographic projection matrix from scale, aspect ratio, and depth range.
    /// The `ortho_scale` controls the visible world-space height. Width is
    /// derived as `ortho_scale * aspect`. Near/far define the depth clipping range.
    pub fn build_matrix(ortho_scale: f32, aspect: f32, znear: f32, zfar: f32) -> Matrix4<f32> {
        let half_height = ortho_scale * 0.5;
        let half_width = half_height * aspect;
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

/// Projection-related methods for the Camera struct.
/// These methods delegate to `PerspectiveProjection` and `OrthographicProjection`
/// based on the current `ProjectionMode`, keeping the projection math isolated
/// in dedicated stateless types while preserving the Camera's unified API.
impl super::Camera {
    /// Builds the appropriate projection matrix based on the camera's current mode.
    /// Delegates to `PerspectiveProjection::build_matrix()` or
    /// `OrthographicProjection::build_matrix()` depending on `self.mode`.
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

    /// Builds a view-projection matrix with a shorter far-plane, used ONLY for frustum culling.
    /// This decouples visual render depth (zfar=2000) from CPU culling range,
    /// restoring the original frustum culling performance by aggressively eliminating
    /// distant objects from the CPU instance list before they ever reach the GPU.
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