// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Geometry, Draw Call & Video RAM Metrics Declarative Builder
//!
//! Renders the geometry metrics card and VRAM allocation distribution card
//! purely using zero-`let`, zero-heap [`UiScope`] and [`StackString`] primitives.

use super::types::StatsPanelParams;
use irisui::prelude::*;
use std::fmt::Write;

#[inline]
fn instanced_percent(total_dc: u32, instanced: u32) -> u32 {
    if total_dc > 0 {
        ((instanced as f32 / total_dc as f32) * 100.0).round() as u32
    } else {
        100
    }
}

/// Builds the Scene & Geometry Metrics section inside the declarative stats card.
pub fn build_scene_geometry_content(scope: &mut UiScope<'_>, params: &StatsPanelParams<'_>) {
    scope.container(Style::new().flex_col().gap(4.0), |s| {
        s.container(
            Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .height(16.0),
            |hr| {
                hr.text_colored("Draw Calls:", Color::rgba(0.86, 0.88, 0.92, 1.0));
                hr.text_colored_fmt::<16, _>(Color::rgba(0.0, 0.82, 0.63, 1.0), |buf| {
                    buf.write_fmt(format_args!("{}", params.draw_call_stats.total_draw_calls))
                        .ok();
                });
                hr.spacer();
                hr.text_colored_fmt::<24, _>(Color::rgba(0.55, 0.57, 0.63, 1.0), |buf| {
                    buf.write_fmt(format_args!(
                        "{}% Instanced",
                        instanced_percent(
                            params.draw_call_stats.total_draw_calls,
                            params.draw_call_stats.instanced_draw_calls
                        )
                    ))
                    .ok();
                });
            },
        );

        [
            (
                "Batched Primitives",
                params.draw_call_stats.batched_draw_calls,
                Color::rgba(0.0, 0.75, 0.90, 1.0),
            ),
            (
                "Hardware Instanced",
                params.draw_call_stats.instanced_draw_calls,
                Color::rgba(0.39, 0.86, 0.39, 1.0),
            ),
            (
                "Dispatched Compute",
                params.draw_call_stats.dispatched_compute,
                Color::rgba(0.94, 0.71, 0.20, 1.0),
            ),
            (
                "Culled / Occluded",
                params.draw_call_stats.culled_meshes,
                Color::rgba(0.71, 0.55, 0.86, 1.0),
            ),
        ]
        .iter()
        .for_each(|&(name, val, dot_col)| {
            s.metric_dot_row_fmt::<16, _>(name, 0, dot_col, None, |buf| {
                buf.write_fmt(format_args!("{val}")).ok();
            });
        });

        s.metric_dot_row_fmt::<24, _>(
            "Triangles Rendered",
            0,
            Color::rgba(0.78, 0.90, 1.0, 1.0),
            None,
            |buf| write_compact_count(buf, params.render_triangles),
        );
        s.metric_dot_row_fmt::<24, _>(
            "Vertices Rendered",
            0,
            Color::rgba(0.78, 0.90, 1.0, 1.0),
            None,
            |buf| write_compact_count(buf, params.render_vertices),
        );
        s.metric_dot_row_fmt::<24, _>(
            "Scene Entities",
            0,
            Color::rgba(0.47, 0.55, 0.71, 1.0),
            None,
            |buf| {
                buf.write_fmt(format_args!("{}", params.active_entities_count))
                    .ok();
            },
        );
    });
}

/// Builds the Video RAM & Memory Allocations section inside the declarative stats card.
pub fn build_vram_breakdown_content(scope: &mut UiScope<'_>, params: &StatsPanelParams<'_>) {
    build_vram_with_total(
        scope,
        params.vram_stats,
        params.vram_stats.total_vram_mb.max(0.001),
    );
}

fn build_vram_with_total(
    scope: &mut UiScope<'_>,
    vram: &ae_core::telemetry::VramStats,
    total_vram_mb: f32,
) {
    scope.container(Style::new().flex_col().gap(4.0), |s| {
        s.segmented_bar(&[
            (
                vram.texture_vram_mb / total_vram_mb,
                Color::rgba(0.0, 0.82, 0.63, 1.0),
            ),
            (
                vram.mesh_index_vram_mb / total_vram_mb,
                Color::rgba(0.0, 0.75, 0.90, 1.0),
            ),
            (
                vram.dynamic_uniform_vram_mb / total_vram_mb,
                Color::rgba(0.63, 0.39, 0.86, 1.0),
            ),
        ]);

        [
            (
                "Texture Assets",
                vram.texture_vram_mb,
                Color::rgba(0.0, 0.82, 0.63, 1.0),
            ),
            (
                "Mesh & Index Buffers",
                vram.mesh_index_vram_mb,
                Color::rgba(0.0, 0.75, 0.90, 1.0),
            ),
            (
                "Dynamic Uniform Buffers",
                vram.dynamic_uniform_vram_mb,
                Color::rgba(0.63, 0.39, 0.86, 1.0),
            ),
        ]
        .iter()
        .for_each(|&(name, mb, col)| {
            s.metric_dot_row_fmt::<24, _>(name, 0, col, None, |buf| {
                buf.write_fmt(format_args!("{mb:.1} MB")).ok();
            });
        });

        s.container(
            Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .height(18.0)
                .margin_insets(Insets::new(2.0, 0.0, 0.0, 0.0)),
            |r| {
                r.text_colored("Total VRAM Allocated:", Color::rgba(0.86, 0.88, 0.92, 1.0));
                r.spacer();
                r.text_colored_fmt::<24, _>(Color::rgba(0.78, 0.90, 1.0, 1.0), |buf| {
                    buf.write_fmt(format_args!("{total_vram_mb:.1} MB")).ok();
                });
            },
        );
    });
}

#[inline]
fn write_compact_count<const N: usize>(buf: &mut StackString<N>, count: u64) {
    if count >= 1_000_000 {
        buf.write_fmt(format_args!("{:.2}M", count as f64 / 1_000_000.0))
            .ok();
    } else if count >= 1_000 {
        buf.write_fmt(format_args!("{:.1}K", count as f64 / 1_000.0))
            .ok();
    } else {
        buf.write_fmt(format_args!("{count}")).ok();
    }
}