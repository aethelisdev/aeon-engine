// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Console Rows Virtualized Renderer
//!
//! Efficiently renders the visible slice of log entries within the scrollable
//! console viewport using high-performance Iris UI retained-mode widget nodes.
//!

use super::types::{ConsoleFilterExt, ConsolePanelParams};
use irisui::prelude::*;

/// Standard height in physical pixels for a single console log row.
pub const CONSOLE_ROW_HEIGHT: f32 = 26.0;

/// Standard line height for text elements within a log row.
pub const CONSOLE_ROW_LINE_HEIGHT: f32 = 18.0;

/// Renders the virtualized slice of filtered log rows into the widget tree via declarative [`UiScope::virtual_scroll_area`].
///
/// Automatically manages frustum culling, overscan buffer margins, and sub-pixel scroll offset.
/// Returns the computed maximum scroll limit `max_scroll_y` in physical pixels.
pub fn build_console_rows(panel_scope: &mut UiScope<'_>, params: &ConsolePanelParams<'_>) -> f32 {
    let query_lower = params.search_query.to_lowercase();
    let has_query = !query_lower.is_empty();

    // 1. First pass: count and collect indices of matching entries
    let mut matching_indices: Vec<usize> = Vec::with_capacity(params.entries.len());
    for (idx, entry) in params.entries.iter().enumerate() {
        if !params.filter.matches(entry.level) {
            continue;
        }
        if has_query {
            let msg_matches = entry.msg.to_lowercase().contains(&query_lower);
            let target_matches = entry.target.to_lowercase().contains(&query_lower);
            if !msg_matches && !target_matches {
                continue;
            }
        }
        matching_indices.push(idx);
    }

    let total_filtered = matching_indices.len();
    let vp_height = (params.panel_rect.height - super::panel::CONSOLE_TOOLBAR_HEIGHT).max(10.0);

    if total_filtered == 0 {
        panel_scope.scroll_area_tagged(
            "ConsoleViewport",
            CONSOLE_TAG_VIEWPORT,
            Color::rgba(0.05, 0.06, 0.08, 0.98),
            |vp| {
                let msg = if has_query {
                    "No logs matching the search filter."
                } else {
                    "Console log is empty."
                };
                vp.console_empty_notice(msg);
            },
        );
        return 0.0;
    }

    // 2. Delegate windowing, overscan, and sub-pixel scrolling to pure declarative virtual_scroll_area
    let effective_scroll_y = if params.auto_scroll {
        f32::MAX
    } else {
        params.scroll_y
    };

    let config = VirtualScrollConfig::fixed(total_filtered, CONSOLE_ROW_HEIGHT)
        .with_overscan(2)
        .with_bg(Color::rgba(0.05, 0.06, 0.08, 0.98));

    panel_scope.virtual_scroll_area(
        "ConsoleViewport",
        CONSOLE_TAG_VIEWPORT,
        vp_height,
        effective_scroll_y,
        config,
        |row_scope, filtered_idx| {
            let entry_idx = matching_indices[filtered_idx];
            let entry = &params.entries[entry_idx];
            let is_striped = !filtered_idx.is_multiple_of(2);

            row_scope.console_row(
                convert_log_level(entry.level),
                &entry.timestamp,
                &entry.target,
                &entry.msg,
                false,
                is_striped,
            );
        },
    )
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