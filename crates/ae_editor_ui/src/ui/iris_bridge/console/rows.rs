// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Console Rows Virtualized Renderer
//!
//! Efficiently renders the visible slice of log entries within the scrollable
//! console viewport using high-performance Iris UI retained-mode widget nodes.

use super::types::{
    CONSOLE_TAG_ROW, CONSOLE_TAG_SCROLLBAR_THUMB, CONSOLE_TAG_SCROLLBAR_TRACK,
    CONSOLE_TAG_VIEWPORT, ConsoleEntry, ConsoleFilterExt, ConsoleFilterLevel, ConsoleLogLevel,
    ConsolePanelParams,
};
use irisui::prelude::*;

/// Standard height in physical pixels for a single console log row.
pub const CONSOLE_ROW_HEIGHT: f32 = 26.0;

/// Standard line height for text elements within a log row.
pub const CONSOLE_ROW_LINE_HEIGHT: f32 = 18.0;

/// Filters and collects the indices of log entries that match the severity level and search query.
fn collect_matching_log_indices(
    entries: &[ConsoleEntry],
    filter: ConsoleFilterLevel,
    search_query: &str,
) -> Vec<usize> {
    entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| filter.matches(entry.level))
        .filter(|(_, entry)| {
            search_query.is_empty()
                || entry
                    .msg
                    .to_lowercase()
                    .contains(&search_query.to_lowercase())
                || entry
                    .target
                    .to_lowercase()
                    .contains(&search_query.to_lowercase())
        })
        .map(|(idx, _)| idx)
        .collect()
}

/// Renders the virtualized slice of filtered log rows into the widget tree via declarative [`UiScope::virtual_list`].
///
/// Automatically manages frustum culling, overscan buffer margins, and sub-pixel scroll offset.
/// Connects to the declarative `.auto_scroll(params.auto_scroll)` hook to automatically disable
/// auto-scrolling when the user scrolls up via mouse wheel or scrollbar gestures.
pub fn build_console_rows(panel_scope: &mut UiScope<'_>, params: &mut ConsolePanelParams<'_>) {
    render_console_matching_rows(
        panel_scope,
        params,
        collect_matching_log_indices(params.entries, params.filter, params.search_query),
    );
}

/// Renders the virtualized slice of matching log rows inside the viewport.
fn render_console_matching_rows(
    panel_scope: &mut UiScope<'_>,
    params: &mut ConsolePanelParams<'_>,
    matching_indices: Vec<usize>,
) {
    panel_scope
        .virtual_list(matching_indices.len(), CONSOLE_ROW_HEIGHT)
        .name("ConsoleViewport")
        .tag(CONSOLE_TAG_VIEWPORT)
        .background(Color::rgba(0.05, 0.06, 0.08, 0.98))
        .scrollbar_tags(CONSOLE_TAG_SCROLLBAR_TRACK, CONSOLE_TAG_SCROLLBAR_THUMB)
        .overscan(2)
        .auto_scroll(params.auto_scroll)
        .empty_notice(if params.search_query.is_empty() {
            "Console log is empty."
        } else {
            "No logs matching the search filter."
        })
        .render_item(|row_scope, filtered_idx| {
            render_console_row(
                row_scope,
                convert_log_level(params.entries[matching_indices[filtered_idx]].level),
                &params.entries[matching_indices[filtered_idx]].timestamp,
                &params.entries[matching_indices[filtered_idx]].target,
                &params.entries[matching_indices[filtered_idx]].msg,
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

/// Converts a standard `log::Level` to the engine-independent `ConsoleLogLevel`.
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