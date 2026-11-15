// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative IDE Log and Telemetry Row Widget
//!
//! Provides a standardized, high-performance IDE log entry row primitive
//! with alternating stripes, leading accent indicator, severity badge,
//! timestamp, target pill, and message label.

use crate::declarative::scope::core::UiScope;
use iris_core::{Color, Insets, Style, TextAlign, WidgetRole};

/// Severity badge configuration for an IDE log entry row.
#[derive(Debug, Clone, Copy)]
pub struct LogRowBadge<'a> {
    /// Severity badge label (e.g. "ERR", "WRN", "INF", "DBG", "TRC").
    pub text: &'a str,
    /// Foreground text color for the severity badge.
    pub color: Color,
    /// Background fill color for the severity badge container.
    pub bg: Color,
}

impl<'a> LogRowBadge<'a> {
    /// Constructs a log badge configuration.
    #[inline]
    pub const fn new(text: &'a str, color: Color, bg: Color) -> Self {
        Self { text, color, bg }
    }
}

/// Descriptor configuring a single declarative IDE log entry row.
#[derive(Debug, Clone, Copy)]
pub struct LogRowDescriptor<'a> {
    /// Semantic hit-testing tag for the row container.
    pub tag: u64,
    /// Severity badge configuration.
    pub badge: LogRowBadge<'a>,
    /// Timestamp string (e.g. "12:00:00").
    pub timestamp: &'a str,
    /// Originating subsystem or module target tag (e.g. "ae_renderer").
    pub target: &'a str,
    /// Formatted log message text.
    pub message: &'a str,
    /// Whether the row displays an alternating striped background.
    pub is_striped: bool,
    /// Whether the row is currently hovered by the pointer.
    pub is_hovered: bool,
    /// Optional accent bar color rendered on the leading edge when hovered.
    pub accent_color: Option<Color>,
}

impl<'a> LogRowDescriptor<'a> {
    /// Constructs a log row descriptor with canonical defaults.
    #[inline]
    pub const fn new(
        tag: u64,
        badge: LogRowBadge<'a>,
        timestamp: &'a str,
        target: &'a str,
        message: &'a str,
        is_striped: bool,
    ) -> Self {
        Self {
            tag,
            badge,
            timestamp,
            target,
            message,
            is_striped,
            is_hovered: false,
            accent_color: Some(Color::rgba(0.0, 0.85, 1.0, 0.95)),
        }
    }

    /// Sets whether the row is currently hovered by the pointer.
    #[inline]
    #[must_use]
    pub const fn hovered(mut self, is_hovered: bool) -> Self {
        self.is_hovered = is_hovered;
        self
    }

    /// Overrides the accent bar color.
    #[inline]
    #[must_use]
    pub const fn accent(mut self, accent_color: Option<Color>) -> Self {
        self.accent_color = accent_color;
        self
    }
}

impl UiScope<'_> {
    /// Renders a standardized declarative IDE log entry row.
    ///
    /// Emits alternating row backgrounds, leading hover accent bar,
    /// severity badge, timestamp, target pill, and message label.
    pub fn log_row(&mut self, desc: &LogRowDescriptor<'_>) {
        let bg_color = if desc.is_hovered {
            Color::rgba(0.10, 0.13, 0.18, 1.0)
        } else if desc.is_striped {
            Color::rgba(0.04, 0.048, 0.065, 0.98)
        } else {
            Color::rgba(0.05, 0.06, 0.08, 0.98)
        };

        let row_style = Style::new()
            .flex_row()
            .align_items(iris_core::AlignItems::Center)
            .height(26.0)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .background(bg_color);

        self.container_tagged(
            "LogRow",
            row_style,
            WidgetRole::Default,
            desc.tag,
            |row_scope| {
                if desc.is_hovered
                    && let Some(accent) = desc.accent_color
                {
                    row_scope
                        .empty_box_passive(Style::new().width(3.0).height(26.0).background(accent));
                }

                // Severity Badge
                row_scope.container_named(
                    "LogBadge",
                    Style::new()
                        .width(36.0)
                        .height(18.0)
                        .margin_insets(Insets::new(0.0, 8.0, 0.0, 0.0))
                        .padding_insets(Insets::new(1.0, 0.0, 0.0, 0.0))
                        .border_radius(4.0)
                        .background(desc.badge.bg),
                    |badge_scope| {
                        badge_scope.label_styled_passive(
                            "LogBadgeLabel",
                            desc.badge.text,
                            9.5,
                            desc.badge.color,
                            TextAlign::Center,
                            Style::new(),
                        );
                    },
                );

                // Timestamp
                row_scope.label_styled_passive(
                    "LogTimestamp",
                    desc.timestamp,
                    11.0,
                    Color::rgba(0.45, 0.48, 0.55, 1.0),
                    TextAlign::Left,
                    Style::new().width(80.0),
                );

                // Target Subsystem Tag
                row_scope.container_named(
                    "LogTarget",
                    Style::new()
                        .margin_insets(Insets::new(0.0, 8.0, 0.0, 0.0))
                        .padding_insets(Insets::new(1.0, 6.0, 1.0, 6.0))
                        .border_radius(3.0)
                        .background(Color::rgba(0.12, 0.14, 0.20, 0.80)),
                    |target_scope| {
                        target_scope.label_styled_passive(
                            "LogTargetLabel",
                            desc.target,
                            10.5,
                            Color::rgba(0.65, 0.70, 0.80, 1.0),
                            TextAlign::Left,
                            Style::new(),
                        );
                    },
                );

                // Message Text
                let text_color = if desc.is_hovered {
                    Color::rgba(1.0, 1.0, 1.0, 1.0)
                } else {
                    Color::rgba(0.85, 0.88, 0.94, 1.0)
                };
                row_scope.label_styled_passive(
                    "LogMessage",
                    desc.message,
                    12.0,
                    text_color,
                    TextAlign::Left,
                    Style::new().flex_grow(1.0),
                );
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::tree::UiTree;

    #[test]
    fn test_ui_scope_log_row_renders_structure() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");

        let mut scope = UiScope::new(&mut tree, root);
        let badge = LogRowBadge::new(
            "ERR",
            Color::rgba(1.0, 0.3, 0.3, 1.0),
            Color::rgba(0.3, 0.05, 0.05, 1.0),
        );
        let desc = LogRowDescriptor::new(
            0x500c,
            badge,
            "12:34:56",
            "ae_renderer",
            "Pipeline compile failed",
            false,
        );

        scope.log_row(&desc);

        // Verify row node was created
        let root_node = tree.get(root).expect("root node");
        assert_eq!(root_node.children.len(), 1);
        let row_node = tree.get(root_node.children[0]).expect("row node");
        assert_eq!(row_node.tag, 0x500c);
        // Verify child columns (badge container, timestamp label, target container, message label)
        assert_eq!(row_node.children.len(), 4);
    }
}