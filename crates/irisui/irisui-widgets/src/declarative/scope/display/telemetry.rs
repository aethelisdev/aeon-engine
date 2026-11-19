// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Telemetry Cards, Metric Grids, Pills & Segmented Bars
//!
//! Provides high-level, percentage-flex telemetry layout primitives on [`UiScope`]
//! backed by zero-heap [`StackString`] formatting so consumer diagnostics and profiler
//! panels require no manual width subtraction (`panel_rect.width - 12.0`) or `format!`.

use irisui_core::{AlignItems, Color, Insets, StackString, Style, TextAlign};

use super::super::UiScope;
use super::live::LiveTextProps;

impl UiScope<'_> {
    /// Emits a full-width dark telemetry card container with an icon, title header, and child slot.
    ///
    /// Uses `width_percent(100.0)` flex sizing to eliminate manual parent-width subtraction.
    pub fn telemetry_card<F>(&mut self, icon: &str, title: &str, build_body: F)
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let card_style = Style::new()
            .flex_col()
            .gap(6.0)
            .padding_insets(Insets::new(8.0, 10.0, 8.0, 10.0))
            .background(Color::rgba(0.07, 0.08, 0.10, 0.95))
            .border(1.0, Color::rgba(0.15, 0.16, 0.21, 0.90))
            .border_radius(5.0);

        self.container(card_style, |card| {
            let header_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .gap(6.0)
                .height(18.0);
            card.container(header_style, |header| {
                header.text_colored(icon, Color::rgba(0.86, 0.88, 0.92, 1.0));
                header.text_colored(title, Color::rgba(0.86, 0.88, 0.92, 1.0));
            });
            build_body(card);
        });
    }

    /// Emits a vertical column container organizing stacked [`Self::metric_row`] strips.
    pub fn metric_grid<F>(&mut self, gap: f32, build_rows: F)
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        self.container(Style::new().flex_col().gap(gap), build_rows);
    }

    /// Emits a horizontal row container distributing [`Self::metric_pill`] items evenly via flex grow.
    pub fn metric_row<F>(&mut self, gap: f32, build_pills: F)
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        self.container(Style::new().flex_row().gap(gap), build_pills);
    }

    /// Emits an equal-flex telemetry metric pill with left title and tagged live-updatable right value.
    ///
    /// Registers `value_tag` as a live node when non-zero so subsequent frames can update text and
    /// foreground color in place via [`Self::update_live_text_colored_fmt`].
    pub fn metric_pill(&mut self, label: &str, value_tag: u64, value: &str, value_color: Color) {
        self.container(
            Style::new()
                .flex_row()
                .flex_grow(1.0)
                .align_items(AlignItems::Center)
                .height(24.0)
                .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
                .background(Color::rgba(0.04, 0.04, 0.06, 0.90))
                .border(1.0, Color::rgba(0.14, 0.15, 0.20, 0.80))
                .border_radius(4.0),
            |pill| {
                pill.text_colored(label, Color::rgba(0.60, 0.62, 0.70, 1.0));
                pill.spacer();
                if value_tag != 0 {
                    let est_w = (value.len() as f32 * 14.0 * 0.65).ceil().max(16.0);
                    pill.live_text(
                        LiveTextProps::new(
                            "MetricPillValue",
                            value_tag,
                            value,
                            14.0,
                            value_color,
                            Style::new().width(est_w),
                        )
                        .with_align(TextAlign::Left),
                    );
                } else {
                    pill.text_colored(value, value_color);
                }
            },
        );
    }

    /// Emits an equal-flex telemetry metric pill whose value is formatted into a stack [`StackString`].
    ///
    /// Avoids all heap allocations (`format!`) during both initial tree construction and rebuilds.
    pub fn metric_pill_fmt<const N: usize, F>(
        &mut self,
        label: &str,
        value_tag: u64,
        value_color: Color,
        fmt_fn: F,
    ) where
        F: FnOnce(&mut StackString<N>),
    {
        let mut buf = StackString::<N>::new();
        fmt_fn(&mut buf);
        self.metric_pill(label, value_tag, buf.as_str(), value_color);
    }

    /// Emits a proportional multi-segment horizontal bar from normalized `(ratio, color)` pairs.
    pub fn segmented_bar(&mut self, segments: &[(f32, Color)]) {
        let bar_style = Style::new()
            .flex_row()
            .height(4.0)
            .border_radius(2.0)
            .background(Color::rgba(0.08, 0.09, 0.12, 0.95))
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.80))
            .clip_children(true);

        self.container(bar_style, |bar| {
            for &(ratio, color) in segments {
                bar.empty_box(
                    Style::new()
                        .background(color)
                        .flex_grow(ratio.clamp(0.001, 1.0))
                        .height(4.0),
                );
            }
        });
    }

    /// Emits a metric row with a colored dot indicator, left label, spacer, and static right value.
    pub fn metric_dot_row(&mut self, label: &str, value: &str, dot_color: Color) {
        self.metric_dot_row_styled(label, 0, value, dot_color, None);
    }

    /// Emits a metric row with a colored dot indicator and tagged live-updatable right value.
    pub fn metric_dot_row_tagged(
        &mut self,
        label: &str,
        value_tag: u64,
        value: &str,
        dot_color: Color,
    ) {
        self.metric_dot_row_styled(label, value_tag, value, dot_color, None);
    }

    /// Emits a metric row whose right value is formatted into a stack [`StackString`] without heap allocation.
    pub fn metric_dot_row_fmt<const N: usize, F>(
        &mut self,
        label: &str,
        value_tag: u64,
        dot_color: Color,
        value_color: Option<Color>,
        fmt_fn: F,
    ) where
        F: FnOnce(&mut StackString<N>),
    {
        let mut buf = StackString::<N>::new();
        fmt_fn(&mut buf);
        self.metric_dot_row_styled(label, value_tag, buf.as_str(), dot_color, value_color);
    }

    fn metric_dot_row_styled(
        &mut self,
        label: &str,
        value_tag: u64,
        value: &str,
        dot_color: Color,
        value_color: Option<Color>,
    ) {
        let resolved_val_color = value_color.unwrap_or(Color::rgba(0.86, 0.88, 0.92, 1.0));
        self.container(
            Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .gap(6.0)
                .height(16.0),
            |row| {
                row.empty_box(
                    Style::new()
                        .width(6.0)
                        .height(6.0)
                        .border_radius(3.0)
                        .background(dot_color),
                );
                row.text_colored(label, Color::rgba(0.78, 0.80, 0.86, 1.0));
                row.spacer();
                if value_tag != 0 {
                    let est_w = (value.len() as f32 * 14.0 * 0.65).ceil().max(16.0);
                    row.live_text(
                        LiveTextProps::new(
                            "TelemetryMetricValue",
                            value_tag,
                            value,
                            14.0,
                            resolved_val_color,
                            Style::new().width(est_w),
                        )
                        .with_align(TextAlign::Left),
                    );
                } else {
                    row.text_colored(value, resolved_val_color);
                }
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::UiTree;
    use std::fmt::Write;

    #[test]
    fn test_telemetry_primitives_stack_format_and_live_update() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        const TAG_FPS: u64 = 0xCC01;
        const TAG_CPU: u64 = 0xCC02;

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.telemetry_card("📈", "Telemetry", |card| {
                card.metric_grid(4.0, |grid| {
                    grid.metric_row(6.0, |row| {
                        row.metric_pill_fmt::<32, _>("Avg FPS", TAG_FPS, Color::GREEN, |buf| {
                            let _ = write!(buf, "{:.0} FPS", 120.0_f32);
                        });
                    });
                });
                card.segmented_bar(&[(0.4, Color::CYAN), (0.6, Color::RED)]);
                card.metric_dot_row_fmt::<24, _>("Main Loop", TAG_CPU, Color::CYAN, None, |buf| {
                    let _ = write!(buf, "{:.2} ms", 4.25_f32);
                });
            });
        }

        assert_eq!(
            tree.node_tagged(TAG_FPS).and_then(|n| n.text.as_deref()),
            Some("120 FPS")
        );
        assert_eq!(
            tree.node_tagged(TAG_CPU).and_then(|n| n.text.as_deref()),
            Some("4.25 ms")
        );
        assert!(tree.is_live_tag(TAG_FPS));
        assert!(tree.is_live_tag(TAG_CPU));
    }
}