// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # GPU Render Asset Data Models
//!
//! Structures for uploaded GPU textures, 3D model geometries, submesh materials, and bounding computations.

use crate::render::types::vertex::{SkinVertex, Vertex};

/// GPU-uploaded texture with its bind group, canonical source path, and dimensions.
pub struct TextureAsset {
    /// Egui/WGPU compatible GPU bind group containing texture view and sampler bindings.
    pub bind_group: wgpu::BindGroup,
    /// Absolute canonical path on local disk for memory deduplication.
    pub source_path: String,
    /// Width of the texture image in pixels.
    pub width: u32,
    /// Height of the texture image in pixels.
    pub height: u32,
}

/// Submesh alpha blending and testing mode (glTF 2.0 standard).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmeshAlphaMode {
    /// Fully opaque rendering with depth writes enabled (no discard).
    Opaque,
    /// Masked/Cutout rendering with depth writes enabled (discards pixels where alpha < cutoff).
    Mask,
    /// Alpha-blended rendering over background geometry (depth writes disabled).
    Blend,
}

/// Submesh index range with its material and texture binding information.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelSubmesh {
    pub start_index: u32,
    pub index_count: u32,
    pub texture_index: Option<usize>,
    pub base_color: [f32; 4],
    pub alpha_mode: SubmeshAlphaMode,
    pub alpha_cutoff: f32,
}

/// GPU-uploaded 3D model asset with vertex/index buffers, AABB bounds,
/// and raw mesh data for physics shape generation.
pub struct ModelAsset {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub source_path: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// Raw positions extracted for physics shape generation (Trimesh / ConvexHull)
    pub raw_vertices: Vec<[f32; 3]>,
    /// Raw indices extracted for physics shape generation (Trimesh / ConvexHull)
    pub raw_indices: Vec<u32>,
    /// Raw skinning vertex data for CPU animation matrix evaluation
    pub raw_skin_vertices: Vec<SkinVertex>,
    /// CPU mirror of GPU vertices for real-time WGPU queue.write_buffer updates
    pub gpu_vertices: Vec<Vertex>,
    /// Skeleton hierarchy (if model contains bones/skinning)
    pub skeleton: Option<ae_animation::Skeleton>,
    /// Animation clips embedded in the 3D model
    pub animations: Vec<ae_animation::AnimationClip>,
    /// Default embedded texture handle extracted from GLTF/GLB/FBX materials
    pub default_texture: Option<crate::asset::AssetHandle>,
    /// All textures embedded in the model mapped by image index
    pub embedded_textures: Vec<crate::asset::AssetHandle>,
    /// Submesh index ranges with their corresponding material/texture bindings
    pub submeshes: Vec<ModelSubmesh>,
}

impl ModelAsset {
    /// Computes the bounding sphere radius from AABB center and half-diagonal.
    /// Uses the distance from AABB center to any corner (half-diagonal), which
    /// correctly handles off-center models where `min`/`max` are far from origin.
    /// Falls back to `max(origin_distances)` for backward compatibility with
    /// origin-centered models. Minimum clamped to `1.0` to prevent zero-radius culling.
    pub fn bounding_radius(&self) -> f32 {
        // AABB center-to-corner half-diagonal: correct for all model origins
        let cx = (self.min[0] + self.max[0]) * 0.5;
        let cy = (self.min[1] + self.max[1]) * 0.5;
        let cz = (self.min[2] + self.max[2]) * 0.5;
        let hx = self.max[0] - cx;
        let hy = self.max[1] - cy;
        let hz = self.max[2] - cz;
        let r_half_diag = (hx * hx + hy * hy + hz * hz).sqrt();
        // Distance from origin to AABB center + half-diagonal covers the full extent
        let center_dist = (cx * cx + cy * cy + cz * cz).sqrt();
        (center_dist + r_half_diag).max(1.0)
    }
}