// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU text rendering pass integrating with WGPU and Glyphon text atlas.

use crate::section::TextSection;
use crate::system::TextSystem;
use cosmic_text::Buffer;
use glyphon::{
    Cache, Resolution, TextArea, TextAtlas, TextBounds, TextRenderer as GlyphonTextRenderer,
    Viewport,
};

/// GPU text renderer backed by Glyphon and Cosmic-Text.
pub struct TextRenderer {
    text_renderer: GlyphonTextRenderer,
    text_atlas: TextAtlas,
    viewport: Viewport,
    _cache: Cache,
    buffers: Vec<Buffer>,
    last_resolution: Option<(u32, u32)>,
}

impl TextRenderer {
    /// Creates a new `TextRenderer` for the given WGPU device, queue, and surface format.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let cache = Cache::new(device);
        let mut text_atlas = TextAtlas::new(device, queue, &cache, target_format);
        let text_renderer = GlyphonTextRenderer::new(
            &mut text_atlas,
            device,
            wgpu::MultisampleState::default(),
            None,
        );
        let viewport = Viewport::new(device, &cache);

        Self {
            text_renderer,
            text_atlas,
            viewport,
            _cache: cache,
            buffers: Vec::with_capacity(128),
            last_resolution: None,
        }
    }

    /// Prepares text buffers and uploads font glyphs to the GPU text atlas.
    /// `physical_screen_size` defines the physical target framebuffer resolution (e.g. 1920x1080).
    /// `zoom_factor` specifies the active UI scaling factor (e.g. 1.25, 1.50) to render
    /// font glyphs at exact 1:1 physical pixel resolution without magnification blur or distortion.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        text_system: &mut TextSystem,
        physical_screen_size: (u32, u32),
        zoom_factor: f32,
        sections: &[TextSection<'_>],
    ) {
        let width = physical_screen_size.0.max(1);
        let height = physical_screen_size.1.max(1);
        let zoom = if zoom_factor.is_finite() && zoom_factor > 0.1 {
            zoom_factor
        } else {
            1.0
        };

        if self.last_resolution != Some((width, height)) {
            self.viewport.update(queue, Resolution { width, height });
            self.last_resolution = Some((width, height));
        }

        // Clear and rebuild buffers reusing existing allocation capacity
        self.buffers.clear();
        for section in sections {
            let buffer = text_system.shape_text(
                &section.text,
                section.font_size * zoom,
                section.line_height * zoom,
                section.bounds.width * zoom,
                section.bounds.height * zoom,
                section.align,
            );
            self.buffers.push(buffer);
        }

        let mut text_areas: Vec<TextArea> = Vec::with_capacity(sections.len());
        for (sec, buf) in sections.iter().zip(self.buffers.iter()) {
            let scaled_bounds_h = sec.bounds.height * zoom;
            let scaled_line_h = sec.line_height * zoom;
            let y_offset = ((scaled_bounds_h - scaled_line_h) * 0.5).max(0.0);
            let bounds = if let Some(clip) = sec.clip_bounds {
                TextBounds {
                    left: (clip.x * zoom).max(0.0) as i32,
                    top: (clip.y * zoom).max(0.0) as i32,
                    right: (clip.right() * zoom).max(0.0) as i32,
                    bottom: (clip.bottom() * zoom).max(0.0) as i32,
                }
            } else {
                TextBounds {
                    left: (sec.bounds.x * zoom).max(0.0) as i32,
                    top: (sec.bounds.y * zoom).max(0.0) as i32,
                    right: ((sec.bounds.x + sec.bounds.width.max(200.0)) * zoom) as i32,
                    bottom: ((sec.bounds.y + sec.bounds.height.max(40.0)) * zoom) as i32,
                }
            };
            text_areas.push(TextArea {
                buffer: buf,
                left: (sec.bounds.x * zoom).round(),
                top: ((sec.bounds.y * zoom) + y_offset).round(),
                scale: 1.0,
                bounds,
                default_color: cosmic_text::Color::rgba(
                    (sec.color.r * 255.0) as u8,
                    (sec.color.g * 255.0) as u8,
                    (sec.color.b * 255.0) as u8,
                    (sec.color.a * 255.0) as u8,
                ),
                custom_glyphs: &[],
            });
        }

        let (font_sys, swash_c) = text_system.components_mut();

        let _ = self.text_renderer.prepare(
            device,
            queue,
            font_sys,
            &mut self.text_atlas,
            &self.viewport,
            text_areas,
            swash_c,
        );
    }

    /// Renders all prepared text glyphs into the active WGPU render pass.
    pub fn render<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        let _ = self
            .text_renderer
            .render(&self.text_atlas, &self.viewport, render_pass);
    }
}