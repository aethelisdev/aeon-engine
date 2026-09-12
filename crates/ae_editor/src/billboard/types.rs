// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport 3D Billboard Data Types
//!
//! Type definitions, GPU vertex layouts, and uniform representations
//! for 3D camera-facing editor billboard badges.

use bytemuck::{Pod, Zeroable};
use cgmath::Matrix4;

/// GPU vertex for camera-facing billboard quad rendering.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq)]
pub struct BillboardVertex {
    /// 3D world space coordinate of the entity anchor point.
    pub world_pos: [f32; 3],
    /// Normalized quad vertex offset `[-1.0, -1.0]` to `[1.0, 1.0]`.
    pub quad_offset: [f32; 2],
    /// Texture UV coordinates `[0.0, 0.0]` to `[1.0, 1.0]`.
    pub uv: [f32; 2],
    /// Texture array layer index (e.g. 8.0 for Light, 9.0 for Camera, 15.0 for Audio).
    pub layer: f32,
    /// Circular badge background color `[r, g, b, a]`.
    pub bg_color: [f32; 4],
    /// Circular badge outer border color `[r, g, b, a]`.
    pub border_color: [f32; 4],
    /// Center icon tint multiplier `[r, g, b, a]`.
    pub icon_tint: [f32; 4],
}

impl BillboardVertex {
    /// WGPU vertex attribute bindings for billboard pipeline layout.
    const ATTRIBS: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        0 => Float32x3, // world_pos
        1 => Float32x2, // quad_offset
        2 => Float32x2, // uv
        3 => Float32,   // layer
        4 => Float32x4, // bg_color
        5 => Float32x4, // border_color
        6 => Float32x4  // icon_tint
    ];

    /// Returns the vertex buffer memory layout descriptor.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BillboardVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Uniform representation for camera view-projection and screen sizing.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct BillboardUniform {
    /// Combined Camera View-Projection Matrix (Column-Major).
    pub view_proj: [[f32; 4]; 4],
    /// Logical viewport dimensions in pixels `[width, height]`.
    pub viewport_size: [f32; 2],
    /// Desired billboard badge screen diameter in pixels.
    pub icon_size_px: f32,
    /// 32-bit scalar alignment padding.
    pub _pad: f32,
}

impl Default for BillboardUniform {
    fn default() -> Self {
        Self {
            view_proj: Matrix4::from_scale(1.0).into(),
            viewport_size: [800.0, 600.0],
            icon_size_px: 28.0,
            _pad: 0.0,
        }
    }
}

/// Category of entity billboard icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillboardIconType {
    /// Light source entity (point light, directional light, spot light).
    Light,
    /// Audio emitter / playback component.
    AudioSource,
    /// Audio listener / microphone or camera reference.
    Camera,
}

impl BillboardIconType {
    /// Returns the canonical 2D texture array layer index in `editor_atlas.png`.
    pub fn layer_index(&self) -> f32 {
        match self {
            BillboardIconType::Light => 8.0,
            BillboardIconType::Camera => 9.0,
            BillboardIconType::AudioSource => 15.0,
        }
    }

    /// Returns default icon tint color `[r, g, b, a]`.
    pub fn default_tint(&self) -> [f32; 4] {
        match self {
            BillboardIconType::Light => [1.0, 0.88, 0.35, 1.0],
            BillboardIconType::Camera => [0.85, 0.88, 0.95, 1.0],
            BillboardIconType::AudioSource => [0.40, 0.75, 1.0, 1.0],
        }
    }
}