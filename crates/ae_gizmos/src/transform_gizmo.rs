// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! CPU transform gizmo coordinator, interaction math, picking, and procedural
//! handle geometry builders.

pub mod core;
pub mod geometry;
pub mod input;
pub mod math;
pub mod picking;
pub mod rotate;
pub mod rotation_geometry;
pub mod space;
pub mod translate;
pub mod vertex;

pub use core::{ActiveAxis, GizmoMode, GizmoScreenParams, TransformGizmo};
pub use input::GizmoInputParams;
pub use rotation_geometry::TorusRingDescriptor;
pub use space::GizmoSpace;
pub use vertex::GizmoVertex;