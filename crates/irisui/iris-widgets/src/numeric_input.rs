// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hardware-Accelerated Numeric Input Pill & Drag Widget
//!
//! Provides a standardized, GPU SDF-rendered numeric input pill component
//! for inspector panels, property grids, and transform vectors.
//!
//! Handles idle, hover, and active editing states, dynamic cursor blinking,
//! Select-All highlight capsules, and optional prefixes (e.g. `X: `) or suffixes (e.g. `m/s`, `°`).

use iris_core::color::Color;
use iris_core::geometry::Rect;
use iris_core::id::WidgetId;
use iris_core::node::{WidgetCursor, WidgetRole};
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Visual styling configuration for a numeric input pill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericInputStyle {
    /// Background color in normal/idle state.
    pub bg_idle: Color,
    /// Background color when hovered by cursor.
    pub bg_hover: Color,
    /// Background color when actively focused/editing.
    pub bg_active: Color,
    /// Border color in normal/idle state.
    pub border_idle: Color,
    /// Border color when hovered by cursor.
    pub border_hover: Color,
    /// Border color when actively focused/editing.
    pub border_active: Color,
    /// Border width in logical pixels.
    pub border_width: f32,
    /// Corner rounding radius in logical pixels.
    pub border_radius: f32,
    /// Text color in normal/idle state.
    pub text_color_idle: Color,
    /// Text color when hovered by cursor.
    pub text_color_hover: Color,
    /// Text color when actively focused/editing.
    pub text_color_active: Color,
    /// Background color of the selection pill when all text is selected.
    pub selection_bg: Color,
    /// Corner radius of the selection pill.
    pub selection_radius: f32,
    /// Font size in logical points.
    pub font_size: f32,
}

impl Default for NumericInputStyle {
    fn default() -> Self {
        Self {
            bg_idle: Color::rgba(0.125, 0.133, 0.153, 0.98),
            bg_hover: Color::rgba(0.157, 0.169, 0.200, 1.0),
            bg_active: Color::rgba(0.118, 0.125, 0.145, 1.0),
            border_idle: Color::rgba(0.180, 0.192, 0.227, 0.85),
            border_hover: Color::rgba(0.235, 0.247, 0.286, 0.95),
            border_active: Color::rgba(0.0, 0.80, 1.00, 0.95),
            border_width: 1.0,
            border_radius: 5.0,
            text_color_idle: Color::rgba(0.886, 0.894, 0.918, 1.0),
            text_color_hover: Color::WHITE,
            text_color_active: Color::WHITE,
            selection_bg: Color::rgba(0.14, 0.46, 0.88, 0.95),
            selection_radius: 3.0,
            font_size: 10.5,
        }
    }
}

/// Active text editing and cursor selection state for numeric inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumericInputEditState<'a> {
    /// In-progress text buffer entered by the user.
    pub buffer: &'a str,
    /// Caret insertion index within the string buffer.
    pub cursor_idx: usize,
    /// Whether all characters in the buffer are currently selected.
    pub is_all_selected: bool,
    /// Caret blinking phase (true = visible cursor pipe, false = hidden).
    pub blink_caret: bool,
}

/// Fluent builder for constructing standardized numeric input pills.
pub struct NumericInputPillBuilder<'a> {
    rect: Rect,
    value: f32,
    decimals: usize,
    prefix: Option<&'a str>,
    suffix: Option<&'a str>,
    custom_text: Option<String>,
    edit_state: Option<NumericInputEditState<'a>>,
    is_hovered: bool,
    style: NumericInputStyle,
    name: Option<String>,
    tag: u64,
}

impl<'a> NumericInputPillBuilder<'a> {
    /// Initializes a new numeric input builder targeting the given bounding rectangle.
    #[must_use]
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            value: 0.0,
            decimals: 2,
            prefix: None,
            suffix: None,
            custom_text: None,
            edit_state: None,
            is_hovered: false,
            style: NumericInputStyle::default(),
            name: None,
            tag: 0,
        }
    }

    /// Sets the underlying numeric value displayed when not actively editing.
    #[must_use]
    pub fn value(mut self, val: f32) -> Self {
        self.value = val;
        self
    }

    /// Sets the decimal precision for floating point formatting.
    #[must_use]
    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    /// Sets an optional prefix string prepended to the value (e.g. `"X: "`, `"Y: "`, `"Z: "`).
    #[must_use]
    pub fn prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = Some(prefix);
        self
    }

    /// Sets an optional unit suffix string appended to the formatted text (e.g. `"°"`, `" m/s"`).
    #[must_use]
    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = Some(suffix);
        self
    }

    /// Sets an explicit custom display text overriding default numerical formatting.
    #[must_use]
    pub fn custom_text(mut self, text: impl Into<String>) -> Self {
        self.custom_text = Some(text.into());
        self
    }

    /// Informs the builder of the active editing session state, enabling caret and selection highlight rendering.
    #[must_use]
    pub fn edit_state(mut self, state: Option<NumericInputEditState<'a>>) -> Self {
        self.edit_state = state;
        self
    }

    /// Sets whether the input pill is currently hovered by the mouse cursor.
    #[must_use]
    pub fn is_hovered(mut self, is_hovered: bool) -> Self {
        self.is_hovered = is_hovered;
        self
    }

    /// Overrides the visual styling of the numeric input pill.
    #[must_use]
    pub fn style(mut self, style: NumericInputStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets a descriptive debug name for the widget tree node.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Assigns a semantic identifier tag to the numeric input pill container (`node.tag`).
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Assembles the numeric input pill hierarchy into the provided [`UiTree`] and links it to `parent_id`.
    ///
    /// Returns the [`WidgetId`] of the outer pill container node.
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> WidgetId {
        let is_editing = self.edit_state.is_some();
        let (bg, border_col) = if is_editing {
            (self.style.bg_active, self.style.border_active)
        } else if self.is_hovered {
            (self.style.bg_hover, self.style.border_hover)
        } else {
            (self.style.bg_idle, self.style.border_idle)
        };
        let base_name = self.name.as_deref().unwrap_or("NumericInputPill");
        let (sel_name, txt_name) = if let Some(suffix) = base_name.strip_prefix("NumBox_") {
            (format!("NumSel_{}", suffix), format!("NumText_{}", suffix))
        } else if let Some(suffix) = base_name.strip_prefix("NumPill_") {
            (format!("NumSel_{}", suffix), format!("NumVal_{}", suffix))
        } else {
            (
                format!("{}_Selection", base_name),
                format!("{}_Value", base_name),
            )
        };

        // 1. Container Pill Box
        let box_id = tree.create_node();
        if let Some(node) = tree.get_mut(box_id) {
            node.set_name(base_name);
            node.computed_rect = self.rect;
            node.interactive = true;
            node.role = WidgetRole::NumericInput;
            node.tag = self.tag;
            node.cursor = Some(if is_editing {
                WidgetCursor::Text
            } else {
                WidgetCursor::EwResize
            });
            node.style = Style::new()
                .background(bg)
                .border(self.style.border_width, border_col)
                .border_radius(self.style.border_radius);
        }
        let _ = tree.add_child(parent_id, box_id);

        let prefix_str = self.prefix.unwrap_or("");
        let suffix_str = self.suffix.unwrap_or("");

        // 2. Select-All Highlight Pill
        if let Some(s) = self.edit_state.filter(|s| s.is_all_selected) {
            let buf = s.buffer;
            let display_str = format!("{}{}", prefix_str, buf);
            let approx_char_w = 6.2;
            let tot_w = display_str.len() as f32 * approx_char_w;
            let cx = self.rect.x + self.rect.width * 0.5;
            let text_start_x = cx - tot_w * 0.5;
            let prefix_w = prefix_str.len() as f32 * approx_char_w;
            let val_start_x = text_start_x + prefix_w;
            let val_w = (buf.len() as f32 * approx_char_w).max(12.0);

            let sel_x = (val_start_x - 2.0).clamp(self.rect.x + 2.0, self.rect.right() - 6.0);
            let sel_max_w = (self.rect.right() - 2.0 - sel_x).max(4.0);
            let sel_w = (val_w + 4.0).min(sel_max_w);
            let sel_rect = Rect::new(
                sel_x,
                self.rect.y + 2.5,
                sel_w,
                (self.rect.height - 5.0).max(4.0),
            );

            let sel_id = tree.create_node();
            if let Some(node) = tree.get_mut(sel_id) {
                node.set_name(sel_name);
                node.computed_rect = sel_rect;
                node.style = Style::new()
                    .background(self.style.selection_bg)
                    .border_radius(self.style.selection_radius);
            }
            let _ = tree.add_child(box_id, sel_id);
        }

        // 3. Formatted Display String Resolution
        let display_str = if let Some(s) = self.edit_state {
            let buf = s.buffer;
            let cursor = s.cursor_idx.min(buf.len());
            let (left, right) = buf.split_at(cursor);
            if s.is_all_selected {
                format!("{}{}", prefix_str, buf)
            } else if s.blink_caret {
                format!("{}{}|{}", prefix_str, left, right)
            } else {
                format!("{}{}{}", prefix_str, left, right)
            }
        } else if let Some(custom) = self.custom_text {
            format!("{}{}{}", prefix_str, custom, suffix_str)
        } else {
            let num_str = format!("{:.precision$}", self.value, precision = self.decimals);
            format!("{}{}{}", prefix_str, num_str, suffix_str)
        };

        // 4. Text Node
        let txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(txt_id) {
            node.set_name(txt_name);
            node.set_text(display_str);
            node.font_size = self.style.font_size;
            node.line_height = self.rect.height;
            node.text_align = TextAlign::Center;
            node.text_color = if is_editing {
                self.style.text_color_active
            } else if self.is_hovered {
                self.style.text_color_hover
            } else {
                self.style.text_color_idle
            };
            node.computed_rect = self.rect;
        }
        let _ = tree.add_child(box_id, txt_id);

        box_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_input_pill_idle() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let rect = Rect::new(10.0, 20.0, 60.0, 22.0);

        let box_id = NumericInputPillBuilder::new(rect)
            .value(42.5)
            .decimals(2)
            .name("MyInput")
            .build(&mut tree, root);

        let box_node = tree.get(box_id).expect("Pill box exists");
        assert_eq!(box_node.name.as_deref(), Some("MyInput"));
        assert_eq!(box_node.computed_rect, rect);
        assert_eq!(box_node.children.len(), 1);

        let txt_id = box_node.children[0];
        let txt_node = tree.get(txt_id).expect("Text node exists");
        assert_eq!(txt_node.text.as_deref(), Some("42.50"));
    }

    #[test]
    fn test_numeric_input_pill_prefix_and_suffix() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let rect = Rect::new(10.0, 20.0, 80.0, 22.0);

        let box_id = NumericInputPillBuilder::new(rect)
            .value(180.0)
            .decimals(0)
            .prefix("Angle: ")
            .suffix("°")
            .build(&mut tree, root);

        let box_node = tree.get(box_id).expect("Pill box exists");
        let txt_id = box_node.children[0];
        let txt_node = tree.get(txt_id).expect("Text node exists");
        assert_eq!(txt_node.text.as_deref(), Some("Angle: 180°"));
    }

    #[test]
    fn test_numeric_input_pill_editing_with_caret() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let rect = Rect::new(10.0, 20.0, 60.0, 22.0);

        let edit_state = NumericInputEditState {
            buffer: "1234",
            cursor_idx: 2,
            is_all_selected: false,
            blink_caret: true,
        };

        let box_id = NumericInputPillBuilder::new(rect)
            .prefix("X: ")
            .edit_state(Some(edit_state))
            .build(&mut tree, root);

        let box_node = tree.get(box_id).expect("Pill box exists");
        assert_eq!(box_node.children.len(), 1);

        let txt_id = box_node.children[0];
        let txt_node = tree.get(txt_id).expect("Text node exists");
        assert_eq!(txt_node.text.as_deref(), Some("X: 12|34"));
    }

    #[test]
    fn test_numeric_input_pill_select_all_highlight() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let rect = Rect::new(10.0, 20.0, 60.0, 22.0);

        let edit_state = NumericInputEditState {
            buffer: "500.0",
            cursor_idx: 5,
            is_all_selected: true,
            blink_caret: false,
        };

        let box_id = NumericInputPillBuilder::new(rect)
            .edit_state(Some(edit_state))
            .build(&mut tree, root);

        let box_node = tree.get(box_id).expect("Pill box exists");
        assert_eq!(box_node.children.len(), 2);

        let sel_id = box_node.children[0];
        let sel_node = tree.get(sel_id).expect("Selection node exists");
        assert_eq!(sel_node.name.as_deref(), Some("NumericInputPill_Selection"));

        let txt_id = box_node.children[1];
        let txt_node = tree.get(txt_id).expect("Text node exists");
        assert_eq!(txt_node.text.as_deref(), Some("500.0"));
    }
}