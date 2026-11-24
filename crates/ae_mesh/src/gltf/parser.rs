// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Disk and VFS-slice glTF 2.0 parsers alongside workspace path security validation.

use super::animation::parse_gltf_skin_and_animations;
use super::geometry::parse_gltf_scene_geometry;
use super::textures::extract_gltf_all_embedded_textures;
use crate::model::ParsedModelData;
use std::path::{Path, PathBuf};

/// Verifies whether `path` is safe for asset loading by rejecting parent directory
/// traversal (`..`), UNC network prefixes (`\\`), URI protocol schemes (`://`), and NUL bytes.
#[must_use]
pub fn is_safe_path(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("\\\\")
        || trimmed.contains("://")
        || trimmed.contains('\0')
    {
        return false;
    }
    Path::new(trimmed)
        .components()
        .all(|component| component != std::path::Component::ParentDir)
}

/// Parses a `.gltf` or `.glb` file on disk into CPU [`ParsedModelData`].
///
/// # Errors
/// Returns an error string if `path` fails [`is_safe_path`], canonicalization fails,
/// or `gltf::import` fails to decode the asset.
pub fn parse_gltf_file(path: &str, final_name: String) -> Result<ParsedModelData, String> {
    if !is_safe_path(path) {
        core::hint::cold_path();
        return Err(format!("Security Error: Blocked unsafe GLTF path: {path}"));
    }

    let canonical_path = std::fs::canonicalize(path).map_err(|e| {
        core::hint::cold_path();
        format!("Failed to canonicalize model path '{path}': {e}")
    })?;

    let (document, buffers, images) = gltf::import(path).map_err(|e| {
        core::hint::cold_path();
        format!("Failed to load GLTF Model file {path}: {e:?}")
    })?;

    Ok(assemble_parsed_gltf_data(
        &document,
        &buffers,
        &images,
        canonical_path,
        path,
        final_name,
    ))
}

/// Parses an in-memory `.glb` or self-contained `.gltf` byte slice into CPU [`ParsedModelData`].
///
/// # Errors
/// Returns an error string if `gltf::import_slice` rejects the payload.
pub fn parse_gltf_bytes(
    bytes: &[u8],
    canonical_key: PathBuf,
    original_path: &str,
    final_name: String,
) -> Result<ParsedModelData, String> {
    let (document, buffers, images) = gltf::import_slice(bytes)
        .map_err(|e| format!("Failed to decode GLTF bytes for '{original_path}': {e:?}"))?;
    Ok(assemble_parsed_gltf_data(
        &document,
        &buffers,
        &images,
        canonical_key,
        original_path,
        final_name,
    ))
}

fn assemble_parsed_gltf_data(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    canonical_path: PathBuf,
    original_path: &str,
    final_name: String,
) -> ParsedModelData {
    let (skeleton, animations) = parse_gltf_skin_and_animations(document, buffers);
    let embedded_textures = extract_gltf_all_embedded_textures(document, images);
    let default_texture = embedded_textures.first().cloned();
    let parsed_geom = parse_gltf_scene_geometry(document, buffers, images, skeleton.is_some());

    ParsedModelData {
        all_vertices: parsed_geom.all_vertices,
        all_indices: parsed_geom.all_indices,
        raw_positions: parsed_geom.raw_positions,
        raw_skin_vertices: parsed_geom.raw_skin_vertices,
        min: parsed_geom.min,
        max: parsed_geom.max,
        canonical_path,
        original_path: original_path.to_owned(),
        final_name,
        skeleton,
        animations,
        default_texture,
        embedded_textures,
        submeshes: parsed_geom.submeshes,
    }
}