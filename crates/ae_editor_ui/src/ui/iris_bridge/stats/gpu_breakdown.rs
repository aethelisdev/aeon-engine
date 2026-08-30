// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # GPU Render Passes Breakdown Builder
//!
//! Renders the GPU multi-segmented pass execution bar, hardware adapter name,
//! and pass durations in retained mode.

use super::cpu_breakdown::{build_multi_segment_bar, build_timing_row, update_multi_segment_bar};
use super::types::StatsPanelParams;
use irisui::prelude::*;

/// Node handles for the GPU breakdown card in retained mode.
pub struct GpuBreakdownNodes {
    /// GPU Device Name node ID.
    pub dev_id: WidgetId,
    /// 4 GPU bar segment node IDs.
    pub bar_seg_ids: [WidgetId; 4],
    /// Bounding rectangle of the GPU bar.
    pub bar_rect: Rect,
    /// 4 GPU Pass timing value node IDs.
    pub pass_val_ids: [WidgetId; 4],
    /// Total GPU workload value node ID.
    pub total_val_id: WidgetId,
}

/// Builds the static GPU Render Passes breakdown card layout.
pub fn build_gpu_breakdown_content(
    tree: &mut UiTree,
    parent_id: WidgetId,
    content_rect: Rect,
) -> GpuBreakdownNodes {
    let mut cur_y = content_rect.y;

    // 1. GPU Adapter Device Name & Backend Badge
    let dev_id = tree.create_node();
    if let Some(node) = tree.get_mut(dev_id) {
        node.set_name("GpuDeviceText");
        node.set_text("Graphics Adapter");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.78, 0.82, 0.88, 1.0);
        node.computed_rect = Rect::new(content_rect.x, cur_y, content_rect.width, 16.0);
    }
    let _ = tree.add_child(parent_id, dev_id);
    cur_y += 18.0;

    // 2. Multi-segmented composite bar
    let bar_rect = Rect::new(content_rect.x, cur_y, content_rect.width, 4.0);
    let bar_seg_ids = build_multi_segment_bar(
        tree,
        parent_id,
        4,
        &[
            Color::rgba(0.96, 0.57, 0.12, 1.0), // Shadow (Orange)
            Color::rgba(0.35, 0.51, 0.94, 1.0), // Forward Pass (Blue)
            Color::rgba(0.86, 0.35, 0.67, 1.0), // Post-Process (Magenta)
            Color::rgba(0.86, 0.71, 0.20, 1.0), // UI Composite (Gold)
        ],
        bar_rect,
    );
    cur_y += 10.0;

    // 3. GPU Pass Timing Rows
    let rows = [
        ("Shadow Pass (Cascades)", Color::rgba(0.96, 0.57, 0.12, 1.0)),
        ("Main Forward Pass", Color::rgba(0.35, 0.51, 0.94, 1.0)),
        ("Post-Process & Outline", Color::rgba(0.86, 0.35, 0.67, 1.0)),
        ("UI Composite Pass", Color::rgba(0.86, 0.71, 0.20, 1.0)),
    ];

    let mut pass_val_ids = [WidgetId::default(); 4];
    for (idx, (name, col)) in rows.into_iter().enumerate() {
        pass_val_ids[idx] = build_timing_row(
            tree,
            parent_id,
            name,
            col,
            Rect::new(content_rect.x, cur_y, content_rect.width, 16.0),
        );
        cur_y += 18.0;
    }

    // 4. Total GPU Workload Footer
    let tot_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(tot_lbl_id) {
        node.set_name("TotalGpuLabel");
        node.set_text("Total GPU Workload:");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.86, 0.88, 0.92, 1.0);
        node.computed_rect = Rect::new(content_rect.x, cur_y, 140.0, 16.0);
    }
    let _ = tree.add_child(parent_id, tot_lbl_id);

    let tot_val_id = tree.create_node();
    if let Some(node) = tree.get_mut(tot_val_id) {
        node.set_name("TotalGpuValue");
        node.set_text("-");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_align = TextAlign::Right;
        node.text_color = Color::rgba(0.78, 0.90, 1.0, 1.0);
        node.computed_rect = Rect::new(
            content_rect.x + 140.0,
            cur_y,
            content_rect.width - 140.0,
            16.0,
        );
    }
    let _ = tree.add_child(parent_id, tot_val_id);

    let mut bar_seg_arr = [WidgetId::default(); 4];
    bar_seg_arr.copy_from_slice(&bar_seg_ids[0..4]);

    GpuBreakdownNodes {
        dev_id,
        bar_seg_ids: bar_seg_arr,
        bar_rect,
        pass_val_ids,
        total_val_id: tot_val_id,
    }
}

/// Updates the dynamic values of the GPU Breakdown card in place (0 allocations).
pub fn update_gpu_breakdown_values(
    tree: &mut UiTree,
    nodes: &GpuBreakdownNodes,
    params: &StatsPanelParams<'_>,
) {
    let gpu = params.gpu_pass_timings;

    // 1. Update Device Name
    let gpu_text = if !params.gpu_adapter_name.is_empty() {
        format!("{} ({})", params.gpu_adapter_name, params.gpu_backend)
    } else {
        format!("Graphics Adapter ({})", params.gpu_backend)
    };
    if let Some(node) = tree.get_mut(nodes.dev_id) {
        node.set_text(gpu_text);
    }

    // 2. Update Progress Bar Segments
    let bar_total = gpu.total_gpu_ms.max(0.001);
    let ratios = [
        gpu.shadow_pass_ms / bar_total,
        gpu.main_opaque_pass_ms / bar_total,
        gpu.post_process_pass_ms / bar_total,
        gpu.ui_pass_ms / bar_total,
    ];
    update_multi_segment_bar(tree, &nodes.bar_seg_ids, &ratios, nodes.bar_rect);

    // 3. Update Pass Timing Rows
    let ms_values = [
        gpu.shadow_pass_ms,
        gpu.main_opaque_pass_ms,
        gpu.post_process_pass_ms,
        gpu.ui_pass_ms,
    ];

    for (idx, &ms) in ms_values.iter().enumerate() {
        let pct = if bar_total > 0.001 {
            (ms / bar_total) * 100.0
        } else {
            0.0
        };
        let val_text = format!("{:.2} ms ({:.0}%)", ms, pct);
        if let Some(node) = tree.get_mut(nodes.pass_val_ids[idx]) {
            node.set_text(val_text);
        }
    }

    // 4. Update Total GPU Workload
    if let Some(node) = tree.get_mut(nodes.total_val_id) {
        node.set_text(format!("{:.2} ms", gpu.total_gpu_ms));
    }
}