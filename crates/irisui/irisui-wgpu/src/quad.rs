// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU instance representations and buffer layouts for SDF Quads.

use bytemuck::{Pod, Zeroable};
use irisui_core::{Color, Point, Rect, Style};

/// Per-instance vertex buffer data uploaded to the GPU for each SDF Quad or Line Segment.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Pod, Zeroable)]
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
    /// Endpoints `[start_x, start_y, end_x, end_y]` in screen pixels for analytical capsule lines.
    pub line_endpoints: [f32; 4],
    /// Shape mode & auxiliary parameters `[mode (0=box, 1=line), half_thickness, 0.0, 0.0]`.
    pub shape_params: [f32; 4],
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
            line_endpoints: [0.0; 4],
            shape_params: [0.0; 4],
        }
    }

    /// Constructs a hardware anti-aliased analytical capsule line segment between `start` and `end`.
    ///
    /// Computes an exact covering axis-aligned bounding box expanded by `thickness * 0.5` and encodes
    /// screen-space endpoints for arbitrary-angle signed distance evaluation in `sdf_quad.wgsl`.
    pub fn line(
        start: Point,
        end: Point,
        color: Color,
        thickness: f32,
        clip_rect: Option<Rect>,
    ) -> Self {
        let clamped_thickness = thickness.max(0.5);
        let half_thick = clamped_thickness * 0.5;
        let min_x = start.x.min(end.x) - half_thick;
        let min_y = start.y.min(end.y) - half_thick;
        let max_x = start.x.max(end.x) + half_thick;
        let max_y = start.y.max(end.y) + half_thick;
        let clip = clip_rect.unwrap_or(Rect::ZERO);

        Self {
            rect: [min_x, min_y, max_x - min_x, max_y - min_y],
            color: color.to_linear().to_array(),
            border_color: [0.0; 4],
            border_width: [0.0; 4],
            corner_radii: [0.0; 4],
            shadow_color: [0.0; 4],
            shadow_params: [0.0; 4],
            clip_rect: [clip.x, clip.y, clip.right(), clip.bottom()],
            line_endpoints: [start.x, start.y, end.x, end.y],
            shape_params: [1.0, half_thick, 0.0, 0.0],
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
            wgpu::VertexAttribute {
                offset: (std::mem::size_of::<[f32; 4]>() * 8) as wgpu::BufferAddress,
                shader_location: 9,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: (std::mem::size_of::<[f32; 4]>() * 9) as wgpu::BufferAddress,
                shader_location: 10,
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