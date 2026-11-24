// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU-independent 3D mesh data, procedural primitives, CPU skeletal skinning,
//! bounding volume helpers, and glTF 2.0 parsing for Aeon Engine.

/// glTF 2.0 CPU geometry, skeletal animation, embedded texture, and safe path loaders.
pub mod gltf;
/// CPU model payload container and AABB bounding volume helpers.
pub mod model;
/// Static primitive vertex arrays and procedural 3D shape generators.
pub mod primitives;
/// CPU skeletal vertex skinning evaluation against animation joint palettes.
pub mod skinning;
/// Submesh index slicing and surface alpha blending metadata.
pub mod submesh;
/// Plain-old-data vertex and skinning attribute layouts.
pub mod vertex;

pub use gltf::{
    ParsedGltfGeometry, RawMeshGeometry, extract_gltf_all_embedded_textures,
    extract_gltf_embedded_texture, is_safe_path, parse_gltf_bytes, parse_gltf_file,
    parse_gltf_raw_geometry, parse_gltf_scene_geometry, parse_gltf_skin_and_animations,
};
pub use model::{ParsedModelData, compute_aabb_bounding_radius};
pub use primitives::{
    CUBE_VERTICES, GRID_QUAD_VERTICES, PYRAMID_VERTICES, QUAD_VERTICES, VERTICES, generate_capsule,
    generate_cylinder, generate_sphere, generate_torus,
};
pub use skinning::apply_skinning_palette;
pub use submesh::{ModelSubmesh, SubmeshAlphaMode};
pub use vertex::{SkinVertex, SpriteVertex, Vertex};

#[cfg(test)]
mod tests;