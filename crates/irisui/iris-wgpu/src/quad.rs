// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU instance representations and buffer layouts for SDF Quads.

use bytemuck::{Pod, Zeroable};
use iris_core::{Rect, Style};

/// Per-instance vertex buffer data uploaded to the GPU for each SDF Quad.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct QuadInstance {
    /// Screen-space bounding rectangle `[x, y, width, height]`.
    pub rect: [f32; 4],
    /// Background RGBA color `[r, g, b, a]`.
    pub color: [f32; 4],
    /// Border stroke RGBA color `[r, g, b, a]`.
    pub border_color: [f32; 4],
    /// Border thickness insets `[top, right, bottom, left]`.
    pub border_width: [f32; 4],
    /// Corner radii `[top_left, top_right, bottom_right, bottom_left]`.
    pub corner_radii: [f32; 4],
    /// Drop shadow RGBA color `[r, g, b, a]`.
    pub shadow_color: [f32; 4],
    /// Drop shadow parameters `[offset_x, offset_y, blur, spread]`.
    pub shadow_params: [f32; 4],
    /// Scissor clipping rectangle `[min_x, min_y, max_x, max_y]`.
    pub clip_rect: [f32; 4],
}

impl QuadInstance {
    /// Constructs a `QuadInstance` from an absolute layout rectangle, style, and optional clip rect.
    pub fn from_style(rect: Rect, style: &Style, clip_rect: Option<Rect>) -> Self {
        let shadow = style.box_shadow.unwrap_or_default();
        let clip = clip_rect.unwrap_or(Rect::ZERO);

        Self {
            rect: rect.to_array(),
            color: (style
                .background_color
                .with_alpha(style.background_color.a * style.opacity))
            .to_linear()
            .to_array(),
            border_color: style.border.color.to_linear().to_array(),
            border_width: style.border.width.to_array(),
            corner_radii: style.corner_radii.to_array(),
            shadow_color: shadow.color.to_linear().to_array(),
            shadow_params: [shadow.offset.x, shadow.offset.y, shadow.blur, shadow.spread],
            clip_rect: [clip.x, clip.y, clip.right(), clip.bottom()],
        }
    }

    /// Returns the WGPU `VertexBufferLayout` for instanced quad rendering.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] = &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: (std::mem::size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: (std::mem::size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: (std::mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: (std::mem::size_of::<[f32; 4]>() * 5) as wgpu::BufferAddress,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: (std::mem::size_of::<[f32; 4]>() * 6) as wgpu::BufferAddress,
                shader_location: 7,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: (std::mem::size_of::<[f32; 4]>() * 7) as wgpu::BufferAddress,
                shader_location: 8,
                format: wgpu::VertexFormat::Float32x4,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRIBUTES,
        }
    }
}