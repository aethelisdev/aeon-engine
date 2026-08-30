// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Profiler Iris UI Module
//!
//! Orchestrates the retained-mode frame pacing oscillograph, CPU/GPU pass breakdowns,
//! draw call distribution, granular VRAM memory cards, and viewport overlay toggles.

pub mod cpu_breakdown;
pub mod gpu_breakdown;
pub mod graph;
pub mod metrics;
pub mod overlays;
pub mod types;

pub use graph::append_oscilloscope_quads;
pub use types::{StatsPanelAction, StatsPanelNodes, StatsPanelParams, StatsPanelTargets};

use cpu_breakdown::{CpuBreakdownNodes, build_cpu_breakdown_content, update_cpu_breakdown_values};
use gpu_breakdown::{GpuBreakdownNodes, build_gpu_breakdown_content, update_gpu_breakdown_values};
use graph::{FramePacingNodes, build_frame_pacing_content, update_frame_pacing_values};
use irisui::prelude::*;
use metrics::{
    SceneGeometryNodes, VramBreakdownNodes, build_scene_geometry_content,
    build_vram_breakdown_content, update_scene_geometry_values, update_vram_values,
};
use overlays::{OverlaysNodes, build_viewport_overlays_content, update_viewport_overlays_values};

/// Builds the static Stats & Profiler panel layout tree once and returns persistent node handles.
pub fn build_stats_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &StatsPanelParams<'_>,
    targets: &mut StatsPanelTargets,
) -> StatsPanelNodes {
    targets.panel_rect = params.panel_rect;

    // Panel Base Container
    let root_id = tree.create_node();
    if let Some(node) = tree.get_mut(root_id) {
        node.set_name("StatsPanelRoot");
        node.computed_rect = params.panel_rect;
        node.style = Style::new().background(Color::rgba(0.06, 0.07, 0.09, 1.0));
    }
    let _ = tree.add_child(parent_id, root_id);

    let card_padding_x = 6.0;
    let card_w = (params.panel_rect.width - card_padding_x * 2.0).max(180.0);
    let card_x = params.panel_rect.x + card_padding_x;
    let mut cur_y = params.panel_rect.y + 6.0 - params.scroll_y;

    // ── 1. Frame Pacing & Stutter Analyzer Card ──
    let card1_h = 210.0;
    let (c1_id, c1_content) = build_stats_card(
        tree,
        root_id,
        "📈",
        "Frame Pacing & Stutter Analyzer",
        Rect::new(card_x, cur_y, card_w, card1_h),
    );
    let FramePacingNodes {
        pill_val_ids,
        footer_id: pacing_footer_id,
        canvas_rect,
    } = build_frame_pacing_content(tree, c1_id, c1_content);
    cur_y += card1_h + 6.0;

    // ── 2. CPU Thread & Synchronization Card ──
    let card2_h = 178.0;
    let (c2_id, c2_content) = build_stats_card(
        tree,
        root_id,
        "⏱",
        "CPU Thread & Synchronization",
        Rect::new(card_x, cur_y, card_w, card2_h),
    );
    let CpuBreakdownNodes {
        tb_val_id: cpu_tb_val_id,
        bar_seg_ids: cpu_bar_seg_ids,
        bar_rect: cpu_bar_rect,
        timing_val_ids: cpu_timing_val_ids,
        total_val_id: cpu_total_val_id,
    } = build_cpu_breakdown_content(tree, c2_id, c2_content);
    cur_y += card2_h + 6.0;

    // ── 3. GPU Render Passes Card ──
    let card3_h = 160.0;
    let (c3_id, c3_content) = build_stats_card(
        tree,
        root_id,
        "⚡",
        "GPU Render Passes",
        Rect::new(card_x, cur_y, card_w, card3_h),
    );
    let GpuBreakdownNodes {
        dev_id: gpu_dev_id,
        bar_seg_ids: gpu_bar_seg_ids,
        bar_rect: gpu_bar_rect,
        pass_val_ids: gpu_pass_val_ids,
        total_val_id: gpu_total_val_id,
    } = build_gpu_breakdown_content(tree, c3_id, c3_content);
    cur_y += card3_h + 6.0;

    // ── 4. Scene & Geometry Metrics Card ──
    let card4_h = 182.0;
    let (c4_id, c4_content) = build_stats_card(
        tree,
        root_id,
        "📐",
        "Scene & Geometry Metrics",
        Rect::new(card_x, cur_y, card_w, card4_h),
    );
    let SceneGeometryNodes {
        dc_val_id,
        inst_pct_id,
        dc_subrow_val_ids,
        triangles_val_id,
        vertices_val_id,
        entities_val_id,
    } = build_scene_geometry_content(tree, c4_id, c4_content);
    cur_y += card4_h + 6.0;

    // ── 5. Video RAM & Memory Allocations Card ──
    let card5_h = 120.0;
    let (c5_id, c5_content) = build_stats_card(
        tree,
        root_id,
        "💾",
        "Video RAM & Memory Allocations",
        Rect::new(card_x, cur_y, card_w, card5_h),
    );
    let VramBreakdownNodes {
        bar_seg_ids: vram_bar_seg_ids,
        bar_rect: vram_bar_rect,
        row_val_ids: vram_row_val_ids,
        total_val_id: vram_total_val_id,
    } = build_vram_breakdown_content(tree, c5_id, c5_content);
    cur_y += card5_h + 6.0;

    // ── 6. Viewport Overlays Card ──
    let card6_h = 82.0;
    let (c6_id, c6_content) = build_stats_card(
        tree,
        root_id,
        "🎛",
        "Viewport Overlays",
        Rect::new(card_x, cur_y, card_w, card6_h),
    );
    let OverlaysNodes {
        wireframe_box_id,
        wireframe_check_id,
        grid_box_id,
        grid_check_id,
    } = build_viewport_overlays_content(tree, c6_id, c6_content, targets);

    StatsPanelNodes {
        root_id,
        metric_pill_val_ids: pill_val_ids,
        pacing_footer_id,
        cpu_tb_val_id,
        cpu_bar_seg_ids,
        cpu_timing_val_ids,
        cpu_total_val_id,
        gpu_dev_id,
        gpu_bar_seg_ids,
        gpu_pass_val_ids,
        gpu_total_val_id,
        dc_val_id,
        inst_pct_id,
        dc_subrow_val_ids,
        triangles_val_id,
        vertices_val_id,
        entities_val_id,
        vram_bar_seg_ids,
        vram_row_val_ids,
        vram_total_val_id,
        wireframe_box_id,
        wireframe_check_id,
        grid_box_id,
        grid_check_id,
        canvas_rect,
        cpu_bar_rect,
        gpu_bar_rect,
        vram_bar_rect,
    }
}

/// Updates the dynamic telemetry values in place with zero node allocations.
pub fn update_stats_panel_values(
    tree: &mut UiTree,
    nodes: &StatsPanelNodes,
    params: &StatsPanelParams<'_>,
    targets: &StatsPanelTargets,
) {
    // 1. Frame Pacing Card Values
    update_frame_pacing_values(
        tree,
        &nodes.metric_pill_val_ids,
        nodes.pacing_footer_id,
        params,
    );

    // 2. CPU Breakdown Values
    update_cpu_breakdown_values(
        tree,
        &CpuBreakdownNodes {
            tb_val_id: nodes.cpu_tb_val_id,
            bar_seg_ids: nodes.cpu_bar_seg_ids,
            bar_rect: nodes.cpu_bar_rect,
            timing_val_ids: nodes.cpu_timing_val_ids,
            total_val_id: nodes.cpu_total_val_id,
        },
        params,
    );

    // 3. GPU Breakdown Values
    update_gpu_breakdown_values(
        tree,
        &GpuBreakdownNodes {
            dev_id: nodes.gpu_dev_id,
            bar_seg_ids: nodes.gpu_bar_seg_ids,
            bar_rect: nodes.gpu_bar_rect,
            pass_val_ids: nodes.gpu_pass_val_ids,
            total_val_id: nodes.gpu_total_val_id,
        },
        params,
    );

    // 4. Scene Geometry Values
    update_scene_geometry_values(
        tree,
        &SceneGeometryNodes {
            dc_val_id: nodes.dc_val_id,
            inst_pct_id: nodes.inst_pct_id,
            dc_subrow_val_ids: nodes.dc_subrow_val_ids,
            triangles_val_id: nodes.triangles_val_id,
            vertices_val_id: nodes.vertices_val_id,
            entities_val_id: nodes.entities_val_id,
        },
        params,
    );

    // 5. Video RAM Values
    update_vram_values(
        tree,
        &VramBreakdownNodes {
            bar_seg_ids: nodes.vram_bar_seg_ids,
            bar_rect: nodes.vram_bar_rect,
            row_val_ids: nodes.vram_row_val_ids,
            total_val_id: nodes.vram_total_val_id,
        },
        params,
    );

    // 6. Viewport Overlays Values
    update_viewport_overlays_values(
        tree,
        &OverlaysNodes {
            wireframe_box_id: nodes.wireframe_box_id,
            wireframe_check_id: nodes.wireframe_check_id,
            grid_box_id: nodes.grid_box_id,
            grid_check_id: nodes.grid_check_id,
        },
        params,
        targets,
    );
}

/// Helper function to build a standard dark stats card frame with header and inner content rect.
fn build_stats_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    icon: &str,
    title: &str,
    rect: Rect,
) -> (WidgetId, Rect) {
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("StatsCard");
        node.computed_rect = rect;
        node.style = Style::new()
            .background(Color::rgba(0.07, 0.08, 0.10, 0.95))
            .border(1.0, Color::rgba(0.15, 0.16, 0.21, 0.90))
            .border_radius(5.0)
            .box_shadow(0.0, 4.0, 10.0, Color::rgba(0.0, 0.0, 0.0, 0.40));
    }
    let _ = tree.add_child(parent_id, card_id);

    // Card Header Row
    let head_text = format!("{} {}", icon, title);
    let head_id = tree.create_node();
    if let Some(node) = tree.get_mut(head_id) {
        node.set_name("CardHeader");
        node.set_text(head_text);
        node.font_size = 11.5;
        node.line_height = 18.0;
        node.text_color = Color::rgba(0.86, 0.88, 0.92, 1.0);
        node.computed_rect = Rect::new(rect.x + 10.0, rect.y + 8.0, rect.width - 20.0, 18.0);
    }
    let _ = tree.add_child(card_id, head_id);

    let content_rect = Rect::new(
        rect.x + 10.0,
        rect.y + 30.0,
        rect.width - 20.0,
        rect.height - 38.0,
    );

    (card_id, content_rect)
}