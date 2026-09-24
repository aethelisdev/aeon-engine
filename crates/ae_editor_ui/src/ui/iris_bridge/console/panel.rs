// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Main Panel Builder
//!
//! Orchestrates the top toolbar and virtualized scrollable log entries viewport
//! exclusively through declarative [`UiScope`] sub-methods.
//!

use super::rows::build_console_rows;
use super::types::ConsolePanelParams;
use irisui::prelude::*;

/// Height of the console header toolbar in physical pixels.
pub const CONSOLE_TOOLBAR_HEIGHT: f32 = 34.0;

/// Constructs the complete Developer Console panel widget hierarchy into the Iris `UiTree`.
///
/// Uses pure declarative [`UiScope`] containers (`panel_tagged`, `toolbar_tagged`, `scroll_area_tagged`).
/// Returns the computed maximum vertical scroll limit in physical pixels.
pub fn build_console_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &ConsolePanelParams<'_>,
) -> f32 {
    // 1. Count metrics for toolbar badges
    let mut count_err = 0;
    let mut count_warn = 0;
    let mut count_info = 0;
    let mut count_debug = 0;
    for entry in params.entries {
        match entry.level {
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

    let mut max_scroll = 0.0_f32;

    // 2. Panel Base Container via declarative UiScope
    let mut scope = UiScope::with_tagged_interactions(tree, parent_id, &[], params.hovered_tag);
    scope.panel_tagged(
        "ConsolePanelRoot",
        params.panel_rect,
        CONSOLE_TAG_PANEL_ROOT,
        Color::rgba(0.05, 0.06, 0.08, 1.0),
        |panel_scope| {
            // 3. Declarative Toolbar Header Bar
            panel_scope.toolbar_tagged(
                "ConsoleToolbar",
                CONSOLE_TOOLBAR_HEIGHT,
                CONSOLE_TAG_TOOLBAR,
                Color::rgba(0.02, 0.02, 0.03, 0.98),
                |tb| {
                    tb.toolbar_button_tagged("🧹 Clear", 76.0, CONSOLE_TAG_CLEAR);
                    tb.vertical_divider(18.0, Color::rgba(1.0, 1.0, 1.0, 0.12));

                    let filters = [
                        (
                            ConsoleFilterLevel::All,
                            format!("All ({})", counts.all),
                            58.0_f32,
                            Color::rgba(0.0, 0.85, 1.0, 0.95),
                            CONSOLE_TAG_FILTER_ALL,
                        ),
                        (
                            ConsoleFilterLevel::Error,
                            format!("Errors ({})", counts.error),
                            76.0_f32,
                            Color::rgba(0.95, 0.30, 0.30, 0.95),
                            CONSOLE_TAG_FILTER_ERROR,
                        ),
                        (
                            ConsoleFilterLevel::Warn,
                            format!("Warnings ({})", counts.warn),
                            88.0_f32,
                            Color::rgba(0.95, 0.70, 0.15, 0.95),
                            CONSOLE_TAG_FILTER_WARN,
                        ),
                        (
                            ConsoleFilterLevel::Info,
                            format!("Info ({})", counts.info),
                            68.0_f32,
                            Color::rgba(0.20, 0.70, 0.95, 0.95),
                            CONSOLE_TAG_FILTER_INFO,
                        ),
                        (
                            ConsoleFilterLevel::Debug,
                            format!("Debug ({})", counts.debug),
                            76.0_f32,
                            Color::rgba(0.65, 0.50, 0.95, 0.95),
                            CONSOLE_TAG_FILTER_DEBUG,
                        ),
                    ];
                    for (filter_lvl, label, width, color, tag) in filters {
                        let is_active = params.filter == filter_lvl;
                        tb.filter_pill_tagged(&label, is_active, color, width, tag);
                    }

                    tb.vertical_divider(18.0, Color::rgba(1.0, 1.0, 1.0, 0.12));
                    let blink = params.is_search_focused && params.blink_caret;
                    let search_props = InputBoxProps::new(
                        params.search_query,
                        "Search logs...",
                        210.0,
                        params.is_search_focused,
                        blink,
                    )
                    .with_icon("🔍");
                    tb.input_box_tagged(&search_props, CONSOLE_TAG_SEARCH_INPUT);
                    if !params.search_query.is_empty() {
                        tb.toolbar_button_tagged("✖", 20.0, CONSOLE_TAG_SEARCH_CLEAR);
                    }

                    // Spacer pushes Auto-Scroll and log count to the right edge
                    tb.spacer();

                    tb.label_with_width(
                        format!("{} logs", counts.all),
                        64.0,
                        10.5,
                        Color::rgba(0.45, 0.49, 0.58, 1.0),
                        TextAlign::Right,
                    );

                    let auto_scroll_label = if params.auto_scroll {
                        "✓ Auto-Scroll"
                    } else {
                        "⏸ Auto-Scroll"
                    };
                    tb.toggle_pill_tagged(
                        auto_scroll_label,
                        params.auto_scroll,
                        CONSOLE_TAG_AUTOSCROLL,
                    );
                },
            );

            // 4. Scrollable Log Rows Viewport via declarative virtual_scroll_area
            max_scroll = build_console_rows(panel_scope, params);

            // 5. Scrollbar overlay indicator when content overflows viewport
            let vp_h = (params.panel_rect.height - CONSOLE_TOOLBAR_HEIGHT).max(10.0);
            if max_scroll > 0.0 {
                let content_h = vp_h + max_scroll;
                let effective_scroll_y = if params.auto_scroll {
                    max_scroll
                } else {
                    params.scroll_y.clamp(0.0, max_scroll)
                };
                let vp_rect = Rect::new(
                    params.panel_rect.x,
                    params.panel_rect.y + CONSOLE_TOOLBAR_HEIGHT,
                    params.panel_rect.width,
                    vp_h,
                );
                let scroll_style = ScrollAreaStyle::dark_default();
                if let Some(geom) = ScrollBarGeometry::compute_vertical(
                    vp_rect,
                    content_h,
                    effective_scroll_y,
                    &scroll_style,
                ) {
                    panel_scope.scrollbar_vertical(
                        geom,
                        params.panel_rect,
                        params.is_scrollbar_dragging,
                        Some(params.cursor_pos),
                        CONSOLE_TAG_SCROLLBAR_TRACK,
                        CONSOLE_TAG_SCROLLBAR_THUMB,
                    );
                }
            }
        },
    );

    max_scroll
}