// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! POD vertex representation for wireframe debug lines.

/// Vertex for debug line rendering (position + RGB color).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct DebugLineVertex {
    /// 3D world-space line endpoint position.
    pub position: [f32; 3],
    /// Linear RGB line color.
    pub color: [f32; 3],
}