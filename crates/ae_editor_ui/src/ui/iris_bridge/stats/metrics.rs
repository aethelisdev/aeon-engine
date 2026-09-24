// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Geometry, Draw Call & Video RAM Metrics Declarative Builder
//!
//! Renders the geometry metrics card and VRAM allocation distribution card
//! purely using [`UiScope`].
//!

use super::cpu_breakdown::build_multi_segment_bar;
use super::types::StatsPanelParams;
use irisui::prelude::*;

/// Builds the Scene & Geometry Metrics section inside the declarative stats card.
pub fn build_scene_geometry_content(scope: &mut UiScope<'_>, params: &StatsPanelParams<'_>) {
    let dc = params.draw_call_stats;
    let total_dc = dc.total_draw_calls;
    let inst_pct = if total_dc > 0 {
        ((dc.instanced_draw_calls as f32 / total_dc as f32) * 100.0).round() as u32
    } else {
        100
    };

    let container_style = Style::new().flex_col().gap(4.0);

    scope.container(container_style, |s| {
        // 1. Draw Calls Header Row
        let header_row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(16.0);

        s.container(header_row_style, |hr| {
            hr.text_colored("Draw Calls:", Color::rgba(0.86, 0.88, 0.92, 1.0));

            hr.text_colored(total_dc.to_string(), Color::rgba(0.0, 0.82, 0.63, 1.0));

            hr.spacer();

            let inst_text = format!("{}% Instanced", inst_pct);
            hr.text_colored(&inst_text, Color::rgba(0.55, 0.57, 0.63, 1.0));
        });

        // 2. Draw Call Subrows
        let batched_str = dc.batched_draw_calls.to_string();
        let instanced_str = dc.instanced_draw_calls.to_string();
        let compute_str = dc.dispatched_compute.to_string();
        let culled_str = dc.culled_meshes.to_string();

        let dc_rows: [(&str, &str, Color); 4] = [
            (
                "Batched Primitives",
                &batched_str,
                Color::rgba(0.0, 0.75, 0.90, 1.0),
            ),
            (
                "Hardware Instanced",
                &instanced_str,
                Color::rgba(0.39, 0.86, 0.39, 1.0),
            ),
            (
                "Dispatched Compute",
                &compute_str,
                Color::rgba(0.94, 0.71, 0.20, 1.0),
            ),
            (
                "Culled / Occluded",
                &culled_str,
                Color::rgba(0.71, 0.55, 0.86, 1.0),
            ),
        ];

        for (name, val, col) in dc_rows {
            build_metric_string_row(s, name, val, col);
        }

        // 3. Triangles, Vertices & Active Entities
        build_metric_string_row(
            s,
            "Triangles Rendered",
            &format_count(params.render_triangles),
            Color::rgba(0.78, 0.90, 1.0, 1.0),
        );
        build_metric_string_row(
            s,
            "Vertices Rendered",
            &format_count(params.render_vertices),
            Color::rgba(0.78, 0.90, 1.0, 1.0),
        );
        let entities_str = params.active_entities_count.to_string();
        build_metric_string_row(
            s,
            "Scene Entities",
            &entities_str,
            Color::rgba(0.47, 0.55, 0.71, 1.0),
        );
    });
}

/// Builds the Video RAM & Memory Allocations section inside the declarative stats card.
pub fn build_vram_breakdown_content(scope: &mut UiScope<'_>, params: &StatsPanelParams<'_>) {
    let vram = params.vram_stats;
    let total_vram_mb = vram.total_vram_mb.max(0.001);

    let container_style = Style::new().flex_col().gap(4.0);

    scope.container(container_style, |s| {
        // 1. Multi-segmented composite VRAM bar
        let segments = [
            (
                vram.texture_vram_mb / total_vram_mb,
                Color::rgba(0.0, 0.82, 0.63, 1.0),
            ), // Textures (Emerald)
            (
                vram.mesh_index_vram_mb / total_vram_mb,
                Color::rgba(0.0, 0.75, 0.90, 1.0),
            ), // Mesh/Index (Cyan)
            (
                vram.dynamic_uniform_vram_mb / total_vram_mb,
                Color::rgba(0.63, 0.39, 0.86, 1.0),
            ), // Dynamic/Uniform (Purple)
        ];
        build_multi_segment_bar(s, &segments);

        // 2. VRAM Category Rows
        let tex_str = format!("{:.1} MB", vram.texture_vram_mb);
        let mesh_str = format!("{:.1} MB", vram.mesh_index_vram_mb);
        let dyn_str = format!("{:.1} MB", vram.dynamic_uniform_vram_mb);

        let rows: [(&str, &str, Color); 3] = [
            (
                "Texture Assets",
                &tex_str,
                Color::rgba(0.0, 0.82, 0.63, 1.0),
            ),
            (
                "Mesh & Index Buffers",
                &mesh_str,
                Color::rgba(0.0, 0.75, 0.90, 1.0),
            ),
            (
                "Dynamic Uniform Buffers",
                &dyn_str,
                Color::rgba(0.63, 0.39, 0.86, 1.0),
            ),
        ];

        for (name, val, col) in rows {
            build_metric_string_row(s, name, val, col);
        }

        // 3. Total VRAM Allocated Footer
        let total_row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(18.0)
            .margin_insets(Insets::new(2.0, 0.0, 0.0, 0.0));

        s.container(total_row_style, |r| {
            r.text_colored("Total VRAM Allocated:", Color::rgba(0.86, 0.88, 0.92, 1.0));

            r.spacer();

            let val_text = format!("{:.1} MB", total_vram_mb);
            r.text_colored(&val_text, Color::rgba(0.78, 0.90, 1.0, 1.0));
        });
    });
}

/// Helper emitting a metric row with color dot, label, spacer, and value string.
fn build_metric_string_row(scope: &mut UiScope<'_>, name: &str, val_str: &str, dot_col: Color) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .gap(6.0)
        .height(16.0);

    scope.container(row_style, |r| {
        let dot_style = Style::new()
            .width(6.0)
            .height(6.0)
            .border_radius(3.0)
            .background(dot_col);
        r.empty_box(dot_style);

        r.text_colored(name, Color::rgba(0.78, 0.80, 0.86, 1.0));

        r.spacer();

        r.text_colored(val_str, Color::rgba(0.86, 0.88, 0.92, 1.0));
    });
}

/// Formats large geometry counts with thousands separators for readability.
fn format_count(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.2}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}