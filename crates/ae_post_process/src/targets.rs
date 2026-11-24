// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Intermediate color, MSAA resolve, and depth render-target allocation and sanitation helpers.

/// Depth texture format used by MSAA and post-process scene depth targets.
pub const POST_PROCESS_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Sanitizes an arbitrary MSAA sample count into a WebGPU-valid tier (`1`, `2`, or `4`).
#[must_use]
pub const fn sanitize_msaa_samples(samples: u32) -> u32 {
    if samples >= 4 {
        4
    } else if samples >= 2 {
        2
    } else {
        1
    }
}

/// Sanitizes a bloom intensity scalar, clamping negative or non-finite inputs to `0.0`.
#[must_use]
pub const fn sanitize_bloom_intensity(intensity: f32) -> f32 {
    if intensity.is_finite() && intensity > 0.0 {
        intensity
    } else {
        0.0
    }
}

/// Estimates dynamic GPU VRAM usage (bytes) for post-process scene, depth, MSAA, and bloom targets.
#[must_use]
pub fn estimate_post_process_vram_bytes(width: u32, height: u32, msaa_samples: u32) -> usize {
    let vp_w = width.max(1) as usize;
    let vp_h = height.max(1) as usize;
    let base_targets = vp_w.saturating_mul(vp_h).saturating_mul(8); // Scene + Depth (4 bytes each)
    let msaa = sanitize_msaa_samples(msaa_samples);
    let msaa_bytes = if msaa > 1 {
        vp_w.saturating_mul(vp_h)
            .saturating_mul(4)
            .saturating_mul(msaa as usize)
    } else {
        0
    };
    let bloom_chain = vp_w.saturating_mul(vp_h).saturating_mul(2); // Half-resolution bloom chain estimate
    base_targets
        .saturating_add(msaa_bytes)
        .saturating_add(bloom_chain)
}

/// Creates a depth texture view for the given surface dimensions and MSAA sample count.
pub(crate) fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    sample_count: u32,
    label: &str,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: sanitize_msaa_samples(sample_count),
        dimension: wgpu::TextureDimension::D2,
        format: POST_PROCESS_DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

/// Creates a single-sample or multisampled color render-target texture view.
pub(crate) fn create_target_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    sample_count: u32,
    label: &str,
) -> wgpu::TextureView {
    create_target_texture_and_view(device, config, sample_count, label).1
}

/// Creates both a color render-target texture and its associated view.
///
/// Returns the backing texture alongside its view so callers can inspect the exact
/// driver-selected `wgpu::TextureFormat` at runtime.
pub(crate) fn create_target_texture_and_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    sample_count: u32,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: sanitize_msaa_samples(sample_count),
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

#[cfg(test)]
mod tests {
    use super::{
        estimate_post_process_vram_bytes, sanitize_bloom_intensity, sanitize_msaa_samples,
    };

    #[test]
    fn test_sanitize_msaa_and_bloom_intensity() {
        assert_eq!(sanitize_msaa_samples(0), 1);
        assert_eq!(sanitize_msaa_samples(1), 1);
        assert_eq!(sanitize_msaa_samples(2), 2);
        assert_eq!(sanitize_msaa_samples(3), 2);
        assert_eq!(sanitize_msaa_samples(4), 4);
        assert_eq!(sanitize_msaa_samples(16), 4);

        assert_eq!(sanitize_bloom_intensity(1.25), 1.25);
        assert_eq!(sanitize_bloom_intensity(-0.5), 0.0);
        assert_eq!(sanitize_bloom_intensity(f32::NAN), 0.0);
        assert_eq!(sanitize_bloom_intensity(f32::INFINITY), 0.0);
    }

    #[test]
    fn test_estimate_post_process_vram_bytes_with_msaa_and_zero_dims() {
        let single = estimate_post_process_vram_bytes(100, 100, 1);
        let quad = estimate_post_process_vram_bytes(100, 100, 4);
        assert_eq!(single, 100 * 100 * 10);
        assert_eq!(quad, 100 * 100 * (10 + 16));
        assert_eq!(estimate_post_process_vram_bytes(0, 0, 0), 10);
    }
}