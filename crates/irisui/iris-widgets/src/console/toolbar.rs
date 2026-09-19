// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Header Toolbar Widget (`iris-widgets::console::toolbar`)
//!
//! Provides the fluent builder, layout generator, and interactive styling for the
//! top toolbar of the Developer Console in Iris UI.
//!

use super::types::{
    CONSOLE_TAG_AUTOSCROLL, CONSOLE_TAG_CLEAR, CONSOLE_TAG_FILTER_ALL, CONSOLE_TAG_FILTER_DEBUG,
    CONSOLE_TAG_FILTER_ERROR, CONSOLE_TAG_FILTER_INFO, CONSOLE_TAG_FILTER_WARN,
    CONSOLE_TAG_SEARCH_CLEAR, CONSOLE_TAG_SEARCH_INPUT, ConsoleFilterLevel, ConsoleLogCounts,
};
use iris_core::WidgetRole;
use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Visual styling configuration for the Developer Console header toolbar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConsoleToolbarStyle {
    /// Background color of the toolbar strip.
    pub bg: Color,
    /// Bottom border outline color.
    pub border_color: Color,
    /// Bottom border outline thickness in physical pixels.
    pub border_width: f32,
    /// Background color of the Clear button when idle.
    pub clear_bg_idle: Color,
    /// Background color of the Clear button when hovered.
    pub clear_bg_hover: Color,
    /// Border color of the Clear button.
    pub clear_border: Color,
    /// Text color of the Clear button.
    pub clear_text: Color,
    /// Background color of the search input field.
    pub search_bg: Color,
    /// Border color of the search input field when focused.
    pub search_border_focus: Color,
    /// Border color of the search input field when idle.
    pub search_border_idle: Color,
    /// Blinking caret cursor color.
    pub caret_color: Color,
}

impl Default for ConsoleToolbarStyle {
    fn default() -> Self {
        Self::dark_default()
    }
}

impl ConsoleToolbarStyle {
    /// Standard dark slate theme for the console toolbar.
    #[must_use]
    pub const fn dark_default() -> Self {
        Self {
            bg: Color::rgba(0.08, 0.09, 0.12, 0.98),
            border_color: Color::rgba(0.18, 0.21, 0.28, 0.70),
            border_width: 1.0,
            clear_bg_idle: Color::rgba(0.13, 0.15, 0.20, 0.95),
            clear_bg_hover: Color::rgba(0.20, 0.24, 0.32, 1.0),
            clear_border: Color::rgba(0.24, 0.27, 0.35, 0.60),
            clear_text: Color::rgba(0.80, 0.84, 0.90, 1.0),
            search_bg: Color::rgba(0.06, 0.07, 0.09, 0.95),
            search_border_focus: Color::rgba(0.0, 0.90, 1.0, 0.95),
            search_border_idle: Color::rgba(0.20, 0.23, 0.30, 0.60),
            caret_color: Color::rgba(0.0, 0.90, 1.0, 1.0),
        }
    }
}

/// Output layout frame returned after constructing a console toolbar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConsoleToolbarFrame {
    /// Root node ID of the constructed toolbar container widget.
    pub toolbar_id: WidgetId,
    /// Bounding rectangle allocated for the toolbar strip.
    pub toolbar_rect: Rect,
    /// Bounding rectangle of the search input box, if width permitted its display.
    pub search_input_rect: Option<Rect>,
}

/// Fluent builder for the hardware-accelerated Developer Console toolbar.
///
/// Handles the "🧹 Clear" button, elevated filter tabs with badge counts,
/// interactive search query input with focus/caret/clear controls, auto-scroll toggle,
/// and total log counter.
pub struct ConsoleToolbarBuilder<'a> {
    rect: Rect,
    active_filter: ConsoleFilterLevel,
    counts: ConsoleLogCounts,
    search_query: &'a str,
    is_search_focused: bool,
    blink_caret: bool,
    auto_scroll: bool,
    cursor_pos: Point,
    style: ConsoleToolbarStyle,
}

impl<'a> ConsoleToolbarBuilder<'a> {
    /// Creates a new console toolbar builder positioned at `rect`.
    #[inline]
    #[must_use]
    pub const fn new(rect: Rect) -> Self {
        Self {
            rect,
            active_filter: ConsoleFilterLevel::All,
            counts: ConsoleLogCounts::new(0, 0, 0, 0, 0),
            search_query: "",
            is_search_focused: false,
            blink_caret: false,
            auto_scroll: true,
            cursor_pos: Point::new(0.0, 0.0),
            style: ConsoleToolbarStyle::dark_default(),
        }
    }

    /// Sets the currently active log severity filter.
    #[inline]
    #[must_use]
    pub const fn active_filter(mut self, filter: ConsoleFilterLevel) -> Self {
        self.active_filter = filter;
        self
    }

    /// Sets the severity count metrics for badge labels.
    #[inline]
    #[must_use]
    pub const fn counts(mut self, counts: ConsoleLogCounts) -> Self {
        self.counts = counts;
        self
    }

    /// Sets the current search query text.
    #[inline]
    #[must_use]
    pub const fn search_query(mut self, query: &'a str) -> Self {
        self.search_query = query;
        self
    }

    /// Sets whether the search text input field currently has keyboard focus.
    #[inline]
    #[must_use]
    pub const fn is_search_focused(mut self, focused: bool) -> Self {
        self.is_search_focused = focused;
        self
    }

    /// Sets whether the blinking caret should be visible during this render frame.
    #[inline]
    #[must_use]
    pub const fn blink_caret(mut self, blink: bool) -> Self {
        self.blink_caret = blink;
        self
    }

    /// Sets whether auto-scroll to bottom is currently active.
    #[inline]
    #[must_use]
    pub const fn auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }

    /// Sets the current mouse cursor position for interactive hover state detection.
    #[inline]
    #[must_use]
    pub const fn cursor_pos(mut self, pos: Point) -> Self {
        self.cursor_pos = pos;
        self
    }

    /// Sets visual styling overrides for toolbar buttons and inputs.
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: ConsoleToolbarStyle) -> Self {
        self.style = style;
        self
    }

    /// Constructs the toolbar into the specified `UiTree`.
    pub fn build(&self, tree: &mut UiTree, parent_id: WidgetId) -> ConsoleToolbarFrame {
        // 1. Toolbar Container
        let tb_id = tree.create_node();
        if let Some(node) = tree.get_mut(tb_id) {
            node.set_name("ConsoleToolbar");
            node.computed_rect = self.rect;
            node.style = Style::new()
                .background(self.style.bg)
                .border(self.style.border_width, self.style.border_color);
        }
        let _ = tree.add_child(parent_id, tb_id);

        let mut cur_x = self.rect.x + 8.0;
        let btn_y = self.rect.y + 5.0;
        let btn_h = (self.rect.height - 10.0).max(18.0);

        // 2. Clear Logs Button
        let clear_w = 76.0;
        let clear_rect = Rect::new(cur_x, btn_y, clear_w, btn_h);
        let is_clear_hovered = clear_rect.contains_point(self.cursor_pos);

        let clear_id = tree.create_node();
        if let Some(node) = tree.get_mut(clear_id) {
            node.set_name("ConsoleClearBtn");
            node.role = WidgetRole::Button;
            node.tag = CONSOLE_TAG_CLEAR;
            node.set_text("🧹 Clear");
            node.font_size = 11.0;
            node.line_height = btn_h;
            node.text_align = TextAlign::Center;
            node.text_color = if is_clear_hovered {
                Color::WHITE
            } else {
                self.style.clear_text
            };
            node.computed_rect = clear_rect;
            node.style = Style::new()
                .background(if is_clear_hovered {
                    self.style.clear_bg_hover
                } else {
                    self.style.clear_bg_idle
                })
                .border_radius(4.0)
                .border(
                    1.0,
                    if is_clear_hovered {
                        Color::rgba(0.40, 0.46, 0.60, 0.80)
                    } else {
                        self.style.clear_border
                    },
                );
        }
        let _ = tree.add_child(tb_id, clear_id);
        cur_x += clear_w + 10.0;

        // 3. Filter Tabs (All, Errors, Warnings, Info, Debug)
        let filters = [
            (
                ConsoleFilterLevel::All,
                "All",
                self.counts.all,
                58.0,
                Color::rgba(0.0, 0.85, 1.0, 0.95),
                CONSOLE_TAG_FILTER_ALL,
            ),
            (
                ConsoleFilterLevel::Error,
                "Errors",
                self.counts.error,
                76.0,
                Color::rgba(0.95, 0.30, 0.30, 0.95),
                CONSOLE_TAG_FILTER_ERROR,
            ),
            (
                ConsoleFilterLevel::Warn,
                "Warnings",
                self.counts.warn,
                88.0,
                Color::rgba(0.95, 0.70, 0.15, 0.95),
                CONSOLE_TAG_FILTER_WARN,
            ),
            (
                ConsoleFilterLevel::Info,
                "Info",
                self.counts.info,
                68.0,
                Color::rgba(0.20, 0.70, 0.95, 0.95),
                CONSOLE_TAG_FILTER_INFO,
            ),
            (
                ConsoleFilterLevel::Debug,
                "Debug",
                self.counts.debug,
                76.0,
                Color::rgba(0.65, 0.50, 0.95, 0.95),
                CONSOLE_TAG_FILTER_DEBUG,
            ),
        ];

        for (lvl, label, count, width, active_color, tag) in filters {
            let is_active = self.active_filter == lvl;
            let tab_rect = Rect::new(cur_x, btn_y, width, btn_h);
            let is_hovered = tab_rect.contains_point(self.cursor_pos);

            let btn_id = tree.create_node();
            if let Some(node) = tree.get_mut(btn_id) {
                node.set_name("FilterBtn");
                node.role = WidgetRole::Button;
                node.tag = tag;
                node.set_text(format!("{} ({})", label, count));
                node.font_size = 11.0;
                node.line_height = btn_h;
                node.text_align = TextAlign::Center;
                node.text_color = if is_active {
                    Color::WHITE
                } else if is_hovered {
                    Color::rgba(0.92, 0.94, 0.98, 1.0)
                } else {
                    Color::rgba(0.68, 0.72, 0.80, 1.0)
                };
                node.computed_rect = tab_rect;

                let (bg, border_c, border_w) = if is_active {
                    (Color::rgba(0.12, 0.16, 0.24, 0.95), active_color, 1.5)
                } else if is_hovered {
                    (
                        Color::rgba(0.16, 0.19, 0.26, 0.90),
                        Color::rgba(0.35, 0.40, 0.52, 0.70),
                        1.0,
                    )
                } else {
                    (
                        Color::rgba(0.10, 0.12, 0.16, 0.85),
                        Color::rgba(0.20, 0.23, 0.30, 0.55),
                        1.0,
                    )
                };

                node.style = Style::new()
                    .background(bg)
                    .border_radius(4.0)
                    .border(border_w, border_c);
            }
            let _ = tree.add_child(tb_id, btn_id);
            cur_x += width + 4.0;
        }
        cur_x += 8.0;

        // 4. Search Input Box
        let search_w = 210.0;
        let mut search_input_rect = None;

        if self.rect.right() - cur_x > search_w + 150.0 {
            let search_rect = Rect::new(cur_x, btn_y, search_w, btn_h);
            search_input_rect = Some(search_rect);
            let is_search_hovered = search_rect.contains_point(self.cursor_pos);

            let search_box_id = tree.create_node();
            if let Some(node) = tree.get_mut(search_box_id) {
                node.set_name("ConsoleSearchBox");
                node.tag = CONSOLE_TAG_SEARCH_INPUT;
                node.computed_rect = search_rect;
                let (border_c, border_w) = if self.is_search_focused {
                    (self.style.search_border_focus, 1.5)
                } else if is_search_hovered {
                    (Color::rgba(0.35, 0.40, 0.52, 0.70), 1.0)
                } else {
                    (self.style.search_border_idle, 1.0)
                };
                node.style = Style::new()
                    .background(self.style.search_bg)
                    .border_radius(4.0)
                    .border(border_w, border_c);
            }
            let _ = tree.add_child(tb_id, search_box_id);

            // Search Icon "🔍"
            let icon_id = tree.create_node();
            if let Some(node) = tree.get_mut(icon_id) {
                node.set_name("SearchIcon");
                node.interactive = false;
                node.set_text("🔍");
                node.font_size = 11.0;
                node.line_height = btn_h;
                node.text_color = Color::rgba(0.50, 0.54, 0.64, 1.0);
                node.computed_rect = Rect::new(cur_x + 7.0, btn_y, 14.0, btn_h);
            }
            let _ = tree.add_child(search_box_id, icon_id);

            // Display text or hint
            let search_text_id = tree.create_node();
            let display_text = if self.search_query.is_empty() {
                "Search logs..."
            } else {
                self.search_query
            };
            let text_color = if self.search_query.is_empty() {
                Color::rgba(0.40, 0.44, 0.54, 1.0)
            } else {
                Color::rgba(0.95, 0.96, 0.98, 1.0)
            };
            let text_start_x = if self.is_search_focused && self.search_query.is_empty() {
                cur_x + 26.5
            } else {
                cur_x + 24.0
            };
            let text_w = search_w - 44.0;
            if let Some(node) = tree.get_mut(search_text_id) {
                node.set_name("SearchQueryText");
                node.interactive = false;
                node.set_text(display_text);
                node.font_size = 11.0;
                node.line_height = btn_h;
                node.text_color = text_color;
                node.computed_rect = Rect::new(text_start_x, btn_y, text_w, btn_h);
            }
            let _ = tree.add_child(search_box_id, search_text_id);

            // Caret cursor
            if self.is_search_focused && self.blink_caret {
                let caret_x = if self.search_query.is_empty() {
                    cur_x + 24.0
                } else {
                    (cur_x + 24.0 + (self.search_query.len() as f32 * 6.6))
                        .min(cur_x + search_w - 24.0)
                };
                let caret_id = tree.create_node();
                if let Some(node) = tree.get_mut(caret_id) {
                    node.set_name("ConsoleSearchCaret");
                    node.interactive = false;
                    node.computed_rect = Rect::new(caret_x, btn_y + 4.0, 1.5, btn_h - 8.0);
                    node.style = Style::new()
                        .background(self.style.caret_color)
                        .border_radius(0.75);
                }
                let _ = tree.add_child(search_box_id, caret_id);
            }

            // Clear search "✖" button
            if !self.search_query.is_empty() {
                let clear_rect = Rect::new(cur_x + search_w - 20.0, btn_y + 3.0, 16.0, 18.0);
                let clr_id = tree.create_node();
                if let Some(node) = tree.get_mut(clr_id) {
                    node.set_name("SearchClearButton");
                    node.role = WidgetRole::Button;
                    node.tag = CONSOLE_TAG_SEARCH_CLEAR;
                    node.set_text("✖");
                    node.font_size = 9.5;
                    node.line_height = 18.0;
                    node.text_align = TextAlign::Center;
                    node.text_color = Color::rgba(0.60, 0.65, 0.75, 1.0);
                    node.computed_rect = clear_rect;
                }
                let _ = tree.add_child(search_box_id, clr_id);
            }
        }

        // 5. Auto-Scroll Toggle Button (Right-aligned)
        let auto_w = 98.0;
        let auto_rect = Rect::new(self.rect.right() - auto_w - 8.0, btn_y, auto_w, btn_h);
        let is_auto_hovered = auto_rect.contains_point(self.cursor_pos);

        let auto_id = tree.create_node();
        if let Some(node) = tree.get_mut(auto_id) {
            node.set_name("AutoScrollToggle");
            node.role = WidgetRole::Button;
            node.tag = CONSOLE_TAG_AUTOSCROLL;
            node.set_text(if self.auto_scroll {
                "✓ Auto-Scroll"
            } else {
                "⏸ Scroll Lock"
            });
            node.font_size = 11.0;
            node.line_height = btn_h;
            node.text_align = TextAlign::Center;
            node.text_color = if self.auto_scroll {
                Color::rgba(0.25, 0.85, 1.0, 1.0)
            } else if is_auto_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.65, 0.70, 0.78, 1.0)
            };
            node.computed_rect = auto_rect;

            let (bg, border_c) = if self.auto_scroll {
                (
                    Color::rgba(0.06, 0.22, 0.32, 0.75),
                    Color::rgba(0.14, 0.65, 0.95, 0.65),
                )
            } else if is_auto_hovered {
                (
                    Color::rgba(0.18, 0.21, 0.28, 0.85),
                    Color::rgba(0.35, 0.40, 0.50, 0.65),
                )
            } else {
                (
                    Color::rgba(0.11, 0.13, 0.17, 0.70),
                    Color::rgba(0.22, 0.25, 0.32, 0.50),
                )
            };

            node.style = Style::new()
                .background(bg)
                .border_radius(4.0)
                .border(1.0, border_c);
        }
        let _ = tree.add_child(tb_id, auto_id);

        // 6. Total logs count label to the left of Auto-Scroll
        let total_lbl_w = 64.0;
        let total_lbl_x = auto_rect.x - total_lbl_w - 8.0;
        if total_lbl_x > cur_x + search_w + 10.0 {
            let total_id = tree.create_node();
            if let Some(node) = tree.get_mut(total_id) {
                node.set_name("ConsoleTotalLabel");
                node.set_text(format!("{} logs", self.counts.all));
                node.font_size = 10.5;
                node.line_height = btn_h;
                node.text_align = TextAlign::Right;
                node.text_color = Color::rgba(0.45, 0.49, 0.58, 1.0);
                node.computed_rect = Rect::new(total_lbl_x, btn_y, total_lbl_w, btn_h);
            }
            let _ = tree.add_child(tb_id, total_id);
        }

        ConsoleToolbarFrame {
            toolbar_id: tb_id,
            toolbar_rect: self.rect,
            search_input_rect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_toolbar_builder_full_layout() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation");

        let tb_rect = Rect::new(0.0, 0.0, 1000.0, 34.0);
        let counts = ConsoleLogCounts::new(42, 2, 5, 20, 15);

        let frame = ConsoleToolbarBuilder::new(tb_rect)
            .active_filter(ConsoleFilterLevel::Error)
            .counts(counts)
            .search_query("test")
            .is_search_focused(true)
            .blink_caret(true)
            .auto_scroll(true)
            .build(&mut tree, root);

        assert_eq!(frame.toolbar_rect, tb_rect);
        assert!(frame.search_input_rect.is_some());

        // Verify toolbar container node
        let tb_node = tree.get(frame.toolbar_id).expect("Toolbar node");
        assert_eq!(tb_node.name.as_deref(), Some("ConsoleToolbar"));
        assert_eq!(tb_node.computed_rect, tb_rect);

        // Verify tags can be found in children
        let mut found_clear = false;
        let mut found_error_filter = false;
        let mut found_search_input = false;
        let mut found_autoscroll = false;

        for &child_id in &tb_node.children {
            if let Some(child) = tree.get(child_id) {
                if child.tag == CONSOLE_TAG_CLEAR {
                    found_clear = true;
                } else if child.tag == CONSOLE_TAG_FILTER_ERROR {
                    found_error_filter = true;
                } else if child.tag == CONSOLE_TAG_SEARCH_INPUT {
                    found_search_input = true;
                } else if child.tag == CONSOLE_TAG_AUTOSCROLL {
                    found_autoscroll = true;
                }
            }
        }

        assert!(found_clear, "Clear button was not found in toolbar");
        assert!(
            found_error_filter,
            "Error filter button was not found in toolbar"
        );
        assert!(
            found_search_input,
            "Search input box was not found in toolbar"
        );
        assert!(
            found_autoscroll,
            "Auto-scroll toggle was not found in toolbar"
        );
    }
}