// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Geometry, Draw Call & Video RAM Metrics Builder
//!
//! Renders the geometry metrics card and VRAM allocation distribution card in retained mode.

use super::cpu_breakdown::{build_multi_segment_bar, update_multi_segment_bar};
use super::types::StatsPanelParams;
use irisui::prelude::*;

/// Node handles for the Scene & Geometry Metrics card in retained mode.
pub struct SceneGeometryNodes {
    /// Draw calls value node ID.
    pub dc_val_id: WidgetId,
    /// Instanced ratio value node ID.
    pub inst_pct_id: WidgetId,
    /// 4 Draw call subrow value node IDs (Batched, Instanced, Compute, Culled).
    pub dc_subrow_val_ids: [WidgetId; 4],
    /// Triangles rendered value node ID.
    pub triangles_val_id: WidgetId,
    /// Vertices rendered value node ID.
    pub vertices_val_id: WidgetId,
    /// Scene entities value node ID.
    pub entities_val_id: WidgetId,
}

/// Node handles for the Video RAM & Memory Allocations card in retained mode.
pub struct VramBreakdownNodes {
    /// 3 VRAM bar segment node IDs.
    pub bar_seg_ids: [WidgetId; 3],
    /// Bounding rectangle of the VRAM bar.
    pub bar_rect: Rect,
    /// 3 VRAM row value node IDs.
    pub row_val_ids: [WidgetId; 3],
    /// Total allocated VRAM value node ID.
    pub total_val_id: WidgetId,
}

/// Builds the static Scene & Geometry Metrics card layout.
pub fn build_scene_geometry_content(
    tree: &mut UiTree,
    parent_id: WidgetId,
    content_rect: Rect,
) -> SceneGeometryNodes {
    let mut cur_y = content_rect.y;

    // 1. Draw Calls Header Row
    let dc_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(dc_lbl_id) {
        node.set_name("DrawCallsLabel");
        node.set_text("Draw Calls:");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.86, 0.88, 0.92, 1.0);
        node.computed_rect = Rect::new(content_rect.x, cur_y, 70.0, 16.0);
    }
    let _ = tree.add_child(parent_id, dc_lbl_id);

    let dc_val_id = tree.create_node();
    if let Some(node) = tree.get_mut(dc_val_id) {
        node.set_name("DrawCallsValue");
        node.set_text("0");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.0, 0.82, 0.63, 1.0);
        node.computed_rect = Rect::new(content_rect.x + 72.0, cur_y, 40.0, 16.0);
    }
    let _ = tree.add_child(parent_id, dc_val_id);

    let inst_pct_id = tree.create_node();
    if let Some(node) = tree.get_mut(inst_pct_id) {
        node.set_name("InstancedRatioText");
        node.set_text("100% Instanced");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_align = TextAlign::Right;
        node.text_color = Color::rgba(0.55, 0.57, 0.63, 1.0);
        node.computed_rect = Rect::new(
            content_rect.x + 112.0,
            cur_y,
            content_rect.width - 112.0,
            16.0,
        );
    }
    let _ = tree.add_child(parent_id, inst_pct_id);
    cur_y += 18.0;

    // 2. Draw Call Subrows
    let dc_rows = [
        ("Batched Primitives", Color::rgba(0.0, 0.75, 0.90, 1.0)),
        ("Hardware Instanced", Color::rgba(0.39, 0.86, 0.39, 1.0)),
        ("Dispatched Compute", Color::rgba(0.94, 0.71, 0.20, 1.0)),
        ("Culled / Occluded", Color::rgba(0.71, 0.55, 0.86, 1.0)),
    ];

    let mut dc_subrow_val_ids = [WidgetId::default(); 4];
    for (idx, (name, col)) in dc_rows.into_iter().enumerate() {
        dc_subrow_val_ids[idx] = build_metric_row(
            tree,
            parent_id,
            name,
            col,
            Rect::new(content_rect.x, cur_y, content_rect.width, 16.0),
        );
        cur_y += 18.0;
    }

    // 3. Triangles & Vertices & Entities
    let triangles_val_id = build_metric_row(
        tree,
        parent_id,
        "Triangles Rendered",
        Color::rgba(0.78, 0.90, 1.0, 1.0),
        Rect::new(content_rect.x, cur_y, content_rect.width, 16.0),
    );
    cur_y += 18.0;

    let vertices_val_id = build_metric_row(
        tree,
        parent_id,
        "Vertices Rendered",
        Color::rgba(0.78, 0.90, 1.0, 1.0),
        Rect::new(content_rect.x, cur_y, content_rect.width, 16.0),
    );
    cur_y += 18.0;

    let entities_val_id = build_metric_row(
        tree,
        parent_id,
        "Scene Entities",
        Color::rgba(0.47, 0.55, 0.71, 1.0),
        Rect::new(content_rect.x, cur_y, content_rect.width, 16.0),
    );

    SceneGeometryNodes {
        dc_val_id,
        inst_pct_id,
        dc_subrow_val_ids,
        triangles_val_id,
        vertices_val_id,
        entities_val_id,
    }
}

/// Updates the dynamic values of the Scene & Geometry card in place (0 allocations).
pub fn update_scene_geometry_values(
    tree: &mut UiTree,
    nodes: &SceneGeometryNodes,
    params: &StatsPanelParams<'_>,
) {
    let dc = params.draw_call_stats;

    // 1. Draw Calls Count and Instancing Ratio
    if let Some(node) = tree.get_mut(nodes.dc_val_id) {
        node.set_text(format!("{}", dc.total_draw_calls));
    }
    if let Some(node) = tree.get_mut(nodes.inst_pct_id) {
        node.set_text(format!("{:.0}% Instanced", dc.instancing_ratio_percent()));
    }

    // 2. Draw Call Subrows
    let vals = [
        dc.batched_draw_calls,
        dc.instanced_draw_calls,
        dc.dispatched_compute,
        dc.culled_meshes,
    ];
    for (idx, &val) in vals.iter().enumerate() {
        if let Some(node) = tree.get_mut(nodes.dc_subrow_val_ids[idx]) {
            node.set_text(format!("{}", val));
        }
    }

    // 3. Triangles, Vertices, Entities
    if let Some(node) = tree.get_mut(nodes.triangles_val_id) {
        node.set_text(format!("{}", params.render_triangles));
    }
    if let Some(node) = tree.get_mut(nodes.vertices_val_id) {
        node.set_text(format!("{}", params.render_vertices));
    }
    if let Some(node) = tree.get_mut(nodes.entities_val_id) {
        let selected_suffix = if params.selected_entity.is_some() {
            " (1 selected)"
        } else {
            ""
        };
        node.set_text(format!(
            "{}{}",
            params.active_entities_count, selected_suffix
        ));
    }
}

/// Builds the static Video RAM & Memory Allocations card layout.
pub fn build_vram_breakdown_content(
    tree: &mut UiTree,
    parent_id: WidgetId,
    content_rect: Rect,
) -> VramBreakdownNodes {
    let mut cur_y = content_rect.y;

    // 1. Multi-segmented VRAM Bar
    let bar_rect = Rect::new(content_rect.x, cur_y, content_rect.width, 4.0);
    let bar_seg_ids = build_multi_segment_bar(
        tree,
        parent_id,
        3,
        &[
            Color::rgba(0.0, 0.75, 0.90, 1.0),  // Textures (Cyan)
            Color::rgba(0.39, 0.86, 0.39, 1.0), // Mesh (Green)
            Color::rgba(0.94, 0.71, 0.20, 1.0), // Uniforms (Gold)
        ],
        bar_rect,
    );
    cur_y += 10.0;

    // 2. VRAM Rows
    let rows = [
        ("Texture VRAM (2D/3D)", Color::rgba(0.0, 0.75, 0.90, 1.0)),
        ("Mesh & Index VRAM", Color::rgba(0.39, 0.86, 0.39, 1.0)),
        ("Uniform & Target VRAM", Color::rgba(0.94, 0.71, 0.20, 1.0)),
    ];

    let mut row_val_ids = [WidgetId::default(); 3];
    for (idx, (name, col)) in rows.into_iter().enumerate() {
        row_val_ids[idx] = build_metric_row(
            tree,
            parent_id,
            name,
            col,
            Rect::new(content_rect.x, cur_y, content_rect.width, 16.0),
        );
        cur_y += 18.0;
    }

    // 3. Total Allocated VRAM Footer
    let tot_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(tot_lbl_id) {
        node.set_name("TotalVramLabel");
        node.set_text("Total Allocated VRAM:");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.86, 0.88, 0.92, 1.0);
        node.computed_rect = Rect::new(content_rect.x, cur_y, 140.0, 16.0);
    }
    let _ = tree.add_child(parent_id, tot_lbl_id);

    let tot_val_id = tree.create_node();
    if let Some(node) = tree.get_mut(tot_val_id) {
        node.set_name("TotalVramValue");
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

    let mut bar_seg_arr = [WidgetId::default(); 3];
    bar_seg_arr.copy_from_slice(&bar_seg_ids[0..3]);

    VramBreakdownNodes {
        bar_seg_ids: bar_seg_arr,
        bar_rect,
        row_val_ids,
        total_val_id: tot_val_id,
    }
}

/// Updates the dynamic values of the Video RAM card in place (0 allocations).
pub fn update_vram_values(
    tree: &mut UiTree,
    nodes: &VramBreakdownNodes,
    params: &StatsPanelParams<'_>,
) {
    let vram = params.vram_stats;

    // 1. Update Progress Bar Segments
    let bar_total = vram.total_vram_mb.max(0.001);
    let ratios = [
        vram.texture_vram_mb / bar_total,
        vram.mesh_index_vram_mb / bar_total,
        vram.dynamic_uniform_vram_mb / bar_total,
    ];
    update_multi_segment_bar(tree, &nodes.bar_seg_ids, &ratios, nodes.bar_rect);

    // 2. Update VRAM Rows
    let mb_values = [
        vram.texture_vram_mb,
        vram.mesh_index_vram_mb,
        vram.dynamic_uniform_vram_mb,
    ];
    for (idx, &mb) in mb_values.iter().enumerate() {
        if let Some(node) = tree.get_mut(nodes.row_val_ids[idx]) {
            node.set_text(format!("{:.2} MB", mb));
        }
    }

    // 3. Update Total Allocated VRAM
    if let Some(node) = tree.get_mut(nodes.total_val_id) {
        node.set_text(format!("{:.2} MB", vram.total_vram_mb));
    }
}

/// Helper to render a static metric row and return the value node ID.
fn build_metric_row(
    tree: &mut UiTree,
    parent_id: WidgetId,
    name: &str,
    color: Color,
    rect: Rect,
) -> WidgetId {
    // 1. Colored Dot
    let dot_id = tree.create_node();
    if let Some(node) = tree.get_mut(dot_id) {
        node.set_name("MetricDot");
        node.computed_rect = Rect::new(rect.x + 2.0, rect.y + 5.0, 6.0, 6.0);
        node.style = Style::new().background(color).border_radius(3.0);
    }
    let _ = tree.add_child(parent_id, dot_id);

    // 2. Name
    let name_w = (rect.width - 110.0).max(60.0);
    let name_id = tree.create_node();
    if let Some(node) = tree.get_mut(name_id) {
        node.set_name("MetricRowName");
        node.set_text(name);
        node.font_size = 11.0;
        node.line_height = rect.height;
        node.text_color = Color::rgba(0.80, 0.83, 0.90, 1.0);
        node.computed_rect = Rect::new(rect.x + 14.0, rect.y, name_w, rect.height);
    }
    let _ = tree.add_child(parent_id, name_id);

    // 3. Value
    let val_w = 96.0;
    let val_id = tree.create_node();
    if let Some(node) = tree.get_mut(val_id) {
        node.set_name("MetricRowValue");
        node.set_text("-");
        node.font_size = 11.0;
        node.line_height = rect.height;
        node.text_align = TextAlign::Right;
        node.text_color = color;
        node.computed_rect = Rect::new(rect.x + rect.width - val_w, rect.y, val_w, rect.height);
    }
    let _ = tree.add_child(parent_id, val_id);

    val_id
}