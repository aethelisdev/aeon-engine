// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Submesh slicing and PBR alpha-blending metadata for 3D meshes.

/// Submesh alpha blending and testing mode following the glTF 2.0 specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SubmeshAlphaMode {
    /// Fully opaque surface with depth writes enabled and no alpha discard.
    #[default]
    Opaque,
    /// Alpha-tested cutout surface discarding fragments below `alpha_cutoff`.
    Mask,
    /// Back-to-front alpha-blended translucent surface with depth writes disabled.
    Blend,
}

/// Contiguous index slice of a mesh paired with surface material and texture parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelSubmesh {
    /// Starting element offset within the parent mesh index buffer.
    pub start_index: u32,
    /// Number of indices drawn for this submesh slice.
    pub index_count: u32,
    /// Optional index into the parent model's embedded texture list.
    pub texture_index: Option<usize>,
    /// Linear RGBA base color factor multiplied with vertex color and albedo texture.
    pub base_color: [f32; 4],
    /// Surface alpha compositing mode.
    pub alpha_mode: SubmeshAlphaMode,
    /// Threshold below which fragments are discarded when `alpha_mode` is [`SubmeshAlphaMode::Mask`].
    pub alpha_cutoff: f32,
    /// Uniform metallic factor in `[0.0, 1.0]`.
    pub metallic_factor: f32,
    /// Uniform roughness factor in `[0.0, 1.0]`.
    pub roughness_factor: f32,
    /// UV tiling multiplier `[u_scale, v_scale]`.
    pub uv_scale: [f32; 2],
}

impl Default for ModelSubmesh {
    fn default() -> Self {
        Self {
            start_index: 0,
            index_count: 0,
            texture_index: None,
            base_color: [1.0, 1.0, 1.0, 1.0],
            alpha_mode: SubmeshAlphaMode::Opaque,
            alpha_cutoff: 0.5,
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            uv_scale: [1.0, 1.0],
        }
    }
}