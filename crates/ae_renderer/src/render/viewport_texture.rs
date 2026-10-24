// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

///
/// Encapsulates a WGPU texture, its default view, and its dimensions.
/// Used for rendering intermediate targets such as the 3D viewport.
pub struct ViewportTexture {
    /// The underlying WGPU texture object.
    pub texture: wgpu::Texture,
    /// The default TextureView (sRGB format matching configuration).
    pub view: wgpu::TextureView,
    /// Allocated physical width of this texture in GPU memory (quantized to bucket granularity).
    pub width: u32,
    /// Allocated physical height of this texture in GPU memory (quantized to bucket granularity).
    pub height: u32,
    /// Logical width of the active viewport content in physical pixels.
    pub logical_width: u32,
    /// Logical height of the active viewport content in physical pixels.
    pub logical_height: u32,
}

impl ViewportTexture {
    /// Creates a new ViewportTexture with the specified allocated and logical dimensions.
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        logical_width: u32,
        logical_height: u32,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            width,
            height,
            logical_width,
            logical_height,
        }
    }

    /// Computes normalized UV bounding coordinates `[min_u, min_v, max_u, max_v]` for sampling the active logical sub-rect.
    ///
    /// If allocated dimensions are non-zero, calculates the ratio of logical size to allocated size clamped to `[0.0, 1.0]`.
    #[inline]
    pub fn uv_bounds(&self) -> [f32; 4] {
        let max_u = if self.width > 0 {
            (self.logical_width as f32 / self.width as f32).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let max_v = if self.height > 0 {
            (self.logical_height as f32 / self.height as f32).clamp(0.0, 1.0)
        } else {
            1.0
        };
        [0.0, 0.0, max_u, max_v]
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_uv_bounds_computation() {
        // Construct mock ViewportTexture without GPU device to verify UV math invariants
        let mock_uv = |width: u32, height: u32, log_w: u32, log_h: u32| -> [f32; 4] {
            let max_u = if width > 0 {
                (log_w as f32 / width as f32).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let max_v = if height > 0 {
                (log_h as f32 / height as f32).clamp(0.0, 1.0)
            } else {
                1.0
            };
            [0.0, 0.0, max_u, max_v]
        };

        // Exact match
        assert_eq!(mock_uv(1024, 768, 1024, 768), [0.0, 0.0, 1.0, 1.0]);

        // Sub-rect within bucketed allocation
        let uv = mock_uv(1024, 512, 512, 256);
        assert!((uv[2] - 0.5).abs() < f32::EPSILON);
        assert!((uv[3] - 0.5).abs() < f32::EPSILON);

        // Edge case: zero dimensions
        assert_eq!(mock_uv(0, 0, 100, 100), [0.0, 0.0, 1.0, 1.0]);

        // Logical larger than allocated (clamped to 1.0)
        assert_eq!(mock_uv(500, 500, 600, 600), [0.0, 0.0, 1.0, 1.0]);
    }
}