// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Procedural fallback texture generators for missing or default material textures.

use crate::data::{ColorSpace, CpuTextureData};

/// Generator for procedural fallback textures (default white, flat normal, missing checkerboard).
pub struct FallbackTextureGenerator;

impl FallbackTextureGenerator {
    /// Generates a 1x1 solid white RGBA texture for default untextured materials.
    pub fn white_1x1() -> CpuTextureData {
        CpuTextureData::new(
            1,
            1,
            vec![255, 255, 255, 255],
            ColorSpace::Srgb,
            "fallback_white_1x1",
        )
        .with_mipmaps()
    }

    /// Generates a 1x1 flat normal map texture `[128, 128, 255, 255]` representing surface normal `(0, 0, 1)`.
    pub fn flat_normal_1x1() -> CpuTextureData {
        CpuTextureData::new(
            1,
            1,
            vec![128, 128, 255, 255],
            ColorSpace::Linear,
            "fallback_flat_normal_1x1",
        )
        .with_mipmaps()
    }

    /// Generates a 1x1 default Metallic-Roughness texture `[0, 255, 0, 255]` (Metallic=0.0, Roughness=1.0).
    pub fn default_metallic_roughness_1x1() -> CpuTextureData {
        CpuTextureData::new(
            1,
            1,
            vec![0, 255, 0, 255],
            ColorSpace::Linear,
            "fallback_metallic_roughness_1x1",
        )
        .with_mipmaps()
    }

    /// Generates a 1x1 default Ambient Occlusion texture `[255, 255, 255, 255]` (AO=1.0).
    pub fn default_ao_1x1() -> CpuTextureData {
        CpuTextureData::new(
            1,
            1,
            vec![255, 255, 255, 255],
            ColorSpace::Linear,
            "fallback_ao_1x1",
        )
        .with_mipmaps()
    }

    /// Generates a 1x1 black Emissive texture `[0, 0, 0, 255]` (Emissive=0.0).
    pub fn black_emissive_1x1() -> CpuTextureData {
        CpuTextureData::new(
            1,
            1,
            vec![0, 0, 0, 255],
            ColorSpace::Srgb,
            "fallback_emissive_1x1",
        )
        .with_mipmaps()
    }

    /// Generates a magenta-black checkerboard pattern texture for missing or corrupted assets.
    /// # Arguments
    /// * `width` - Target width in pixels.
    /// * `height` - Target height in pixels.
    /// * `grid_size` - Size of each checker square in pixels.
    pub fn checkerboard_missing(width: u32, height: u32, grid_size: u32) -> CpuTextureData {
        let w = width.max(2);
        let h = height.max(2);
        let size = grid_size.max(1);
        let mut pixels = Vec::with_capacity((w * h * 4) as usize);

        for y in 0..h {
            for x in 0..w {
                let is_magenta = ((x / size) + (y / size)) % 2 == 0;
                if is_magenta {
                    pixels.extend_from_slice(&[255, 0, 255, 255]); // Magenta
                } else {
                    pixels.extend_from_slice(&[0, 0, 0, 255]); // Black
                }
            }
        }

        CpuTextureData::new(
            w,
            h,
            pixels,
            ColorSpace::Srgb,
            "fallback_checkerboard_missing",
        )
        .with_mipmaps()
    }
}