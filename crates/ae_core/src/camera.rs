// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! AE Core - Camera Subsystem
//!
//! Provides the primary camera abstraction, projection models (perspective and
//! orthographic), coordinate remapping matrices for WGPU, and view-projection
//! transformation calculation pipelines.
//!

pub mod projection;
pub mod transform;

pub use projection::{
    OPENGL_TO_WGPU_MATRIX, OrthographicProjection, PerspectiveProjection, ProjectionMode,
};
pub use transform::Camera;