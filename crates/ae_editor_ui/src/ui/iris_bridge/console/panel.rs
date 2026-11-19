// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Main Panel Builder
//!
//! Orchestrates the wrapping toolbar, severity filter pills, zero-heap telemetry labels,
//! virtualized log feed, and multi-line scratchpad purely through zero-`let` [`UiScope`] calls.

use super::rows::build_console_rows;
use super::types::{
    CONSOLE_TAG_AUTOSCROLL, CONSOLE_TAG_CLEAR, CONSOLE_TAG_FILTER_ALL, CONSOLE_TAG_FILTER_DEBUG,
    CONSOLE_TAG_FILTER_ERROR, CONSOLE_TAG_FILTER_INFO, CONSOLE_TAG_FILTER_WARN,
    CONSOLE_TAG_PANEL_ROOT, CONSOLE_TAG_SCRATCHPAD_CLEAR, CONSOLE_TAG_SCRATCHPAD_EDITOR,
    CONSOLE_TAG_SCRATCHPAD_RESET, CONSOLE_TAG_SEARCH_CLEAR, CONSOLE_TAG_SEARCH_INPUT,
    CONSOLE_TAG_TAB_LOGS, CONSOLE_TAG_TAB_SCRATCHPAD, CONSOLE_TAG_TOOLBAR, ConsoleEntry,
    ConsoleFilterLevel, ConsoleLogCounts, ConsolePanelState, ConsoleTab,
};
use irisui::prelude::*;
use irisui::widgets::declarative::InputBoxProps;
use std::fmt::Write;

/// Height of the console header toolbar in physical pixels.
pub const CONSOLE_TOOLBAR_HEIGHT: f32 = 34.0;

const CONSOLE_FILTER_SPECS: &[(ConsoleFilterLevel, f32, Color, u64)] = &[
    (
        ConsoleFilterLevel::All,
        58.0,
        Color::rgba(0.0, 0.85, 1.0, 0.95),
        CONSOLE_TAG_FILTER_ALL,
    ),
    (
        ConsoleFilterLevel::Error,
        76.0,
        Color::rgba(0.95, 0.30, 0.30, 0.95),
        CONSOLE_TAG_FILTER_ERROR,
    ),
    (
        ConsoleFilterLevel::Warn,
        88.0,
        Color::rgba(0.95, 0.70, 0.15, 0.95),
        CONSOLE_TAG_FILTER_WARN,
    ),
    (
        ConsoleFilterLevel::Info,
        68.0,
        Color::rgba(0.20, 0.70, 0.95, 0.95),
        CONSOLE_TAG_FILTER_INFO,
    ),
    (
        ConsoleFilterLevel::Debug,
        76.0,
        Color::rgba(0.65, 0.50, 0.95, 0.95),
        CONSOLE_TAG_FILTER_DEBUG,
    ),
];

/// Builds the complete Developer Console panel tree purely using [`UiScope`].
pub fn build_console_panel(
    scope: &mut UiScope<'_>,
    state: &mut ConsolePanelState,
    entries: &[ConsoleEntry],
    blink_caret: bool,
) {
    scope
        .panel("ConsolePanelRoot")
        .tagged(CONSOLE_TAG_PANEL_ROOT)
        .background(Color::rgba(0.05, 0.06, 0.08, 1.0))
        .show(|panel_scope| {
            Toolbar::new("ConsoleToolbar")
                .height(CONSOLE_TOOLBAR_HEIGHT)
                .gap(6.0)
                .wrap()
                .tag(CONSOLE_TAG_TOOLBAR)
                .background(Color::rgba(0.02, 0.02, 0.03, 0.98))
                .show(panel_scope, |tb| {
                    tb.filter_pill_tagged(
                        "📋 Logs",
                        state.active_tab == ConsoleTab::Logs,
                        Color::rgba(0.0, 0.85, 1.0, 0.95),
                        78.0,
                        CONSOLE_TAG_TAB_LOGS,
                    )
                    .on_click(|| state.select_tab(ConsoleTab::Logs));

                    tb.filter_pill_tagged(
                        "📝 Scratchpad",
                        state.active_tab == ConsoleTab::Scratchpad,
                        Color::rgba(0.95, 0.70, 0.15, 0.95),
                        116.0,
                        CONSOLE_TAG_TAB_SCRATCHPAD,
                    )
                    .on_click(|| state.select_tab(ConsoleTab::Scratchpad));

                    tb.vertical_divider(18.0, Color::rgba(1.0, 1.0, 1.0, 0.12));

                    if state.active_tab == ConsoleTab::Logs {
                        build_logs_toolbar(
                            tb,
                            state,
                            ConsoleLogCounts::from_entries(entries),
                            blink_caret,
                        );
                    } else {
                        build_scratchpad_toolbar(tb, state);
                    }
                });

            match state.active_tab {
                ConsoleTab::Logs => build_console_rows(panel_scope, state, entries),
                ConsoleTab::Scratchpad => {
                    panel_scope.multiline_editor_remaining(
                        CONSOLE_TAG_SCRATCHPAD_EDITOR,
                        CONSOLE_TOOLBAR_HEIGHT,
                        &mut state.scratchpad_state,
                    );
                }
            }
        });
}

fn build_logs_toolbar(
    tb: &mut irisui::widgets::declarative::ToolbarScope<'_, '_>,
    state: &mut ConsolePanelState,
    counts: ConsoleLogCounts,
    blink_caret: bool,
) {
    tb.add(
        ToolbarButton::new("🧹 Clear")
            .width(76.0)
            .tag(CONSOLE_TAG_CLEAR),
    )
    .on_click(|| state.clear_logs(tb.scope_mut()));

    tb.vertical_divider(18.0, Color::rgba(1.0, 1.0, 1.0, 0.12));

    tb.input_box_tagged(
        &InputBoxProps::new(
            state.search_input.text(),
            "Search logs...",
            190.0,
            false,
            blink_caret,
        )
        .with_icon("🔍")
        .with_cursor_byte_idx(state.search_input.cursor_byte_idx)
        .with_clear_tag(CONSOLE_TAG_SEARCH_CLEAR),
        CONSOLE_TAG_SEARCH_INPUT,
    );
    if tb.is_tag_clicked(CONSOLE_TAG_SEARCH_CLEAR) {
        state.clear_search(tb.scope_mut());
    }

    tb.vertical_divider(18.0, Color::rgba(1.0, 1.0, 1.0, 0.12));

    CONSOLE_FILTER_SPECS
        .iter()
        .for_each(|&(level, width, accent, tag)| {
            tb.filter_pill_tagged_fmt::<24, _>(state.filter == level, accent, width, tag, |buf| {
                buf.write_fmt(format_args!(
                    "{} ({})",
                    level.label(),
                    counts.count_for(level)
                ))
                .ok();
            })
            .on_click(|| state.set_filter(level));
        });

    tb.spacer();

    tb.label_fmt::<24, _>(
        64.0,
        10.5,
        Color::rgba(0.45, 0.49, 0.58, 1.0),
        TextAlign::Right,
        |buf| {
            buf.write_fmt(format_args!("{} logs", counts.all)).ok();
        },
    );

    tb.add(
        Button::new(if state.auto_scroll {
            "✓ Auto-Scroll"
        } else {
            "⏸ Auto-Scroll"
        })
        .small()
        .active(state.auto_scroll)
        .tag(CONSOLE_TAG_AUTOSCROLL),
    )
    .on_click(|| state.toggle_auto_scroll());
}

fn build_scratchpad_toolbar(
    tb: &mut irisui::widgets::declarative::ToolbarScope<'_, '_>,
    state: &mut ConsolePanelState,
) {
    tb.add(
        ToolbarButton::new("🧹 Clear")
            .width(76.0)
            .tag(CONSOLE_TAG_SCRATCHPAD_CLEAR),
    )
    .on_click(|| state.clear_scratchpad());

    tb.add(
        ToolbarButton::new("↺ Reset Sample")
            .width(112.0)
            .tag(CONSOLE_TAG_SCRATCHPAD_RESET),
    )
    .on_click(|| state.reset_scratchpad());

    tb.vertical_divider(18.0, Color::rgba(1.0, 1.0, 1.0, 0.12));

    tb.label_fmt::<32, _>(
        100.0,
        11.0,
        Color::rgba(0.0, 0.85, 1.0, 0.9),
        TextAlign::Left,
        |buf| {
            buf.write_fmt(format_args!(
                "Line {}, Col {}",
                state.scratchpad_state.cursor.0 + 1,
                state.scratchpad_state.cursor.1 + 1
            ))
            .ok();
        },
    );

    tb.label_fmt::<24, _>(
        60.0,
        11.0,
        Color::rgba(0.65, 0.70, 0.80, 1.0),
        TextAlign::Left,
        |buf| {
            buf.write_fmt(format_args!(
                "{} lines",
                state.scratchpad_state.line_count()
            ))
            .ok();
        },
    );

    tb.spacer();

    tb.label_with_width(
        "Click to focus • Type/Arrow/Enter/Backspace",
        270.0,
        11.0,
        Color::rgba(0.45, 0.49, 0.58, 1.0),
        TextAlign::Right,
    );
}