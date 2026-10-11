// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Central font system, text layout measurement, and shaped glyph caching engine.

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache};
use iris_core::{Size, TextAlign, TextWrap, UiTree, WidgetId};
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
    wrap: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AdvanceCacheKey {
    text: String,
    font_size_bits: u32,
}

/// Parameters describing typography metrics, bounding constraints, and alignment for text shaping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextShapeParams {
    /// Font size in pixels.
    pub font_size: f32,
    /// Line height in pixels.
    pub line_height: f32,
    /// Bounding width constraint in pixels.
    pub bounds_width: f32,
    /// Bounding height constraint in pixels.
    pub bounds_height: f32,
    /// Horizontal text alignment.
    pub align: TextAlign,
    /// Text wrapping behavior mode.
    pub wrap: TextWrap,
}

/// Core text system managing font discovery, shaping caches, and layout measurement.
pub struct TextSystem {
    font_system: FontSystem,
    swash_cache: SwashCache,
    default_family: Option<String>,
    measure_cache: HashMap<MeasureCacheKey, Size>,
    shape_cache: HashMap<ShapeCacheKey, Buffer>,
    advance_cache: HashMap<AdvanceCacheKey, f32>,
}

impl Default for TextSystem {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TextSystem {
    /// Initializes a new `TextSystem` loading embedded fallback typography fonts.
    pub fn new() -> Self {
        let mut db = cosmic_text::fontdb::Database::new();

        // Load embedded default fonts for isolated standalone typography rendering
        db.load_font_data(include_bytes!("../assets/fonts/NotoSans-Regular.ttf").to_vec());
        db.load_font_data(include_bytes!("../assets/fonts/NotoSansSymbols-Regular.ttf").to_vec());
        db.load_font_data(include_bytes!("../assets/fonts/NotoSansSymbols2-Regular.ttf").to_vec());
        db.load_font_data(include_bytes!("../assets/fonts/NotoSansMath-Regular.ttf").to_vec());
        db.load_font_data(include_bytes!("../assets/fonts/NotoEmoji-Regular.ttf").to_vec());

        db.set_sans_serif_family("Noto Sans");
        db.set_serif_family("Noto Sans");
        db.set_monospace_family("Noto Sans");

        let font_system = FontSystem::new_with_locale_and_db("en-US".into(), db);

        Self {
            font_system,
            swash_cache: SwashCache::new(),
            default_family: None,
            measure_cache: HashMap::with_capacity(256),
            shape_cache: HashMap::with_capacity(256),
            advance_cache: HashMap::with_capacity(256),
        }
    }

    /// Resolves the default text styling attributes using the active family or generic sans-serif.
    #[inline]
    fn default_attrs(&self) -> Attrs<'_> {
        match &self.default_family {
            Some(family) if !family.is_empty() && !family.eq_ignore_ascii_case("sans-serif") => {
                Attrs::new().family(Family::Name(family))
            }
            _ => Attrs::new().family(Family::SansSerif),
        }
    }

    /// Sets the active default font family used for text shaping and layout measurement.
    ///
    /// If an empty string or `"sans-serif"` is specified, the engine falls back to generic `Family::SansSerif`.
    /// Internal layout and shaping caches are invalidated upon modification.
    ///
    /// # Arguments
    /// * `family` - Target font family name (e.g. `"Inter"`, `"Roboto"`, `"sans-serif"`).
    pub fn set_default_family(&mut self, family: &str) {
        let new_family = if family.is_empty() || family.eq_ignore_ascii_case("sans-serif") {
            None
        } else {
            Some(family.to_string())
        };
        if self.default_family != new_family {
            self.default_family = new_family;
            self.clear_cache();
        }
    }

    /// Returns the active default font family name if explicitly configured, or `"sans-serif"`.
    #[inline]
    pub fn default_family(&self) -> &str {
        self.default_family.as_deref().unwrap_or("sans-serif")
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
        self.advance_cache.clear();
    }

    /// Measures the dimensions of a text string given font size and line height constraints with caching.
    ///
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
        let attrs = self.default_attrs();
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

    /// Measures intrinsic text dimensions for all nodes with text content in the subtree.
    ///
    /// Traverses the subtree starting from `root`, calculates formatted dimensions using
    /// the internal font system and measurement cache, and updates `node.content_size`.
    ///
    /// # Arguments
    /// * `tree` - Mutable reference to the UI node arena.
    /// * `root` - Root widget identifier of the subtree to measure.
    pub fn measure_subtree_text(&mut self, tree: &mut UiTree, root: WidgetId) {
        tree.traverse_depth_first_mut(root, &mut |_id, node| {
            if let Some(ref text) = node.text {
                let measured = self.measure_text(text, node.font_size, node.line_height, None);
                node.content_size = measured;
            }
        });
    }

    /// Creates and shapes a `cosmic_text::Buffer` for rendering a text section with caching.
    pub fn shape_text(&mut self, text: &str, params: TextShapeParams) -> Buffer {
        let align_code = match params.align {
            TextAlign::Left => 0,
            TextAlign::Center => 1,
            TextAlign::Right => 2,
        };

        let wrap_code = match params.wrap {
            TextWrap::Auto => 0,
            TextWrap::None => 1,
            TextWrap::Word => 2,
        };

        let key = ShapeCacheKey {
            text: text.to_string(),
            font_size_bits: params.font_size.to_bits(),
            line_height_bits: params.line_height.to_bits(),
            bounds_width_bits: params.bounds_width.to_bits(),
            bounds_height_bits: params.bounds_height.to_bits(),
            align: align_code,
            wrap: wrap_code,
        };

        if let Some(cached) = self.shape_cache.get(&key) {
            return cached.clone();
        }

        let metrics = Metrics::new(params.font_size, params.line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        buffer.set_size(
            Some(params.bounds_width.max(1.0)),
            Some(params.bounds_height.max(1.0)),
        );

        let should_wrap = match params.wrap {
            TextWrap::None => false,
            TextWrap::Word => true,
            TextWrap::Auto => params.bounds_height >= params.line_height * 1.4,
        };

        let cosmic_wrap = if should_wrap {
            cosmic_text::Wrap::Word
        } else {
            cosmic_text::Wrap::None
        };
        buffer.set_wrap(cosmic_wrap);

        let cosmic_align = match params.align {
            TextAlign::Left => cosmic_text::Align::Left,
            TextAlign::Center => cosmic_text::Align::Center,
            TextAlign::Right => cosmic_text::Align::Right,
        };

        let attrs = self.default_attrs();
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

    /// Measures the exact horizontal layout advance of a single line of text dynamically.
    ///
    /// Shapes the glyphs at the requested font size and computes the total horizontal advance
    /// width in physical pixels. Results are cached dynamically.
    ///
    /// # Arguments
    /// * `text` - Text string to shape and measure.
    /// * `font_size` - Target font size in physical pixels.
    ///
    /// # Returns
    /// Horizontal advance width in pixels. Returns `0.0` if `text` is empty.
    pub fn measure_advance(&mut self, text: &str, font_size: f32) -> f32 {
        if text.is_empty() {
            return 0.0;
        }

        let key = AdvanceCacheKey {
            text: text.to_string(),
            font_size_bits: font_size.to_bits(),
        };

        if let Some(&cached) = self.advance_cache.get(&key) {
            return cached;
        }

        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(None, None);
        let attrs = self.default_attrs();
        let shaping = if text.is_ascii() {
            Shaping::Basic
        } else {
            Shaping::Advanced
        };
        buffer.set_text(text, &attrs, shaping, None);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let advance = buffer.layout_runs().next().map_or(0.0, |run| run.line_w);

        if self.advance_cache.len() > 1024 {
            self.advance_cache.clear();
        }
        self.advance_cache.insert(key, advance);
        advance
    }
}

impl iris_core::TextMeasurer for TextSystem {
    #[inline]
    fn measure_text_advance(&mut self, text: &str, font_size: f32) -> f32 {
        self.measure_advance(text, font_size)
    }
}

std::thread_local! {
    static SHARED_TEXT_SYSTEM: std::cell::RefCell<TextSystem> = std::cell::RefCell::new(TextSystem::new());
}

/// Measures the dynamic horizontal advance width of a string in physical pixels at the specified font size.
///
/// Backed by a thread-local [`TextSystem`] instance with glyph shaping and metric caching.
///
/// # Arguments
/// * `text` - Text string to measure.
/// * `font_size` - Font size in physical pixels.
///
/// # Returns
/// Total advance width in physical pixels. Returns `0.0` if `text` is empty.
pub fn measure_text_advance(text: &str, font_size: f32) -> f32 {
    SHARED_TEXT_SYSTEM.with(|sys| sys.borrow_mut().measure_advance(text, font_size))
}

/// Configures the default font family name for the thread-local [`TextSystem`] instance.
pub fn set_default_text_family(family: &str) {
    SHARED_TEXT_SYSTEM.with(|sys| sys.borrow_mut().set_default_family(family));
}