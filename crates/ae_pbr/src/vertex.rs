// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU vertex buffer layouts, 80-byte per-instance matrix/tint payloads, and depth format constants.

use ae_mesh::{SpriteVertex, Vertex};
use ae_plugin_api::AssetHandle;

/// Default depth-stencil format across 3D PBR, shadow, sky, grid, and sprite pipelines.
pub const PBR_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Returns the static WGPU vertex buffer layout for [`ae_mesh::Vertex`].
#[must_use]
pub const fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                shader_location: 8,
                format: wgpu::VertexFormat::Float32x2,
            },
        ],
    }
}

/// Returns the static WGPU vertex buffer layout for [`ae_mesh::SpriteVertex`].
#[must_use]
pub const fn sprite_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x3,
            },
        ],
    }
}

/// Per-instance GPU payload: 4x4 model matrix + RGBA color (80 bytes per instance).
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    /// Column-major 4x4 world transform matrix.
    pub model_matrix: [[f32; 4]; 4],
    /// Linear RGBA tint or selection mask metadata.
    pub color: [f32; 4],
}

/// Byte stride of a single [`Instance`] in GPU instance buffers.
pub const INSTANCE_SIZE: usize = std::mem::size_of::<Instance>();

impl Instance {
    /// Returns the static WGPU vertex buffer layout for [`Instance`].
    #[must_use]
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }

    /// Returns `true` when the upper-left 3x3 transform determinant is negative (mirrored winding).
    #[inline]
    #[must_use]
    pub fn is_mirrored(&self) -> bool {
        let m = &self.model_matrix;
        let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
        det < 0.0
    }
}

/// Pre-evaluated render instance pairing GPU matrix/color with cached translation and reflection parity.
#[derive(Copy, Clone, Debug)]
pub struct RenderInstanceItem {
    /// Raw GPU instance data.
    pub instance: Instance,
    /// Cached world translation `[x, y, z]`.
    pub world_position: [f32; 3],
    /// Optional diffuse texture override handle.
    pub texture: Option<AssetHandle>,
    /// Precomputed negative-scale reflection indicator selecting CW vs CCW pipelines.
    pub is_mirrored: bool,
}

impl RenderInstanceItem {
    /// Constructs a [`RenderInstanceItem`], caching world translation and reflection parity.
    #[inline]
    #[must_use]
    pub fn new(instance: Instance, texture: Option<AssetHandle>) -> Self {
        let is_mirrored = instance.is_mirrored();
        let world_position = [
            instance.model_matrix[3][0],
            instance.model_matrix[3][1],
            instance.model_matrix[3][2],
        ];
        Self {
            instance,
            world_position,
            texture,
            is_mirrored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_instance(m: [[f32; 4]; 4]) -> Instance {
        Instance {
            model_matrix: m,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    #[test]
    fn test_identity_matrix_is_not_mirrored() {
        let inst = make_test_instance([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        assert!(!inst.is_mirrored());
    }

    #[test]
    fn test_single_axis_negative_scale_is_mirrored() {
        for diag in [[-1.0, 1.0, 1.0], [1.0, -2.5, 1.0], [1.0, 1.0, -0.5]] {
            let inst = make_test_instance([
                [diag[0], 0.0, 0.0, 0.0],
                [0.0, diag[1], 0.0, 0.0],
                [0.0, 0.0, diag[2], 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]);
            assert!(inst.is_mirrored());
        }
    }

    #[test]
    fn test_two_and_three_axis_negative_scales_and_zero_singularity() {
        let even_mirror = make_test_instance([
            [-1.0, 0.0, 0.0, 0.0],
            [0.0, -1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        assert!(!even_mirror.is_mirrored());

        let odd_mirror = make_test_instance([
            [-1.0, 0.0, 0.0, 0.0],
            [0.0, -1.0, 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        assert!(odd_mirror.is_mirrored());

        let singular = make_test_instance([[0.0; 4], [0.0; 4], [0.0; 4], [0.0, 0.0, 0.0, 1.0]]);
        assert!(!singular.is_mirrored());
    }

    #[test]
    fn test_render_instance_item_caches_reflection_and_position() {
        let inst = make_test_instance([
            [-2.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [3.0, 4.0, 5.0, 1.0],
        ]);
        let item = RenderInstanceItem::new(inst, None);
        assert!(item.is_mirrored);
        assert_eq!(item.texture, None);
        assert_eq!(item.world_position, [3.0, 4.0, 5.0]);
    }
}