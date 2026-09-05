// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Main Panel Builder
//!
//! Orchestrates the top toolbar (clear button, severity filters, auto-scroll toggle,
//! search query box) and the virtualized scrollable log entries viewport.
//!

use super::rows::build_console_rows;
use super::types::{ConsoleFilterLevel, ConsolePanelParams, ConsolePanelTargets};
use irisui::prelude::*;

/// Height of the console header toolbar in physical pixels.
pub const CONSOLE_TOOLBAR_HEIGHT: f32 = 34.0;

/// Constructs the complete Developer Console panel widget hierarchy into the Iris `UiTree`.
pub fn build_console_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &ConsolePanelParams<'_>,
    targets: &mut ConsolePanelTargets,
) {
    targets.panel_rect = params.panel_rect;

    // 1. Panel Base Container
    let root_id = tree.create_node();
    if let Some(node) = tree.get_mut(root_id) {
        node.set_name("ConsolePanelRoot");
        node.computed_rect = params.panel_rect;
        node.style = Style::new()
            .background(Color::rgba(0.05, 0.06, 0.08, 1.0))
            .clip_children(true);
    }
    let _ = tree.add_child(parent_id, root_id);

    // 2. Toolbar Header Bar
    let tb_rect = Rect::new(
        params.panel_rect.x,
        params.panel_rect.y,
        params.panel_rect.width,
        CONSOLE_TOOLBAR_HEIGHT,
    );
    let tb_id = tree.create_node();
    if let Some(node) = tree.get_mut(tb_id) {
        node.set_name("ConsoleToolbar");
        node.computed_rect = tb_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .border(1.0, Color::rgba(0.18, 0.21, 0.28, 0.70));
    }
    let _ = tree.add_child(root_id, tb_id);

    let mut cur_x = tb_rect.x + 8.0;
    let btn_y = tb_rect.y + 5.0;
    let btn_h = 24.0;

    // ── Clear Logs Button ──
    let clear_w = 76.0;
    let clear_rect = Rect::new(cur_x, btn_y, clear_w, btn_h);
    let is_clear_hovered = clear_rect.contains_point(params.cursor_pos);
    targets.clear_btn_rect = clear_rect;

    let clear_id = tree.create_node();
    if let Some(node) = tree.get_mut(clear_id) {
        node.set_name("ConsoleClearBtn");
        node.set_text("🧹 Clear");
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_clear_hovered {
            Color::WHITE
        } else {
            Color::rgba(0.80, 0.84, 0.90, 1.0)
        };
        node.computed_rect = clear_rect;
        node.style = Style::new()
            .background(if is_clear_hovered {
                Color::rgba(0.20, 0.24, 0.32, 1.0)
            } else {
                Color::rgba(0.13, 0.15, 0.20, 0.95)
            })
            .border_radius(4.0)
            .border(
                1.0,
                if is_clear_hovered {
                    Color::rgba(0.40, 0.46, 0.60, 0.80)
                } else {
                    Color::rgba(0.24, 0.27, 0.35, 0.60)
                },
            );
    }
    let _ = tree.add_child(tb_id, clear_id);
    cur_x += clear_w + 10.0;

    // ── Count metrics for badges ──
    let mut count_err = 0;
    let mut count_warn = 0;
    let mut count_info = 0;
    let mut count_debug = 0;
    for e in params.entries {
        match e.level {
            log::Level::Error => count_err += 1,
            log::Level::Warn => count_warn += 1,
            log::Level::Info => count_info += 1,
            log::Level::Debug | log::Level::Trace => count_debug += 1,
        }
    }
    let count_all = params.entries.len();

    // ── Filter Buttons (Elevated Modern Tabs) ──
    let build_filter_btn = |tree: &mut UiTree,
                            label: &str,
                            count: usize,
                            is_active: bool,
                            active_color: Color,
                            rect: Rect| {
        let is_hovered = rect.contains_point(params.cursor_pos);
        let btn_id = tree.create_node();
        if let Some(node) = tree.get_mut(btn_id) {
            node.set_name("FilterBtn");
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
            node.computed_rect = rect;

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
    };

    // 1. All
    let all_w = 58.0;
    let all_rect = Rect::new(cur_x, btn_y, all_w, btn_h);
    targets.filter_all_rect = all_rect;
    build_filter_btn(
        tree,
        "All",
        count_all,
        params.filter == ConsoleFilterLevel::All,
        Color::rgba(0.0, 0.85, 1.0, 0.95),
        all_rect,
    );
    cur_x += all_w + 4.0;

    // 2. Errors
    let err_w = 76.0;
    let err_rect = Rect::new(cur_x, btn_y, err_w, btn_h);
    targets.filter_error_rect = err_rect;
    build_filter_btn(
        tree,
        "Errors",
        count_err,
        params.filter == ConsoleFilterLevel::Error,
        Color::rgba(0.95, 0.30, 0.30, 0.95),
        err_rect,
    );
    cur_x += err_w + 4.0;

    // 3. Warnings
    let warn_w = 88.0;
    let warn_rect = Rect::new(cur_x, btn_y, warn_w, btn_h);
    targets.filter_warn_rect = warn_rect;
    build_filter_btn(
        tree,
        "Warnings",
        count_warn,
        params.filter == ConsoleFilterLevel::Warn,
        Color::rgba(0.95, 0.70, 0.15, 0.95),
        warn_rect,
    );
    cur_x += warn_w + 4.0;

    // 4. Info
    let info_w = 68.0;
    let info_rect = Rect::new(cur_x, btn_y, info_w, btn_h);
    targets.filter_info_rect = info_rect;
    build_filter_btn(
        tree,
        "Info",
        count_info,
        params.filter == ConsoleFilterLevel::Info,
        Color::rgba(0.20, 0.70, 0.95, 0.95),
        info_rect,
    );
    cur_x += info_w + 4.0;

    // 5. Debug
    let dbg_w = 76.0;
    let dbg_rect = Rect::new(cur_x, btn_y, dbg_w, btn_h);
    targets.filter_debug_rect = dbg_rect;
    build_filter_btn(
        tree,
        "Debug",
        count_debug,
        params.filter == ConsoleFilterLevel::Debug,
        Color::rgba(0.65, 0.50, 0.95, 0.95),
        dbg_rect,
    );
    cur_x += dbg_w + 12.0;

    // ── Search Input Field ──
    let search_w = 210.0;
    targets.search_clear_btn_rect = None;
    if tb_rect.right() - cur_x > search_w + 150.0 {
        let search_rect = Rect::new(cur_x, btn_y, search_w, btn_h);
        targets.search_input_rect = search_rect;
        let is_search_hovered = search_rect.contains_point(params.cursor_pos);

        // Outer Search Box container
        let search_box_id = tree.create_node();
        if let Some(node) = tree.get_mut(search_box_id) {
            node.set_name("ConsoleSearchBox");
            node.computed_rect = search_rect;
            let (border_c, border_w) = if params.is_search_focused {
                (Color::rgba(0.0, 0.90, 1.0, 0.95), 1.5)
            } else if is_search_hovered {
                (Color::rgba(0.35, 0.40, 0.52, 0.70), 1.0)
            } else {
                (Color::rgba(0.20, 0.23, 0.30, 0.60), 1.0)
            };
            node.style = Style::new()
                .background(Color::rgba(0.06, 0.07, 0.09, 0.95))
                .border_radius(4.0)
                .border(border_w, border_c);
        }
        let _ = tree.add_child(tb_id, search_box_id);

        // Search Icon "🔍"
        let icon_id = tree.create_node();
        if let Some(node) = tree.get_mut(icon_id) {
            node.set_name("SearchIcon");
            node.set_text("🔍");
            node.font_size = 11.0;
            node.line_height = btn_h;
            node.text_color = Color::rgba(0.50, 0.54, 0.64, 1.0);
            node.computed_rect = Rect::new(cur_x + 7.0, btn_y, 14.0, btn_h);
        }
        let _ = tree.add_child(search_box_id, icon_id);

        // Search Query or Hint Text
        let search_text_id = tree.create_node();
        let display_text = if params.search_query.is_empty() {
            "Search logs..."
        } else {
            params.search_query
        };
        let text_color = if params.search_query.is_empty() {
            Color::rgba(0.40, 0.44, 0.54, 1.0)
        } else {
            Color::rgba(0.95, 0.96, 0.98, 1.0)
        };
        let text_start_x = if params.is_search_focused && params.search_query.is_empty() {
            cur_x + 26.5
        } else {
            cur_x + 24.0
        };
        let text_w = search_w - 44.0;
        if let Some(node) = tree.get_mut(search_text_id) {
            node.set_name("SearchQueryText");
            node.set_text(display_text);
            node.font_size = 11.0;
            node.line_height = btn_h;
            node.text_color = text_color;
            node.computed_rect = Rect::new(text_start_x, btn_y, text_w, btn_h);
        }
        let _ = tree.add_child(search_box_id, search_text_id);

        // Blinking Caret Cursor (500ms cycle)
        if params.is_search_focused && params.blink_caret {
            let caret_x = if params.search_query.is_empty() {
                cur_x + 24.0
            } else {
                (cur_x + 24.0 + (params.search_query.len() as f32 * 6.6))
                    .min(cur_x + search_w - 24.0)
            };
            let caret_id = tree.create_node();
            if let Some(node) = tree.get_mut(caret_id) {
                node.set_name("ConsoleSearchCaret");
                node.computed_rect = Rect::new(caret_x, btn_y + 4.0, 1.5, btn_h - 8.0);
                node.style = Style::new()
                    .background(Color::rgba(0.0, 0.90, 1.0, 1.0))
                    .border_radius(0.75);
            }
            let _ = tree.add_child(search_box_id, caret_id);
        }

        // Clear Search "✖" Button
        if !params.search_query.is_empty() {
            let clear_rect = Rect::new(cur_x + search_w - 20.0, btn_y + 3.0, 16.0, 18.0);
            targets.search_clear_btn_rect = Some(clear_rect);

            let clr_id = tree.create_node();
            if let Some(node) = tree.get_mut(clr_id) {
                node.set_name("SearchClearButton");
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

    // ── Auto-Scroll Toggle & Status (Right-aligned) ──
    let auto_w = 98.0;
    let auto_rect = Rect::new(tb_rect.right() - auto_w - 8.0, btn_y, auto_w, btn_h);
    targets.autoscroll_toggle_rect = auto_rect;
    let is_auto_hovered = auto_rect.contains_point(params.cursor_pos);

    let auto_id = tree.create_node();
    if let Some(node) = tree.get_mut(auto_id) {
        node.set_name("AutoScrollToggle");
        node.set_text(if params.auto_scroll {
            "✓ Auto-Scroll"
        } else {
            "⏸ Scroll Lock"
        });
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = if params.auto_scroll {
            Color::rgba(0.25, 0.85, 1.0, 1.0)
        } else if is_auto_hovered {
            Color::WHITE
        } else {
            Color::rgba(0.65, 0.70, 0.78, 1.0)
        };
        node.computed_rect = auto_rect;

        let (bg, border_c) = if params.auto_scroll {
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

    // Total logs count label to the left of Auto-Scroll
    let total_lbl_w = 64.0;
    let total_lbl_x = auto_rect.x - total_lbl_w - 8.0;
    if total_lbl_x > cur_x + search_w + 10.0 {
        let total_id = tree.create_node();
        if let Some(node) = tree.get_mut(total_id) {
            node.set_name("ConsoleTotalLabel");
            node.set_text(format!("{} logs", count_all));
            node.font_size = 10.5;
            node.line_height = btn_h;
            node.text_align = TextAlign::Right;
            node.text_color = Color::rgba(0.45, 0.49, 0.58, 1.0);
            node.computed_rect = Rect::new(total_lbl_x, btn_y, total_lbl_w, btn_h);
        }
        let _ = tree.add_child(tb_id, total_id);
    }

    // 3. Scrollable Log Rows Viewport
    let vp_y = params.panel_rect.y + CONSOLE_TOOLBAR_HEIGHT + 1.0;
    let vp_h = (params.panel_rect.height - CONSOLE_TOOLBAR_HEIGHT - 2.0).max(10.0);
    let vp_rect = Rect::new(params.panel_rect.x, vp_y, params.panel_rect.width, vp_h);
    targets.rows_viewport_rect = vp_rect;

    let vp_id = tree.create_node();
    if let Some(node) = tree.get_mut(vp_id) {
        node.set_name("ConsoleViewport");
        node.computed_rect = vp_rect;
        node.style = Style::new()
            .background(Color::rgba(0.05, 0.06, 0.08, 0.98))
            .clip_children(true);
    }
    let _ = tree.add_child(root_id, vp_id);

    // 4. Render Rows
    let (content_h, max_scroll) = build_console_rows(tree, vp_id, params, vp_rect);
    targets.total_content_height = content_h;
    targets.max_scroll_y = max_scroll;
}