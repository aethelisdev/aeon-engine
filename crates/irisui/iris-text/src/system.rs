// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Central font system, text layout measurement, and shaped glyph caching engine.

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache};
use iris_core::{Size, TextAlign};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MeasureCacheKey {
    text: String,
    font_size_bits: u32,
    line_height_bits: u32,
    max_width_bits: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ShapeCacheKey {
    text: String,
    font_size_bits: u32,
    line_height_bits: u32,
    bounds_width_bits: u32,
    bounds_height_bits: u32,
    align: u8,
}

/// Core text system managing font discovery, shaping caches, and layout measurement.
pub struct TextSystem {
    font_system: FontSystem,
    swash_cache: SwashCache,
    measure_cache: HashMap<MeasureCacheKey, Size>,
    shape_cache: HashMap<ShapeCacheKey, Buffer>,
}

impl Default for TextSystem {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TextSystem {
    /// Initializes a new `TextSystem` loading embedded Noto fonts (standard, symbols, math, and monochrome emojis).
    pub fn new() -> Self {
        let mut db = cosmic_text::fontdb::Database::new();

        // Load Aeon Engine standard embedded fonts for 1-to-1 visual parity with egui
        db.load_font_data(
            include_bytes!("../../../ae_engine/assets/fonts/NotoSans-Regular.ttf").to_vec(),
        );
        db.load_font_data(
            include_bytes!("../../../ae_engine/assets/fonts/NotoSansSymbols-Regular.ttf").to_vec(),
        );
        db.load_font_data(
            include_bytes!("../../../ae_engine/assets/fonts/NotoSansSymbols2-Regular.ttf").to_vec(),
        );
        db.load_font_data(
            include_bytes!("../../../ae_engine/assets/fonts/NotoSansMath-Regular.ttf").to_vec(),
        );
        db.load_font_data(
            include_bytes!("../../../ae_engine/assets/fonts/NotoEmoji-Regular.ttf").to_vec(),
        );

        db.set_sans_serif_family("Noto Sans");
        db.set_serif_family("Noto Sans");
        db.set_monospace_family("Noto Sans");

        let font_system = FontSystem::new_with_locale_and_db("en-US".into(), db);

        Self {
            font_system,
            swash_cache: SwashCache::new(),
            measure_cache: HashMap::with_capacity(256),
            shape_cache: HashMap::with_capacity(256),
        }
    }

    /// Provides mutable access to the underlying `FontSystem`.
    #[inline]
    pub fn font_system_mut(&mut self) -> &mut FontSystem {
        &mut self.font_system
    }

    /// Provides mutable access to the underlying `SwashCache`.
    #[inline]
    pub fn swash_cache_mut(&mut self) -> &mut SwashCache {
        &mut self.swash_cache
    }

    /// Provides simultaneous disjoint mutable access to both `FontSystem` and `SwashCache`.
    #[inline]
    pub fn components_mut(&mut self) -> (&mut FontSystem, &mut SwashCache) {
        (&mut self.font_system, &mut self.swash_cache)
    }

    /// Clears internal shaping and measurement caches if needed.
    #[inline]
    pub fn clear_cache(&mut self) {
        self.measure_cache.clear();
        self.shape_cache.clear();
    }

    /// Measures the dimensions of a text string given font size and line height constraints with caching.
    /// Used by the layout engine to calculate intrinsic widget `content_size`.
    pub fn measure_text(
        &mut self,
        text: &str,
        font_size: f32,
        line_height: f32,
        max_width: Option<f32>,
    ) -> Size {
        if text.is_empty() {
            return Size::ZERO;
        }

        let max_w_bits = max_width.unwrap_or(0.0).to_bits();
        let key = MeasureCacheKey {
            text: text.to_string(),
            font_size_bits: font_size.to_bits(),
            line_height_bits: line_height.to_bits(),
            max_width_bits: max_w_bits,
        };

        if let Some(&cached) = self.measure_cache.get(&key) {
            return cached;
        }

        let metrics = Metrics::new(font_size, line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        buffer.set_size(max_width, None);
        let attrs = Attrs::new().family(Family::Name("Noto Sans"));
        let shaping = if text.is_ascii() {
            Shaping::Basic
        } else {
            Shaping::Advanced
        };
        buffer.set_text(text, &attrs, shaping, None);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut measured_width: f32 = 0.0;
        let mut line_count: usize = 0;

        for run in buffer.layout_runs() {
            measured_width = measured_width.max(run.line_w);
            line_count += 1;
        }

        let measured_height = (line_count as f32) * line_height;
        let size = Size::new(measured_width.ceil(), measured_height.ceil());

        if self.measure_cache.len() > 1024 {
            self.measure_cache.clear();
        }
        self.measure_cache.insert(key, size);
        size
    }

    /// Creates and shapes a `cosmic_text::Buffer` for rendering a text section with caching.
    pub fn shape_text(
        &mut self,
        text: &str,
        font_size: f32,
        line_height: f32,
        bounds_width: f32,
        bounds_height: f32,
        align: TextAlign,
    ) -> Buffer {
        let align_code = match align {
            TextAlign::Left => 0,
            TextAlign::Center => 1,
            TextAlign::Right => 2,
        };

        let key = ShapeCacheKey {
            text: text.to_string(),
            font_size_bits: font_size.to_bits(),
            line_height_bits: line_height.to_bits(),
            bounds_width_bits: bounds_width.to_bits(),
            bounds_height_bits: bounds_height.to_bits(),
            align: align_code,
        };

        if let Some(cached) = self.shape_cache.get(&key) {
            return cached.clone();
        }

        let metrics = Metrics::new(font_size, line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        buffer.set_size(Some(bounds_width.max(1.0)), Some(bounds_height.max(1.0)));

        let cosmic_align = match align {
            TextAlign::Left => cosmic_text::Align::Left,
            TextAlign::Center => cosmic_text::Align::Center,
            TextAlign::Right => cosmic_text::Align::Right,
        };

        let attrs = Attrs::new().family(Family::Name("Noto Sans"));
        let shaping = if text.is_ascii() {
            Shaping::Basic
        } else {
            Shaping::Advanced
        };
        buffer.set_text(text, &attrs, shaping, Some(cosmic_align));

        buffer.shape_until_scroll(&mut self.font_system, false);

        if self.shape_cache.len() > 1024 {
            self.shape_cache.clear();
        }
        self.shape_cache.insert(key, buffer.clone());
        buffer
    }
}