// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Aeon Engine — Gizmo Subsystem
//!
//! Native 3D Transform Gizmo system, modularized by responsibility.
//!

pub mod core;
pub mod geometry;
pub mod input;
pub mod math;
pub mod picking;
pub mod render;
pub mod rotate;
pub mod rotation_geometry;
pub mod space;
pub mod translate;

// Re-export public API for backwards compatibility
pub use self::core::*;
pub use self::input::GizmoInputParams;
pub use self::space::*;