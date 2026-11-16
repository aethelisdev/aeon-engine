// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Input Box Properties & Slicing Types
//!
//! Provides property descriptors and horizontal visibility calculation helpers for single-line
//! text inputs with clipping, prefix icons, and caret positioning.

use irisui_core::{Color, Theme};

/// Customizable visual style descriptor for text input boxes, derived from [`Theme`] tokens by default.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputBoxStyle {
    /// Background color when idle.
    pub background: Color,
    /// Background color when hovered.
    pub hover_background: Color,
    /// Background color when focused.
    pub focus_background: Color,
    /// Border stroke color when idle.
    pub border_color: Color,
    /// Border stroke color when hovered.
    pub hover_border_color: Color,
    /// Border stroke color when focused.
    pub focus_border_color: Color,
    /// Primary text color when non-empty.
    pub text_color: Color,
    /// Placeholder text color when empty.
    pub placeholder_color: Color,
    /// Leading icon glyph color.
    pub icon_color: Color,
    /// Clear button text color when idle.
    pub clear_color: Color,
    /// Clear button text color when hovered.
    pub clear_hover_color: Color,
    /// Caret line color.
    pub caret_color: Color,
    /// Highlight background color behind selected text spans.
    pub selection_background: Color,
    /// Input box container height in logical pixels.
    pub height: f32,
    /// Container corner radius in logical pixels.
    pub corner_radius: f32,
    /// Caret vertical bar width in logical pixels.
    pub caret_width: f32,
    /// Caret vertical bar height in logical pixels.
    pub caret_height: f32,
}

impl InputBoxStyle {
    /// Derives a complete input box visual style from the provided [`Theme`] tokens.
    #[must_use]
    pub const fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.toolbar_bg,
            hover_background: theme.button_hover_bg,
            focus_background: theme.card_bg,
            border_color: theme.panel_border,
            hover_border_color: theme.button_hover_border,
            focus_border_color: theme.accent,
            text_color: theme.text_primary,
            placeholder_color: theme.text_muted,
            icon_color: theme.text_muted,
            clear_color: theme.text_muted,
            clear_hover_color: theme.text_primary,
            caret_color: theme.text_caret,
            selection_background: theme.selection_background,
            height: 24.0,
            corner_radius: theme.corner_radius_md,
            caret_width: 1.5,
            caret_height: 14.0,
        }
    }

    /// Overrides the idle background color.
    #[must_use]
    pub const fn with_background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Overrides the focused border accent color.
    #[must_use]
    pub const fn with_focus_border_color(mut self, color: Color) -> Self {
        self.focus_border_color = color;
        self
    }

    /// Overrides the vertical caret color.
    #[must_use]
    pub const fn with_caret_color(mut self, color: Color) -> Self {
        self.caret_color = color;
        self
    }

    /// Overrides primary and placeholder text colors.
    #[must_use]
    pub const fn with_text_colors(mut self, text: Color, placeholder: Color) -> Self {
        self.text_color = text;
        self.placeholder_color = placeholder;
        self
    }

    /// Overrides selection highlight background color.
    #[must_use]
    pub const fn with_selection_background(mut self, color: Color) -> Self {
        self.selection_background = color;
        self
    }

    /// Overrides box height and corner radius in logical pixels.
    #[must_use]
    pub const fn with_dimensions(mut self, height: f32, corner_radius: f32) -> Self {
        self.height = height;
        self.corner_radius = corner_radius;
        self
    }

    /// Overrides caret bar dimensions in logical pixels.
    #[must_use]
    pub const fn with_caret_size(mut self, width: f32, height: f32) -> Self {
        self.caret_width = width;
        self.caret_height = height;
        self
    }
}

impl Default for InputBoxStyle {
    #[inline]
    fn default() -> Self {
        Self::from_theme(&Theme::dark_default())
    }
}

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
    pub caret_color: Option<irisui_core::Color>,
    /// Optional custom visual style descriptor overriding [`Theme`] tokens.
    pub style: Option<InputBoxStyle>,
    /// Optional semantic tag for the "✖" clear button rendered inside the right edge of the input.
    pub clear_tag: Option<u64>,
    /// Whether clipboard shortcuts (`Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V`) are enabled for this input box.
    pub clipboard_shortcuts_enabled: bool,
    /// Whether autonomous undo/redo shortcuts (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`) are enabled for this input box.
    pub undo_shortcuts_enabled: bool,
}

impl<'a> InputBoxProps<'a> {
    /// Creates a new [`InputBoxProps`] descriptor using autonomous retained caret emission
    /// (`cursor_blink_visible = true`), deferring blink phase timing to `irisui_text::TextSystem`
    /// and GPU compilation options without manual per-frame booleans or tree rebuilds.
    pub fn focused(text: &'a str, placeholder: &'a str, width: f32, is_focused: bool) -> Self {
        Self::new(text, placeholder, width, is_focused, true)
    }

    /// Creates a new [`InputBoxProps`] descriptor with default horizontal layout metrics and shortcuts enabled.
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
        let text_width = irisui_text::measure_text_advance(text, font_size);
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
            style: None,
            clear_tag: None,
            clipboard_shortcuts_enabled: true,
            undo_shortcuts_enabled: true,
        }
    }

    /// Attaches a custom [`InputBoxStyle`] descriptor overriding active theme colors and metrics.
    pub const fn with_style(mut self, style: InputBoxStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Sets explicit caret visibility when overriding retained blink compiler emission.
    pub const fn with_blink_caret(mut self, visible: bool) -> Self {
        self.cursor_blink_visible = visible;
        self
    }

    /// Sets the font size in physical pixels and recomputes the text layout width dynamically.
    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self.text_width = irisui_text::measure_text_advance(self.text, font_size);
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
    pub fn with_caret_color(mut self, color: irisui_core::Color) -> Self {
        self.caret_color = Some(color);
        self
    }

    /// Attaches an optional clear button tag to display a "✖" glyph on the right edge when text is non-empty.
    pub fn with_clear_tag(mut self, clear_tag: u64) -> Self {
        self.clear_tag = Some(clear_tag);
        self
    }

    /// Sets whether clipboard keyboard shortcuts (`Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V`) are enabled.
    pub fn with_clipboard_shortcuts(mut self, enabled: bool) -> Self {
        self.clipboard_shortcuts_enabled = enabled;
        self
    }

    /// Sets whether undo and redo keyboard shortcuts (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`) are enabled.
    pub fn with_undo_shortcuts(mut self, enabled: bool) -> Self {
        self.undo_shortcuts_enabled = enabled;
        self
    }

    /// Sets whether both clipboard and undo/redo keyboard shortcuts are enabled.
    pub fn with_shortcuts(mut self, enabled: bool) -> Self {
        self.clipboard_shortcuts_enabled = enabled;
        self.undo_shortcuts_enabled = enabled;
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

    let total_w = irisui_text::measure_text_advance(text, font_size);
    if total_w <= max_visible_w {
        let caret_offset = irisui_text::measure_text_advance(&text[..valid_idx], font_size);
        return (0, text.len(), caret_offset);
    }

    let mut visible_start_byte = 0;
    for (idx, ch) in text[..valid_idx].char_indices().rev() {
        let candidate_slice = &text[idx..valid_idx];
        let slice_w = irisui_text::measure_text_advance(candidate_slice, font_size);
        if slice_w > max_visible_w {
            visible_start_byte = idx + ch.len_utf8();
            break;
        }
    }

    let mut visible_end_byte = text.len();
    for (idx, ch) in text[visible_start_byte..].char_indices() {
        let candidate_end = visible_start_byte + idx + ch.len_utf8();
        let candidate_slice = &text[visible_start_byte..candidate_end];
        let slice_w = irisui_text::measure_text_advance(candidate_slice, font_size);
        if slice_w > max_visible_w {
            visible_end_byte = visible_start_byte + idx;
            break;
        }
    }

    let caret_offset =
        irisui_text::measure_text_advance(&text[visible_start_byte..valid_idx], font_size);
    (visible_start_byte, visible_end_byte, caret_offset)
}