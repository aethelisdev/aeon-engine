// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

/// Encapsulates a WGPU texture, its default view, and its dimensions.
/// Used for rendering intermediate targets such as the 3D viewport.
pub struct ViewportTexture {
    /// The underlying WGPU texture object.
    pub texture: wgpu::Texture,
    /// The default TextureView (sRGB format matching configuration).
    pub view: wgpu::TextureView,
    /// The reinterpreted non-sRGB (linear) view passed to Egui to prevent double-gamma correction.
    pub egui_view: wgpu::TextureView,
    /// Width of this texture.
    pub width: u32,
    /// Height of this texture.
    pub height: u32,
}

impl ViewportTexture {
    /// Creates a new ViewportTexture with the specified dimensions and format.
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        };

        // Determine compatible linear format for Egui texture sampling to bypass hardware decoding
        let linear_format = match format {
            wgpu::TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8Unorm,
            other => other,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[linear_format],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let egui_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Egui Viewport View"),
            format: Some(linear_format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
            usage: None,
        });

        Self {
            texture,
            view,
            egui_view,
            width,
            height,
        }
    }
}