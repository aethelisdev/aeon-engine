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
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        text_system: &mut TextSystem,
        screen_size: [f32; 2],
        sections: &[TextSection<'_>],
    ) {
        let width = screen_size[0].max(1.0) as u32;
        let height = screen_size[1].max(1.0) as u32;

        if self.last_resolution != Some((width, height)) {
            self.viewport.update(queue, Resolution { width, height });
            self.last_resolution = Some((width, height));
        }

        // Clear and rebuild buffers reusing existing allocation capacity
        self.buffers.clear();
        for section in sections {
            let buffer = text_system.shape_text(
                &section.text,
                section.font_size,
                section.line_height,
                section.bounds.width,
                section.bounds.height,
                section.align,
            );
            self.buffers.push(buffer);
        }

        let mut text_areas: Vec<TextArea> = Vec::with_capacity(sections.len());
        for (sec, buf) in sections.iter().zip(self.buffers.iter()) {
            let y_offset = ((sec.bounds.height - sec.line_height) * 0.5).max(0.0);
            let bounds = if let Some(clip) = sec.clip_bounds {
                TextBounds {
                    left: clip.x.max(0.0) as i32,
                    top: clip.y.max(0.0) as i32,
                    right: clip.right().max(0.0) as i32,
                    bottom: clip.bottom().max(0.0) as i32,
                }
            } else {
                TextBounds {
                    left: sec.bounds.x as i32,
                    top: sec.bounds.y as i32,
                    right: (sec.bounds.x + sec.bounds.width.max(200.0)) as i32,
                    bottom: (sec.bounds.y + sec.bounds.height.max(40.0)) as i32,
                }
            };
            text_areas.push(TextArea {
                buffer: buf,
                left: sec.bounds.x,
                top: sec.bounds.y + y_offset,
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