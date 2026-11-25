// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Central font system, text layout measurement, and shaped glyph caching engine.

use crate::cache::{CacheEntry, StringKeyedArenaCache, hash_str_fnv1a};
use crate::caret::{CaretBlinkTimer, CaretBlinkUpdate};
use crate::collector::TextCollectionOptions;
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache};
use irisui_core::{Size, TextAlign, TextWrap, UiTree, WidgetId};
use std::time::{Duration, Instant};

type CaretFocusKey = (Option<WidgetId>, Option<u64>, Option<u64>);

const MAX_METRIC_CACHE_ENTRIES: usize = 1024;
const MAX_SHAPE_CACHE_ENTRIES: usize = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MeasureCacheKey {
    text_hash: u64,
    text_len: usize,
    font_size_bits: u32,
    line_height_bits: u32,
    max_width_bits: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ShapeCacheKey {
    text_hash: u64,
    text_len: usize,
    font_size_bits: u32,
    line_height_bits: u32,
    bounds_width_bits: u32,
    bounds_height_bits: u32,
    align: u8,
    wrap: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AdvanceCacheKey {
    text_hash: u64,
    text_len: usize,
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
    measure_cache: StringKeyedArenaCache<MeasureCacheKey, Size>,
    shape_cache: StringKeyedArenaCache<ShapeCacheKey, Buffer>,
    advance_cache: StringKeyedArenaCache<AdvanceCacheKey, f32>,
    caret_timer: CaretBlinkTimer,
    caret_subms_ns: u128,
    last_caret_tick: Option<Instant>,
    last_blink_focus_key: Option<CaretFocusKey>,
    caret_blink_enabled: bool,
}

impl Default for TextSystem {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TextSystem {
    /// Initializes a new `TextSystem`, optionally loading bundled Noto fonts enabled by Cargo
    /// features (`bundled-sans`, `bundled-symbols`, `bundled-emoji`, all active via default
    /// `bundled-fonts`).
    ///
    /// When all `bundled-*` features are disabled (`default-features = false`), callers can
    /// register fonts dynamically via [`Self::load_static_font`], [`Self::load_font_bytes`],
    /// or [`Self::load_system_fonts`].
    pub fn new() -> Self {
        let mut font_system =
            FontSystem::new_with_locale_and_db("en-US".into(), Self::build_bundled_font_db());
        Self::sync_fallback_family_on_db(font_system.db_mut());

        Self {
            font_system,
            swash_cache: SwashCache::new(),
            default_family: None,
            measure_cache: StringKeyedArenaCache::with_capacity(256),
            shape_cache: StringKeyedArenaCache::with_capacity(256),
            advance_cache: StringKeyedArenaCache::with_capacity(256),
            caret_timer: CaretBlinkTimer::new(),
            caret_subms_ns: 0,
            last_caret_tick: None,
            last_blink_focus_key: None,
            caret_blink_enabled: true,
        }
    }

    #[cfg(any(
        feature = "bundled-sans",
        feature = "bundled-symbols",
        feature = "bundled-emoji"
    ))]
    fn build_bundled_font_db() -> cosmic_text::fontdb::Database {
        let mut db = cosmic_text::fontdb::Database::new();

        #[cfg(feature = "bundled-sans")]
        db.load_font_source(cosmic_text::fontdb::Source::Binary(std::sync::Arc::new(
            include_bytes!("../assets/fonts/NotoSans-Regular.ttf"),
        )));

        #[cfg(feature = "bundled-symbols")]
        for bytes in [
            include_bytes!("../assets/fonts/NotoSansSymbols-Regular.ttf") as &[u8],
            include_bytes!("../assets/fonts/NotoSansSymbols2-Regular.ttf"),
            include_bytes!("../assets/fonts/NotoSansMath-Regular.ttf"),
        ] {
            db.load_font_source(cosmic_text::fontdb::Source::Binary(std::sync::Arc::new(
                bytes,
            )));
        }

        #[cfg(feature = "bundled-emoji")]
        db.load_font_source(cosmic_text::fontdb::Source::Binary(std::sync::Arc::new(
            include_bytes!("../assets/fonts/NotoEmoji-Regular.ttf"),
        )));

        db
    }

    #[cfg(not(any(
        feature = "bundled-sans",
        feature = "bundled-symbols",
        feature = "bundled-emoji"
    )))]
    fn build_bundled_font_db() -> cosmic_text::fontdb::Database {
        cosmic_text::fontdb::Database::new()
    }

    fn sync_fallback_family_on_db(db: &mut cosmic_text::fontdb::Database) {
        let fallback = if cfg!(feature = "bundled-sans")
            && db.faces().any(|f| {
                f.families
                    .iter()
                    .any(|(name, _)| name.eq_ignore_ascii_case("Noto Sans"))
            }) {
            Some(String::from("Noto Sans"))
        } else {
            db.faces()
                .find_map(|face| face.families.first().map(|(name, _)| name.clone()))
        };

        if let Some(family) = fallback {
            db.set_sans_serif_family(family.clone());
            db.set_serif_family(family.clone());
            db.set_monospace_family(family);
        }
    }

    /// Loads a static font binary slice (TTF/OTF/TTC) into the font database and clears caches.
    pub fn load_static_font(&mut self, bytes: &'static [u8]) {
        let db = self.font_system.db_mut();
        db.load_font_source(cosmic_text::fontdb::Source::Binary(std::sync::Arc::new(
            bytes,
        )));
        Self::sync_fallback_family_on_db(db);
        self.clear_cache();
    }

    /// Loads an owned font byte buffer (TTF/OTF/TTC) into the font database and clears caches.
    pub fn load_font_bytes(&mut self, bytes: Vec<u8>) {
        let db = self.font_system.db_mut();
        db.load_font_data(bytes);
        Self::sync_fallback_family_on_db(db);
        self.clear_cache();
    }

    /// Discovers installed OS system fonts into the font database and clears caches.
    pub fn load_system_fonts(&mut self) {
        let db = self.font_system.db_mut();
        db.load_system_fonts();
        Self::sync_fallback_family_on_db(db);
        self.clear_cache();
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

    /// Provides simultaneous disjoint mutable access to `FontSystem`, `SwashCache`, and shaped buffer entries.
    #[inline]
    pub(crate) fn parts_for_prepare(
        &mut self,
    ) -> (&mut FontSystem, &mut SwashCache, &[CacheEntry<Buffer>]) {
        (
            &mut self.font_system,
            &mut self.swash_cache,
            self.shape_cache.entries(),
        )
    }

    /// Prunes the shape cache cleanly before a render prepare frame so indices minted during
    /// that frame remain valid until `parts_for_prepare`.
    #[inline]
    pub(crate) fn begin_prepare_frame(&mut self) {
        if self.shape_cache.len() > MAX_SHAPE_CACHE_ENTRIES {
            self.shape_cache.clear();
        }
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
        if text.is_empty() || self.font_system.db().is_empty() {
            return Size::ZERO;
        }

        let max_w_bits = max_width.unwrap_or(0.0).to_bits();
        let key = MeasureCacheKey {
            text_hash: hash_str_fnv1a(text),
            text_len: text.len(),
            font_size_bits: font_size.to_bits(),
            line_height_bits: line_height.to_bits(),
            max_width_bits: max_w_bits,
        };

        if let Some((_, &cached)) = self.measure_cache.get_indexed(&key, text) {
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

        self.measure_cache
            .insert_bounded(key, text, size, MAX_METRIC_CACHE_ENTRIES);
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

    /// Creates or retrieves a cached shaped `cosmic_text::Buffer`, returning its arena index
    /// without cloning `Buffer` or allocating lookup strings on cache hits.
    ///
    /// Callers executing within a prepare batch should call `begin_prepare_frame` beforehand;
    /// this method preserves all existing indices for the active batch.
    pub fn shape_text_indexed(&mut self, text: &str, params: TextShapeParams) -> usize {
        let key = build_shape_cache_key(text, params);
        if let Some((idx, _)) = self.shape_cache.get_indexed(&key, text) {
            return idx;
        }
        let buffer = self.build_shaped_buffer(text, params);
        self.shape_cache
            .insert_preserving_indices(key, text, buffer)
    }

    /// Returns a borrowed reference to the cached shaped buffer at `index`, if present.
    #[inline]
    #[must_use]
    pub fn shaped_buffer(&self, index: usize) -> Option<&Buffer> {
        self.shape_cache.get_value(index)
    }

    /// Returns a zero-clone reference to a cached shaped `cosmic_text::Buffer`, computing and
    /// caching it on miss with zero `String` allocation on cache hits.
    pub fn shape_text_ref(&mut self, text: &str, params: TextShapeParams) -> &Buffer {
        let key = build_shape_cache_key(text, params);
        let idx = if let Some((existing_idx, _)) = self.shape_cache.get_indexed(&key, text) {
            existing_idx
        } else {
            let buffer = self.build_shaped_buffer(text, params);
            self.shape_cache
                .insert_bounded(key, text, buffer, MAX_SHAPE_CACHE_ENTRIES)
        };
        self.shape_cache
            .get_value(idx)
            .expect("shape cache entry inserted at idx must exist")
    }

    /// Creates and shapes a cloned `cosmic_text::Buffer` for callers requiring ownership.
    pub fn shape_text(&mut self, text: &str, params: TextShapeParams) -> Buffer {
        self.shape_text_ref(text, params).clone()
    }

    fn build_shaped_buffer(&mut self, text: &str, params: TextShapeParams) -> Buffer {
        let metrics = Metrics::new(params.font_size, params.line_height);
        if self.font_system.db().is_empty() {
            return Buffer::new_empty(metrics);
        }
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
        if text.is_empty() || self.font_system.db().is_empty() {
            return 0.0;
        }

        let key = AdvanceCacheKey {
            text_hash: hash_str_fnv1a(text),
            text_len: text.len(),
            font_size_bits: font_size.to_bits(),
        };

        if let Some((_, &cached)) = self.advance_cache.get_indexed(&key, text) {
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

        self.advance_cache
            .insert_bounded(key, text, advance, MAX_METRIC_CACHE_ENTRIES);
        advance
    }

    /// Resolves the nearest UTF-8 character boundary index for a given horizontal pixel offset.
    ///
    /// Evaluates glyph bounding boxes shaped by the font system and computes the closest
    /// caret index based on mid-glyph rounding.
    ///
    /// # Arguments
    /// * `text` - Source text line being hit-tested.
    /// * `font_size` - Physical font size in pixels.
    /// * `local_x` - Horizontal click position relative to the text line origin.
    ///
    /// # Returns
    /// Valid UTF-8 byte offset within `text` (from `0` to `text.len()`).
    pub fn index_for_position(&mut self, text: &str, font_size: f32, local_x: f32) -> usize {
        crate::hit_test::index_for_position_in_system(self, text, font_size, local_x)
    }

    /// Returns an immutable reference to the internal [`CaretBlinkTimer`].
    #[inline]
    #[must_use]
    pub const fn caret_blink_timer(&self) -> &CaretBlinkTimer {
        &self.caret_timer
    }

    /// Returns a mutable reference to the internal [`CaretBlinkTimer`].
    #[inline]
    pub fn caret_blink_timer_mut(&mut self) -> &mut CaretBlinkTimer {
        &mut self.caret_timer
    }

    /// Returns the configured half-cycle caret blink duration in milliseconds.
    #[inline]
    #[must_use]
    pub const fn caret_blink_interval_ms(&self) -> u64 {
        self.caret_timer.interval_ms()
    }

    /// Customizes the half-cycle caret blink interval in milliseconds without hardcoded lock-in.
    ///
    /// # Arguments
    /// * `interval_ms` - Half-period duration in milliseconds (visible duration = hidden duration).
    #[inline]
    pub fn set_caret_blink_interval_ms(&mut self, interval_ms: u64) {
        self.caret_timer.set_interval_ms(interval_ms);
    }

    /// Returns `true` when periodic caret blinking animation is enabled (`false` yields a steady solid caret).
    #[inline]
    #[must_use]
    pub const fn is_caret_blink_enabled(&self) -> bool {
        self.caret_blink_enabled
    }

    /// Enables or disables periodic caret blinking animation (steady solid caret when `false`).
    #[inline]
    pub fn set_caret_blink_enabled(&mut self, enabled: bool) {
        self.caret_blink_enabled = enabled;
        if !enabled {
            self.caret_timer.reset();
            self.caret_subms_ns = 0;
        }
    }

    /// Returns `true` if text input carets are currently in the visible blink phase.
    #[inline]
    #[must_use]
    pub const fn is_caret_visible(&self) -> bool {
        !self.caret_blink_enabled || self.caret_timer.is_visible()
    }

    /// Resets the internal caret blink animation timer so the caret immediately appears visible.
    #[inline]
    pub fn reset_caret_blink(&mut self) {
        self.reset_caret_blink_at(Instant::now());
    }

    /// Resets the internal caret blink animation timer anchored at an explicit monotonic timestamp.
    #[inline]
    pub fn reset_caret_blink_at(&mut self, now: Instant) {
        self.caret_timer.reset();
        self.caret_subms_ns = 0;
        self.last_caret_tick = Some(now);
    }

    /// Advances the internal caret blink timer by `delta`, preserving sub-millisecond remainders
    /// across high-refresh-rate frames, and returns `true` if caret visibility transitioned.
    pub fn advance_caret_blink(&mut self, delta: Duration) -> bool {
        if !self.caret_blink_enabled {
            let was_visible = self.caret_timer.is_visible();
            self.caret_timer.reset();
            self.caret_subms_ns = 0;
            return !was_visible;
        }
        self.caret_subms_ns = self.caret_subms_ns.saturating_add(delta.as_nanos());
        let whole_ms = (self.caret_subms_ns / 1_000_000) as u64;
        self.caret_subms_ns %= 1_000_000;
        if whole_ms == 0 {
            return false;
        }
        self.caret_timer.tick(whole_ms)
    }

    /// Advances the internal caret blink timer using an explicit monotonic timestamp `now`
    /// and returns `true` if caret visibility transitioned.
    pub fn tick_caret_blink_at(&mut self, now: Instant) -> bool {
        if let Some(prev) = self.last_caret_tick {
            let delta = now.saturating_duration_since(prev);
            self.last_caret_tick = Some(now);
            self.advance_caret_blink(delta)
        } else {
            self.reset_caret_blink_at(now);
            false
        }
    }

    /// Advances the internal caret blink timer using the OS monotonic clock (`Instant::now()`)
    /// and returns `true` if caret visibility transitioned.
    #[inline]
    pub fn tick_caret_blink(&mut self) -> bool {
        self.tick_caret_blink_at(Instant::now())
    }

    /// Synchronizes internal caret blink animation against `tree` using an explicit frame `delta`.
    ///
    /// Automatically detects focus transitions, resets the timer to visible on focus gain or
    /// keyboard typing input, and advances periodic blinking otherwise.
    pub fn sync_caret_blink_with_delta(
        &mut self,
        tree: &UiTree,
        delta: Duration,
    ) -> CaretBlinkUpdate {
        self.evaluate_tree_blink(tree, None, Some(delta))
    }

    /// Synchronizes internal caret blink animation against `tree` at an explicit monotonic timestamp `now`.
    pub fn sync_caret_blink_at(&mut self, tree: &UiTree, now: Instant) -> CaretBlinkUpdate {
        self.evaluate_tree_blink(tree, Some(now), None)
    }

    /// Synchronizes internal caret blink animation against `tree` using `Instant::now()`.
    #[inline]
    pub fn sync_caret_blink(&mut self, tree: &UiTree) -> CaretBlinkUpdate {
        self.sync_caret_blink_at(tree, Instant::now())
    }

    /// Builds [`TextCollectionOptions`] pre-populated with focus, hover, and live blink state
    /// directly from `tree` and this `TextSystem` instance.
    #[must_use]
    pub fn text_collection_options(&self, tree: &UiTree) -> TextCollectionOptions {
        TextCollectionOptions::new()
            .with_hovered_id(tree.event_state.hovered)
            .with_focused_id(tree.event_state.focused)
            .with_focused_tag(tree.event_state.focused_tag)
            .with_blink_caret(self.is_caret_visible())
    }

    fn evaluate_tree_blink(
        &mut self,
        tree: &UiTree,
        now: Option<Instant>,
        delta: Option<Duration>,
    ) -> CaretBlinkUpdate {
        let has_text_focus = tree.is_text_input_focused();
        let focus_key = if has_text_focus {
            Some((
                tree.event_state.focused,
                tree.event_state.focused_tag,
                tree.event_state.active_text_edit.as_ref().map(|s| s.tag),
            ))
        } else {
            None
        };
        let focus_changed = focus_key != self.last_blink_focus_key;
        self.last_blink_focus_key = focus_key;

        if !has_text_focus {
            self.caret_timer.reset();
            self.caret_subms_ns = 0;
            if let Some(ts) = now {
                self.last_caret_tick = Some(ts);
            }
            return CaretBlinkUpdate {
                is_visible: false,
                has_text_focus: false,
                focus_changed,
                blink_changed: false,
            };
        }

        let had_typing_input = (tree.event_state.frame_key_pressed
            && tree.event_state.frame_key.is_some())
            || tree.event_state.frame_text_input.is_some();

        if focus_changed || had_typing_input {
            let was_visible = self.is_caret_visible();
            self.caret_timer.reset();
            self.caret_subms_ns = 0;
            if let Some(ts) = now {
                self.last_caret_tick = Some(ts);
            }
            let now_visible = self.is_caret_visible();
            return CaretBlinkUpdate {
                is_visible: now_visible,
                has_text_focus: true,
                focus_changed,
                blink_changed: was_visible != now_visible,
            };
        }

        let blink_changed = if let Some(d) = delta {
            self.advance_caret_blink(d)
        } else if let Some(ts) = now {
            self.tick_caret_blink_at(ts)
        } else {
            false
        };

        CaretBlinkUpdate {
            is_visible: self.is_caret_visible(),
            has_text_focus: true,
            focus_changed: false,
            blink_changed,
        }
    }
}

impl irisui_core::TextMeasurer for TextSystem {
    #[inline]
    fn measure_text_advance(&mut self, text: &str, font_size: f32) -> f32 {
        self.measure_advance(text, font_size)
    }
}

std::thread_local! {
    static SHARED_TEXT_SYSTEM: std::sync::Mutex<TextSystem> = std::sync::Mutex::new(TextSystem::new());
}

/// Measures the dynamic horizontal advance width of a string in physical pixels at the specified font size.
///
/// Backed by a thread-local [`TextSystem`] instance with glyph shaping and metric caching.
/// Thread-safe and free from runtime borrow mutations (`RefCell::borrow_mut`).
///
/// # Arguments
/// * `text` - Text string to measure.
/// * `font_size` - Font size in physical pixels.
///
/// # Returns
/// Total advance width in physical pixels. Returns `0.0` if `text` is empty.
pub fn measure_text_advance(text: &str, font_size: f32) -> f32 {
    SHARED_TEXT_SYSTEM.with(|sys| {
        sys.lock()
            .map(|mut guard| guard.measure_advance(text, font_size))
            .unwrap_or(0.0)
    })
}

/// Configures the default font family name for the thread-local [`TextSystem`] instance.
pub fn set_default_text_family(family: &str) {
    SHARED_TEXT_SYSTEM.with(|sys| {
        if let Ok(mut guard) = sys.lock() {
            guard.set_default_family(family);
        }
    });
}

#[inline]
fn build_shape_cache_key(text: &str, params: TextShapeParams) -> ShapeCacheKey {
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
    ShapeCacheKey {
        text_hash: hash_str_fnv1a(text),
        text_len: text.len(),
        font_size_bits: params.font_size.to_bits(),
        line_height_bits: params.line_height.to_bits(),
        bounds_width_bits: params.bounds_width.to_bits(),
        bounds_height_bits: params.bounds_height.to_bits(),
        align: align_code,
        wrap: wrap_code,
    }
}

#[cfg(test)]
#[path = "system_tests.rs"]
mod tests;