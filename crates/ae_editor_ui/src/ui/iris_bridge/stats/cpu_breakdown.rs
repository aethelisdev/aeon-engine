// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # CPU Thread Synchronization Breakdown Builder
//!
//! Renders the CPU multi-segmented subsystem timing bar, detailed subsystem timings,
//! and thread balance bottleneck indicator purely using declarative zero-`let` [`UiScope`]
//! calls and zero-heap [`StackString`] formatting.

use super::types::{
    STATS_TAG_CPU_BALANCE, STATS_TAG_CPU_ECS, STATS_TAG_CPU_PHYSICS, STATS_TAG_CPU_RENDER,
    STATS_TAG_CPU_TOTAL, STATS_TAG_CPU_UI, STATS_TAG_CPU_WAIT,
};
use ae_core::telemetry::CpuSyncTimings;
use irisui::prelude::*;
use std::fmt::Write;

#[inline]
fn total_cpu_ms(cpu: &CpuSyncTimings) -> f32 {
    (cpu.main_logic_ms
        + cpu.physics_ms
        + cpu.render_prep_ms
        + cpu.wait_for_gpu_ms
        + cpu.ui_editor_ms)
        .max(0.0001)
}

/// Builds the CPU Thread & Synchronization breakdown section inside the stats card.
pub fn build_cpu_breakdown_content(scope: &mut UiScope<'_>, cpu_timings: &CpuSyncTimings) {
    build_cpu_breakdown_with_total(scope, cpu_timings, total_cpu_ms(cpu_timings));
}

fn build_cpu_breakdown_with_total(scope: &mut UiScope<'_>, cpu: &CpuSyncTimings, total_ms: f32) {
    scope.container(Style::new().flex_col().gap(4.0), |s| {
        build_thread_balance_row(s, total_ms);

        s.segmented_bar(&[
            (
                cpu.main_logic_ms / total_ms,
                Color::rgba(0.0, 0.75, 0.90, 1.0),
            ),
            (
                cpu.physics_ms / total_ms,
                Color::rgba(0.96, 0.57, 0.12, 1.0),
            ),
            (
                cpu.render_prep_ms / total_ms,
                Color::rgba(0.35, 0.51, 0.94, 1.0),
            ),
            (
                cpu.wait_for_gpu_ms / total_ms,
                Color::rgba(0.63, 0.39, 0.86, 1.0),
            ),
            (
                cpu.ui_editor_ms / total_ms,
                Color::rgba(0.86, 0.71, 0.20, 1.0),
            ),
        ]);

        [
            (
                "ECS / Logic",
                cpu.main_logic_ms,
                Color::rgba(0.0, 0.75, 0.90, 1.0),
                STATS_TAG_CPU_ECS,
            ),
            (
                "Physics Simulation",
                cpu.physics_ms,
                Color::rgba(0.96, 0.57, 0.12, 1.0),
                STATS_TAG_CPU_PHYSICS,
            ),
            (
                "Render Preparation",
                cpu.render_prep_ms,
                Color::rgba(0.35, 0.51, 0.94, 1.0),
                STATS_TAG_CPU_RENDER,
            ),
            (
                "Wait for GPU (VSync)",
                cpu.wait_for_gpu_ms,
                Color::rgba(0.63, 0.39, 0.86, 1.0),
                STATS_TAG_CPU_WAIT,
            ),
            (
                "UI / Editor Passes",
                cpu.ui_editor_ms,
                Color::rgba(0.86, 0.71, 0.20, 1.0),
                STATS_TAG_CPU_UI,
            ),
        ]
        .iter()
        .for_each(|&(name, val, col, tag)| build_timing_row_tagged(s, name, val, col, tag));

        build_total_row(s, "Total CPU Frame:", total_ms, STATS_TAG_CPU_TOTAL);
    });
}

fn build_thread_balance_row(scope: &mut UiScope<'_>, total_ms: f32) {
    scope.container(
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(16.0),
        |r| {
            r.text_colored("Thread Balance:", Color::rgba(0.59, 0.61, 0.67, 1.0));
            r.spacer();
            r.text_colored_tagged(
                cpu_balance_label(total_ms),
                cpu_balance_color(total_ms),
                STATS_TAG_CPU_BALANCE,
            );
        },
    );
}

/// Emits a standard timing or metric row with color dot indicator, name, and right-aligned value.
pub fn build_timing_row(scope: &mut UiScope<'_>, name: &str, val_ms: f32, dot_col: Color) {
    build_timing_row_tagged(scope, name, val_ms, dot_col, 0);
}

/// Emits a standard timing or metric row with semantic tag for reactive in-place text updates.
pub fn build_timing_row_tagged(
    scope: &mut UiScope<'_>,
    name: &str,
    val_ms: f32,
    dot_col: Color,
    tag: u64,
) {
    scope.metric_dot_row_fmt::<24, _>(name, tag, dot_col, None, |buf| {
        write_ms_fixed2(buf, val_ms);
    });
}

fn build_total_row(scope: &mut UiScope<'_>, label: &str, total_ms: f32, tag: u64) {
    scope.container(
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(18.0)
            .margin_insets(Insets::new(2.0, 0.0, 0.0, 0.0)),
        |r| {
            r.text_colored(label, Color::rgba(0.86, 0.88, 0.92, 1.0));
            r.spacer();
            r.text_colored_tagged_fmt::<24, _>(Color::rgba(0.78, 0.90, 1.0, 1.0), tag, |buf| {
                write_ms_fixed2(buf, total_ms)
            });
        },
    );
}

/// Updates CPU timing breakdown labels in-place without tree reconstruction or layout passes.
pub fn update_cpu_breakdown_texts(scope: &mut UiScope<'_>, cpu: &CpuSyncTimings) -> bool {
    update_cpu_breakdown_with_total(scope, cpu, total_cpu_ms(cpu))
}

#[inline]
fn cpu_balance_label(total_cpu_ms: f32) -> &'static str {
    if total_cpu_ms < 8.33 {
        "Optimal (120+ FPS)"
    } else if total_cpu_ms < 16.67 {
        "Good (60+ FPS)"
    } else if total_cpu_ms < 33.33 {
        "Moderate Load"
    } else {
        "Heavy Bottleneck"
    }
}

#[inline]
fn cpu_balance_color(total_cpu_ms: f32) -> Color {
    if total_cpu_ms < 8.33 {
        Color::rgba(0.0, 0.82, 0.63, 1.0)
    } else if total_cpu_ms < 16.67 {
        Color::rgba(0.0, 0.75, 0.90, 1.0)
    } else if total_cpu_ms < 33.33 {
        Color::rgba(1.0, 0.73, 0.0, 1.0)
    } else {
        Color::rgba(0.92, 0.24, 0.24, 1.0)
    }
}

#[inline]
pub(super) fn write_ms_fixed2<const N: usize>(buf: &mut StackString<N>, val_ms: f32) {
    buf.write_fmt(format_args!(
        "{:.2} ms",
        if val_ms.is_finite() {
            val_ms.clamp(0.0, 99_999.99)
        } else {
            0.0
        }
    ))
    .ok();
}

fn update_cpu_breakdown_with_total(
    scope: &mut UiScope<'_>,
    cpu: &CpuSyncTimings,
    total_cpu_ms: f32,
) -> bool {
    scope.update_live_text_colored(
        STATS_TAG_CPU_BALANCE,
        cpu_balance_label(total_cpu_ms),
        cpu_balance_color(total_cpu_ms),
    ) | scope.update_live_text_fmt::<24, _>(STATS_TAG_CPU_ECS, |b| {
        write_ms_fixed2(b, cpu.main_logic_ms);
    }) | scope.update_live_text_fmt::<24, _>(STATS_TAG_CPU_PHYSICS, |b| {
        write_ms_fixed2(b, cpu.physics_ms);
    }) | scope.update_live_text_fmt::<24, _>(STATS_TAG_CPU_RENDER, |b| {
        write_ms_fixed2(b, cpu.render_prep_ms);
    }) | scope.update_live_text_fmt::<24, _>(STATS_TAG_CPU_WAIT, |b| {
        write_ms_fixed2(b, cpu.wait_for_gpu_ms);
    }) | scope.update_live_text_fmt::<24, _>(STATS_TAG_CPU_UI, |b| {
        write_ms_fixed2(b, cpu.ui_editor_ms);
    }) | scope.update_live_text_fmt::<24, _>(STATS_TAG_CPU_TOTAL, |b| {
        write_ms_fixed2(b, total_cpu_ms);
    })
}