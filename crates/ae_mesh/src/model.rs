// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! CPU-side 3D model payload container and AABB bounding volume calculations.

use crate::submesh::ModelSubmesh;
use crate::vertex::{SkinVertex, Vertex};
use std::path::PathBuf;

/// Complete CPU-parsed 3D model payload ready for GPU upload or offline binary cooking.
///
/// Aggregates vertex/index streams, raw triangle positions for collision shape
/// construction, skeletal animation data, embedded textures, and submesh ranges.
#[derive(Debug, Clone)]
pub struct ParsedModelData {
    /// Combined vertex buffer in object/baked node space.
    pub all_vertices: Vec<Vertex>,
    /// Triangle index buffer referencing `all_vertices`.
    pub all_indices: Vec<u32>,
    /// Raw 3D vertex positions retained for collision mesh construction.
    pub raw_positions: Vec<[f32; 3]>,
    /// Optional per-vertex skinning bind attributes matching `all_vertices`.
    pub raw_skin_vertices: Vec<SkinVertex>,
    /// Minimum corner `[x, y, z]` of the model's axis-aligned bounding box.
    pub min: [f32; 3],
    /// Maximum corner `[x, y, z]` of the model's axis-aligned bounding box.
    pub max: [f32; 3],
    /// Canonicalized disk path or virtual archive key used for deduplication.
    pub canonical_path: PathBuf,
    /// Original source path string for diagnostics and GPU buffer labels.
    pub original_path: String,
    /// Display name assigned when spawning an entity from this model.
    pub final_name: String,
    /// Optional skeletal joint hierarchy extracted from the source model.
    pub skeleton: Option<ae_animation::Skeleton>,
    /// Skeletal or hierarchical node animation clips embedded in the model.
    pub animations: Vec<ae_animation::AnimationClip>,
    /// First embedded diffuse texture, if any, decoded on the CPU.
    pub default_texture: Option<ae_texture::CpuTextureData>,
    /// All embedded CPU textures indexed by submesh `texture_index`.
    pub embedded_textures: Vec<ae_texture::CpuTextureData>,
    /// Ordered submesh index slices and surface parameters.
    pub submeshes: Vec<ModelSubmesh>,
}

impl ParsedModelData {
    /// Computes an origin-enclosing bounding sphere radius from [`Self::min`] and [`Self::max`].
    ///
    /// Uses [`compute_aabb_bounding_radius`] and clamps the minimum radius to `1.0`
    /// so degenerate or empty meshes do not vanish prematurely during frustum culling.
    #[must_use]
    pub fn bounding_radius(&self) -> f32 {
        compute_aabb_bounding_radius(self.min, self.max)
    }
}

/// Computes an origin-enclosing conservative bounding sphere radius from an AABB `(min, max)`.
///
/// Adds the distance from local origin to the AABB center and the center-to-corner
/// half-diagonal, ensuring off-center meshes remain inside the culling sphere around
/// the entity's local origin. Non-finite bounds return `1.0`; degenerate bounds are
/// clamped to a minimum of `1.0`.
#[must_use]
pub fn compute_aabb_bounding_radius(min: [f32; 3], max: [f32; 3]) -> f32 {
    if !min.iter().chain(max.iter()).all(|v| v.is_finite()) {
        return 1.0;
    }
    let cx = (min[0] + max[0]) * 0.5;
    let cy = (min[1] + max[1]) * 0.5;
    let cz = (min[2] + max[2]) * 0.5;
    let hx = (max[0] - cx).abs();
    let hy = (max[1] - cy).abs();
    let hz = (max[2] - cz).abs();
    let r_half_diag = (hx * hx + hy * hy + hz * hz).sqrt();
    let center_dist = (cx * cx + cy * cy + cz * cz).sqrt();
    (center_dist + r_half_diag).max(1.0)
}