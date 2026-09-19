// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Main Panel Builder
//!
//! Orchestrates the top toolbar via `ConsoleToolbarBuilder` and the
//! virtualized scrollable log entries viewport.
//!

use super::rows::build_console_rows;
use super::types::{ConsolePanelParams, ConsolePanelTargets};
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

    // 2. Count metrics for toolbar badges
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
    let counts = ConsoleLogCounts::new(
        params.entries.len(),
        count_err,
        count_warn,
        count_info,
        count_debug,
    );

    // 3. Toolbar Header Bar via iris-widgets ConsoleToolbarBuilder
    let tb_rect = Rect::new(
        params.panel_rect.x,
        params.panel_rect.y,
        params.panel_rect.width,
        CONSOLE_TOOLBAR_HEIGHT,
    );
    let tb_frame = ConsoleToolbarBuilder::new(tb_rect)
        .active_filter(params.filter)
        .counts(counts)
        .search_query(params.search_query)
        .is_search_focused(params.is_search_focused)
        .blink_caret(params.blink_caret)
        .auto_scroll(params.auto_scroll)
        .cursor_pos(params.cursor_pos)
        .build(tree, root_id);

    if let Some(search_rect) = tb_frame.search_input_rect {
        targets.search_input_rect = search_rect;
    }

    // 4. Scrollable Log Rows Viewport
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

    // 5. Render Rows
    let (content_h, max_scroll) = build_console_rows(tree, vp_id, params, vp_rect);
    targets.total_content_height = content_h;
    targets.max_scroll_y = max_scroll;
}