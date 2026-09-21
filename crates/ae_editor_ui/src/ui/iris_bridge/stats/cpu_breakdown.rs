// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # CPU Thread Synchronization Breakdown Builder
//!
//! Renders the CPU multi-segmented subsystem timing bar, detailed subsystem timings,
//! and thread balance bottleneck indicator in retained mode.

use super::types::StatsPanelParams;
use irisui::prelude::*;

/// Node handles for the CPU breakdown card in retained mode.
pub struct CpuBreakdownNodes {
    /// Thread Balance value node ID.
    pub tb_val_id: WidgetId,
    /// 5 CPU bar segment node IDs.
    pub bar_seg_ids: [WidgetId; 5],
    /// Bounding rectangle of the CPU bar.
    pub bar_rect: Rect,
    /// 5 Subsystem timing value node IDs.
    pub timing_val_ids: [WidgetId; 5],
    /// Total CPU frame value node ID.
    pub total_val_id: WidgetId,
}

/// Builds the static CPU Thread & Synchronization breakdown card layout.
pub fn build_cpu_breakdown_content(
    tree: &mut UiTree,
    parent_id: WidgetId,
    content_rect: Rect,
) -> CpuBreakdownNodes {
    let mut cur_y = content_rect.y;

    // 1. Thread Balance Status Row
    let tb_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(tb_lbl_id) {
        node.set_name("ThreadBalanceLabel");
        node.set_text("Thread Balance:");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.59, 0.61, 0.67, 1.0);
        node.computed_rect = Rect::new(content_rect.x, cur_y, 90.0, 16.0);
    }
    let _ = tree.add_child(parent_id, tb_lbl_id);

    let tb_val_id = tree.create_node();
    if let Some(node) = tree.get_mut(tb_val_id) {
        node.set_name("ThreadBalanceValue");
        node.set_text("Optimal (120+ FPS)");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.0, 0.82, 0.63, 1.0);
        node.computed_rect = Rect::new(
            content_rect.x + 92.0,
            cur_y,
            content_rect.width - 92.0,
            16.0,
        );
    }
    let _ = tree.add_child(parent_id, tb_val_id);
    cur_y += 18.0;

    // 2. Multi-segmented composite bar
    let bar_rect = Rect::new(content_rect.x, cur_y, content_rect.width, 4.0);
    let bar_seg_ids = build_multi_segment_bar(
        tree,
        parent_id,
        5,
        &[
            Color::rgba(0.0, 0.75, 0.90, 1.0),  // ECS (Cyan)
            Color::rgba(0.96, 0.57, 0.12, 1.0), // Physics (Orange)
            Color::rgba(0.35, 0.51, 0.94, 1.0), // Render Prep (Blue)
            Color::rgba(0.63, 0.39, 0.86, 1.0), // Wait for GPU (Purple)
            Color::rgba(0.86, 0.71, 0.20, 1.0), // UI (Yellow)
        ],
        bar_rect,
    );
    cur_y += 10.0;

    // 3. Subsystem Timing Rows
    let rows = [
        ("ECS / Logic", Color::rgba(0.0, 0.75, 0.90, 1.0)),
        ("Physics Simulation", Color::rgba(0.96, 0.57, 0.12, 1.0)),
        ("Render Preparation", Color::rgba(0.35, 0.51, 0.94, 1.0)),
        ("Wait for GPU (VSync)", Color::rgba(0.63, 0.39, 0.86, 1.0)),
        ("UI / Editor Passes", Color::rgba(0.86, 0.71, 0.20, 1.0)),
    ];

    let mut timing_val_ids = [WidgetId::default(); 5];
    for (idx, (name, col)) in rows.into_iter().enumerate() {
        timing_val_ids[idx] = build_timing_row(
            tree,
            parent_id,
            name,
            col,
            Rect::new(content_rect.x, cur_y, content_rect.width, 16.0),
        );
        cur_y += 18.0;
    }

    // 4. Total CPU Frame Footer
    let tot_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(tot_lbl_id) {
        node.set_name("TotalCpuLabel");
        node.set_text("Total CPU Frame:");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.86, 0.88, 0.92, 1.0);
        node.computed_rect = Rect::new(content_rect.x, cur_y, 120.0, 16.0);
    }
    let _ = tree.add_child(parent_id, tot_lbl_id);

    let tot_val_id = tree.create_node();
    if let Some(node) = tree.get_mut(tot_val_id) {
        node.set_name("TotalCpuValue");
        node.set_text("-");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_align = TextAlign::Right;
        node.text_color = Color::rgba(0.78, 0.90, 1.0, 1.0);
        node.computed_rect = Rect::new(
            content_rect.x + 120.0,
            cur_y,
            content_rect.width - 120.0,
            16.0,
        );
    }
    let _ = tree.add_child(parent_id, tot_val_id);

    let mut bar_seg_arr = [WidgetId::default(); 5];
    bar_seg_arr.copy_from_slice(&bar_seg_ids[0..5]);

    CpuBreakdownNodes {
        tb_val_id,
        bar_seg_ids: bar_seg_arr,
        bar_rect,
        timing_val_ids,
        total_val_id: tot_val_id,
    }
}

/// Updates the dynamic values of the CPU Breakdown card in place (0 allocations).
pub fn update_cpu_breakdown_values(
    tree: &mut UiTree,
    nodes: &CpuBreakdownNodes,
    params: &StatsPanelParams<'_>,
) {
    let cpu = params.cpu_timings;
    let total_gpu_ms = params.gpu_pass_timings.total_gpu_ms;
    let total_frame_ms = cpu.total_cpu_ms.max(total_gpu_ms);
    let is_cpu_bound = cpu.is_cpu_bound(total_gpu_ms);

    let (bottleneck_text, bottleneck_color) = if total_frame_ms <= 8.33 {
        ("Optimal (120+ FPS)", Color::rgba(0.0, 0.82, 0.63, 1.0))
    } else if total_frame_ms <= 16.67 {
        ("Within Budget (60+ FPS)", Color::rgba(0.0, 0.75, 0.90, 1.0))
    } else if is_cpu_bound {
        ("CPU Bound", Color::rgba(0.96, 0.57, 0.12, 1.0))
    } else {
        ("GPU Bound / VSync", Color::rgba(0.94, 0.39, 0.31, 1.0))
    };

    // 1. Update Thread Balance Status
    if let Some(node) = tree.get_mut(nodes.tb_val_id) {
        node.set_text(bottleneck_text);
        node.text_color = bottleneck_color;
    }

    // 2. Update Progress Bar Segments
    let bar_total = cpu.total_cpu_ms.max(0.001);
    let ratios = [
        cpu.main_logic_ms / bar_total,
        cpu.physics_ms / bar_total,
        cpu.render_prep_ms / bar_total,
        cpu.wait_for_gpu_ms / bar_total,
        cpu.ui_editor_ms / bar_total,
    ];
    update_multi_segment_bar(tree, &nodes.bar_seg_ids, &ratios, nodes.bar_rect);

    // 3. Update Timing Rows
    let ms_values = [
        cpu.main_logic_ms,
        cpu.physics_ms,
        cpu.render_prep_ms,
        cpu.wait_for_gpu_ms,
        cpu.ui_editor_ms,
    ];

    for (idx, &ms) in ms_values.iter().enumerate() {
        let pct = if bar_total > 0.001 {
            (ms / bar_total) * 100.0
        } else {
            0.0
        };
        let val_text = format!("{:.2} ms ({:.0}%)", ms, pct);
        if let Some(node) = tree.get_mut(nodes.timing_val_ids[idx]) {
            node.set_text(val_text);
        }
    }

    // 4. Update Total CPU Frame Footer
    if let Some(node) = tree.get_mut(nodes.total_val_id) {
        node.set_text(format!("{:.2} ms", cpu.total_cpu_ms));
    }
}

/// Helper to build a multi-segmented progress bar track and pre-allocate segment nodes.
pub fn build_multi_segment_bar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    segment_count: usize,
    colors: &[Color],
    rect: Rect,
) -> Vec<WidgetId> {
    let bg_id = tree.create_node();
    if let Some(node) = tree.get_mut(bg_id) {
        node.set_name("MultiSegmentTrack");
        node.computed_rect = rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.09, 0.12, 1.0))
            .border_radius(2.0);
    }
    let _ = tree.add_child(parent_id, bg_id);

    let mut seg_ids = Vec::with_capacity(segment_count);
    for idx in 0..segment_count {
        let col = colors.get(idx).copied().unwrap_or(Color::WHITE);
        let seg_id = tree.create_node();
        if let Some(node) = tree.get_mut(seg_id) {
            node.set_name(format!("SegmentFill_{}", idx));
            node.computed_rect = Rect::ZERO;
            node.style = Style::new().background(col);
        }
        let _ = tree.add_child(bg_id, seg_id);
        seg_ids.push(seg_id);
    }

    seg_ids
}

/// Helper to update segment rects without any node allocations.
pub fn update_multi_segment_bar(
    tree: &mut UiTree,
    seg_ids: &[WidgetId],
    ratios: &[f32],
    rect: Rect,
) {
    let mut seg_x = rect.x;
    for (idx, &seg_id) in seg_ids.iter().enumerate() {
        let ratio = ratios.get(idx).copied().unwrap_or(0.0);
        if ratio > 0.001 {
            let seg_w = (rect.width * ratio.clamp(0.0, 1.0)).min(rect.x + rect.width - seg_x);
            if seg_w > 0.5 {
                if let Some(node) = tree.get_mut(seg_id) {
                    node.computed_rect = Rect::new(seg_x, rect.y, seg_w, rect.height);
                    node.visible = true;
                }
                seg_x += seg_w;
                continue;
            }
        }
        if let Some(node) = tree.get_mut(seg_id) {
            node.computed_rect = Rect::ZERO;
            node.visible = false;
        }
    }
}

/// Helper to render a static subsystem timing row and return value node ID.
pub fn build_timing_row(
    tree: &mut UiTree,
    parent_id: WidgetId,
    name: &str,
    color: Color,
    rect: Rect,
) -> WidgetId {
    // 1. Colored Dot
    let dot_id = tree.create_node();
    if let Some(node) = tree.get_mut(dot_id) {
        node.set_name("TimingDot");
        node.computed_rect = Rect::new(rect.x + 2.0, rect.y + 5.0, 6.0, 6.0);
        node.style = Style::new().background(color).border_radius(3.0);
    }
    let _ = tree.add_child(parent_id, dot_id);

    // 2. Subsystem Name
    let name_w = (rect.width - 110.0).max(60.0);
    let name_id = tree.create_node();
    if let Some(node) = tree.get_mut(name_id) {
        node.set_name("TimingName");
        node.set_text(name);
        node.font_size = 11.0;
        node.line_height = rect.height;
        node.text_color = Color::rgba(0.80, 0.83, 0.90, 1.0);
        node.computed_rect = Rect::new(rect.x + 14.0, rect.y, name_w, rect.height);
    }
    let _ = tree.add_child(parent_id, name_id);

    // 3. Timing Value (ms and %)
    let val_w = 96.0;
    let val_id = tree.create_node();
    if let Some(node) = tree.get_mut(val_id) {
        node.set_name("TimingValue");
        node.set_text("-");
        node.font_size = 11.0;
        node.line_height = rect.height;
        node.text_align = TextAlign::Right;
        node.text_color = Color::rgba(0.70, 0.73, 0.80, 1.0);
        node.computed_rect = Rect::new(rect.x + rect.width - val_w, rect.y, val_w, rect.height);
    }
    let _ = tree.add_child(parent_id, val_id);

    val_id
}