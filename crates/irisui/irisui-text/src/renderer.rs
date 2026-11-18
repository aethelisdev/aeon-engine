// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU text rendering pass integrating with WGPU and Glyphon text atlas.

use crate::section::TextSection;
use crate::system::TextSystem;
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
    buffer_indices: Vec<usize>,
    last_resolution: Option<(u32, u32)>,
    has_prepared_text: bool,
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
            buffer_indices: Vec::with_capacity(128),
            last_resolution: None,
            has_prepared_text: false,
        }
    }

    /// Prepares text buffers and uploads font glyphs to the GPU text atlas.
    ///
    /// `physical_screen_size` defines the physical target framebuffer resolution (e.g. 1920x1080).
    /// `zoom_factor` specifies the active UI scaling factor (e.g. 1.25, 1.50) to render
    /// font glyphs at exact 1:1 physical pixel resolution without magnification blur or distortion.
    ///
    /// Shapes or resolves cached glyph buffers directly inside `text_system` with zero
    /// `cosmic_text::Buffer` cloning.
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

        text_system.begin_prepare_frame();
        self.buffer_indices.clear();
        for section in sections {
            let idx = text_system.shape_text_indexed(
                &section.text,
                crate::system::TextShapeParams {
                    font_size: section.font_size * zoom,
                    line_height: section.line_height * zoom,
                    bounds_width: section.bounds.width * zoom,
                    bounds_height: section.bounds.height * zoom,
                    align: section.align,
                    wrap: section.wrap,
                },
            );
            self.buffer_indices.push(idx);
        }

        let (font_sys, swash_c, shape_entries) = text_system.parts_for_prepare();
        let max_right = width as i32;
        let max_bottom = height as i32;
        let mut text_areas: Vec<TextArea<'_>> = Vec::with_capacity(sections.len());
        for (sec, &buf_idx) in sections.iter().zip(self.buffer_indices.iter()) {
            let buf = &shape_entries[buf_idx].value;
            let scaled_bounds_h = sec.bounds.height * zoom;
            let scaled_line_h = sec.line_height * zoom;
            let line_count = buf.layout_runs().count().max(1) as f32;
            let total_text_h = line_count * scaled_line_h;
            let y_offset = ((scaled_bounds_h - total_text_h) * 0.5).max(0.0);
            let raw_clip = sec.clip_bounds.unwrap_or(sec.bounds);
            let bounds = TextBounds {
                left: (raw_clip.x * zoom).clamp(0.0, max_right as f32) as i32,
                top: (raw_clip.y * zoom).clamp(0.0, max_bottom as f32) as i32,
                right: (raw_clip.right() * zoom).clamp(0.0, max_right as f32) as i32,
                bottom: (raw_clip.bottom() * zoom).clamp(0.0, max_bottom as f32) as i32,
            };
            if bounds.right <= bounds.left || bounds.bottom <= bounds.top {
                continue;
            }
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

        self.has_prepared_text = !text_areas.is_empty();
        if !self.has_prepared_text {
            return;
        }
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
        if !self.has_prepared_text {
            return;
        }
        let _ = self
            .text_renderer
            .render(&self.text_atlas, &self.viewport, render_pass);
    }
}

/// Two-layer GPU text renderer isolating static panel labels from high-frequency volatile
/// telemetry and canvas text (`Camera HUD`, `Compass`, live `Stats` readouts).
///
/// Updating the volatile layer leaves the static panel layer's prepared glyph buffers and
/// atlas geometry completely untouched in GPU memory.
pub struct LayeredTextRenderer {
    static_layer: TextRenderer,
    volatile_layer: TextRenderer,
}

impl LayeredTextRenderer {
    /// Creates a new [`LayeredTextRenderer`] with isolated static and volatile GPU text layers.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            static_layer: TextRenderer::new(device, queue, target_format),
            volatile_layer: TextRenderer::new(device, queue, target_format),
        }
    }

    /// Returns an immutable reference to the static panel text renderer.
    #[inline]
    #[must_use]
    pub fn static_renderer(&self) -> &TextRenderer {
        &self.static_layer
    }

    /// Returns an immutable reference to the volatile/live text renderer.
    #[inline]
    #[must_use]
    pub fn volatile_renderer(&self) -> &TextRenderer {
        &self.volatile_layer
    }

    /// Re-prepares the static text layer without modifying the volatile layer.
    pub fn prepare_static(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        text_system: &mut TextSystem,
        physical_screen_size: (u32, u32),
        zoom_factor: f32,
        sections: &[TextSection<'_>],
    ) {
        self.static_layer.prepare(
            device,
            queue,
            text_system,
            physical_screen_size,
            zoom_factor,
            sections,
        );
    }

    /// Re-prepares only the volatile/live text layer, leaving static panel glyphs untouched.
    pub fn prepare_volatile(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        text_system: &mut TextSystem,
        physical_screen_size: (u32, u32),
        zoom_factor: f32,
        sections: &[TextSection<'_>],
    ) {
        self.volatile_layer.prepare(
            device,
            queue,
            text_system,
            physical_screen_size,
            zoom_factor,
            sections,
        );
    }

    /// Renders both the static and volatile text layers into `render_pass`.
    pub fn render<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        self.static_layer.render(render_pass);
        self.volatile_layer.render(render_pass);
    }
}