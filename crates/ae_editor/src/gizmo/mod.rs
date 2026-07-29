// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Aeon Engine — Gizmo Module
/// Native 3D Transform Gizmo system, modularized by responsibility.

/// Core types, state, and coordinator struct (`GizmoSystem`).
pub mod core;
/// Pure procedural geometry algorithms for 3D translation, rotation, and scale gizmo handles.
pub mod geometry;
/// Drag lifecycle management (start, calculate, end).
pub mod input;
/// Pure math helpers — ray/segment/plane intersection.
pub mod math;
/// Raycast intersection against gizmo axes and plane handles.
pub mod picking;
/// GPU mesh generation, pipeline setup, and draw calls for gizmo rendering.
pub mod render;
/// Euler angle delta computation from drag vectors.
pub mod rotate;
/// Coordinate space enum (`GizmoSpace`) — World vs Local axis orientation.
pub mod space;
/// Axis-constrained translation delta computation.
pub mod translate;

// Re-export public API for backwards compatibility
pub use self::core::*;
pub use self::space::*;