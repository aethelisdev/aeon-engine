// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Aeon Camera Crate
//!
//! GPU-independent camera representation, perspective/orthographic projections,
//! view frustum culling planes, logical viewport rectangles, and uniform payloads.

pub mod frustum;
pub mod projection;
pub mod transform;
pub mod uniform;
pub mod viewport;

pub use frustum::Frustum;
pub use projection::{
    OPENGL_TO_WGPU_MATRIX, OrthographicProjection, PerspectiveProjection, ProjectionMode,
};
pub use transform::Camera;
pub use uniform::CameraUniform;
pub use viewport::{Viewport, ViewportRect};