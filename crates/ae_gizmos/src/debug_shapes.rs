// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Wireframe collider shape generators and debug line vertex POD types.

pub mod convex_hull;
pub mod shapes;
pub mod vertex;

pub use convex_hull::{ConvexHullWireframe, build_convex_hull_wireframe};
pub use shapes::{DebugArcParams, DebugCircleParams, DebugShapes};
pub use vertex::DebugLineVertex;