// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Input Box Properties & Slicing Types
//!
//! Provides property descriptors and horizontal visibility calculation helpers for single-line
//! text inputs with clipping, prefix icons, and caret positioning.

/// Configuration parameters for emitting a text input box in [`crate::declarative::UiScope`].
///
/// Encapsulates textual content, sizing constraints, focus state, optional leading icon,
/// and caret animation to prevent excessive argument counts and keep call sites readable and clean.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputBoxProps<'a> {
    /// Current textual content within the input box.
    pub text: &'a str,
    /// Grayed-out hint text displayed when [`Self::text`] is empty.
    pub placeholder: &'a str,
    /// Optional prefix icon glyph (e.g. `"🔍"`) displayed inside the left edge of the input.
    pub icon: Option<&'a str>,
    /// Font size in physical pixels used for dynamic glyph metric calculations.
    pub font_size: f32,
    /// Approximate visual layout width of [`Self::text`] in pixels for caret positioning.
    pub text_width: f32,
    /// Explicit physical width constraint for the input box widget.
    pub width: f32,
    /// Indicates whether this input box currently possesses active keyboard focus.
    pub is_focused: bool,
    /// Controls whether the text caret indicator is rendered visible on the current frame.
    pub cursor_blink_visible: bool,
    /// Optional selection range (start_byte_idx, end_byte_idx) for selection highlight capsule.
    pub selection: Option<(usize, usize)>,
    /// Byte index of the text cursor inside [`Self::text`] for accurate horizontal caret positioning.
    pub cursor_byte_idx: usize,
    /// Optional custom color override for the blinking vertical caret.
    pub caret_color: Option<iris_core::Color>,
    /// Optional semantic tag for the "✖" clear button rendered inside the right edge of the input.
    pub clear_tag: Option<u64>,
}

impl<'a> InputBoxProps<'a> {
    /// Creates a new [`InputBoxProps`] descriptor with default horizontal layout metrics.
    ///
    /// # Arguments
    /// * `text` - Current text value.
    /// * `placeholder` - Grayed-out hint text.
    /// * `width` - Total width constraint of the input box in physical pixels.
    /// * `is_focused` - Focus flag.
    /// * `cursor_blink_visible` - Caret visibility flag.
    pub fn new(
        text: &'a str,
        placeholder: &'a str,
        width: f32,
        is_focused: bool,
        cursor_blink_visible: bool,
    ) -> Self {
        let font_size = 11.0;
        let text_width = iris_text::measure_text_advance(text, font_size);
        let cursor_byte_idx = text.len();
        Self {
            text,
            placeholder,
            icon: None,
            font_size,
            text_width,
            width,
            is_focused,
            cursor_blink_visible,
            selection: None,
            cursor_byte_idx,
            caret_color: None,
            clear_tag: None,
        }
    }

    /// Sets the font size in physical pixels and recomputes the text layout width dynamically.
    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self.text_width = iris_text::measure_text_advance(self.text, font_size);
        self
    }

    /// Attaches an optional prefix icon glyph (e.g. `"🔍"`) to this input box.
    pub fn with_icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Attaches an active text selection range to display a selection highlight capsule.
    pub fn with_selection(mut self, selection: Option<(usize, usize)>) -> Self {
        self.selection = selection;
        self
    }

    /// Explicitly sets the byte offset of the text cursor inside [`Self::text`] for precise caret alignment.
    pub fn with_cursor_byte_idx(mut self, cursor_byte_idx: usize) -> Self {
        self.cursor_byte_idx = cursor_byte_idx;
        self
    }

    /// Overrides the visual color of the vertical blinking caret.
    pub fn with_caret_color(mut self, color: iris_core::Color) -> Self {
        self.caret_color = Some(color);
        self
    }

    /// Attaches an optional clear button tag to display a "✖" glyph on the right edge when text is non-empty.
    pub fn with_clear_tag(mut self, clear_tag: u64) -> Self {
        self.clear_tag = Some(clear_tag);
        self
    }
}

/// Calculates the visible substring range `(start_byte, end_byte)` and caret horizontal offset.
///
/// Ensures the active cursor is strictly visible within `max_visible_w` without overflowing
/// the text input boundaries or overlapping prefix icons or suffix clear buttons.
/// Dynamically calculates glyph advances using the font engine at runtime.
///
/// # Arguments
/// * `text` - Full text buffer of the input box.
/// * `cursor_byte_idx` - Current byte offset of the editing cursor.
/// * `font_size` - Font size in physical pixels for dynamic glyph metric calculation.
/// * `max_visible_w` - Available horizontal space in physical pixels for visible text.
///
/// # Returns
/// A tuple `(visible_start_byte, visible_end_byte, caret_offset_within_visible_slice)`.
pub fn calculate_visible_slice_and_caret(
    text: &str,
    cursor_byte_idx: usize,
    font_size: f32,
    max_visible_w: f32,
) -> (usize, usize, f32) {
    if text.is_empty() || max_visible_w <= 0.0 {
        return (0, 0, 0.0);
    }

    let valid_idx = cursor_byte_idx.min(text.len());
    let valid_idx = if text.is_char_boundary(valid_idx) {
        valid_idx
    } else {
        text.char_indices()
            .map(|(idx, _)| idx)
            .take_while(|&idx| idx <= valid_idx)
            .last()
            .unwrap_or(0)
    };

    let total_w = iris_text::measure_text_advance(text, font_size);
    if total_w <= max_visible_w {
        let caret_offset = iris_text::measure_text_advance(&text[..valid_idx], font_size);
        return (0, text.len(), caret_offset);
    }

    let mut visible_start_byte = 0;
    for (idx, ch) in text[..valid_idx].char_indices().rev() {
        let candidate_slice = &text[idx..valid_idx];
        let slice_w = iris_text::measure_text_advance(candidate_slice, font_size);
        if slice_w > max_visible_w {
            visible_start_byte = idx + ch.len_utf8();
            break;
        }
    }

    let mut visible_end_byte = text.len();
    for (idx, ch) in text[visible_start_byte..].char_indices() {
        let candidate_end = visible_start_byte + idx + ch.len_utf8();
        let candidate_slice = &text[visible_start_byte..candidate_end];
        let slice_w = iris_text::measure_text_advance(candidate_slice, font_size);
        if slice_w > max_visible_w {
            visible_end_byte = visible_start_byte + idx;
            break;
        }
    }

    let caret_offset =
        iris_text::measure_text_advance(&text[visible_start_byte..valid_idx], font_size);
    (visible_start_byte, visible_end_byte, caret_offset)
}