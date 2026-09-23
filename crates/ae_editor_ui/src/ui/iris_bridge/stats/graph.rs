// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Frame Pacing & Stutter Analyzer Oscilloscope Builder
//!
//! Visualizes real-time historical frametimes with milestone threshold lines,
//! dynamic stutter indicators, and retained 2x2 statistical metrics with zero per-frame allocations.

use super::types::StatsPanelParams;
use ae_core::telemetry::FrameRingBuffer;
use irisui::prelude::*;

/// Output node handles created during static layout initialization of the frame pacing card.
pub struct FramePacingNodes {
    /// Metric pill value node IDs (`[Avg FPS, 1% Low, 0.1% Low, Jitter]`).
    pub pill_val_ids: [WidgetId; 4],
    /// Pacing summary footer text node ID.
    pub footer_id: WidgetId,
    /// Oscilloscope canvas bounding rect.
    pub canvas_rect: Rect,
}

/// Builds the static Frame Pacing & Stutter Analyzer card layout using native `CanvasBuilder`.
pub fn build_frame_pacing_content(
    tree: &mut UiTree,
    parent_id: WidgetId,
    content_rect: Rect,
) -> FramePacingNodes {
    let grid_w = content_rect.width;
    let col_w = ((grid_w - 6.0) * 0.5).max(80.0);
    let pill_h = 24.0;
    let row_gap = 4.0;

    let r1_y = content_rect.y;
    let r2_y = r1_y + pill_h + row_gap;

    // Row 1: Avg FPS & 1% Low
    let avg_val_id = build_metric_pill(
        tree,
        parent_id,
        "Avg FPS",
        "-",
        Color::rgba(0.0, 0.82, 0.63, 1.0),
        Rect::new(content_rect.x, r1_y, col_w, pill_h),
    );

    let low_1_val_id = build_metric_pill(
        tree,
        parent_id,
        "1% Low",
        "-",
        Color::rgba(0.0, 0.75, 0.90, 1.0),
        Rect::new(content_rect.x + col_w + 6.0, r1_y, col_w, pill_h),
    );

    // Row 2: 0.1% Low & Jitter
    let low_01_val_id = build_metric_pill(
        tree,
        parent_id,
        "0.1% Low",
        "-",
        Color::rgba(0.0, 0.75, 0.90, 1.0),
        Rect::new(content_rect.x, r2_y, col_w, pill_h),
    );

    let jitter_val_id = build_metric_pill(
        tree,
        parent_id,
        "Jitter",
        "-",
        Color::rgba(0.0, 0.82, 0.63, 1.0),
        Rect::new(content_rect.x + col_w + 6.0, r2_y, col_w, pill_h),
    );

    // Oscilloscope Canvas Container using Iris UI native CanvasBuilder
    let canvas_y = r2_y + pill_h + 6.0;
    let canvas_h = 92.0;
    let canvas_rect = Rect::new(content_rect.x, canvas_y, grid_w, canvas_h);

    CanvasBuilder::new(tree)
        .parent(parent_id)
        .name("OscilloscopeCanvas")
        .rect(canvas_rect)
        .style(ChartStyle {
            max_scale_value: 36.0,
            ..Default::default()
        })
        .add_threshold(ChartThreshold::new(
            8.33,
            "120 FPS (8.3ms)",
            Color::rgba(0.20, 0.80, 0.20, 0.45),
        ))
        .add_threshold(ChartThreshold::new(
            16.67,
            "60 FPS (16.6ms)",
            Color::rgba(1.0, 0.73, 0.0, 0.50),
        ))
        .add_threshold(ChartThreshold::new(
            33.33,
            "30 FPS (33.3ms)",
            Color::rgba(0.92, 0.24, 0.24, 0.50),
        ))
        .build();

    // Pacing Summary Footer
    let footer_y = canvas_y + canvas_h + 6.0;
    let foot_id = tree.create_node();
    if let Some(node) = tree.get_mut(foot_id) {
        node.set_name("PacingSummaryFooter");
        node.set_text("✓ Spikes (>16ms): 0  •  Stutter Rate: 0.0%");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.0, 0.82, 0.63, 1.0);
        node.computed_rect = Rect::new(content_rect.x, footer_y, grid_w, 16.0);
    }
    let _ = tree.add_child(parent_id, foot_id);

    FramePacingNodes {
        pill_val_ids: [avg_val_id, low_1_val_id, low_01_val_id, jitter_val_id],
        footer_id: foot_id,
        canvas_rect,
    }
}

/// Updates the dynamic metric values of the Frame Pacing card in place (0 allocations).
pub fn update_frame_pacing_values(
    tree: &mut UiTree,
    nodes: &[WidgetId; 4],
    footer_id: WidgetId,
    params: &StatsPanelParams<'_>,
) {
    let stats = params.frame_pacing_stats;

    // 1. Update 2x2 Metric Pills
    let avg_fps_text = format!(
        "{:.0} ({:.2}ms)",
        stats.average_fps, stats.average_frametime_ms
    );
    if let Some(node) = tree.get_mut(nodes[0]) {
        node.set_text(avg_fps_text);
        node.text_color = get_fps_color(stats.average_fps);
    }

    let low_1_text = format!("{:.0} FPS", stats.low_1_percent_fps);
    if let Some(node) = tree.get_mut(nodes[1]) {
        node.set_text(low_1_text);
        node.text_color = get_fps_color(stats.low_1_percent_fps);
    }

    let low_01_text = format!("{:.0} FPS", stats.low_0_1_percent_fps);
    if let Some(node) = tree.get_mut(nodes[2]) {
        node.set_text(low_01_text);
        node.text_color = get_fps_color(stats.low_0_1_percent_fps);
    }

    let jitter_color = if stats.variance_ms < 1.5 {
        Color::rgba(0.0, 0.82, 0.63, 1.0)
    } else if stats.variance_ms < 4.0 {
        Color::rgba(1.0, 0.73, 0.0, 1.0)
    } else {
        Color::rgba(0.92, 0.24, 0.24, 1.0)
    };
    let jitter_text = format!("±{:.2} ms", stats.variance_ms);
    if let Some(node) = tree.get_mut(nodes[3]) {
        node.set_text(jitter_text);
        node.text_color = jitter_color;
    }

    // 2. Update Pacing Summary Footer
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

    if let Some(node) = tree.get_mut(footer_id) {
        node.set_text(summary_text);
        node.text_color = summary_color;
    }
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

/// Helper function to build a single metric pill container with static label and dynamic value nodes.
fn build_metric_pill(
    tree: &mut UiTree,
    parent_id: WidgetId,
    label: &str,
    initial_value: &str,
    val_color: Color,
    rect: Rect,
) -> WidgetId {
    let pill_id = tree.create_node();
    if let Some(node) = tree.get_mut(pill_id) {
        node.set_name("MetricPill");
        node.computed_rect = rect;
        node.style = Style::new()
            .background(Color::rgba(0.04, 0.04, 0.06, 0.90))
            .border(1.0, Color::rgba(0.14, 0.15, 0.20, 0.80))
            .border_radius(4.0);
    }
    let _ = tree.add_child(parent_id, pill_id);

    let lbl_w = (rect.width * 0.45).max(40.0);
    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name("PillLabel");
        node.set_text(label);
        node.font_size = 10.0;
        node.line_height = rect.height;
        node.text_color = Color::rgba(0.60, 0.62, 0.70, 1.0);
        node.computed_rect = Rect::new(rect.x + 6.0, rect.y, lbl_w, rect.height);
    }
    let _ = tree.add_child(pill_id, lbl_id);

    let val_id = tree.create_node();
    if let Some(node) = tree.get_mut(val_id) {
        node.set_name("PillValue");
        node.set_text(initial_value);
        node.font_size = 11.0;
        node.line_height = rect.height;
        node.text_align = TextAlign::Right;
        node.text_color = val_color;
        node.computed_rect = Rect::new(
            rect.x + lbl_w + 2.0,
            rect.y,
            rect.width - lbl_w - 8.0,
            rect.height,
        );
    }
    let _ = tree.add_child(pill_id, val_id);

    val_id
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