// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Frame Pacing & Stutter Analyzer Oscilloscope Renderer
//!
//! Visualizes real-time historical frametimes with milestone threshold lines,
//! dynamic stutter indicators, and oscilloscope quads using zero per-frame allocations.
//!

use ae_core::telemetry::FrameRingBuffer;
use irisui::prelude::*;

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