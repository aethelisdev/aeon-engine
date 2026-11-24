// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU-agnostic vertex representation for transform gizmo handles.

/// Vertex payload for transform gizmo line and mesh geometry (position + RGBA color + SDF UV).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct GizmoVertex {
    /// Local gizmo-space vertex coordinate.
    pub position: [f32; 3],
    /// Linear RGBA handle color.
    pub color: [f32; 4],
    /// Normalized signed-distance-field coordinates for analytical ring anti-aliasing.
    pub uv: [f32; 2],
}