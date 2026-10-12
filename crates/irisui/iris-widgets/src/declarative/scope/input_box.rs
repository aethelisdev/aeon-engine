// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Text Input Box Widget Primitives
//!
//! Provides text input rendering, caret positioning, visible text slicing, and selection highlight
//! primitives on [`UiScope`] for search inputs, text fields, and inspector inline string editing.
//!

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::event::KeyCode;
use iris_core::{
    AlignItems, Color, JustifyContent, Style, TextAlign, WidgetCursor, WidgetId, WidgetRole,
};

pub use iris_text::measure_text_advance;

impl<'a> UiScope<'a> {
    /// Emits a single-line declarative text input field bound to a mutable string buffer in-place.
    ///
    /// Automatically manages keyboard focus, text insertion, backspace deletion,
    /// and cursor rendering. Generates a deterministic persistent tag from `placeholder`.
    ///
    /// # Arguments
    /// * `placeholder` - Prompt string displayed when `*text` is empty.
    /// * `text` - Mutable reference to the underlying string buffer.
    ///
    /// # Returns
    /// A [`WidgetResponse`] reflecting hovered, clicked, focused, and changed state flags.
    pub fn text_input(
        &mut self,
        placeholder: impl Into<String>,
        text: &mut String,
    ) -> WidgetResponse {
        self.text_input_with_width(placeholder, 0.0, text)
    }

    /// Emits a single-line declarative text input field constrained to an explicit width.
    ///
    /// Generates a deterministic persistent tag from `placeholder` and delegates to [`Self::text_input_with_width_tagged`].
    ///
    /// # Arguments
    /// * `placeholder` - Prompt string displayed when `*text` is empty.
    /// * `width` - Explicit width constraint in physical pixels (uses `flex_grow = 1.0` if `<= 0.0`).
    /// * `text` - Mutable reference to the underlying string buffer.
    ///
    /// # Returns
    /// A [`WidgetResponse`] reflecting hovered, clicked, focused, and changed state flags.
    pub fn text_input_with_width(
        &mut self,
        placeholder: impl Into<String>,
        width: f32,
        text: &mut String,
    ) -> WidgetResponse {
        let placeholder_str = placeholder.into();
        let (visible, _) = crate::declarative::types::split_label_id(&placeholder_str);
        let tag = self.tag_for(&placeholder_str);
        self.text_input_with_width_tagged(visible, tag, width, text)
    }

    /// Emits a single-line declarative text input field with an explicit semantic tag.
    ///
    /// # Arguments
    /// * `placeholder` - Prompt string displayed when `*text` is empty.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    /// * `text` - Mutable reference to the underlying string buffer.
    ///
    /// # Returns
    /// A [`WidgetResponse`] reflecting hovered, clicked, focused, and changed state flags.
    pub fn text_input_tagged(
        &mut self,
        placeholder: impl Into<String>,
        tag: u64,
        text: &mut String,
    ) -> WidgetResponse {
        self.text_input_with_width_tagged(placeholder, tag, 0.0, text)
    }

    /// Emits a single-line declarative text input field with an explicit semantic tag and fixed width.
    ///
    /// Tracks click interactions to gain focus, processes incoming keyboard characters
    /// and Backspace/Delete events when focused, updates `*text` in-place, and renders
    /// an interactive text input with focus outline and blinking caret.
    ///
    /// # Arguments
    /// * `placeholder` - Prompt string displayed when `*text` is empty.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    /// * `width` - Explicit width constraint in physical pixels (uses `flex_grow = 1.0` if `<= 0.0`).
    /// * `text` - Mutable reference to the underlying string buffer.
    ///
    /// # Returns
    /// A [`WidgetResponse`] reflecting hovered, clicked, focused, and changed state flags.
    pub fn text_input_with_width_tagged(
        &mut self,
        placeholder: impl Into<String>,
        tag: u64,
        width: f32,
        text: &mut String,
    ) -> WidgetResponse {
        let node_id = self.tree.create_node();
        let placeholder_str = placeholder.into();

        let _ = self.tree.add_child(self.parent, node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("TextInput");
            node.tag = tag;
        }

        let (clicked, hovered, _) = self.check_interaction(node_id);
        if clicked {
            self.tree.event_state.focused = Some(node_id);
            self.tree.event_state.focused_tag = if tag != 0 { Some(tag) } else { None };
        }

        let is_focused = self.is_focused(node_id);
        let mut changed = false;

        if is_focused {
            if let Some(ref incoming) = self.tree.event_state.frame_text_input {
                text.push_str(incoming);
                changed = true;
            } else if let Some(key) = self.tree.event_state.frame_key
                && self.tree.event_state.frame_key_pressed
            {
                match key {
                    KeyCode::Backspace => {
                        if text.pop().is_some() {
                            changed = true;
                        }
                    }
                    KeyCode::Escape => {
                        self.tree.event_state.clear_focus();
                    }
                    _ => {}
                }
            }

            if !changed && let Some(incoming) = self.check_interaction_text(node_id) {
                text.push_str(incoming);
                changed = true;
            }
        }

        let border_color = if is_focused {
            Color::rgba(0.0, 0.90, 1.0, 0.95)
        } else if hovered {
            Color::rgba(0.35, 0.40, 0.52, 0.95)
        } else {
            Color::rgba(0.20, 0.23, 0.30, 0.60)
        };

        let bg_color = if hovered || is_focused {
            Color::rgba(0.08, 0.09, 0.12, 0.98)
        } else {
            Color::rgba(0.06, 0.07, 0.09, 0.95)
        };

        let mut box_style = Style::new()
            .height(24.0)
            .border_radius(4.0)
            .border(1.0, border_color)
            .hover_border(1.0, Color::rgba(0.35, 0.40, 0.52, 0.95))
            .focus_border(1.0, Color::rgba(0.0, 0.90, 1.0, 0.95))
            .background(bg_color)
            .hover_background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .focus_background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .clip_children(true);

        if width > 0.0 {
            box_style = box_style.width(width);
        } else {
            box_style = box_style.flex_grow(1.0);
        }

        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::TextInput;
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Text);
            node.set_style(box_style);
        }

        let font_size = 11.0;
        let text_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(text_id) {
            node.role = WidgetRole::Default;
            node.tag = tag;
            let display_str = if text.is_empty() {
                placeholder_str.as_str()
            } else {
                text.as_str()
            };
            node.set_text(display_str);
            node.font_size = font_size;
            node.line_height = 24.0;
            node.text_align = TextAlign::Left;
            node.text_color = if text.is_empty() {
                Color::rgba(0.42, 0.45, 0.55, 1.0)
            } else {
                Color::rgba(0.92, 0.94, 0.98, 1.0)
            };
            node.focus_text_color = Some(Color::rgba(0.92, 0.94, 0.98, 1.0));
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(8.0)
                    .top(0.0)
                    .height(24.0),
            );
        }
        let _ = self.tree.add_child(node_id, text_id);

        if is_focused {
            let text_w = iris_text::measure_text_advance(text, font_size);
            let caret_x = 8.0 + text_w;
            let caret_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(caret_id) {
                node.set_name("TextInputCaret");
                node.role = WidgetRole::TextInputCaret;
                node.interactive = false;
                node.set_style(
                    Style::new()
                        .position_absolute()
                        .left(caret_x)
                        .top(5.0)
                        .width(1.5)
                        .height(14.0)
                        .background(Color::rgba(0.0, 0.90, 1.0, 0.95))
                        .border_radius(0.75),
                );
            }
            let _ = self.tree.add_child(node_id, caret_id);
        }

        let mut resp = WidgetResponse::new(node_id, clicked, hovered, changed);
        if is_focused {
            resp = resp.with_focused(true);
        }
        resp
    }
    /// Emits a styled text input display box with placeholder support and an optional blinking caret.
    ///
    /// # Arguments
    /// * `text` - Currently entered string value.
    /// * `placeholder` - Prompt string displayed when `text` is empty.
    /// * `text_width` - Measured width of the active text string for caret positioning.
    /// * `cursor_blink_visible` - Whether the editing caret is rendered in this animation cycle.
    ///
    /// Returns the allocated [`WidgetId`] of the input box container.
    pub fn input_box(
        &mut self,
        text: &str,
        placeholder: &str,
        text_width: f32,
        cursor_blink_visible: bool,
    ) -> WidgetId {
        let box_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(box_id) {
            node.role = WidgetRole::TextInput;
            node.cursor = Some(WidgetCursor::Text);
            node.set_style(
                Style::new()
                    .height(28.0)
                    .border_radius(4.0)
                    .border(1.0, Color::rgba(0.0, 0.85, 0.95, 0.80))
                    .background(Color::rgba(0.05, 0.05, 0.07, 1.0)),
            );
        }
        let _ = self.tree.add_child(self.parent, box_id);

        let text_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(text_id) {
            node.role = WidgetRole::Default;
            let display = if text.is_empty() { placeholder } else { text };
            node.set_text(display);
            node.font_size = 12.0;
            node.line_height = 28.0;
            node.text_align = TextAlign::Left;
            node.text_color = if text.is_empty() {
                Color::rgba(0.45, 0.45, 0.52, 1.0)
            } else {
                Color::WHITE
            };
        }
        let _ = self.tree.add_child(box_id, text_id);

        if cursor_blink_visible {
            let caret_offset = if text.is_empty() { 0.0_f32 } else { text_width };
            let caret_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(caret_id) {
                node.role = WidgetRole::TextInputCaret;
                node.tag = caret_offset.to_bits() as u64;
                node.set_style(
                    Style::new()
                        .background(Color::rgba(0.0, 0.90, 1.0, 0.95))
                        .border_radius(0.75),
                );
            }
            let _ = self.tree.add_child(box_id, caret_id);
        }

        box_id
    }
}

/// Configuration parameters for emitting a text input box in [`UiScope`].
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

    // Accumulate backwards from valid_idx to find the start byte that fits within max_visible_w
    let mut visible_start_byte = 0;
    for (idx, ch) in text[..valid_idx].char_indices().rev() {
        let candidate_slice = &text[idx..valid_idx];
        let slice_w = iris_text::measure_text_advance(candidate_slice, font_size);
        if slice_w > max_visible_w {
            visible_start_byte = idx + ch.len_utf8();
            break;
        }
    }

    // Accumulate forwards from visible_start_byte to find the end byte that fits within max_visible_w
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

impl<'a> UiScope<'a> {
    /// Emits a text input box with placeholder, explicit width, and caret blink indicator, tagged with a semantic tag.
    ///
    /// # Arguments
    /// * `props` - Text, sizing, focus, and caret visual properties.
    /// * `tag` - Semantic hit-testing tag for focus and typing events.
    pub fn input_box_tagged(&mut self, props: &InputBoxProps<'_>, tag: u64) -> WidgetId {
        let box_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(box_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, box_id);
        let (_, hovered, _) = self.check_interaction(box_id);

        let border_color = if props.is_focused {
            Color::rgba(0.0, 0.90, 1.0, 0.95)
        } else if hovered {
            Color::rgba(0.35, 0.40, 0.52, 0.95)
        } else {
            Color::rgba(0.20, 0.23, 0.30, 0.60)
        };
        let bg_color = if hovered {
            Color::rgba(0.08, 0.09, 0.12, 0.98)
        } else {
            Color::rgba(0.06, 0.07, 0.09, 0.95)
        };

        let mut box_style = Style::new()
            .height(24.0)
            .border_radius(4.0)
            .border(1.0, border_color)
            .hover_border(1.0, Color::rgba(0.35, 0.40, 0.52, 0.95))
            .focus_border(1.0, Color::rgba(0.0, 0.90, 1.0, 0.95))
            .background(bg_color)
            .hover_background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .focus_background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .clip_children(true);

        if props.width > 0.0 {
            box_style = box_style.width(props.width);
        } else {
            box_style = box_style.flex_grow(1.0);
        }

        if let Some(node) = self.tree.get_mut(box_id) {
            node.role = WidgetRole::TextInput;
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Text);
            node.set_style(box_style);
        }

        if let Some(icon_str) = props.icon {
            let icon_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(icon_id) {
                node.tag = tag;
                node.set_name("SearchIcon");
                node.interactive = false;
                node.role = WidgetRole::TextInputIcon;
                node.set_text(icon_str);
                node.font_size = 11.0;
                node.line_height = 24.0;
                node.text_align = TextAlign::Center;
                node.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
                node.set_style(
                    Style::new()
                        .position_absolute()
                        .left(7.0)
                        .top(0.0)
                        .width(14.0)
                        .height(24.0),
                );
            }
            let _ = self.tree.add_child(box_id, icon_id);
        }

        let text_start_x = if props.icon.is_some() { 26.0 } else { 8.0 };

        let pad_end = if props.clear_tag.is_some() && !props.text.is_empty() {
            24.0
        } else {
            8.0
        };
        let max_visible_w = if props.width > 0.0 {
            (props.width - text_start_x - pad_end).max(0.0)
        } else {
            0.0
        };

        let font_size = props.font_size;
        let (visible_start, visible_end, caret_offset) =
            if !props.text.is_empty() && max_visible_w > 0.0 {
                calculate_visible_slice_and_caret(
                    props.text,
                    props.cursor_byte_idx,
                    font_size,
                    max_visible_w,
                )
            } else if !props.text.is_empty() {
                let valid_idx = props.cursor_byte_idx.min(props.text.len());
                let offset = iris_text::measure_text_advance(&props.text[..valid_idx], font_size);
                (0, props.text.len(), offset)
            } else {
                (0, 0, 0.0)
            };

        if let Some((start_byte, end_byte)) = props.selection
            && !props.text.is_empty()
        {
            let start_byte = start_byte.min(props.text.len());
            let end_byte = end_byte.min(props.text.len());
            let (sel_min_b, sel_max_b) = if start_byte <= end_byte {
                (start_byte, end_byte)
            } else {
                (end_byte, start_byte)
            };
            let sel_vis_min = sel_min_b.max(visible_start).min(visible_end);
            let sel_vis_max = sel_max_b.max(visible_start).min(visible_end);
            if sel_vis_max > sel_vis_min {
                let sel_start_x = iris_text::measure_text_advance(
                    &props.text[visible_start..sel_vis_min],
                    font_size,
                );
                let sel_end_x = iris_text::measure_text_advance(
                    &props.text[visible_start..sel_vis_max],
                    font_size,
                );
                let sel_w = (sel_end_x - sel_start_x).max(2.0);
                let sel_x = text_start_x + sel_start_x;
                let sel_id = self.tree.create_node();
                if let Some(node) = self.tree.get_mut(sel_id) {
                    node.set_name("SelectionCapsule");
                    node.interactive = false;
                    node.set_style(
                        Style::new()
                            .position_absolute()
                            .left(sel_x)
                            .top(3.5)
                            .width(sel_w)
                            .height(17.0)
                            .background(Color::rgba(0.14, 0.46, 0.88, 0.95))
                            .border_radius(3.0),
                    );
                }
                let _ = self.tree.add_child(box_id, sel_id);
            }
        }

        let display_text = if props.text.is_empty() {
            props.placeholder
        } else {
            &props.text[visible_start..visible_end]
        };

        let text_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(text_id) {
            node.role = WidgetRole::Default;
            node.tag = tag;
            node.set_text(display_text);
            node.font_size = font_size;
            node.line_height = 24.0;
            node.text_align = TextAlign::Left;
            node.text_color = if props.text.is_empty() {
                Color::rgba(0.42, 0.45, 0.55, 1.0)
            } else {
                Color::rgba(0.92, 0.94, 0.98, 1.0)
            };
            node.focus_text_color = Some(Color::rgba(0.92, 0.94, 0.98, 1.0));
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(text_start_x)
                    .top(0.0)
                    .width(max_visible_w)
                    .height(24.0),
            );
        }
        let _ = self.tree.add_child(box_id, text_id);

        let caret_x = text_start_x + caret_offset;

        let caret_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(caret_id) {
            node.set_name("TextInputCaret");
            node.role = WidgetRole::TextInputCaret;
            node.interactive = false;
            node.tag = caret_offset.to_bits() as u64;
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(caret_x)
                    .top(5.0)
                    .width(1.5)
                    .height(14.0)
                    .background(Color::rgba(0.0, 0.90, 1.0, 0.95))
                    .border_radius(0.75),
            );
        }
        let _ = self.tree.add_child(box_id, caret_id);

        if let Some(clear_tag) = props.clear_tag
            && !props.text.is_empty()
        {
            let clear_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(clear_id) {
                node.tag = clear_tag;
                node.set_name("SearchClearButton");
                node.role = WidgetRole::Button;
                node.interactive = true;
                node.cursor = Some(WidgetCursor::Pointer);
                node.set_text("✖");
                node.font_size = 11.0;
                node.line_height = 18.0;
                node.text_align = TextAlign::Center;
                node.text_color = Color::rgba(0.60, 0.63, 0.72, 1.0);
                node.hover_text_color = Some(Color::WHITE);
                node.set_style(
                    Style::new()
                        .position_absolute()
                        .right(4.0)
                        .top(3.0)
                        .width(16.0)
                        .height(18.0)
                        .border_radius(3.0)
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center),
                );
            }
            let _ = self.tree.add_child(box_id, clear_id);
        }

        box_id
    }
}