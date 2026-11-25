// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Aeon Gizmos Crate
//!
//! GPU-independent transform gizmo state and geometry generation, collider
//! wireframe shape builders, viewport navigation compass projection, and
//! 3D entity billboard vertex/uniform descriptors.

pub mod billboard;
pub mod debug_shapes;
pub mod scene_view;
pub mod transform_gizmo;

pub use billboard::{BillboardIconType, BillboardUniform, BillboardVertex};
pub use debug_shapes::{
    ConvexHullWireframe, DebugArcParams, DebugCircleParams, DebugLineVertex, DebugShapes,
    build_convex_hull_wireframe,
};
pub use scene_view::{SceneNavigationGizmo, SceneViewSnap};
pub use transform_gizmo::{
    ActiveAxis, GizmoInputParams, GizmoMode, GizmoScreenParams, GizmoSpace, GizmoVertex,
    TorusRingDescriptor, TransformGizmo,
};