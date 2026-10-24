// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Vertex & Instance GPU Data Layouts
//!
//! Defines memory representations and WGPU vertex buffer layouts for 3D meshes, 2D billboards, and GPU instances.

/// PBR mesh vertex: position, vertex color, surface normal, and UV coordinates.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
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
}

/// Sprite/billboard vertex: position, UV coordinates, and surface normal.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

impl SpriteVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
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
}

/// Vertex skinning bind information (bind position, normal, joint indices and weights).
#[derive(Debug, Clone, Copy)]
pub struct SkinVertex {
    pub bind_position: [f32; 3],
    pub bind_normal: [f32; 3],
    pub joint_indices: [u32; 4],
    pub joint_weights: [f32; 4],
}

/// Per-instance data: 4×4 model matrix + RGBA color (80 bytes per instance).
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub model_matrix: [[f32; 4]; 4],
    pub color: [f32; 4],
}

/// Byte stride of a single `Instance` in GPU buffer. Used for buffer offset calculations.
/// Replaces hardcoded magic numbers throughout the render pipeline.
pub const INSTANCE_SIZE: usize = std::mem::size_of::<Instance>();

impl Instance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
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
    /// Determines whether the model transformation matrix has negative scale (reflection / mirrored winding).
    ///
    /// Computes the determinant of the upper-left 3x3 rotation/scale submatrix.
    #[inline]
    pub fn is_mirrored(&self) -> bool {
        let m = &self.model_matrix;
        let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
        det < 0.0
    }
}

/// Pre-evaluated render instance item pairing GPU instance data with texture bindings and reflection flags.
///
/// Precomputing matrix reflection and texture handles decouples frame-time render loops
/// from CPU determinant calculations and nested pattern-matching logic.
#[derive(Copy, Clone, Debug)]
pub struct RenderInstanceItem {
    /// Raw GPU instance data containing model matrix and tint color.
    pub instance: Instance,
    /// World position vector extracted for sorting, spatial queries, and cache-friendly distance calculations.
    pub world_position: [f32; 3],
    /// Optional texture asset handle override for diffuse mapping.
    pub texture: Option<crate::asset::AssetHandle>,
    /// Precomputed indicator for negative scale reflection requiring clockwise vertex winding.
    pub is_mirrored: bool,
}

impl RenderInstanceItem {
    /// Constructs a new `RenderInstanceItem`, precomputing reflection parity and extracting translation from the instance matrix.
    #[inline]
    pub fn new(instance: Instance, texture: Option<crate::asset::AssetHandle>) -> Self {
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
        let inst_x = make_test_instance([
            [-1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        assert!(inst_x.is_mirrored());

        let inst_y = make_test_instance([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, -2.5, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        assert!(inst_y.is_mirrored());

        let inst_z = make_test_instance([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, -0.5, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        assert!(inst_z.is_mirrored());
    }

    #[test]
    fn test_two_axes_negative_scale_is_not_mirrored() {
        // (-1) * (-1) * 1 = +1 (preserves winding order)
        let inst = make_test_instance([
            [-1.0, 0.0, 0.0, 0.0],
            [0.0, -1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        assert!(!inst.is_mirrored());
    }

    #[test]
    fn test_three_axes_negative_scale_is_mirrored() {
        // (-1) * (-1) * (-1) = -1 (inverts winding order)
        let inst = make_test_instance([
            [-1.0, 0.0, 0.0, 0.0],
            [0.0, -1.0, 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        assert!(inst.is_mirrored());
    }

    #[test]
    fn test_zero_scale_singularity_matrix() {
        let inst = make_test_instance([
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        assert!(!inst.is_mirrored());
    }

    #[test]
    fn test_render_instance_item_new_caches_reflection() {
        let inst = make_test_instance([
            [-2.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let item = RenderInstanceItem::new(inst, None);
        assert!(item.is_mirrored);
        assert_eq!(item.texture, None);
        assert_eq!(item.world_position, [0.0, 0.0, 0.0]);
    }
}