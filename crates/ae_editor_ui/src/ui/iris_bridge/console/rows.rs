// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Console Rows Virtualized Renderer
//!
//! Efficiently renders the visible slice of log entries within the scrollable
//! console viewport using high-performance Iris UI retained-mode widget nodes.
//!

use super::types::ConsolePanelParams;
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
        // Empty state placeholder
        let empty_id = tree.create_node();
        if let Some(node) = tree.get_mut(empty_id) {
            node.set_name("ConsoleEmptyNotice");
            node.set_text(if has_query {
                "No logs matching the search filter."
            } else {
                "Console log is empty."
            });
            node.font_size = 12.0;
            node.line_height = 24.0;
            node.text_color = Color::rgba(0.50, 0.54, 0.64, 1.0);
            node.computed_rect = Rect::new(
                viewport_rect.x + 24.0,
                viewport_rect.y + 24.0,
                viewport_rect.width - 48.0,
                24.0,
            );
        }
        let _ = tree.add_child(viewport_node_id, empty_id);
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

    // 3. Render visible rows
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

        // Row background with subtle contrast zebra striping and smooth hover highlight
        let bg_color = if is_hovered {
            Color::rgba(0.14, 0.17, 0.23, 0.95)
        } else if filtered_idx.is_multiple_of(2) {
            Color::rgba(0.06, 0.07, 0.09, 0.95)
        } else {
            Color::rgba(0.08, 0.09, 0.12, 0.95)
        };

        let row_id = tree.create_node();
        if let Some(node) = tree.get_mut(row_id) {
            node.set_name("ConsoleRow");
            node.computed_rect = row_rect;
            node.style = Style::new().background(bg_color);
        }
        let _ = tree.add_child(viewport_node_id, row_id);

        // Left accent strip indicator on hover ( /  IDE standard)
        if is_hovered {
            let hover_bar_id = tree.create_node();
            if let Some(node) = tree.get_mut(hover_bar_id) {
                node.set_name("RowHoverAccent");
                node.computed_rect = Rect::new(row_rect.x, row_y, 3.0, CONSOLE_ROW_HEIGHT);
                node.style = Style::new().background(Color::rgba(0.0, 0.85, 1.0, 0.85));
            }
            let _ = tree.add_child(row_id, hover_bar_id);
        }

        let mut cur_x = row_rect.x + 10.0;
        let text_y = row_y + (CONSOLE_ROW_HEIGHT - CONSOLE_ROW_LINE_HEIGHT) * 0.5;

        // Level Badge
        let (badge_text, badge_color, badge_bg, badge_border) = match entry.level {
            log::Level::Error => (
                "ERR",
                Color::rgba(0.98, 0.45, 0.45, 1.0),
                Color::rgba(0.38, 0.08, 0.08, 0.90),
                Color::rgba(0.95, 0.30, 0.30, 0.50),
            ),
            log::Level::Warn => (
                "WRN",
                Color::rgba(0.98, 0.78, 0.20, 1.0),
                Color::rgba(0.35, 0.20, 0.02, 0.90),
                Color::rgba(0.95, 0.70, 0.15, 0.50),
            ),
            log::Level::Info => (
                "INF",
                Color::rgba(0.25, 0.78, 0.98, 1.0),
                Color::rgba(0.05, 0.20, 0.32, 0.90),
                Color::rgba(0.20, 0.70, 0.95, 0.50),
            ),
            log::Level::Debug => (
                "DBG",
                Color::rgba(0.75, 0.60, 0.98, 1.0),
                Color::rgba(0.18, 0.10, 0.32, 0.90),
                Color::rgba(0.65, 0.45, 0.95, 0.50),
            ),
            log::Level::Trace => (
                "TRC",
                Color::rgba(0.60, 0.66, 0.75, 1.0),
                Color::rgba(0.12, 0.15, 0.20, 0.90),
                Color::rgba(0.40, 0.45, 0.55, 0.50),
            ),
        };

        let badge_w = 40.0;
        let badge_id = tree.create_node();
        if let Some(node) = tree.get_mut(badge_id) {
            node.set_name("LevelBadge");
            node.set_text(badge_text);
            node.font_size = 9.5;
            node.line_height = CONSOLE_ROW_LINE_HEIGHT;
            node.text_align = TextAlign::Center;
            node.text_color = badge_color;
            node.computed_rect = Rect::new(cur_x, text_y, badge_w, CONSOLE_ROW_LINE_HEIGHT);
            node.style = Style::new()
                .background(badge_bg)
                .border_radius(3.5)
                .border(1.0, badge_border);
        }
        let _ = tree.add_child(row_id, badge_id);
        cur_x += badge_w + 10.0;

        // Timestamp (Clear readable slate)
        let time_w = 66.0;
        let time_id = tree.create_node();
        if let Some(node) = tree.get_mut(time_id) {
            node.set_name("LogTimestamp");
            node.set_text(&entry.timestamp);
            node.font_size = 11.0;
            node.line_height = CONSOLE_ROW_LINE_HEIGHT;
            node.text_color = Color::rgba(0.58, 0.63, 0.72, 1.0);
            node.computed_rect = Rect::new(cur_x, text_y, time_w, CONSOLE_ROW_LINE_HEIGHT);
        }
        let _ = tree.add_child(row_id, time_id);
        cur_x += time_w + 8.0;

        // Target Tag: e.g. "[ae_audio::audio_manager]"
        let target_text = format!("[{}]", entry.target);
        let target_w = (target_text.len() as f32 * 6.6 + 6.0).clamp(44.0, 220.0);
        let target_id = tree.create_node();
        if let Some(node) = tree.get_mut(target_id) {
            node.set_name("LogTarget");
            node.set_text(target_text);
            node.font_size = 11.0;
            node.line_height = CONSOLE_ROW_LINE_HEIGHT;
            node.text_color = Color::rgba(0.22, 0.76, 0.96, 0.95);
            node.computed_rect = Rect::new(cur_x, text_y, target_w, CONSOLE_ROW_LINE_HEIGHT);
        }
        let _ = tree.add_child(row_id, target_id);
        cur_x += target_w + 10.0;

        // Message text
        let msg_w = (row_rect.right() - cur_x - 12.0).max(60.0);
        let msg_id = tree.create_node();
        if let Some(node) = tree.get_mut(msg_id) {
            node.set_name("LogMessage");
            node.set_text(&entry.msg);
            node.font_size = 11.5;
            node.line_height = CONSOLE_ROW_LINE_HEIGHT;
            node.text_color = match entry.level {
                log::Level::Error => Color::rgba(0.99, 0.60, 0.60, 1.0),
                log::Level::Warn => Color::rgba(0.99, 0.88, 0.45, 1.0),
                log::Level::Info => Color::rgba(0.95, 0.96, 0.98, 1.0),
                log::Level::Debug => Color::rgba(0.85, 0.80, 0.98, 1.0),
                log::Level::Trace => Color::rgba(0.70, 0.74, 0.82, 1.0),
            };
            node.computed_rect = Rect::new(cur_x, text_y, msg_w, CONSOLE_ROW_LINE_HEIGHT);
        }
        let _ = tree.add_child(row_id, msg_id);
    }

    (total_content_height, max_scroll_y)
}