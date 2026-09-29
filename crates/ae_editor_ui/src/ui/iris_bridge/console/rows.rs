// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Console Rows Virtualized Renderer
//!
//! Efficiently renders the visible slice of log entries within the scrollable
//! console viewport using high-performance Iris UI retained-mode widget nodes.
//!

use super::types::{
    CONSOLE_TAG_ROW, CONSOLE_TAG_VIEWPORT, ConsoleFilterExt, ConsoleLogLevel, ConsolePanelParams,
};
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
                render_console_empty_notice(vp, msg);
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

            render_console_row(
                row_scope,
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

/// Renders a single console log row inside the virtualized scroll viewport.
pub fn render_console_row(
    row_scope: &mut UiScope<'_>,
    level: ConsoleLogLevel,
    timestamp: &str,
    target: &str,
    message: &str,
    is_hovered: bool,
    is_striped: bool,
) {
    let bg_color = if is_hovered {
        Color::rgba(0.10, 0.13, 0.18, 1.0)
    } else if is_striped {
        Color::rgba(0.04, 0.048, 0.065, 0.98)
    } else {
        Color::rgba(0.05, 0.06, 0.08, 0.98)
    };

    let row_style = Style::new()
        .flex_row()
        .align_items(irisui::prelude::AlignItems::Center)
        .height(26.0)
        .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
        .background(bg_color);

    row_scope.container_tagged(
        "ConsoleRow",
        row_style,
        WidgetRole::Default,
        CONSOLE_TAG_ROW,
        |row_scope| {
            if is_hovered {
                let accent_style = Style::new()
                    .width(3.0)
                    .height(26.0)
                    .background(Color::rgba(0.0, 0.85, 1.0, 0.95));
                let _ = row_scope.empty_box_passive(accent_style);
            }

            let (badge_text, badge_color, badge_bg) = match level {
                ConsoleLogLevel::Error => (
                    "ERR",
                    Color::rgba(0.98, 0.45, 0.45, 1.0),
                    Color::rgba(0.38, 0.08, 0.08, 0.90),
                ),
                ConsoleLogLevel::Warn => (
                    "WRN",
                    Color::rgba(0.98, 0.78, 0.20, 1.0),
                    Color::rgba(0.35, 0.20, 0.02, 0.90),
                ),
                ConsoleLogLevel::Info => (
                    "INF",
                    Color::rgba(0.25, 0.78, 0.98, 1.0),
                    Color::rgba(0.05, 0.20, 0.32, 0.90),
                ),
                ConsoleLogLevel::Debug => (
                    "DBG",
                    Color::rgba(0.70, 0.75, 0.85, 1.0),
                    Color::rgba(0.12, 0.15, 0.22, 0.90),
                ),
                ConsoleLogLevel::Trace => (
                    "TRC",
                    Color::rgba(0.50, 0.55, 0.65, 1.0),
                    Color::rgba(0.08, 0.10, 0.15, 0.90),
                ),
            };

            // Badge Container
            let badge_style = Style::new()
                .width(36.0)
                .height(18.0)
                .margin_insets(Insets::new(0.0, 8.0, 0.0, 0.0))
                .padding_insets(Insets::new(1.0, 0.0, 0.0, 0.0))
                .border_radius(4.0)
                .background(badge_bg);
            row_scope.container_named("ConsoleBadge", badge_style, |badge_scope| {
                badge_scope.label_styled_passive(
                    "ConsoleBadgeLabel",
                    badge_text,
                    10.0,
                    badge_color,
                    TextAlign::Center,
                    Style::new().width(36.0),
                );
            });

            // Timestamp
            row_scope.label_styled_passive(
                "ConsoleTimestamp",
                timestamp,
                11.0,
                Color::rgba(0.45, 0.48, 0.55, 1.0),
                TextAlign::Left,
                Style::new().width(80.0),
            );

            // Target tag
            let target_style = Style::new()
                .margin_insets(Insets::new(0.0, 8.0, 0.0, 0.0))
                .padding_insets(Insets::new(1.0, 6.0, 1.0, 6.0))
                .border_radius(3.0)
                .background(Color::rgba(0.12, 0.14, 0.20, 0.80));
            row_scope.container_named("ConsoleTarget", target_style, |target_scope| {
                target_scope.label_styled_passive(
                    "ConsoleTargetLabel",
                    target,
                    10.5,
                    Color::rgba(0.65, 0.70, 0.80, 1.0),
                    TextAlign::Left,
                    Style::new(),
                );
            });

            // Message Text
            let text_color = if is_hovered {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::rgba(0.85, 0.88, 0.94, 1.0)
            };
            row_scope.label_styled_passive(
                "ConsoleMessage",
                message,
                12.0,
                text_color,
                TextAlign::Left,
                Style::new().flex_grow(1.0),
            );
        },
    );
}

/// Renders a placeholder notice when the console log list is empty.
pub fn render_console_empty_notice(vp: &mut UiScope<'_>, message: &str) {
    let style = Style::new()
        .padding_insets(Insets::new(24.0, 16.0, 24.0, 16.0))
        .background(Color::rgba(0.0, 0.0, 0.0, 0.0));
    vp.container_named("ConsoleEmptyNotice", style, |scope| {
        scope.label_styled_passive(
            "ConsoleEmptyNoticeLabel",
            message,
            12.0,
            Color::rgba(0.50, 0.53, 0.60, 1.0),
            TextAlign::Center,
            Style::new(),
        );
    });
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