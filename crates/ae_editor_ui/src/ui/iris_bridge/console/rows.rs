// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Console Rows Virtualized Renderer
//!
//! Efficiently renders the visible slice of log entries within the scrollable
//! console viewport using zero-`let`, zero-allocation retained [`UiScope::virtual_list`].

use super::types::{
    CONSOLE_TAG_ROW, CONSOLE_TAG_SCROLLBAR_THUMB, CONSOLE_TAG_SCROLLBAR_TRACK,
    CONSOLE_TAG_VIEWPORT, ConsoleEntry, ConsoleFilterLevel, ConsoleLogLevel, ConsolePanelState,
    sync_filtered_with_query,
};
use irisui::prelude::*;

/// Standard height in physical pixels for a single console log row.
pub const CONSOLE_ROW_HEIGHT: f32 = 26.0;

/// Standard line height for text elements within a log row.
pub const CONSOLE_ROW_LINE_HEIGHT: f32 = 18.0;

/// Filters matching entries into `state.filtered_indices` and renders the virtualized list.
pub fn build_console_rows(
    panel_scope: &mut UiScope<'_>,
    state: &mut ConsolePanelState,
    entries: &[ConsoleEntry],
) {
    state.sync_filtered_indices(entries);
    render_console_matching_rows(
        panel_scope,
        entries,
        &state.filtered_indices,
        &mut state.auto_scroll,
        state.search_input.text().is_empty(),
    );
}

pub(super) fn build_console_rows_fields(
    panel_scope: &mut UiScope<'_>,
    entries: &[ConsoleEntry],
    filtered_indices: &mut Vec<usize>,
    auto_scroll: &mut bool,
    filter: ConsoleFilterLevel,
    query: &str,
) {
    filtered_indices.clear();
    sync_filtered_with_query(filtered_indices, entries, filter, query);
    render_console_matching_rows(
        panel_scope,
        entries,
        filtered_indices,
        auto_scroll,
        query.is_empty(),
    );
}

fn render_console_matching_rows(
    panel_scope: &mut UiScope<'_>,
    entries: &[ConsoleEntry],
    matching_indices: &[usize],
    auto_scroll: &mut bool,
    search_empty: bool,
) {
    panel_scope
        .virtual_list(matching_indices.len(), CONSOLE_ROW_HEIGHT)
        .name("ConsoleViewport")
        .tag(CONSOLE_TAG_VIEWPORT)
        .background(Color::rgba(0.05, 0.06, 0.08, 0.98))
        .scrollbar_tags(CONSOLE_TAG_SCROLLBAR_TRACK, CONSOLE_TAG_SCROLLBAR_THUMB)
        .overscan(2)
        .auto_scroll(auto_scroll)
        .empty_notice(if search_empty {
            "Console log is empty."
        } else {
            "No logs matching the search filter."
        })
        .render_item(|row_scope, filtered_idx| {
            render_console_row(
                row_scope,
                convert_log_level(entries[matching_indices[filtered_idx]].level),
                &entries[matching_indices[filtered_idx]].timestamp,
                &entries[matching_indices[filtered_idx]].target,
                &entries[matching_indices[filtered_idx]].msg,
                false,
                !filtered_idx.is_multiple_of(2),
            );
        });
}

/// Renders a single console log row inside the virtualized scroll viewport.
///
/// Delegates directly to the declarative [`UiScope::log_row`] primitive.
pub fn render_console_row(
    row_scope: &mut UiScope<'_>,
    level: ConsoleLogLevel,
    timestamp: &str,
    target: &str,
    message: &str,
    is_hovered: bool,
    is_striped: bool,
) {
    row_scope.log_row(
        &LogRowDescriptor::new(
            CONSOLE_TAG_ROW,
            level.badge(),
            timestamp,
            target,
            message,
            is_striped,
        )
        .hovered(is_hovered),
    );
}

/// Renders a placeholder notice when the console log list is empty.
pub fn render_console_empty_notice(vp: &mut UiScope<'_>, message: &str) {
    vp.empty_notice("ConsoleEmptyNotice", message);
}

#[inline]
#[must_use]
const fn convert_log_level(level: log::Level) -> ConsoleLogLevel {
    match level {
        log::Level::Error => ConsoleLogLevel::Error,
        log::Level::Warn => ConsoleLogLevel::Warn,
        log::Level::Info => ConsoleLogLevel::Info,
        log::Level::Debug => ConsoleLogLevel::Debug,
        log::Level::Trace => ConsoleLogLevel::Trace,
    }
}