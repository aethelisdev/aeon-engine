// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # CPU Thread Synchronization Breakdown Builder
//!
//! Renders the CPU multi-segmented subsystem timing bar, detailed subsystem timings,
//! and thread balance bottleneck indicator purely using [`UiScope`].
//!

use super::types::StatsPanelParams;
use irisui::prelude::*;

/// Builds the CPU Thread & Synchronization breakdown section inside the stats card.
pub fn build_cpu_breakdown_content(scope: &mut UiScope<'_>, params: &StatsPanelParams<'_>) {
    let cpu = params.cpu_timings;
    let total_cpu_ms = (cpu.main_logic_ms
        + cpu.physics_ms
        + cpu.render_prep_ms
        + cpu.wait_for_gpu_ms
        + cpu.ui_editor_ms)
        .max(0.0001);

    let container_style = Style::new().flex_col().gap(4.0);

    scope.container(container_style, |s| {
        // 1. Thread Balance Status Row
        build_thread_balance_row(s, total_cpu_ms);

        // 2. Multi-segmented composite bar
        let segments = [
            (
                cpu.main_logic_ms / total_cpu_ms,
                Color::rgba(0.0, 0.75, 0.90, 1.0),
            ), // ECS (Cyan)
            (
                cpu.physics_ms / total_cpu_ms,
                Color::rgba(0.96, 0.57, 0.12, 1.0),
            ), // Physics (Orange)
            (
                cpu.render_prep_ms / total_cpu_ms,
                Color::rgba(0.35, 0.51, 0.94, 1.0),
            ), // Render Prep (Blue)
            (
                cpu.wait_for_gpu_ms / total_cpu_ms,
                Color::rgba(0.63, 0.39, 0.86, 1.0),
            ), // Wait for GPU (Purple)
            (
                cpu.ui_editor_ms / total_cpu_ms,
                Color::rgba(0.86, 0.71, 0.20, 1.0),
            ), // UI (Yellow)
        ];
        build_multi_segment_bar(s, &segments);

        // 3. Subsystem Timing Rows
        let rows = [
            (
                "ECS / Logic",
                cpu.main_logic_ms,
                Color::rgba(0.0, 0.75, 0.90, 1.0),
            ),
            (
                "Physics Simulation",
                cpu.physics_ms,
                Color::rgba(0.96, 0.57, 0.12, 1.0),
            ),
            (
                "Render Preparation",
                cpu.render_prep_ms,
                Color::rgba(0.35, 0.51, 0.94, 1.0),
            ),
            (
                "Wait for GPU (VSync)",
                cpu.wait_for_gpu_ms,
                Color::rgba(0.63, 0.39, 0.86, 1.0),
            ),
            (
                "UI / Editor Passes",
                cpu.ui_editor_ms,
                Color::rgba(0.86, 0.71, 0.20, 1.0),
            ),
        ];

        for (name, val, col) in rows {
            build_timing_row(s, name, val, col);
        }

        // 4. Total CPU Frame Footer
        build_total_row(s, "Total CPU Frame:", total_cpu_ms);
    });
}

/// Helper emitting the Thread Balance indicator row.
fn build_thread_balance_row(scope: &mut UiScope<'_>, total_ms: f32) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(16.0);

    scope.container(row_style, |r| {
        r.text_colored("Thread Balance:", Color::rgba(0.59, 0.61, 0.67, 1.0));

        r.spacer();

        let (status_text, status_col) = if total_ms < 8.33 {
            ("Optimal (120+ FPS)", Color::rgba(0.0, 0.82, 0.63, 1.0))
        } else if total_ms < 16.67 {
            ("Good (60+ FPS)", Color::rgba(0.0, 0.75, 0.90, 1.0))
        } else if total_ms < 33.33 {
            ("Moderate Load", Color::rgba(1.0, 0.73, 0.0, 1.0))
        } else {
            ("Heavy Bottleneck", Color::rgba(0.92, 0.24, 0.24, 1.0))
        };

        r.text_colored(status_text, status_col);
    });
}

/// Emits a declarative multi-segmented proportional progress bar.
pub fn build_multi_segment_bar(scope: &mut UiScope<'_>, segments: &[(f32, Color)]) {
    let bar_style = Style::new()
        .flex_row()
        .height(4.0)
        .border_radius(2.0)
        .background(Color::rgba(0.08, 0.09, 0.12, 0.95))
        .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.80))
        .clip_children(true);

    scope.container(bar_style, |bar_scope| {
        for &(ratio, col) in segments {
            let flex_ratio = ratio.clamp(0.001, 1.0);
            let seg_style = Style::new()
                .background(col)
                .flex_grow(flex_ratio)
                .height(4.0);
            bar_scope.empty_box(seg_style);
        }
    });
}

/// Emits a standard timing or metric row with color dot indicator, name, and right-aligned value.
pub fn build_timing_row(scope: &mut UiScope<'_>, name: &str, val_ms: f32, dot_col: Color) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .gap(6.0)
        .height(16.0);

    scope.container(row_style, |r| {
        // Color dot indicator
        let dot_style = Style::new()
            .width(6.0)
            .height(6.0)
            .border_radius(3.0)
            .background(dot_col);
        r.empty_box(dot_style);

        // Subsystem label
        r.text_colored(name, Color::rgba(0.78, 0.80, 0.86, 1.0));

        r.spacer();

        // Timing value
        let val_text = format!("{:.2} ms", val_ms);
        r.text_colored(&val_text, Color::rgba(0.86, 0.88, 0.92, 1.0));
    });
}

/// Emits a summary total row with bold accent text.
fn build_total_row(scope: &mut UiScope<'_>, label: &str, total_ms: f32) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(18.0)
        .margin_insets(Insets::new(2.0, 0.0, 0.0, 0.0));

    scope.container(row_style, |r| {
        r.text_colored(label, Color::rgba(0.86, 0.88, 0.92, 1.0));

        r.spacer();

        let val_text = format!("{:.2} ms", total_ms);
        r.text_colored(&val_text, Color::rgba(0.78, 0.90, 1.0, 1.0));
    });
}