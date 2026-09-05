// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 3D Model Interactive Wireframe Orbit Viewport Builder
//!
//! Renders an interactive 3D bounding box wireframe canvas with projected 3D coordinates,
//! orbital yaw and pitch rotation, zoom distance scaling, and 3D coordinate axes.
//!

use crate::ui::iris_bridge::assets::types::AssetPreviewModalState;
use irisui::prelude::*;

/// Renders the 3D model inspection preview with interactive wireframe orbit canvas.
pub(crate) fn render_model_preview_content(
    tree: &mut UiTree,
    parent_id: WidgetId,
    body_x: f32,
    body_y: f32,
    body_w: f32,
    modal: &AssetPreviewModalState,
    cursor_pos: Point,
) -> (Rect, Rect) {
    // 1. Controls Bar: Instruction Hint + Spawn Action Button
    let controls_h = 28.0;
    let hint_rect = Rect::new(body_x, body_y, body_w - 160.0, controls_h);
    let hint_id = tree.create_node();
    if let Some(node) = tree.get_mut(hint_id) {
        node.set_name("ModelPreviewHint");
        node.set_text("Left Drag: 3D Orbit | Wheel: Zoom Distance");
        node.font_size = 11.0;
        node.line_height = controls_h;
        node.text_color = Color::rgba(0.55, 0.60, 0.72, 1.0);
        node.computed_rect = hint_rect;
    }
    let _ = tree.add_child(parent_id, hint_id);

    let spawn_rect = Rect::new(body_x + body_w - 150.0, body_y, 150.0, controls_h);
    let is_spawn_hovered = spawn_rect.contains_point(cursor_pos);
    let spawn_id = tree.create_node();
    if let Some(node) = tree.get_mut(spawn_id) {
        node.set_name("PreviewSpawnBtn");
        node.set_text("Spawn into Scene");
        node.font_size = 11.5;
        node.line_height = controls_h;
        node.text_align = TextAlign::Center;
        node.text_color = Color::WHITE;
        node.computed_rect = spawn_rect;
        node.style = Style::new()
            .background(if is_spawn_hovered {
                Color::rgba(0.0, 0.45, 0.60, 1.0)
            } else {
                Color::rgba(0.0, 0.35, 0.48, 0.90)
            })
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.0, 0.90, 1.0, 0.80));
    }
    let _ = tree.add_child(parent_id, spawn_id);

    // 2. Interactive 3D Wireframe Viewport Canvas (Height: 280 px)
    let canvas_y = body_y + controls_h + 6.0;
    let canvas_h = 280.0;
    let canvas_rect = Rect::new(body_x, canvas_y, body_w, canvas_h);

    let canvas_id = tree.create_node();
    if let Some(node) = tree.get_mut(canvas_id) {
        node.set_name("ModelOrbitCanvas");
        node.computed_rect = canvas_rect;
        node.style = Style::new()
            .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
            .border_radius(6.0)
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.60))
            .clip_children(true);
    }
    let _ = tree.add_child(parent_id, canvas_id);

    // Render projected 3D bounding box corners and coordinate axes
    let cx = canvas_rect.x + canvas_rect.width * 0.5;
    let cy = canvas_rect.y + canvas_rect.height * 0.5;
    let scale = (85.0 / modal.zoom_distance).clamp(25.0, 180.0);

    // 8 Box corners in local 3D unit coordinates: `[-1..1, -1..1, -1..1]`
    let corners_3d = [
        (-1.0, -1.0, -1.0),
        (1.0, -1.0, -1.0),
        (1.0, 1.0, -1.0),
        (-1.0, 1.0, -1.0),
        (-1.0, -1.0, 1.0),
        (1.0, -1.0, 1.0),
        (1.0, 1.0, 1.0),
        (-1.0, 1.0, 1.0),
    ];

    let center = [cx, cy];
    let mut proj_points = Vec::with_capacity(8);
    for &(x, y, z) in &corners_3d {
        let (px, py) =
            project_3d_point([x, y, z], modal.orbit_yaw, modal.orbit_pitch, center, scale);
        proj_points.push((px, py));
    }

    // 12 Wireframe Box Edges
    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0), // Bottom ring
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4), // Top ring
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7), // Vertical pillars
    ];

    for (i, &(p0_idx, p1_idx)) in edges.iter().enumerate() {
        let (x0, y0) = proj_points[p0_idx];
        let (x1, y1) = proj_points[p1_idx];

        let min_x = x0.min(x1);
        let min_y = y0.min(y1);
        let w = (x1 - x0).abs().max(1.5);
        let h = (y1 - y0).abs().max(1.5);

        let edge_rect = Rect::new(min_x, min_y, w, h);
        let edge_id = tree.create_node();
        if let Some(node) = tree.get_mut(edge_id) {
            node.set_name(format!("WireEdge_{}", i));
            node.computed_rect = edge_rect;
            node.style = Style::new()
                .background(Color::rgba(0.0, 0.85, 1.0, 0.45))
                .border_radius(0.75);
        }
        let _ = tree.add_child(canvas_id, edge_id);
    }

    // Center Coordinate Axes Indicator
    let (ox, oy) = project_3d_point(
        [0.0, 0.0, 0.0],
        modal.orbit_yaw,
        modal.orbit_pitch,
        center,
        scale,
    );
    let (xx, xy) = project_3d_point(
        [0.6, 0.0, 0.0],
        modal.orbit_yaw,
        modal.orbit_pitch,
        center,
        scale,
    );
    let (yx, yy) = project_3d_point(
        [0.0, 0.6, 0.0],
        modal.orbit_yaw,
        modal.orbit_pitch,
        center,
        scale,
    );
    let (zx, zy) = project_3d_point(
        [0.0, 0.0, 0.6],
        modal.orbit_yaw,
        modal.orbit_pitch,
        center,
        scale,
    );

    // X Axis (Red)
    let x_rect = Rect::new(
        ox.min(xx),
        oy.min(xy),
        (xx - ox).abs().max(2.0),
        (xy - oy).abs().max(2.0),
    );
    let x_id = tree.create_node();
    if let Some(node) = tree.get_mut(x_id) {
        node.set_name("AxisX");
        node.computed_rect = x_rect;
        node.style = Style::new().background(Color::rgba(1.0, 0.30, 0.30, 0.90));
    }
    let _ = tree.add_child(canvas_id, x_id);

    // Y Axis (Green)
    let y_rect = Rect::new(
        ox.min(yx),
        oy.min(yy),
        (yx - ox).abs().max(2.0),
        (yy - oy).abs().max(2.0),
    );
    let y_id = tree.create_node();
    if let Some(node) = tree.get_mut(y_id) {
        node.set_name("AxisY");
        node.computed_rect = y_rect;
        node.style = Style::new().background(Color::rgba(0.30, 1.0, 0.30, 0.90));
    }
    let _ = tree.add_child(canvas_id, y_id);

    // Z Axis (Blue)
    let z_rect = Rect::new(
        ox.min(zx),
        oy.min(zy),
        (zx - ox).abs().max(2.0),
        (zy - oy).abs().max(2.0),
    );
    let z_id = tree.create_node();
    if let Some(node) = tree.get_mut(z_id) {
        node.set_name("AxisZ");
        node.computed_rect = z_rect;
        node.style = Style::new().background(Color::rgba(0.30, 0.60, 1.0, 0.90));
    }
    let _ = tree.add_child(canvas_id, z_id);

    // 3. Model Metrics Summary Row
    let metrics_y = canvas_y + canvas_h + 6.0;
    let met_rect = Rect::new(body_x, metrics_y, body_w, 20.0);
    let met_id = tree.create_node();
    if let Some(node) = tree.get_mut(met_id) {
        node.set_name("ModelMetrics");
        node.set_text(
            "Format: 3D glTF / GLB • Mesh Pipeline: Indexed PBR • Bounding Box: Normalized [-1.0 .. 1.0]",
        );
        node.font_size = 10.5;
        node.line_height = 20.0;
        node.text_color = Color::rgba(0.60, 0.65, 0.75, 1.0);
        node.computed_rect = met_rect;
    }
    let _ = tree.add_child(parent_id, met_id);

    (canvas_rect, spawn_rect)
}

/// Helper for projecting 3D point through yaw and pitch angles into 2D viewport coordinates.
fn project_3d_point(
    pt: [f32; 3],
    yaw: f32,
    pitch: f32,
    center: [f32; 2],
    scale: f32,
) -> (f32, f32) {
    let cos_y = yaw.cos();
    let sin_y = yaw.sin();
    let x1 = pt[0] * cos_y - pt[2] * sin_y;
    let z1 = pt[0] * sin_y + pt[2] * cos_y;

    let cos_p = pitch.cos();
    let sin_p = pitch.sin();
    let y2 = pt[1] * cos_p - z1 * sin_p;
    let z2 = pt[1] * sin_p + z1 * cos_p;

    let proj_factor = scale / (1.0 + z2 * 0.18).max(0.2);
    (center[0] + x1 * proj_factor, center[1] - y2 * proj_factor)
}