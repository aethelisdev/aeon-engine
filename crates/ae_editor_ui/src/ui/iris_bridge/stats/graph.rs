// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Frame Pacing & Stutter Analyzer Oscilloscope Builder
//!
//! Visualizes real-time historical frametimes with milestone threshold lines,
//! dynamic stutter indicators, and declarative 2x2 statistical metrics purely using [`UiScope`].

use super::types::{STATS_TAG_CANVAS, StatsPanelParams};
use ae_core::telemetry::FrameRingBuffer;
use irisui::prelude::*;
use irisui::widgets::canvas::ChartDrawer;

/// Builds the Frame Pacing & Stutter Analyzer section inside the declarative stats card.
pub fn build_frame_pacing_content(scope: &mut UiScope<'_>, params: &StatsPanelParams<'_>) {
    scope.container(Style::new().flex_col().gap(6.0), |s| {
        s.container(Style::new().flex_row().gap(6.0), |r| {
            build_metric_pill(
                r,
                "Avg FPS",
                &format!(
                    "{:.0} ({:.2}ms)",
                    params.frame_pacing_stats.average_fps,
                    params.frame_pacing_stats.average_frametime_ms
                ),
                get_fps_color(params.frame_pacing_stats.average_fps),
                super::types::STATS_TAG_AVG_FPS,
            );
            build_metric_pill(
                r,
                "1% Low",
                &format!("{:.0} FPS", params.frame_pacing_stats.low_1_percent_fps),
                get_fps_color(params.frame_pacing_stats.low_1_percent_fps),
                super::types::STATS_TAG_LOW_1,
            );
        });

        s.container(Style::new().flex_row().gap(6.0), |r| {
            build_metric_pill(
                r,
                "0.1% Low",
                &format!("{:.0} FPS", params.frame_pacing_stats.low_0_1_percent_fps),
                get_fps_color(params.frame_pacing_stats.low_0_1_percent_fps),
                super::types::STATS_TAG_LOW_01,
            );
            build_metric_pill(
                r,
                "Jitter",
                &format!("±{:.2} ms", params.frame_pacing_stats.variance_ms),
                get_jitter_color(params.frame_pacing_stats.variance_ms),
                super::types::STATS_TAG_JITTER,
            );
        });

        s.live_canvas_tagged(
            "OscilloscopeCanvas",
            STATS_TAG_CANVAS,
            Size::new((params.panel_rect.width - 24.0).max(100.0), 92.0),
            |canvas| {
                draw_oscilloscope_canvas(canvas, Some(params.frame_pacing));
            },
        );

        s.text_colored_tagged(
            format_pacing_summary(params.frame_pacing_stats),
            get_summary_color(params.frame_pacing_stats),
            super::types::STATS_TAG_SUMMARY,
        );
    });
}

/// Helper function to build a single metric pill container purely with flexbox.
fn build_metric_pill(
    scope: &mut UiScope<'_>,
    label: &str,
    value: &str,
    val_color: Color,
    value_tag: u64,
) {
    scope.container(
        Style::new()
            .flex_row()
            .flex_grow(1.0)
            .align_items(AlignItems::Center)
            .height(24.0)
            .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
            .background(Color::rgba(0.04, 0.04, 0.06, 0.90))
            .border(1.0, Color::rgba(0.14, 0.15, 0.20, 0.80))
            .border_radius(4.0),
        |p| {
            p.text_colored(label, Color::rgba(0.60, 0.62, 0.70, 1.0));
            p.spacer();
            p.text_colored_tagged(value, val_color, value_tag);
        },
    );
}

#[inline]
fn format_pacing_summary(stats: &ae_core::telemetry::FramePacingStats) -> String {
    if stats.spikes_over_16ms == 0 {
        format!(
            "✓ Spikes (>16ms): 0  •  Stutter Rate: {:.1}%",
            stats.stutter_rate_percent
        )
    } else {
        format!(
            "⚠ Spikes (>16ms): {}  •  Stutter Rate: {:.1}%",
            stats.spikes_over_16ms, stats.stutter_rate_percent
        )
    }
}

#[inline]
fn get_jitter_color(variance_ms: f32) -> Color {
    if variance_ms < 1.5 {
        Color::rgba(0.0, 0.82, 0.63, 1.0)
    } else if variance_ms < 4.0 {
        Color::rgba(1.0, 0.73, 0.0, 1.0)
    } else {
        Color::rgba(0.92, 0.24, 0.24, 1.0)
    }
}

#[inline]
fn get_summary_color(stats: &ae_core::telemetry::FramePacingStats) -> Color {
    if stats.spikes_over_16ms == 0 && stats.stutter_rate_percent < 1.0 {
        Color::rgba(0.0, 0.82, 0.63, 1.0)
    } else if stats.spikes_over_16ms < 5 && stats.stutter_rate_percent < 5.0 {
        Color::rgba(1.0, 0.73, 0.0, 1.0)
    } else {
        Color::rgba(0.92, 0.24, 0.24, 1.0)
    }
}

/// Renders the oscilloscope background, border, reference threshold lines, and dynamic frametime polyline.
pub fn draw_oscilloscope_canvas(
    canvas: &mut irisui::widgets::canvas::ImmediateCanvas<'_>,
    pacing: Option<&FrameRingBuffer>,
) {
    draw_oscilloscope_with_bounds(canvas, canvas.bounds(), pacing);
}

fn draw_oscilloscope_with_bounds(
    canvas: &mut irisui::widgets::canvas::ImmediateCanvas<'_>,
    bounds: Rect,
    pacing: Option<&FrameRingBuffer>,
) {
    canvas.fill_rect(bounds, Color::rgba(0.04, 0.05, 0.07, 0.95), 4.0);
    canvas.stroke_rect(bounds, Color::rgba(0.14, 0.16, 0.22, 0.85), 1.0, 4.0);

    if bounds.height > 20.0 {
        draw_threshold_line(
            canvas,
            bounds,
            bounds.y + bounds.height * (1.0 - (16.666_f32 / 36.0).min(1.0)),
            Color::rgba(0.0, 0.82, 0.63, 0.25),
        );
        draw_threshold_line(
            canvas,
            bounds,
            bounds.y + bounds.height * (1.0 - (33.333_f32 / 36.0).min(1.0)),
            Color::rgba(0.92, 0.24, 0.24, 0.25),
        );
    }

    if let Some(pacing) = pacing {
        canvas.draw_chart_polyline(
            |idx| pacing.get_chronological(idx),
            pacing.count(),
            36.0,
            60,
            get_frametime_color,
        );
    }
}

#[inline]
fn draw_threshold_line(
    canvas: &mut irisui::widgets::canvas::ImmediateCanvas<'_>,
    bounds: Rect,
    y: f32,
    color: Color,
) {
    canvas.draw_line(
        Point::new(bounds.x + 2.0, y),
        Point::new(bounds.x + bounds.width - 2.0, y),
        color,
        1.0,
    );
}

/// Updates oscilloscope telemetry text labels in place via [`UiScope::update_live_text`].
pub fn update_oscilloscope_texts(
    scope: &mut UiScope<'_>,
    stats: &ae_core::telemetry::FramePacingStats,
) -> bool {
    if scope
        .node_id_tagged(super::types::STATS_TAG_AVG_FPS)
        .is_none()
    {
        return false;
    }
    scope.update_live_text(
        super::types::STATS_TAG_AVG_FPS,
        &format!(
            "{:.0} ({:.2}ms)",
            stats.average_fps, stats.average_frametime_ms
        ),
    ) | scope.update_live_text(
        super::types::STATS_TAG_LOW_1,
        &format!("{:.0} FPS", stats.low_1_percent_fps),
    ) | scope.update_live_text(
        super::types::STATS_TAG_LOW_01,
        &format!("{:.0} FPS", stats.low_0_1_percent_fps),
    ) | scope.update_live_text(
        super::types::STATS_TAG_JITTER,
        &format!("±{:.2} ms", stats.variance_ms),
    ) | scope.update_live_text(
        super::types::STATS_TAG_SUMMARY,
        &format_pacing_summary(stats),
    )
}

/// Refreshes both the oscilloscope live canvas and telemetry labels in place via [`UiScope`].
pub fn refresh_live_oscilloscope(
    scope: &mut UiScope<'_>,
    pacing: &FrameRingBuffer,
    stats: &ae_core::telemetry::FramePacingStats,
) -> bool {
    if scope.node_id_tagged(STATS_TAG_CANVAS).is_none() {
        return false;
    }
    scope.update_live_canvas(STATS_TAG_CANVAS, |canvas| {
        draw_oscilloscope_canvas(canvas, Some(pacing));
    }) | update_oscilloscope_texts(scope, stats)
}

/// Directly appends the 60 oscilloscope curve quad instances to the command list using `ChartDrawer`.
pub fn append_oscilloscope_quads(
    command_list: &mut DrawCommandList,
    canvas_rect: Rect,
    pacing: &FrameRingBuffer,
) {
    ChartDrawer::draw_polyline(
        command_list,
        canvas_rect,
        |idx| pacing.get_chronological(idx),
        pacing.count(),
        36.0,
        60,
        get_frametime_color,
    );
}

/// Computes frametime curve segment color based on millisecond milestones.
pub(crate) fn get_frametime_color(ms: f32) -> Color {
    if ms <= 8.33 {
        Color::rgba(0.0, 0.82, 0.63, 1.0) // 120+ FPS: Mint Emerald
    } else if ms <= 16.67 {
        Color::rgba(0.0, 0.75, 0.90, 1.0) // 60+ FPS: Neon Cyan
    } else if ms <= 33.33 {
        Color::rgba(1.0, 0.73, 0.0, 1.0) // 30+ FPS: Amber
    } else {
        Color::rgba(0.92, 0.24, 0.24, 1.0) // <30 FPS: Crimson Red
    }
}

/// Computes FPS color based on standard performance milestones.
fn get_fps_color(fps: f32) -> Color {
    if fps >= 100.0 {
        Color::rgba(0.0, 0.82, 0.63, 1.0) // Mint Emerald
    } else if fps >= 55.0 {
        Color::rgba(0.0, 0.75, 0.90, 1.0) // Cyan
    } else if fps >= 28.0 {
        Color::rgba(1.0, 0.73, 0.0, 1.0) // Amber
    } else {
        Color::rgba(0.92, 0.24, 0.24, 1.0) // Crimson
    }
}