// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Scene Geometry, Draw Call Breakdown, and Video RAM (VRAM) Metrics Widgets.
//!
//! Provides granular telemetry for batched, instanced, and compute draw calls,
//! frustum culling effectiveness, and categorized VRAM memory consumption.
//!

use ae_core::telemetry::{DrawCallBreakdown, VramStats};
use egui::{Color32, Stroke, Vec2};

/// Parameters passed into the scene geometry metrics card renderer.
pub struct SceneMetricsParams<'a> {
    pub draw_calls: &'a DrawCallBreakdown,
    pub triangles: u64,
    pub vertices: u64,
    pub active_entities: usize,
    pub selected_entity: Option<hecs::Entity>,
}

/// Renders the detailed Scene Geometry and Draw Call Breakdown card.
pub fn draw_scene_geometry_card(ui: &mut egui::Ui, params: SceneMetricsParams<'_>) {
    // 1. Draw Call Breakdown Header
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Draw Calls:")
                .strong()
                .color(Color32::from_rgb(220, 225, 235)),
        );
        ui.label(
            egui::RichText::new(format!("{}", params.draw_calls.total_draw_calls))
                .strong()
                .color(Color32::from_rgb(0, 210, 160)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{:.0}% Instanced",
                    params.draw_calls.instancing_ratio_percent()
                ))
                .color(Color32::from_rgb(140, 145, 160))
                .font(egui::FontId::proportional(11.0)),
            );
        });
    });

    ui.add_space(3.0);

    draw_metric_subrow(
        ui,
        "Batched Primitives",
        &format!("{}", params.draw_calls.batched_draw_calls),
        Color32::from_rgb(0, 190, 230),
    );
    draw_metric_subrow(
        ui,
        "Hardware Instanced",
        &format!("{}", params.draw_calls.instanced_draw_calls),
        Color32::from_rgb(100, 220, 100),
    );
    draw_metric_subrow(
        ui,
        "Dispatched Compute",
        &format!("{}", params.draw_calls.dispatched_compute),
        Color32::from_rgb(240, 180, 50),
    );
    draw_metric_subrow(
        ui,
        "Culled / Occluded",
        &format!("{}", params.draw_calls.culled_meshes),
        Color32::from_rgb(180, 140, 220),
    );

    ui.add_space(5.0);

    // 2. Mesh & Geometry Metrics
    let mut tri_buf = core::fmt::NumBuffer::new();
    let tri_str = params.triangles.format_into(&mut tri_buf);
    draw_metric_subrow(
        ui,
        "Triangles Rendered",
        tri_str,
        Color32::from_rgb(200, 230, 255),
    );

    let mut vert_buf = core::fmt::NumBuffer::new();
    let vert_str = params.vertices.format_into(&mut vert_buf);
    draw_metric_subrow(
        ui,
        "Vertices Rendered",
        vert_str,
        Color32::from_rgb(200, 230, 255),
    );

    let mut ent_buf = core::fmt::NumBuffer::new();
    let ent_str = params.active_entities.format_into(&mut ent_buf);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("●")
                .font(egui::FontId::proportional(9.0))
                .color(Color32::from_rgb(120, 140, 180)),
        );
        ui.label(
            egui::RichText::new("Scene Entities")
                .color(Color32::from_rgb(175, 180, 195))
                .font(egui::FontId::proportional(11.0)),
        );
        let selected_str = if params.selected_entity.is_some() {
            " (1 selected)"
        } else {
            ""
        };
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{}{}", ent_str, selected_str))
                    .color(Color32::from_rgb(220, 225, 235))
                    .font(egui::FontId::proportional(11.0)),
            );
        });
    });
}

/// Renders the granular Video RAM (VRAM) and Asset Memory breakdown card.
pub fn draw_vram_breakdown_card(ui: &mut egui::Ui, vram: &VramStats) {
    let bar_total = vram.total_vram_mb.max(0.001);
    let segments = [
        (
            vram.texture_vram_mb / bar_total,
            Color32::from_rgb(0, 190, 230),
        ), // Textures (Cyan)
        (
            vram.mesh_index_vram_mb / bar_total,
            Color32::from_rgb(100, 220, 100),
        ), // Meshes (Green)
        (
            vram.dynamic_uniform_vram_mb / bar_total,
            Color32::from_rgb(240, 180, 50),
        ), // Uniforms/Targets (Yellow)
    ];

    // Multi-segment VRAM distribution bar
    let (response, painter) =
        ui.allocate_painter(Vec2::new(ui.available_width(), 7.0), egui::Sense::hover());
    let rect = response.rect;
    painter.rect_filled(rect, 3.0, Color32::from_rgb(14, 16, 22));

    let mut current_x = rect.min.x;
    for (fraction, color) in &segments {
        if *fraction <= 0.001 {
            continue;
        }
        let seg_w = (rect.width() * fraction).min(rect.max.x - current_x);
        let seg_rect = egui::Rect::from_min_size(
            egui::Pos2::new(current_x, rect.min.y),
            Vec2::new(seg_w, rect.height()),
        );
        painter.rect_filled(seg_rect, 0.0, *color);
        current_x += seg_w;
    }
    painter.rect_stroke(
        rect,
        3.0,
        Stroke::new(1.0, Color32::from_rgb(34, 38, 48)),
        egui::StrokeKind::Inside,
    );

    ui.add_space(5.0);

    draw_metric_subrow(
        ui,
        "Texture VRAM (2D/3D)",
        &format!("{:.2} MB", vram.texture_vram_mb),
        Color32::from_rgb(0, 190, 230),
    );
    draw_metric_subrow(
        ui,
        "Mesh & Index VRAM",
        &format!("{:.2} MB", vram.mesh_index_vram_mb),
        Color32::from_rgb(100, 220, 100),
    );
    draw_metric_subrow(
        ui,
        "Uniform & Target VRAM",
        &format!("{:.2} MB", vram.dynamic_uniform_vram_mb),
        Color32::from_rgb(240, 180, 50),
    );

    ui.add_space(3.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Total Allocated VRAM:")
                .strong()
                .color(Color32::from_rgb(220, 225, 235)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{:.2} MB", vram.total_vram_mb))
                    .strong()
                    .color(Color32::from_rgb(200, 230, 255)),
            );
        });
    });
}

/// Helper to render a compact telemetry subrow with label and colored right-aligned value.
fn draw_metric_subrow(ui: &mut egui::Ui, label: &str, value: &str, val_color: Color32) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("●")
                .font(egui::FontId::proportional(9.0))
                .color(val_color),
        );
        ui.label(
            egui::RichText::new(label)
                .color(Color32::from_rgb(175, 180, 195))
                .font(egui::FontId::proportional(11.0)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value)
                    .strong()
                    .color(val_color)
                    .font(egui::FontId::proportional(11.0)),
            );
        });
    });
}