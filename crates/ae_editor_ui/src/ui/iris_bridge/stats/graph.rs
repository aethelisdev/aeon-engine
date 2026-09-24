// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Frame Pacing & Stutter Analyzer Oscilloscope Builder
//!
//! Visualizes real-time historical frametimes with milestone threshold lines,
//! dynamic stutter indicators, and declarative 2x2 statistical metrics purely using [`UiScope`].
//!

use super::types::{STATS_TAG_CANVAS, StatsPanelParams};
use ae_core::telemetry::FrameRingBuffer;
use irisui::prelude::*;

/// Builds the Frame Pacing & Stutter Analyzer section inside the declarative stats card.
pub fn build_frame_pacing_content(scope: &mut UiScope<'_>, params: &StatsPanelParams<'_>) {
    let stats = params.frame_pacing_stats;

    let container_style = Style::new().flex_col().gap(6.0);

    scope.container(container_style, |s| {
        // ── Row 1: Avg FPS & 1% Low ──
        let r1_style = Style::new().flex_row().gap(6.0);
        s.container(r1_style, |r| {
            let avg_fps_text = format!(
                "{:.0} ({:.2}ms)",
                stats.average_fps, stats.average_frametime_ms
            );
            build_metric_pill(
                r,
                "Avg FPS",
                &avg_fps_text,
                get_fps_color(stats.average_fps),
            );

            let low_1_text = format!("{:.0} FPS", stats.low_1_percent_fps);
            build_metric_pill(
                r,
                "1% Low",
                &low_1_text,
                get_fps_color(stats.low_1_percent_fps),
            );
        });

        // ── Row 2: 0.1% Low & Jitter ──
        let r2_style = Style::new().flex_row().gap(6.0);
        s.container(r2_style, |r| {
            let low_01_text = format!("{:.0} FPS", stats.low_0_1_percent_fps);
            build_metric_pill(
                r,
                "0.1% Low",
                &low_01_text,
                get_fps_color(stats.low_0_1_percent_fps),
            );

            let jitter_color = if stats.variance_ms < 1.5 {
                Color::rgba(0.0, 0.82, 0.63, 1.0)
            } else if stats.variance_ms < 4.0 {
                Color::rgba(1.0, 0.73, 0.0, 1.0)
            } else {
                Color::rgba(0.92, 0.24, 0.24, 1.0)
            };
            let jitter_text = format!("±{:.2} ms", stats.variance_ms);
            build_metric_pill(r, "Jitter", &jitter_text, jitter_color);
        });

        // ── Oscilloscope Canvas Container ──
        let canvas_style = Style::new()
            .height(92.0)
            .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
            .border(1.0, Color::rgba(0.14, 0.16, 0.22, 0.85))
            .border_radius(4.0)
            .clip_children(true);

        s.canvas_named(
            "OscilloscopeCanvas",
            canvas_style,
            WidgetRole::OscilloscopeCanvas,
            STATS_TAG_CANVAS,
            None,
        );

        // ── Pacing Summary Footer ──
        let spike_count = stats.spikes_over_16ms;
        let stutter_pct = stats.stutter_rate_percent;
        let summary_text = if spike_count == 0 {
            format!("✓ Spikes (>16ms): 0  •  Stutter Rate: {:.1}%", stutter_pct)
        } else {
            format!(
                "⚠ Spikes (>16ms): {}  •  Stutter Rate: {:.1}%",
                spike_count, stutter_pct
            )
        };

        let summary_color = if spike_count == 0 && stutter_pct < 1.0 {
            Color::rgba(0.0, 0.82, 0.63, 1.0)
        } else if spike_count < 5 && stutter_pct < 5.0 {
            Color::rgba(1.0, 0.73, 0.0, 1.0)
        } else {
            Color::rgba(0.92, 0.24, 0.24, 1.0)
        };

        s.text_colored(&summary_text, summary_color);
    });
}

/// Helper function to build a single metric pill container purely with flexbox.
fn build_metric_pill(scope: &mut UiScope<'_>, label: &str, value: &str, val_color: Color) {
    let pill_style = Style::new()
        .flex_row()
        .flex_grow(1.0)
        .align_items(AlignItems::Center)
        .height(24.0)
        .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
        .background(Color::rgba(0.04, 0.04, 0.06, 0.90))
        .border(1.0, Color::rgba(0.14, 0.15, 0.20, 0.80))
        .border_radius(4.0);

    scope.container(pill_style, |p| {
        p.text_colored(label, Color::rgba(0.60, 0.62, 0.70, 1.0));

        p.spacer();

        p.text_colored(value, val_color);
    });
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
fn get_frametime_color(ms: f32) -> Color {
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