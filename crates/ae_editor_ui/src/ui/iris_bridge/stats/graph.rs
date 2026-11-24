// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Frame Pacing & Stutter Analyzer Oscilloscope Builder
//!
//! Visualizes real-time historical frametimes with milestone threshold lines,
//! dynamic stutter indicators, and declarative zero-heap 2x2 statistical metrics.

use super::types::{
    STATS_TAG_AVG_FPS, STATS_TAG_CANVAS, STATS_TAG_JITTER, STATS_TAG_LOW_01, STATS_TAG_LOW_1,
    STATS_TAG_SUMMARY,
};
use ae_diagnostic::{FramePacingStats, FrameRingBuffer};
use irisui::prelude::*;
use irisui::widgets::canvas::ChartDrawer;
use std::fmt::Write;

/// Builds the Frame Pacing & Stutter Analyzer section inside the declarative stats card.
pub fn build_frame_pacing_content(
    scope: &mut UiScope<'_>,
    frame_pacing: &FrameRingBuffer<240>,
    frame_pacing_stats: &FramePacingStats,
) {
    scope.metric_grid(6.0, |s| {
        s.metric_row(6.0, |r| {
            r.metric_pill_fmt::<32, _>(
                "Avg FPS",
                STATS_TAG_AVG_FPS,
                get_fps_color(frame_pacing_stats.average_fps),
                |buf| {
                    buf.write_fmt(format_args!(
                        "{:.0} ({:.2}ms)",
                        frame_pacing_stats.average_fps, frame_pacing_stats.average_frametime_ms
                    ))
                    .ok();
                },
            );
            r.metric_pill_fmt::<24, _>(
                "1% Low",
                STATS_TAG_LOW_1,
                get_fps_color(frame_pacing_stats.low_1_percent_fps),
                |buf| {
                    buf.write_fmt(format_args!(
                        "{:.0} FPS",
                        frame_pacing_stats.low_1_percent_fps
                    ))
                    .ok();
                },
            );
        });

        s.metric_row(6.0, |r| {
            r.metric_pill_fmt::<24, _>(
                "0.1% Low",
                STATS_TAG_LOW_01,
                get_fps_color(frame_pacing_stats.low_0_1_percent_fps),
                |buf| {
                    buf.write_fmt(format_args!(
                        "{:.0} FPS",
                        frame_pacing_stats.low_0_1_percent_fps
                    ))
                    .ok();
                },
            );
            r.metric_pill_fmt::<24, _>(
                "Jitter",
                STATS_TAG_JITTER,
                get_jitter_color(frame_pacing_stats.variance_ms),
                |buf| {
                    buf.write_fmt(format_args!("±{:.2} ms", frame_pacing_stats.variance_ms))
                        .ok();
                },
            );
        });

        s.live_canvas_styled(
            "OscilloscopeCanvas",
            STATS_TAG_CANVAS,
            Style::new().height(92.0),
            |canvas| {
                draw_oscilloscope_canvas(canvas, Some(frame_pacing));
            },
        );

        s.text_colored_tagged_fmt::<64, _>(
            get_summary_color(frame_pacing_stats),
            STATS_TAG_SUMMARY,
            |buf| write_pacing_summary(buf, frame_pacing_stats),
        );
    });
}

#[inline]
fn write_pacing_summary<const N: usize>(buf: &mut StackString<N>, stats: &FramePacingStats) {
    if stats.spikes_over_16ms == 0 {
        buf.write_fmt(format_args!(
            "✓ Spikes (>16ms): 0  •  Stutter Rate: {:.1}%",
            stats.stutter_rate_percent
        ))
        .ok();
    } else {
        buf.write_fmt(format_args!(
            "⚠ Spikes (>16ms): {}  •  Stutter Rate: {:.1}%",
            stats.spikes_over_16ms, stats.stutter_rate_percent
        ))
        .ok();
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
fn get_summary_color(stats: &FramePacingStats) -> Color {
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

    pacing.into_iter().for_each(|pacing| {
        canvas.draw_chart_polyline(
            |idx| pacing.get_chronological(idx),
            pacing.count(),
            36.0,
            60,
            get_frametime_color,
        );
    });
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

/// Updates oscilloscope telemetry text labels in place using zero-heap [`StackString`] formatting.
pub fn update_oscilloscope_texts(scope: &mut UiScope<'_>, stats: &FramePacingStats) -> bool {
    scope.update_live_text_colored_fmt::<32, _>(
        STATS_TAG_AVG_FPS,
        get_fps_color(stats.average_fps),
        |buf| {
            buf.write_fmt(format_args!(
                "{:.0} ({:.2}ms)",
                stats.average_fps, stats.average_frametime_ms
            ))
            .ok();
        },
    ) | scope.update_live_text_colored_fmt::<24, _>(
        STATS_TAG_LOW_1,
        get_fps_color(stats.low_1_percent_fps),
        |buf| {
            buf.write_fmt(format_args!("{:.0} FPS", stats.low_1_percent_fps))
                .ok();
        },
    ) | scope.update_live_text_colored_fmt::<24, _>(
        STATS_TAG_LOW_01,
        get_fps_color(stats.low_0_1_percent_fps),
        |buf| {
            buf.write_fmt(format_args!("{:.0} FPS", stats.low_0_1_percent_fps))
                .ok();
        },
    ) | scope.update_live_text_colored_fmt::<24, _>(
        STATS_TAG_JITTER,
        get_jitter_color(stats.variance_ms),
        |buf| {
            buf.write_fmt(format_args!("±{:.2} ms", stats.variance_ms))
                .ok();
        },
    ) | scope.update_live_text_colored_fmt::<64, _>(
        STATS_TAG_SUMMARY,
        get_summary_color(stats),
        |buf| write_pacing_summary(buf, stats),
    )
}

/// Refreshes both the oscilloscope live canvas and telemetry labels in place via [`UiScope`].
pub fn refresh_live_oscilloscope(
    scope: &mut UiScope<'_>,
    pacing: &FrameRingBuffer,
    stats: &FramePacingStats,
) -> bool {
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
        Color::rgba(0.0, 0.82, 0.63, 1.0)
    } else if ms <= 16.67 {
        Color::rgba(0.0, 0.75, 0.90, 1.0)
    } else if ms <= 33.33 {
        Color::rgba(1.0, 0.73, 0.0, 1.0)
    } else {
        Color::rgba(0.92, 0.24, 0.24, 1.0)
    }
}

/// Computes FPS color based on standard performance milestones.
fn get_fps_color(fps: f32) -> Color {
    if fps >= 100.0 {
        Color::rgba(0.0, 0.82, 0.63, 1.0)
    } else if fps >= 55.0 {
        Color::rgba(0.0, 0.75, 0.90, 1.0)
    } else if fps >= 28.0 {
        Color::rgba(1.0, 0.73, 0.0, 1.0)
    } else {
        Color::rgba(0.92, 0.24, 0.24, 1.0)
    }
}