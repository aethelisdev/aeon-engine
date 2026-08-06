// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! CPU-side Mipmap chain generation algorithms for uncompressed RGBA texture images.

/// Container representing a single mipmap level in a texture mip chain.
#[derive(Debug, Clone)]
pub struct CpuMipmapLevel {
    /// Width of this mip level in pixels.
    pub width: u32,
    /// Height of this mip level in pixels.
    pub height: u32,
    /// Raw uncompressed RGBA8 bytes for this mip level (length = width * height * 4).
    pub bytes: Vec<u8>,
}

impl CpuMipmapLevel {
    /// Constructs a new CPU mipmap level with buffer size verification.
    pub fn new(width: u32, height: u32, bytes: Vec<u8>) -> Self {
        let expected_len = (width as usize) * (height as usize) * 4;
        debug_assert_eq!(
            bytes.len(),
            expected_len,
            "CpuMipmapLevel buffer size mismatch: expected {}, got {}",
            expected_len,
            bytes.len()
        );
        Self {
            width,
            height,
            bytes,
        }
    }
}

/// Generates a complete mipmap chain from base level 0 down to 1x1 using 2x2 box downsampling.
/// # Arguments
/// * `base_width` - Width of mip level 0.
/// * `base_height` - Height of mip level 0.
/// * `base_bytes` - Uncompressed RGBA8 bytes of mip level 0.
/// # Returns
/// A `Vec<CpuMipmapLevel>` containing level 0 followed by progressively halved mipmap levels.
pub fn generate_mipmap_chain(
    base_width: u32,
    base_height: u32,
    base_bytes: &[u8],
) -> Vec<CpuMipmapLevel> {
    if base_width == 0 || base_height == 0 || base_bytes.is_empty() {
        return Vec::new();
    }

    let num_levels = (base_width.max(base_height) as f32).log2().floor() as usize + 1;
    let mut chain = Vec::with_capacity(num_levels);

    // Level 0 is the unscaled base texture
    chain.push(CpuMipmapLevel::new(
        base_width,
        base_height,
        base_bytes.to_vec(),
    ));

    let mut cur_w = base_width;
    let mut cur_h = base_height;
    let mut cur_bytes = base_bytes.to_vec();

    while cur_w > 1 || cur_h > 1 {
        let next_w = (cur_w / 2).max(1);
        let next_h = (cur_h / 2).max(1);
        let mut next_bytes = Vec::with_capacity((next_w * next_h * 4) as usize);

        for ny in 0..next_h {
            for nx in 0..next_w {
                // Compute source 2x2 block coordinates
                let x0 = nx * 2;
                let y0 = ny * 2;
                let x1 = (x0 + 1).min(cur_w - 1);
                let y1 = (y0 + 1).min(cur_h - 1);

                let p00_idx = ((y0 * cur_w + x0) * 4) as usize;
                let p10_idx = ((y0 * cur_w + x1) * 4) as usize;
                let p01_idx = ((y1 * cur_w + x0) * 4) as usize;
                let p11_idx = ((y1 * cur_w + x1) * 4) as usize;

                for c in 0..4 {
                    let r00 = cur_bytes[p00_idx + c] as u32;
                    let r10 = cur_bytes[p10_idx + c] as u32;
                    let r01 = cur_bytes[p01_idx + c] as u32;
                    let r11 = cur_bytes[p11_idx + c] as u32;
                    let avg = ((r00 + r10 + r01 + r11) / 4) as u8;
                    next_bytes.push(avg);
                }
            }
        }

        chain.push(CpuMipmapLevel::new(next_w, next_h, next_bytes.clone()));
        cur_w = next_w;
        cur_h = next_h;
        cur_bytes = next_bytes;
    }

    chain
}