// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Log Rows Widget (`iris-widgets::console::rows`)
//!
//! Provides the fluent builder, layout generator, and interactive styling for individual
//! console log entry rows in Iris UI.
//!

use super::types::ConsoleLogLevel;
use iris_core::color::Color;
use iris_core::geometry::Rect;
use iris_core::id::WidgetId;
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Visual styling configuration for a single console log row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConsoleRowStyle {
    /// Background color for even rows in normal (idle) state.
    pub bg_idle_even: Color,
    /// Background color for odd rows in normal (idle) state (zebra striping).
    pub bg_idle_odd: Color,
    /// Background color when hovered by the cursor.
    pub bg_hover: Color,
    /// Left accent indicator line color when hovered.
    pub hover_accent_color: Color,
    /// Standard row height in logical pixels.
    pub row_height: f32,
    /// Text line height in logical pixels.
    pub line_height: f32,
}

impl Default for ConsoleRowStyle {
    fn default() -> Self {
        Self::dark_default()
    }
}

impl ConsoleRowStyle {
    /// Standard dark slate theme for console log rows.
    #[must_use]
    pub const fn dark_default() -> Self {
        Self {
            bg_idle_even: Color::rgba(0.06, 0.07, 0.09, 0.95),
            bg_idle_odd: Color::rgba(0.08, 0.09, 0.12, 0.95),
            bg_hover: Color::rgba(0.14, 0.17, 0.23, 0.95),
            hover_accent_color: Color::rgba(0.0, 0.85, 1.0, 0.85),
            row_height: 26.0,
            line_height: 18.0,
        }
    }
}

/// Layout frame returned after constructing a console log row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConsoleRowFrame {
    /// Root node ID of the constructed log row widget.
    pub row_id: WidgetId,
    /// Bounding rectangle allocated for the row.
    pub row_rect: Rect,
}

/// Fluent builder for individual console log entry rows.
///
/// Handles row background zebra striping, left hover indicator accent line,
/// color-coded severity level badge (`ERR`, `WRN`, `INF`, `DBG`, `TRC`),
/// timestamp label, target module bracket tag, and message text.
pub struct ConsoleRowBuilder<'a> {
    rect: Rect,
    level: ConsoleLogLevel,
    timestamp: &'a str,
    target: &'a str,
    message: &'a str,
    is_hovered: bool,
    is_striped: bool,
    style: ConsoleRowStyle,
}

impl<'a> ConsoleRowBuilder<'a> {
    /// Creates a new console row builder positioned at `rect`.
    #[inline]
    #[must_use]
    pub const fn new(rect: Rect) -> Self {
        Self {
            rect,
            level: ConsoleLogLevel::Info,
            timestamp: "",
            target: "",
            message: "",
            is_hovered: false,
            is_striped: false,
            style: ConsoleRowStyle::dark_default(),
        }
    }

    /// Sets the severity level of this log entry.
    #[inline]
    #[must_use]
    pub const fn level(mut self, level: ConsoleLogLevel) -> Self {
        self.level = level;
        self
    }

    /// Sets the readable timestamp text (e.g. `14:20:05.123`).
    #[inline]
    #[must_use]
    pub const fn timestamp(mut self, timestamp: &'a str) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the source module or subsystem target tag (e.g. `ae_renderer`).
    #[inline]
    #[must_use]
    pub const fn target(mut self, target: &'a str) -> Self {
        self.target = target;
        self
    }

    /// Sets the primary log message string.
    #[inline]
    #[must_use]
    pub const fn message(mut self, message: &'a str) -> Self {
        self.message = message;
        self
    }

    /// Sets whether this row is currently hovered by the mouse cursor.
    #[inline]
    #[must_use]
    pub const fn is_hovered(mut self, hovered: bool) -> Self {
        self.is_hovered = hovered;
        self
    }

    /// Sets whether this row is zebra-striped (odd indexed).
    #[inline]
    #[must_use]
    pub const fn is_striped(mut self, striped: bool) -> Self {
        self.is_striped = striped;
        self
    }

    /// Sets visual styling overrides for this row.
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: ConsoleRowStyle) -> Self {
        self.style = style;
        self
    }

    /// Constructs the log row into the specified `UiTree`.
    pub fn build(&self, tree: &mut UiTree, parent_id: WidgetId) -> ConsoleRowFrame {
        // 1. Row Base Container
        let bg_color = if self.is_hovered {
            self.style.bg_hover
        } else if self.is_striped {
            self.style.bg_idle_odd
        } else {
            self.style.bg_idle_even
        };

        let row_id = tree.create_node();
        if let Some(node) = tree.get_mut(row_id) {
            node.set_name("ConsoleRow");
            node.computed_rect = self.rect;
            node.style = Style::new().background(bg_color);
        }
        let _ = tree.add_child(parent_id, row_id);

        // 2. Left Hover Accent Line
        if self.is_hovered {
            let hover_bar_id = tree.create_node();
            if let Some(node) = tree.get_mut(hover_bar_id) {
                node.set_name("RowHoverAccent");
                node.computed_rect = Rect::new(self.rect.x, self.rect.y, 3.0, self.rect.height);
                node.style = Style::new().background(self.style.hover_accent_color);
            }
            let _ = tree.add_child(row_id, hover_bar_id);
        }

        let mut cur_x = self.rect.x + 10.0;
        let line_h = self.style.line_height;
        let text_y = self.rect.y + (self.rect.height - line_h) * 0.5;

        // 3. Severity Level Badge
        let (badge_text, badge_color, badge_bg, badge_border) = match self.level {
            ConsoleLogLevel::Error => (
                "ERR",
                Color::rgba(0.98, 0.45, 0.45, 1.0),
                Color::rgba(0.38, 0.08, 0.08, 0.90),
                Color::rgba(0.95, 0.30, 0.30, 0.50),
            ),
            ConsoleLogLevel::Warn => (
                "WRN",
                Color::rgba(0.98, 0.78, 0.20, 1.0),
                Color::rgba(0.35, 0.20, 0.02, 0.90),
                Color::rgba(0.95, 0.70, 0.15, 0.50),
            ),
            ConsoleLogLevel::Info => (
                "INF",
                Color::rgba(0.25, 0.78, 0.98, 1.0),
                Color::rgba(0.05, 0.20, 0.32, 0.90),
                Color::rgba(0.20, 0.70, 0.95, 0.50),
            ),
            ConsoleLogLevel::Debug => (
                "DBG",
                Color::rgba(0.75, 0.60, 0.98, 1.0),
                Color::rgba(0.18, 0.10, 0.32, 0.90),
                Color::rgba(0.65, 0.45, 0.95, 0.50),
            ),
            ConsoleLogLevel::Trace => (
                "TRC",
                Color::rgba(0.60, 0.66, 0.75, 1.0),
                Color::rgba(0.12, 0.15, 0.20, 0.90),
                Color::rgba(0.40, 0.45, 0.55, 0.50),
            ),
        };

        let badge_w = 40.0;
        let badge_id = tree.create_node();
        if let Some(node) = tree.get_mut(badge_id) {
            node.set_name("LevelBadge");
            node.set_text(badge_text);
            node.font_size = 9.5;
            node.line_height = line_h;
            node.text_align = TextAlign::Center;
            node.text_color = badge_color;
            node.computed_rect = Rect::new(cur_x, text_y, badge_w, line_h);
            node.style = Style::new()
                .background(badge_bg)
                .border_radius(3.5)
                .border(1.0, badge_border);
        }
        let _ = tree.add_child(row_id, badge_id);
        cur_x += badge_w + 10.0;

        // 4. Timestamp
        let time_w = 66.0;
        let time_id = tree.create_node();
        if let Some(node) = tree.get_mut(time_id) {
            node.set_name("LogTimestamp");
            node.set_text(self.timestamp);
            node.font_size = 11.0;
            node.line_height = line_h;
            node.text_color = Color::rgba(0.58, 0.63, 0.72, 1.0);
            node.computed_rect = Rect::new(cur_x, text_y, time_w, line_h);
        }
        let _ = tree.add_child(row_id, time_id);
        cur_x += time_w + 8.0;

        // 5. Target Tag: e.g. "[ae_renderer]"
        let target_text = format!("[{}]", self.target);
        let target_w = (target_text.len() as f32 * 6.6 + 6.0).clamp(44.0, 220.0);
        let target_id = tree.create_node();
        if let Some(node) = tree.get_mut(target_id) {
            node.set_name("LogTarget");
            node.set_text(target_text);
            node.font_size = 11.0;
            node.line_height = line_h;
            node.text_color = Color::rgba(0.22, 0.76, 0.96, 0.95);
            node.computed_rect = Rect::new(cur_x, text_y, target_w, line_h);
        }
        let _ = tree.add_child(row_id, target_id);
        cur_x += target_w + 10.0;

        // 6. Message text
        let msg_w = (self.rect.right() - cur_x - 12.0).max(60.0);
        let msg_id = tree.create_node();
        if let Some(node) = tree.get_mut(msg_id) {
            node.set_name("LogMessage");
            node.set_text(self.message);
            node.font_size = 11.5;
            node.line_height = line_h;
            node.text_color = match self.level {
                ConsoleLogLevel::Error => Color::rgba(0.99, 0.60, 0.60, 1.0),
                ConsoleLogLevel::Warn => Color::rgba(0.99, 0.88, 0.45, 1.0),
                ConsoleLogLevel::Info => Color::rgba(0.95, 0.96, 0.98, 1.0),
                ConsoleLogLevel::Debug => Color::rgba(0.85, 0.80, 0.98, 1.0),
                ConsoleLogLevel::Trace => Color::rgba(0.70, 0.74, 0.82, 1.0),
            };
            node.computed_rect = Rect::new(cur_x, text_y, msg_w, line_h);
        }
        let _ = tree.add_child(row_id, msg_id);

        ConsoleRowFrame {
            row_id,
            row_rect: self.rect,
        }
    }
}

/// Fluent builder for the empty notice placeholder when no logs match the current filter.
pub struct ConsoleEmptyNoticeBuilder<'a> {
    rect: Rect,
    message: &'a str,
}

impl<'a> ConsoleEmptyNoticeBuilder<'a> {
    /// Creates a new empty notice builder positioned at `rect`.
    #[inline]
    #[must_use]
    pub const fn new(rect: Rect, message: &'a str) -> Self {
        Self { rect, message }
    }

    /// Constructs the placeholder notice into the specified `UiTree`.
    pub fn build(&self, tree: &mut UiTree, parent_id: WidgetId) -> WidgetId {
        let empty_id = tree.create_node();
        if let Some(node) = tree.get_mut(empty_id) {
            node.set_name("ConsoleEmptyNotice");
            node.set_text(self.message);
            node.font_size = 12.0;
            node.line_height = 24.0;
            node.text_color = Color::rgba(0.50, 0.54, 0.64, 1.0);
            node.computed_rect = self.rect;
        }
        let _ = tree.add_child(parent_id, empty_id);
        empty_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_row_builder_styling_and_badges() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation");

        let row_rect = Rect::new(0.0, 0.0, 800.0, 26.0);
        let frame = ConsoleRowBuilder::new(row_rect)
            .level(ConsoleLogLevel::Error)
            .timestamp("12:00:00.000")
            .target("ae_core")
            .message("Assertion failed")
            .is_hovered(true)
            .is_striped(false)
            .build(&mut tree, root);

        assert_eq!(frame.row_rect, row_rect);

        let row_node = tree.get(frame.row_id).expect("Row node");
        assert_eq!(row_node.name.as_deref(), Some("ConsoleRow"));

        // Hover bar should be created
        let mut found_hover_bar = false;
        let mut found_badge = false;
        let mut found_timestamp = false;
        let mut found_target = false;
        let mut found_msg = false;

        for &child_id in &row_node.children {
            if let Some(child) = tree.get(child_id) {
                match child.name.as_deref() {
                    Some("RowHoverAccent") => found_hover_bar = true,
                    Some("LevelBadge") => {
                        found_badge = true;
                        assert_eq!(child.text.as_deref(), Some("ERR"));
                    }
                    Some("LogTimestamp") => {
                        found_timestamp = true;
                        assert_eq!(child.text.as_deref(), Some("12:00:00.000"));
                    }
                    Some("LogTarget") => {
                        found_target = true;
                        assert_eq!(child.text.as_deref(), Some("[ae_core]"));
                    }
                    Some("LogMessage") => {
                        found_msg = true;
                        assert_eq!(child.text.as_deref(), Some("Assertion failed"));
                    }
                    _ => {}
                }
            }
        }

        assert!(found_hover_bar);
        assert!(found_badge);
        assert!(found_timestamp);
        assert!(found_target);
        assert!(found_msg);
    }

    #[test]
    fn test_console_empty_notice_builder() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation");

        let notice_rect = Rect::new(20.0, 20.0, 200.0, 30.0);
        let notice_id =
            ConsoleEmptyNoticeBuilder::new(notice_rect, "No logs recorded").build(&mut tree, root);

        let node = tree.get(notice_id).expect("Notice node");
        assert_eq!(node.name.as_deref(), Some("ConsoleEmptyNotice"));
        assert_eq!(node.text.as_deref(), Some("No logs recorded"));
        assert_eq!(node.computed_rect, notice_rect);
    }
}