// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! glTF 2.0 CPU geometry, skeletal animation, embedded texture, and path safety loaders.

/// glTF skeletal armature, node hierarchy, and animation clip extraction.
pub mod animation;
/// glTF scene graph and primitive mesh geometry extraction.
pub mod geometry;
/// High-level file and byte-slice glTF 2.0 parsers and path security validation.
pub mod parser;
/// glTF embedded image extraction and sampler wrap-mode mapping.
pub mod textures;

pub use animation::parse_gltf_skin_and_animations;
pub use geometry::{
    ParsedGltfGeometry, RawMeshGeometry, parse_gltf_raw_geometry, parse_gltf_scene_geometry,
};
pub use parser::{
    GltfExternalUris, collect_gltf_external_uris, is_safe_path, parse_gltf_bytes, parse_gltf_file,
};
pub use textures::{extract_gltf_all_embedded_textures, extract_gltf_embedded_texture};