// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! File loading, path security verification, and image parsing utilities for textures.

use crate::data::{ColorSpace, CpuTextureData};
use std::path::Path;

/// Maximum allowable texture dimension in pixels (16384x16384) to prevent VRAM overflow.
pub const MAX_TEXTURE_DIMENSION: u32 = 16384;

/// Sanitizes file paths to prevent directory traversal vulnerabilities (e.g. `../..` manipulation).
/// Returns `false` if the path contains relative traversal sequences (`..`).
pub fn is_safe_path(path: &str) -> bool {
    let p = Path::new(path);
    for component in p.components() {
        if let std::path::Component::ParentDir = component {
            return false;
        }
    }
    true
}

/// Parses a texture file from local disk into uncompressed RGBA8 `CpuTextureData`.
/// Performs security sanitization, checks dimension bounds, and converts pixels to RGBA8.
/// # Errors
/// Returns an error string if path is unsafe, file cannot be read, image format is unknown,
/// or dimensions exceed `MAX_TEXTURE_DIMENSION`.
pub fn parse_texture_file(path: &str, color_space: ColorSpace) -> Result<CpuTextureData, String> {
    if !is_safe_path(path) {
        core::hint::cold_path();
        return Err(format!(
            "[SECURITY ERROR] Blocked unsafe texture path: {}",
            path
        ));
    }

    let img = match image::open(path) {
        Ok(img) => img,
        Err(e) => {
            core::hint::cold_path();
            return Err(format!("[ERROR] Failed to open texture '{}': {}", path, e));
        }
    };

    let (width, height) = (img.width(), img.height());
    if width > MAX_TEXTURE_DIMENSION || height > MAX_TEXTURE_DIMENSION {
        core::hint::cold_path();
        return Err(format!(
            "[ERROR] Texture dimensions {}x{} exceed maximum limit {}x{}",
            width, height, MAX_TEXTURE_DIMENSION, MAX_TEXTURE_DIMENSION
        ));
    }

    let last_mod = std::fs::metadata(path).and_then(|m| m.modified()).ok();
    let rgba = img.to_rgba8();
    Ok(
        CpuTextureData::new(width, height, rgba.into_raw(), color_space, path)
            .with_last_modified(last_mod)
            .with_mipmaps(),
    )
}