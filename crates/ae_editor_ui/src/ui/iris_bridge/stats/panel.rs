// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Stats and telemetry panel layout building using declarative UiScope.
//!

use super::types::{StatsPanelNodes, StatsPanelParams, StatsPanelTargets};
use irisui::prelude::*;

/// Builds the declarative Stats & Profiler panel layout tree using `UiScope`.
pub fn build_stats_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &StatsPanelParams<'_>,
    targets: &mut StatsPanelTargets,
) -> StatsPanelNodes {
    targets.panel_rect = params.panel_rect;

    let mut scope = UiScope::new(tree, parent_id);
    let root_id = scope.container(
        Style::new()
            .flex_col()
            .background(Color::rgba(0.06, 0.07, 0.09, 1.0))
            .padding(6.0)
            .gap(6.0)
            .clip_children(true),
        |col| {
            // 1. Frame Pacing Card
            col.card("📈 Frame Pacing & Stutter Analyzer", |card| {
                card.row(|r| {
                    r.text_colored(
                        format!("FPS: {:.1}", params.fps),
                        Color::rgba(0.0, 0.9, 1.0, 1.0),
                    );
                    r.text(format!(
                        "1% Low: {:.1}",
                        params.frame_pacing_stats.low_1_percent_fps
                    ));
                    r.text(format!(
                        "0.1% Low: {:.1}",
                        params.frame_pacing_stats.low_0_1_percent_fps
                    ));
                });
                card.text(format!(
                    "Avg Frame Time: {:.2} ms | Variance: {:.2} ms",
                    params.frame_pacing_stats.average_frametime_ms,
                    params.frame_pacing_stats.variance_ms,
                ));
            });

            // 2. CPU Thread Breakdown Card
            col.card("⏱ CPU Thread & Synchronization", |card| {
                card.text(format!(
                    "Main Logic: {:.2} ms",
                    params.cpu_timings.main_logic_ms
                ));
                card.text(format!(
                    "Render Prep: {:.2} ms",
                    params.cpu_timings.render_prep_ms
                ));
                card.text(format!("Physics: {:.2} ms", params.cpu_timings.physics_ms));
                card.text(format!(
                    "Wait for GPU: {:.2} ms",
                    params.cpu_timings.wait_for_gpu_ms
                ));
                card.text(format!(
                    "Total CPU: {:.2} ms",
                    params.cpu_timings.total_cpu_ms
                ));
            });

            // 3. GPU Render Passes Card
            col.card("⚡ GPU Render Passes", |card| {
                card.text(format!(
                    "Adapter: {} ({})",
                    params.gpu_adapter_name, params.gpu_backend
                ));
                card.text(format!(
                    "Shadow Pass: {:.2} ms",
                    params.gpu_pass_timings.shadow_pass_ms
                ));
                card.text(format!(
                    "Main Opaque: {:.2} ms",
                    params.gpu_pass_timings.main_opaque_pass_ms
                ));
                card.text(format!(
                    "Post Process: {:.2} ms",
                    params.gpu_pass_timings.post_process_pass_ms
                ));
                card.text(format!(
                    "Total GPU: {:.2} ms",
                    params.gpu_pass_timings.total_gpu_ms
                ));
            });

            // 4. Scene Geometry Card
            col.card("📐 Scene & Geometry Metrics", |card| {
                card.row(|r| {
                    r.text(format!(
                        "Draw Calls: {}",
                        params.draw_call_stats.total_draw_calls
                    ));
                    r.text(format!("Entities: {}", params.active_entities_count));
                });
                card.row(|r| {
                    r.text(format!("Triangles: {}", params.render_triangles));
                    r.text(format!("Vertices: {}", params.render_vertices));
                });
            });

            // 5. Video RAM Card
            col.card("💾 Video RAM & Memory Allocations", |card| {
                card.text(format!(
                    "Textures: {:.1} MB | Mesh/Index: {:.1} MB",
                    params.vram_stats.texture_vram_mb, params.vram_stats.mesh_index_vram_mb,
                ));
                card.text(format!(
                    "Dynamic Uniform: {:.1} MB | Total: {:.1} MB",
                    params.vram_stats.dynamic_uniform_vram_mb, params.vram_stats.total_vram_mb,
                ));
            });

            // 6. Viewport Overlays Card
            col.card("🎛 Viewport Overlays", |card| {
                card.row(|r| {
                    let wire_txt = if params.wireframe_enabled {
                        "☑ Wireframe"
                    } else {
                        "☐ Wireframe"
                    };
                    r.button(wire_txt, 1001);
                    let grid_txt = if params.grid_enabled {
                        "☑ Grid"
                    } else {
                        "☐ Grid"
                    };
                    r.button(grid_txt, 1002);
                });
            });
        },
    );

    if let Some(node) = tree.get_mut(root_id) {
        node.computed_rect = params.panel_rect;
    }

    StatsPanelNodes {
        root_id,
        ..Default::default()
    }
}

/// No-op in declarative UI as values are automatically rebuilt dynamically.
pub fn update_stats_panel_values(
    _tree: &mut UiTree,
    _nodes: &StatsPanelNodes,
    _params: &StatsPanelParams<'_>,
    _targets: &StatsPanelTargets,
) {
}