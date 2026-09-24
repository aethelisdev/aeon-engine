// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # GPU Render Passes Breakdown Builder
//!
//! Renders the GPU multi-segmented pass execution bar, hardware adapter name,
//! and pass durations purely using [`UiScope`].
//!

use super::cpu_breakdown::{build_multi_segment_bar, build_timing_row};
use super::types::StatsPanelParams;
use irisui::prelude::*;

/// Builds the GPU Render Passes breakdown section inside the stats card.
pub fn build_gpu_breakdown_content(scope: &mut UiScope<'_>, params: &StatsPanelParams<'_>) {
    let gpu = params.gpu_pass_timings;
    let total_gpu_ms =
        (gpu.shadow_pass_ms + gpu.main_opaque_pass_ms + gpu.post_process_pass_ms + gpu.ui_pass_ms)
            .max(0.0001);

    let container_style = Style::new().flex_col().gap(4.0);

    scope.container(container_style, |s| {
        // 1. GPU Adapter Device Name & Backend Badge
        build_gpu_device_row(s, params.gpu_adapter_name, params.gpu_backend);

        // 2. Multi-segmented composite bar
        let segments = [
            (
                gpu.shadow_pass_ms / total_gpu_ms,
                Color::rgba(0.96, 0.57, 0.12, 1.0),
            ), // Shadow (Orange)
            (
                gpu.main_opaque_pass_ms / total_gpu_ms,
                Color::rgba(0.35, 0.51, 0.94, 1.0),
            ), // Forward (Blue)
            (
                gpu.post_process_pass_ms / total_gpu_ms,
                Color::rgba(0.86, 0.35, 0.67, 1.0),
            ), // Post-Process (Magenta)
            (
                gpu.ui_pass_ms / total_gpu_ms,
                Color::rgba(0.86, 0.71, 0.20, 1.0),
            ), // UI (Gold)
        ];
        build_multi_segment_bar(s, &segments);

        // 3. GPU Pass Timing Rows
        let rows = [
            (
                "Shadow Pass (Cascades)",
                gpu.shadow_pass_ms,
                Color::rgba(0.96, 0.57, 0.12, 1.0),
            ),
            (
                "Main Forward Pass",
                gpu.main_opaque_pass_ms,
                Color::rgba(0.35, 0.51, 0.94, 1.0),
            ),
            (
                "Post-Process & Outline",
                gpu.post_process_pass_ms,
                Color::rgba(0.86, 0.35, 0.67, 1.0),
            ),
            (
                "UI Composite Pass",
                gpu.ui_pass_ms,
                Color::rgba(0.86, 0.71, 0.20, 1.0),
            ),
        ];

        for (name, val, col) in rows {
            build_timing_row(s, name, val, col);
        }

        // 4. Total GPU Workload Footer
        build_gpu_total_row(s, "Total GPU Workload:", total_gpu_ms);
    });
}

/// Helper emitting GPU adapter name and backend badge.
fn build_gpu_device_row(scope: &mut UiScope<'_>, adapter: &str, backend: &str) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(16.0);

    scope.container(row_style, |r| {
        let name_text = if adapter.is_empty() {
            "Default Graphics Adapter"
        } else {
            adapter
        };
        r.text_colored(name_text, Color::rgba(0.78, 0.82, 0.88, 1.0));

        r.spacer();

        r.text_colored(backend, Color::rgba(0.0, 0.82, 0.63, 1.0));
    });
}

/// Emits the total GPU workload summary row.
fn build_gpu_total_row(scope: &mut UiScope<'_>, label: &str, total_ms: f32) {
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