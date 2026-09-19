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

/// Renders the virtualized slice of filtered log rows into the widget tree.
pub fn build_console_rows(
    tree: &mut UiTree,
    viewport_node_id: WidgetId,
    params: &ConsolePanelParams<'_>,
    viewport_rect: Rect,
) -> (f32, f32) {
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
    let total_content_height =
        (total_filtered as f32 * CONSOLE_ROW_HEIGHT).max(viewport_rect.height);
    let max_scroll_y = (total_content_height - viewport_rect.height).max(0.0);

    if total_filtered == 0 {
        let msg = if has_query {
            "No logs matching the search filter."
        } else {
            "Console log is empty."
        };
        let notice_rect = Rect::new(
            viewport_rect.x + 24.0,
            viewport_rect.y + 24.0,
            viewport_rect.width - 48.0,
            24.0,
        );
        ConsoleEmptyNoticeBuilder::new(notice_rect, msg).build(tree, viewport_node_id);
        return (total_content_height, max_scroll_y);
    }

    // 2. Compute virtualized row slice
    let effective_scroll_y = if params.auto_scroll {
        max_scroll_y
    } else {
        params.scroll_y.clamp(0.0, max_scroll_y)
    };

    let start_idx = (effective_scroll_y / CONSOLE_ROW_HEIGHT).floor() as usize;
    let visible_count = (viewport_rect.height / CONSOLE_ROW_HEIGHT).ceil() as usize + 2;
    let end_idx = (start_idx + visible_count).min(total_filtered);

    // 3. Render visible rows via iris-widgets ConsoleRowBuilder
    for (offset, &entry_idx) in matching_indices[start_idx..end_idx].iter().enumerate() {
        let filtered_idx = start_idx + offset;
        let entry = &params.entries[entry_idx];

        let row_y =
            viewport_rect.y + (filtered_idx as f32 * CONSOLE_ROW_HEIGHT) - effective_scroll_y;
        if row_y + CONSOLE_ROW_HEIGHT <= viewport_rect.y || row_y >= viewport_rect.bottom() {
            continue;
        }

        let row_rect = Rect::new(
            viewport_rect.x,
            row_y,
            viewport_rect.width,
            CONSOLE_ROW_HEIGHT,
        );
        let is_hovered = row_rect.contains_point(params.cursor_pos);
        let is_striped = !filtered_idx.is_multiple_of(2);

        ConsoleRowBuilder::new(row_rect)
            .level(convert_log_level(entry.level))
            .timestamp(&entry.timestamp)
            .target(&entry.target)
            .message(&entry.msg)
            .is_hovered(is_hovered)
            .is_striped(is_striped)
            .build(tree, viewport_node_id);
    }

    (total_content_height, max_scroll_y)
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