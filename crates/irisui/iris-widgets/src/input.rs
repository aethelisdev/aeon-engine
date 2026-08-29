// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Input fields, draggable numeric inputs, sliders, and toggle checkboxes.

use iris_core::{AlignItems, Color, JustifyContent, Style, TextAlign, UiTree, WidgetId};

/// Encapsulated state and UTF-8 safe editing buffer for interactive text input widgets.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextInputState {
    /// Accumulated text string buffer.
    pub buffer: String,
    /// Byte index of the editing cursor.
    pub cursor_byte_idx: usize,
    /// In-progress uncommitted IME composition string.
    pub ime_preedit: Option<String>,
    /// In-progress IME selection/cursor range.
    pub ime_cursor: Option<(usize, usize)>,
}

impl TextInputState {
    /// Creates a new state initialized with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        let buffer = text.into();
        let cursor_byte_idx = buffer.len();
        Self {
            buffer,
            cursor_byte_idx,
            ime_preedit: None,
            ime_cursor: None,
        }
    }

    /// Sets the active in-progress IME preedit composition string.
    pub fn set_ime_preedit(&mut self, text: impl Into<String>, cursor: Option<(usize, usize)>) {
        let txt = text.into();
        if txt.is_empty() {
            self.ime_preedit = None;
            self.ime_cursor = None;
        } else {
            self.ime_preedit = Some(txt);
            self.ime_cursor = cursor;
        }
    }

    /// Clears any active in-progress IME preedit composition.
    pub fn clear_ime_preedit(&mut self) {
        self.ime_preedit = None;
        self.ime_cursor = None;
    }

    /// Commits finalized IME text into the buffer and clears preedit.
    pub fn commit_ime(&mut self, text: &str) {
        self.clear_ime_preedit();
        self.insert_str(text);
    }

    /// Formats the display string with in-progress IME preedit text inserted at cursor position.
    pub fn display_text(&self) -> String {
        if let Some(ref preedit) = self.ime_preedit {
            let (head, tail) = self
                .buffer
                .split_at(self.cursor_byte_idx.min(self.buffer.len()));
            format!("{}[{}]{}", head, preedit, tail)
        } else {
            self.buffer.clone()
        }
    }

    /// Inserts a character or substring at the current cursor position, maintaining UTF-8 boundary integrity.
    pub fn insert_str(&mut self, text: &str) {
        self.cursor_byte_idx = self.cursor_byte_idx.min(self.buffer.len());
        while !self.buffer.is_char_boundary(self.cursor_byte_idx) && self.cursor_byte_idx > 0 {
            self.cursor_byte_idx -= 1;
        }
        self.buffer.insert_str(self.cursor_byte_idx, text);
        self.cursor_byte_idx += text.len();
    }

    /// Deletes the previous UTF-8 character before the cursor (Backspace).
    pub fn backspace(&mut self) {
        if self.cursor_byte_idx == 0 || self.buffer.is_empty() {
            return;
        }
        self.cursor_byte_idx = self.cursor_byte_idx.min(self.buffer.len());
        let prev_boundary = self.buffer[..self.cursor_byte_idx]
            .char_indices()
            .last()
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        self.buffer.drain(prev_boundary..self.cursor_byte_idx);
        self.cursor_byte_idx = prev_boundary;
    }

    /// Deletes the next UTF-8 character after the cursor (Delete).
    pub fn delete(&mut self) {
        if self.cursor_byte_idx >= self.buffer.len() {
            return;
        }
        let next_boundary = self.buffer[self.cursor_byte_idx..]
            .char_indices()
            .nth(1)
            .map(|(idx, _)| self.cursor_byte_idx + idx)
            .unwrap_or(self.buffer.len());
        self.buffer.drain(self.cursor_byte_idx..next_boundary);
    }

    /// Moves cursor one character left safely across multi-byte UTF-8 boundaries.
    pub fn move_left(&mut self) {
        if self.cursor_byte_idx > 0 {
            self.cursor_byte_idx = self.buffer[..self.cursor_byte_idx]
                .char_indices()
                .last()
                .map(|(idx, _)| idx)
                .unwrap_or(0);
        }
    }

    /// Moves cursor one character right safely across multi-byte UTF-8 boundaries.
    pub fn move_right(&mut self) {
        if self.cursor_byte_idx < self.buffer.len() {
            self.cursor_byte_idx = self.buffer[self.cursor_byte_idx..]
                .char_indices()
                .nth(1)
                .map(|(idx, _)| self.cursor_byte_idx + idx)
                .unwrap_or(self.buffer.len());
        }
    }
}

/// Helper builder for editable text input fields with placeholder and focus styling.
pub struct TextInputBuilder {
    node_id: WidgetId,
}

impl TextInputBuilder {
    /// Creates an input box with specified text or placeholder.
    pub fn new(
        tree: &mut UiTree,
        text: impl Into<String>,
        placeholder: &'static str,
        is_focused: bool,
    ) -> Self {
        let node_id = tree.create_node();
        let text_str = text.into();
        let display_text = if text_str.is_empty() {
            placeholder.to_string()
        } else {
            text_str
        };

        if let Some(node) = tree.get_mut(node_id) {
            node.set_text(display_text);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_color = if is_focused {
                Color::WHITE
            } else {
                Color::hex("#94a3b8")
            };

            let border_color = if is_focused {
                Color::hex("#38bdf8")
            } else {
                Color::hex("#20202e")
            };

            node.set_style(
                Style::new()
                    .padding(4.0)
                    .background(Color::hex("#101016"))
                    .border(1.0, border_color)
                    .border_radius(3.0)
                    .align_items(AlignItems::Center),
            );
        }
        Self { node_id }
    }

    /// Consumes the builder and returns the configured `WidgetId`.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.node_id
    }
}

/// Helper builder for interactive numeric drag values with axis indicators.
pub struct DragValueBuilder {
    node_id: WidgetId,
}

impl DragValueBuilder {
    /// Creates a numeric drag widget showing an axis label (e.g. "X") and formatted value.
    pub fn new(
        tree: &mut UiTree,
        axis: &'static str,
        value: f32,
        axis_color: Color,
        is_active: bool,
    ) -> Self {
        let node_id = tree.create_node();
        if let Some(node) = tree.get_mut(node_id) {
            node.set_text(format!("{}: {:.2}", axis, value));
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_color = Color::WHITE;
            node.text_align = TextAlign::Center;

            let border_color = if is_active { Color::WHITE } else { axis_color };

            node.set_style(
                Style::new()
                    .padding(3.0)
                    .margin(1.0)
                    .background(Color::hex("#161622"))
                    .border(1.0, border_color)
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .flex_grow(1.0),
            );
        }
        Self { node_id }
    }

    /// Consumes the builder and returns the configured `WidgetId`.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.node_id
    }
}

/// Helper builder for boolean checkboxes with interactive checkmark indicator.
pub struct CheckboxBuilder {
    node_id: WidgetId,
}

impl CheckboxBuilder {
    /// Creates a checkbox widget with an optional text label and checked state.
    pub fn new(tree: &mut UiTree, label: impl Into<String>, checked: bool) -> Self {
        let node_id = tree.create_node();
        let label_str = label.into();
        let mark = if checked { "✓" } else { " " };
        let display = format!("[{}] {}", mark, label_str);

        if let Some(node) = tree.get_mut(node_id) {
            node.set_text(display);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_color = if checked {
                Color::WHITE
            } else {
                Color::hex("#94a3b8")
            };

            let bg = if checked {
                Color::hex("#0369a1")
            } else {
                Color::hex("#12121a")
            };
            let border_color = if checked {
                Color::hex("#38bdf8")
            } else {
                Color::hex("#252536")
            };

            node.set_style(
                Style::new()
                    .padding(3.0)
                    .margin(1.0)
                    .background(bg)
                    .border(1.0, border_color)
                    .border_radius(3.0)
                    .align_items(AlignItems::Center),
            );
        }
        Self { node_id }
    }

    /// Consumes the builder and returns the configured `WidgetId`.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.node_id
    }
}

/// Helper builder for continuous numeric sliders with percentage fill indicators.
pub struct SliderBuilder {
    node_id: WidgetId,
}

impl SliderBuilder {
    /// Creates a slider widget displaying a label, current value, and normalized progress.
    pub fn new(
        tree: &mut UiTree,
        label: impl Into<String>,
        value: f32,
        min: f32,
        max: f32,
    ) -> Self {
        let node_id = tree.create_node();
        let range = (max - min).max(0.001);
        let pct = ((value - min) / range).clamp(0.0, 1.0) * 100.0;
        let display = format!("{}: {:.2} ({:.0}%)", label.into(), value, pct);

        if let Some(node) = tree.get_mut(node_id) {
            node.set_text(display);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_color = Color::WHITE;
            node.text_align = TextAlign::Center;

            node.set_style(
                Style::new()
                    .padding(3.0)
                    .margin(1.0)
                    .background(Color::hex("#181824"))
                    .border(1.0, Color::hex("#2a2a3e"))
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center),
            );
        }
        Self { node_id }
    }

    /// Consumes the builder and returns the configured `WidgetId`.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.node_id
    }
}