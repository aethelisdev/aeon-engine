// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Mipmapped 2D Texture Array Atlas (`TextureArrayAtlas`)
//!
//! Encapsulates WGPU `D2Array` texture allocation, sampler bind-group creation, and CPU-side 2x2
//! box-filtered mipmap generation/upload for sprite sheets and dynamic UI thumbnail layers.

use crate::texture_pipeline::TextureQuadPipeline;

/// Computes the full mipmap chain depth (`floor(log2(tile_size)) + 1`) down to `1x1`.
#[inline]
#[must_use]
pub const fn compute_mip_level_count(tile_size: u32) -> u32 {
    if tile_size == 0 {
        1
    } else {
        tile_size.ilog2() + 1
    }
}

/// Downsamples an uncompressed RGBA8 buffer by 2x using 2x2 box averaging into `dst`.
///
/// Odd dimensions clamp at the trailing texel edge so non-power-of-two intermediate levels never
/// panic or sample out of bounds. Returns the resulting `(next_width, next_height)`.
pub(crate) fn downsample_rgba_2x2(
    src: &[u8],
    src_w: u32,
    src_h: u32,
    dst: &mut Vec<u8>,
) -> (u32, u32) {
    dst.clear();
    if src_w == 0 || src_h == 0 {
        return (0, 0);
    }
    let next_w = (src_w / 2).max(1);
    let next_h = (src_h / 2).max(1);
    let needed = (next_w as usize) * (next_h as usize) * 4;
    dst.reserve(needed);

    for ny in 0..next_h {
        let y0 = ny * 2;
        let y1 = (y0 + 1).min(src_h - 1);
        for nx in 0..next_w {
            let x0 = nx * 2;
            let x1 = (x0 + 1).min(src_w - 1);

            let p00 = ((y0 * src_w + x0) * 4) as usize;
            let p10 = ((y0 * src_w + x1) * 4) as usize;
            let p01 = ((y1 * src_w + x0) * 4) as usize;
            let p11 = ((y1 * src_w + x1) * 4) as usize;

            for c in 0..4 {
                let sum = u32::from(src[p00 + c])
                    + u32::from(src[p10 + c])
                    + u32::from(src[p01 + c])
                    + u32::from(src[p11 + c]);
                dst.push((sum / 4) as u8);
            }
        }
    }

    (next_w, next_h)
}

/// Copies a single `tile_size x tile_size` RGBA8 region at grid cell `(col, row)` into `dst`.
///
/// Reuses `dst` capacity across calls to eliminate per-tile heap allocations when slicing
/// packed icon sheets. Returns `false` if coordinates exceed the sheet bounds.
pub(crate) fn extract_rgba_tile(
    sheet_rgba: &[u8],
    sheet_width: u32,
    sheet_height: u32,
    tile_size: u32,
    col: u32,
    row: u32,
    dst: &mut Vec<u8>,
) -> bool {
    dst.clear();
    if tile_size == 0
        || (col + 1).saturating_mul(tile_size) > sheet_width
        || (row + 1).saturating_mul(tile_size) > sheet_height
    {
        return false;
    }
    let expected_len = (sheet_width as usize) * (sheet_height as usize) * 4;
    if sheet_rgba.len() < expected_len {
        return false;
    }

    let tile_bytes_len = (tile_size as usize) * (tile_size as usize) * 4;
    dst.reserve(tile_bytes_len);
    let src_x = col * tile_size;
    let row_byte_len = (tile_size as usize) * 4;
    for y in 0..tile_size {
        let src_y = row * tile_size + y;
        let start = ((src_y * sheet_width + src_x) * 4) as usize;
        dst.extend_from_slice(&sheet_rgba[start..start + row_byte_len]);
    }
    true
}

/// Managed `D2Array` GPU texture atlas for tile-uniform icons and thumbnails.
///
/// Isolates each icon/thumbnail in its own array slice with independent mipmaps, preventing
/// cross-tile bilinear/trilinear filtering bleed.
pub struct TextureArrayAtlas {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    tile_size: u32,
    layer_count: u32,
    mip_level_count: u32,
}

impl TextureArrayAtlas {
    /// Allocates a new `D2Array` RGBA8 sRGB texture atlas and binds it to `pipeline`.
    ///
    /// Both `tile_size` and `layer_count` are clamped to at least `1`.
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        pipeline: &TextureQuadPipeline,
        tile_size: u32,
        layer_count: u32,
        label: Option<&str>,
    ) -> Self {
        let tile_size = tile_size.max(1);
        let layer_count = layer_count.max(1);
        let mip_level_count = compute_mip_level_count(tile_size);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: tile_size,
                height: tile_size,
                depth_or_array_layers: layer_count,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let bind_group = pipeline.create_texture_bind_group(device, &view);

        Self {
            texture,
            view,
            bind_group,
            tile_size,
            layer_count,
            mip_level_count,
        }
    }

    /// Returns the square pixel width/height of each array layer.
    #[inline]
    #[must_use]
    pub const fn tile_size(&self) -> u32 {
        self.tile_size
    }

    /// Returns the total number of array layers allocated in this atlas.
    #[inline]
    #[must_use]
    pub const fn layer_count(&self) -> u32 {
        self.layer_count
    }

    /// Returns the mipmap chain depth uploaded per layer.
    #[inline]
    #[must_use]
    pub const fn mip_level_count(&self) -> u32 {
        self.mip_level_count
    }

    /// Returns the underlying `wgpu::Texture` backing this array atlas.
    #[inline]
    #[must_use]
    pub const fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// Returns the `D2Array` `wgpu::TextureView` for custom bindings.
    #[inline]
    #[must_use]
    pub const fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Returns the `TextureQuadPipeline` bind group referencing this atlas.
    #[inline]
    #[must_use]
    pub const fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Uploads a single `tile_size x tile_size` RGBA8 image into `layer` along with its full
    /// 2x2 box-filtered mipmap chain.
    ///
    /// Returns `false` if `layer >= self.layer_count` or `rgba.len() != tile_size * tile_size * 4`.
    pub fn upload_rgba_layer(&self, queue: &wgpu::Queue, layer: u32, rgba: &[u8]) -> bool {
        let expected_len = (self.tile_size as usize) * (self.tile_size as usize) * 4;
        if layer >= self.layer_count || rgba.len() != expected_len {
            log::warn!(
                "Skipping invalid atlas upload (layer={layer}/{}, bytes={}/{}).",
                self.layer_count,
                rgba.len(),
                expected_len
            );
            return false;
        }

        self.write_mip_level(queue, layer, 0, self.tile_size, self.tile_size, rgba);

        let mut cur_w = self.tile_size;
        let mut cur_h = self.tile_size;
        let mut scratch_a = Vec::new();
        let mut scratch_b = Vec::new();
        let mut using_a_as_src = false;

        for mip_level in 1..self.mip_level_count {
            let (next_w, next_h) = if mip_level == 1 {
                using_a_as_src = true;
                downsample_rgba_2x2(rgba, cur_w, cur_h, &mut scratch_a)
            } else if using_a_as_src {
                using_a_as_src = false;
                downsample_rgba_2x2(&scratch_a, cur_w, cur_h, &mut scratch_b)
            } else {
                using_a_as_src = true;
                downsample_rgba_2x2(&scratch_b, cur_w, cur_h, &mut scratch_a)
            };
            let level_bytes = if using_a_as_src {
                &scratch_a
            } else {
                &scratch_b
            };
            self.write_mip_level(queue, layer, mip_level, next_w, next_h, level_bytes);
            cur_w = next_w;
            cur_h = next_h;
        }

        true
    }

    /// Slices a packed RGBA8 sprite sheet into `tile_size x tile_size` tiles in row-major order
    /// and uploads them into consecutive array layers starting at `start_layer`.
    ///
    /// Returns the number of uploaded tile layers.
    pub fn upload_sprite_sheet_tiles(
        &self,
        queue: &wgpu::Queue,
        sheet_width: u32,
        sheet_height: u32,
        sheet_rgba: &[u8],
        start_layer: u32,
    ) -> u32 {
        let cols = sheet_width / self.tile_size;
        let rows = sheet_height / self.tile_size;
        let mut tile_buf = Vec::with_capacity((self.tile_size as usize).pow(2) * 4);
        let mut uploaded = 0u32;

        for r in 0..rows {
            for c in 0..cols {
                let target_layer = start_layer.saturating_add(r * cols + c);
                if target_layer >= self.layer_count {
                    return uploaded;
                }
                if extract_rgba_tile(
                    sheet_rgba,
                    sheet_width,
                    sheet_height,
                    self.tile_size,
                    c,
                    r,
                    &mut tile_buf,
                ) && self.upload_rgba_layer(queue, target_layer, &tile_buf)
                {
                    uploaded += 1;
                }
            }
        }

        uploaded
    }

    fn write_mip_level(
        &self,
        queue: &wgpu::Queue,
        layer: u32,
        mip_level: u32,
        width: u32,
        height: u32,
        bytes: &[u8],
    ) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer,
                },
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{compute_mip_level_count, downsample_rgba_2x2, extract_rgba_tile};

    #[test]
    fn test_compute_mip_level_count() {
        assert_eq!(compute_mip_level_count(0), 1);
        assert_eq!(compute_mip_level_count(1), 1);
        assert_eq!(compute_mip_level_count(2), 2);
        assert_eq!(compute_mip_level_count(64), 7);
        assert_eq!(compute_mip_level_count(256), 9);
    }

    #[test]
    fn test_downsample_rgba_2x2_averages_quadrants_and_handles_edges() {
        #[rustfmt::skip]
        let src = [
            10, 20, 30, 40,   30, 40, 50, 60,
            50, 60, 70, 80,   70, 80, 90, 100,
        ];
        let mut dst = Vec::new();
        let (nw, nh) = downsample_rgba_2x2(&src, 2, 2, &mut dst);
        assert_eq!((nw, nh), (1, 1));
        assert_eq!(dst, [40, 50, 60, 70]);

        let (ew, eh) = downsample_rgba_2x2(&[], 0, 0, &mut dst);
        assert_eq!((ew, eh), (0, 0));
        assert!(dst.is_empty());

        let single = [200u8, 100, 50, 255];
        let (sw, sh) = downsample_rgba_2x2(&single, 1, 1, &mut dst);
        assert_eq!((sw, sh), (1, 1));
        assert_eq!(dst, [200, 100, 50, 255]);
    }

    #[test]
    fn test_extract_rgba_tile_row_major_and_bounds_check() {
        // 4x4 sheet of 2x2 tiles -> 4 distinct solid-color tiles
        let mut sheet = vec![0u8; 4 * 4 * 4];
        for y in 0..4u32 {
            for x in 0..4u32 {
                let tile_id = ((y / 2) * 2 + (x / 2)) as u8;
                let idx = ((y * 4 + x) * 4) as usize;
                sheet[idx..idx + 4].copy_from_slice(&[tile_id, tile_id + 10, tile_id + 20, 255]);
            }
        }

        let mut tile = Vec::new();
        assert!(extract_rgba_tile(&sheet, 4, 4, 2, 1, 1, &mut tile));
        assert_eq!(tile.len(), 2 * 2 * 4);
        assert!(
            tile.as_chunks::<4>()
                .0
                .iter()
                .all(|px| *px == [3, 13, 23, 255])
        );

        assert!(!extract_rgba_tile(&sheet, 4, 4, 2, 2, 0, &mut tile));
        assert!(!extract_rgba_tile(&sheet[..8], 4, 4, 2, 0, 0, &mut tile));
        assert!(!extract_rgba_tile(&sheet, 4, 4, 0, 0, 0, &mut tile));
    }
}