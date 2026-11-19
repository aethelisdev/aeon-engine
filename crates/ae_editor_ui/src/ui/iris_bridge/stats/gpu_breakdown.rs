// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # GPU Render Passes Breakdown Builder
//!
//! Renders the GPU multi-segmented pass execution bar, hardware adapter name,
//! and pass durations purely using [`UiScope`] without intermediate `let`s or heap `format!`.

use super::cpu_breakdown::{build_timing_row, write_ms_fixed2};
use super::types::StatsPanelParams;
use ae_core::telemetry::RenderPassRecordingTimings;
use irisui::prelude::*;

#[inline]
fn total_record_ms(rec: &RenderPassRecordingTimings) -> f32 {
    (rec.shadow_pass_record_ms
        + rec.main_opaque_pass_record_ms
        + rec.post_process_pass_record_ms
        + rec.ui_pass_record_ms)
        .max(0.0001)
}

/// Builds the Render Pass Command Recording breakdown section inside the stats card.
pub fn build_gpu_breakdown_content(scope: &mut UiScope<'_>, params: &StatsPanelParams<'_>) {
    build_gpu_breakdown_with_total(
        scope,
        params.pass_recording_timings,
        total_record_ms(params.pass_recording_timings),
        params.gpu_adapter_name,
        params.gpu_backend,
    );
}

fn build_gpu_breakdown_with_total(
    scope: &mut UiScope<'_>,
    rec: &RenderPassRecordingTimings,
    total_ms: f32,
    adapter: &str,
    backend: &str,
) {
    scope.container(Style::new().flex_col().gap(4.0), |s| {
        build_gpu_device_row(s, adapter, backend);

        s.segmented_bar(&[
            (
                rec.shadow_pass_record_ms / total_ms,
                Color::rgba(0.96, 0.57, 0.12, 1.0),
            ),
            (
                rec.main_opaque_pass_record_ms / total_ms,
                Color::rgba(0.35, 0.51, 0.94, 1.0),
            ),
            (
                rec.post_process_pass_record_ms / total_ms,
                Color::rgba(0.86, 0.35, 0.67, 1.0),
            ),
            (
                rec.ui_pass_record_ms / total_ms,
                Color::rgba(0.86, 0.71, 0.20, 1.0),
            ),
        ]);

        [
            (
                "Shadow Pass Record",
                rec.shadow_pass_record_ms,
                Color::rgba(0.96, 0.57, 0.12, 1.0),
            ),
            (
                "Main Forward Record",
                rec.main_opaque_pass_record_ms,
                Color::rgba(0.35, 0.51, 0.94, 1.0),
            ),
            (
                "Post-Process Record",
                rec.post_process_pass_record_ms,
                Color::rgba(0.86, 0.35, 0.67, 1.0),
            ),
            (
                "UI Composite Record",
                rec.ui_pass_record_ms,
                Color::rgba(0.86, 0.71, 0.20, 1.0),
            ),
        ]
        .iter()
        .for_each(|&(name, val, col)| build_timing_row(s, name, val, col));

        build_gpu_total_row(s, "Total Command Recording:", total_ms);
    });
}

fn build_gpu_device_row(scope: &mut UiScope<'_>, adapter: &str, backend: &str) {
    scope.container(
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(16.0),
        |r| {
            r.text_colored(
                if adapter.is_empty() {
                    "Default Graphics Adapter"
                } else {
                    adapter
                },
                Color::rgba(0.78, 0.82, 0.88, 1.0),
            );
            r.spacer();
            r.text_colored(backend, Color::rgba(0.0, 0.82, 0.63, 1.0));
        },
    );
}

fn build_gpu_total_row(scope: &mut UiScope<'_>, label: &str, total_ms: f32) {
    scope.container(
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(18.0)
            .margin_insets(Insets::new(2.0, 0.0, 0.0, 0.0)),
        |r| {
            r.text_colored(label, Color::rgba(0.86, 0.88, 0.92, 1.0));
            r.spacer();
            r.text_colored_fmt::<24, _>(Color::rgba(0.78, 0.90, 1.0, 1.0), |buf| {
                write_ms_fixed2(buf, total_ms);
            });
        },
    );
}