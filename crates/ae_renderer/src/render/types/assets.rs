// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # GPU Render Asset Data Models
//!
//! Structures for uploaded GPU textures and 3D model buffers paired with `ae_mesh` payloads.

use ae_mesh::{ModelSubmesh, SkinVertex, Vertex, compute_aabb_bounding_radius};

/// GPU-uploaded texture with its bind group, canonical source path, and dimensions.
pub struct TextureAsset {
    /// WGPU compatible GPU bind group containing texture view and sampler bindings.
    pub bind_group: wgpu::BindGroup,
    /// Absolute canonical path on local disk for memory deduplication.
    pub source_path: String,
    /// Width of the texture image in pixels.
    pub width: u32,
    /// Height of the texture image in pixels.
    pub height: u32,
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
    ///
    /// Uses the distance from AABB center to any corner (half-diagonal), which
    /// correctly handles off-center models where `min`/`max` are far from origin.
    /// Falls back to `max(origin_distances)` for backward compatibility with
    /// origin-centered models. Minimum clamped to `1.0` to prevent zero-radius culling.
    pub fn bounding_radius(&self) -> f32 {
        compute_aabb_bounding_radius(self.min, self.max)
    }
}