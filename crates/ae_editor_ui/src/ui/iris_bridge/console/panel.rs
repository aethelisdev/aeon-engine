// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Main Panel Builder
//!
//! Orchestrates the top toolbar and virtualized scrollable log entries viewport
//! exclusively through declarative [`UiScope`] sub-methods.
//!

use super::rows::build_console_rows;
use super::types::{
    CONSOLE_TAG_AUTOSCROLL, CONSOLE_TAG_CLEAR, CONSOLE_TAG_FILTER_ALL, CONSOLE_TAG_FILTER_DEBUG,
    CONSOLE_TAG_FILTER_ERROR, CONSOLE_TAG_FILTER_INFO, CONSOLE_TAG_FILTER_WARN,
    CONSOLE_TAG_PANEL_ROOT, CONSOLE_TAG_SEARCH_CLEAR, CONSOLE_TAG_SEARCH_INPUT,
    CONSOLE_TAG_TOOLBAR, ConsoleAction, ConsoleFilterLevel, ConsoleLogCounts, ConsolePanelParams,
};
use irisui::prelude::*;

/// Height of the console header toolbar in physical pixels.
pub const CONSOLE_TOOLBAR_HEIGHT: f32 = 34.0;

/// Constructs the complete Developer Console panel widget hierarchy into the Iris `UiTree`.
///
/// Uses pure declarative [`UiScope`] containers (`panel_tagged`, `toolbar_tagged`, `scroll_area_tagged`).
pub fn build_console_panel(
    scope: &mut UiScope<'_>,
    params: &ConsolePanelParams<'_>,
    actions: &mut Vec<ConsoleAction>,
) {
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

    // 2. Panel Base Container via declarative UiScope
    scope
        .panel("ConsolePanelRoot")
        .tagged(CONSOLE_TAG_PANEL_ROOT)
        .bounds(params.panel_rect)
        .background(Color::rgba(0.05, 0.06, 0.08, 1.0))
        .show(|panel_scope| {
            // 3. Declarative Toolbar Header Bar
            Toolbar::new("ConsoleToolbar")
                .height(CONSOLE_TOOLBAR_HEIGHT)
                .tag(CONSOLE_TAG_TOOLBAR)
                .background(Color::rgba(0.02, 0.02, 0.03, 0.98))
                .show(panel_scope, |tb| {
                    if tb
                        .add(
                            ToolbarButton::new("🧹 Clear")
                                .width(76.0)
                                .tag(CONSOLE_TAG_CLEAR),
                        )
                        .clicked()
                    {
                        actions.push(ConsoleAction::ClearLogs);
                    }
                    tb.vertical_divider(18.0, Color::rgba(1.0, 1.0, 1.0, 0.12));

                    let is_search_focused =
                        params.focus_state.is_tag_focused(CONSOLE_TAG_SEARCH_INPUT);
                    let search_props = InputBoxProps::new(
                        params.search_query,
                        "Search logs...",
                        200.0,
                        is_search_focused,
                        is_search_focused && params.blink_caret,
                    )
                    .with_icon("🔍")
                    .with_cursor_byte_idx(params.cursor_byte_idx)
                    .with_clear_tag(CONSOLE_TAG_SEARCH_CLEAR);
                    if tb
                        .input_box_tagged(&search_props, CONSOLE_TAG_SEARCH_INPUT)
                        .clicked()
                    {
                        actions.push(ConsoleAction::FocusSearch);
                    }

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
                    for (filter_lvl, label, width, color, tag) in &filters {
                        let is_active = params.filter == *filter_lvl;
                        if tb
                            .filter_pill_tagged(label, is_active, *color, *width, *tag)
                            .clicked()
                        {
                            actions.push(ConsoleAction::SetFilter(*filter_lvl));
                        }
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

                    if tb
                        .add(
                            Button::new(if params.auto_scroll {
                                "✓ Auto-Scroll"
                            } else {
                                "⏸ Auto-Scroll"
                            })
                            .small()
                            .active(params.auto_scroll)
                            .tag(CONSOLE_TAG_AUTOSCROLL),
                        )
                        .clicked()
                    {
                        actions.push(ConsoleAction::ToggleAutoScroll);
                    }
                });

            // 4. Scrollable Log Rows Viewport via declarative virtual_scroll_area
            build_console_rows(panel_scope, params);
        });
}