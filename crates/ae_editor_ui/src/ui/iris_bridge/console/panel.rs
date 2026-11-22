// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Main Panel Builder
//!
//! Orchestrates the wrapping toolbar, severity filter pills, zero-heap telemetry labels,
//! virtualized log feed, and multi-line scratchpad purely through zero-`let` [`UiScope`] calls.

use super::rows::build_console_rows_fields;
use super::types::{
    CONSOLE_TAG_AUTOSCROLL, CONSOLE_TAG_CLEAR, CONSOLE_TAG_FILTER_ALL, CONSOLE_TAG_FILTER_DEBUG,
    CONSOLE_TAG_FILTER_ERROR, CONSOLE_TAG_FILTER_INFO, CONSOLE_TAG_FILTER_WARN,
    CONSOLE_TAG_SCRATCHPAD_CLEAR, CONSOLE_TAG_SCRATCHPAD_EDITOR, CONSOLE_TAG_SCRATCHPAD_RESET,
    CONSOLE_TAG_SEARCH_CLEAR, CONSOLE_TAG_SEARCH_INPUT, CONSOLE_TAG_TAB_LOGS,
    CONSOLE_TAG_TAB_SCRATCHPAD, CONSOLE_TAG_TOOLBAR, ConsoleEntry, ConsoleFilterLevel,
    ConsoleLogCounts, ConsolePanelState, ConsoleTab, DEFAULT_SCRATCHPAD_TEXT,
};
use crate::ui::panel_layout::PanelId;
use crate::ui::panel_registry::EditorPanelContext;
use irisui::prelude::*;
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

struct ConsoleViewMut<'a> {
    search_input: &'a mut TextInputState,
    filter: &'a mut ConsoleFilterLevel,
    auto_scroll: &'a mut bool,
    active_tab: &'a mut ConsoleTab,
    scratchpad_state: &'a mut MultilineTextState,
    filtered_indices: &'a mut Vec<usize>,
}

#[inline]
fn notify_console_redraw(notifier: &mut Option<&mut DeferredNotifier<PanelId>>) {
    for n in notifier.iter_mut() {
        n.tag_redraw(PanelId::Console);
    }
}

impl ConsolePanelState {
    /// Builds the Developer Console panel directly through Iris UI workbench context.
    pub fn draw_panel(&mut self, scope: &mut UiScope<'_>, ctx: &mut EditorPanelContext<'_>) {
        build_console_panel_view(
            scope,
            ConsoleViewMut {
                search_input: &mut self.search_input,
                filter: &mut self.filter,
                auto_scroll: &mut self.auto_scroll,
                active_tab: &mut self.active_tab,
                scratchpad_state: &mut self.scratchpad_state,
                filtered_indices: &mut self.filtered_indices,
            },
            &self.entries,
            Some(ctx.notifier),
        );
    }
}

/// Builds the complete Developer Console panel tree purely using [`UiScope`].
pub fn build_console_panel(
    scope: &mut UiScope<'_>,
    state: &mut ConsolePanelState,
    entries: &[ConsoleEntry],
) {
    build_console_panel_notified(scope, state, entries, None);
}

/// Builds the Developer Console panel tree with an optional deferred redraw notifier.
pub fn build_console_panel_notified(
    scope: &mut UiScope<'_>,
    state: &mut ConsolePanelState,
    entries: &[ConsoleEntry],
    notifier: Option<&mut DeferredNotifier<PanelId>>,
) {
    build_console_panel_view(
        scope,
        ConsoleViewMut {
            search_input: &mut state.search_input,
            filter: &mut state.filter,
            auto_scroll: &mut state.auto_scroll,
            active_tab: &mut state.active_tab,
            scratchpad_state: &mut state.scratchpad_state,
            filtered_indices: &mut state.filtered_indices,
        },
        entries,
        notifier,
    );
}

fn build_console_panel_view(
    scope: &mut UiScope<'_>,
    mut view: ConsoleViewMut<'_>,
    entries: &[ConsoleEntry],
    mut notifier: Option<&mut DeferredNotifier<PanelId>>,
) {
    Toolbar::new("ConsoleToolbar")
        .height(CONSOLE_TOOLBAR_HEIGHT)
        .gap(6.0)
        .wrap()
        .tag(CONSOLE_TAG_TOOLBAR)
        .background(Color::rgba(0.02, 0.02, 0.03, 0.98))
        .show(scope, |tb| {
            tb.filter_pill_tagged(
                "📋 Logs",
                *view.active_tab == ConsoleTab::Logs,
                Color::rgba(0.0, 0.85, 1.0, 0.95),
                78.0,
                CONSOLE_TAG_TAB_LOGS,
            )
            .on_click(|| {
                *view.active_tab = ConsoleTab::Logs;
                notify_console_redraw(&mut notifier);
            });

            tb.filter_pill_tagged(
                "📝 Scratchpad",
                *view.active_tab == ConsoleTab::Scratchpad,
                Color::rgba(0.95, 0.70, 0.15, 0.95),
                116.0,
                CONSOLE_TAG_TAB_SCRATCHPAD,
            )
            .on_click(|| {
                *view.active_tab = ConsoleTab::Scratchpad;
                notify_console_redraw(&mut notifier);
            });

            tb.vertical_divider(18.0, Color::rgba(1.0, 1.0, 1.0, 0.12));

            if *view.active_tab == ConsoleTab::Logs {
                build_logs_toolbar(
                    tb,
                    &mut view,
                    &mut notifier,
                    ConsoleLogCounts::from_entries(entries),
                );
            } else {
                build_scratchpad_toolbar(tb, &mut view, &mut notifier);
            }
        });

    match *view.active_tab {
        ConsoleTab::Logs => build_console_rows_fields(
            scope,
            entries,
            view.filtered_indices,
            view.auto_scroll,
            *view.filter,
            view.search_input.text(),
        ),
        ConsoleTab::Scratchpad => {
            scope.multiline_editor_remaining(
                CONSOLE_TAG_SCRATCHPAD_EDITOR,
                CONSOLE_TOOLBAR_HEIGHT,
                view.scratchpad_state,
            );
        }
    }
}

fn build_logs_toolbar(
    tb: &mut irisui::widgets::declarative::ToolbarScope<'_, '_>,
    view: &mut ConsoleViewMut<'_>,
    notifier: &mut Option<&mut DeferredNotifier<PanelId>>,
    counts: ConsoleLogCounts,
) {
    tb.add(
        ToolbarButton::new("🧹 Clear")
            .width(76.0)
            .tag(CONSOLE_TAG_CLEAR),
    )
    .on_click(|| {
        ae_editor::editor_logger::LOGGER.clear();
        notify_console_redraw(notifier);
    });

    tb.vertical_divider(18.0, Color::rgba(1.0, 1.0, 1.0, 0.12));

    tb.add(
        TextInputField::new(view.search_input)
            .placeholder("Search logs...")
            .width(190.0)
            .search_icon()
            .clearable(true)
            .tag(CONSOLE_TAG_SEARCH_INPUT)
            .clear_tag(CONSOLE_TAG_SEARCH_CLEAR),
    )
    .on_change(|| notify_console_redraw(notifier));

    tb.vertical_divider(18.0, Color::rgba(1.0, 1.0, 1.0, 0.12));

    CONSOLE_FILTER_SPECS
        .iter()
        .for_each(|&(level, width, accent, tag)| {
            tb.filter_pill_tagged_fmt::<24, _>(*view.filter == level, accent, width, tag, |buf| {
                buf.write_fmt(format_args!(
                    "{} ({})",
                    level.label(),
                    counts.count_for(level)
                ))
                .ok();
            })
            .on_click(|| {
                *view.filter = level;
                notify_console_redraw(notifier);
            });
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
        Button::new(if *view.auto_scroll {
            "✓ Auto-Scroll"
        } else {
            "⏸ Auto-Scroll"
        })
        .small()
        .active(*view.auto_scroll)
        .tag(CONSOLE_TAG_AUTOSCROLL),
    )
    .on_click(|| {
        *view.auto_scroll = !*view.auto_scroll;
        notify_console_redraw(notifier);
    });
}

fn build_scratchpad_toolbar(
    tb: &mut irisui::widgets::declarative::ToolbarScope<'_, '_>,
    view: &mut ConsoleViewMut<'_>,
    notifier: &mut Option<&mut DeferredNotifier<PanelId>>,
) {
    tb.add(
        ToolbarButton::new("🧹 Clear")
            .width(76.0)
            .tag(CONSOLE_TAG_SCRATCHPAD_CLEAR),
    )
    .on_click(|| {
        view.scratchpad_state.clear();
        notify_console_redraw(notifier);
    });

    tb.add(
        ToolbarButton::new("↺ Reset Sample")
            .width(112.0)
            .tag(CONSOLE_TAG_SCRATCHPAD_RESET),
    )
    .on_click(|| {
        view.scratchpad_state.set_text(DEFAULT_SCRATCHPAD_TEXT);
        notify_console_redraw(notifier);
    });

    tb.vertical_divider(18.0, Color::rgba(1.0, 1.0, 1.0, 0.12));

    tb.label_fmt::<32, _>(
        100.0,
        11.0,
        Color::rgba(0.0, 0.85, 1.0, 0.9),
        TextAlign::Left,
        |buf| {
            buf.write_fmt(format_args!(
                "Line {}, Col {}",
                view.scratchpad_state.cursor.0 + 1,
                view.scratchpad_state.cursor.1 + 1
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
            buf.write_fmt(format_args!("{} lines", view.scratchpad_state.line_count()))
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