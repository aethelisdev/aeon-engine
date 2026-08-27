// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Visual Anchor Guide and Origin Pin Visualizer.
//!

use ae_core::ecs::UiElement;

/// Draws a golden anchor pin and dotted guideline connecting the anchor origin to the UI element center.
pub fn draw_anchor_pin_and_guide(
    painter: &egui::Painter,
    elem: &UiElement,
    screen_elem_rect: egui::Rect,
    screen_w: f32,
    screen_h: f32,
    base_scale: f32,
    to_screen_pos: &impl Fn(f32, f32) -> egui::Pos2,
) {
    let [anchor_x, anchor_y] = elem.anchor.compute_origin(screen_w, screen_h);
    let screen_anchor_pos = to_screen_pos(anchor_x, anchor_y);
    let elem_center = screen_elem_rect.center();

    // 1. Dotted guideline from anchor origin to element center
    let guide_color = egui::Color32::from_rgba_unmultiplied(255, 200, 50, 160);
    painter.line_segment(
        [screen_anchor_pos, elem_center],
        egui::Stroke::new(1.2 * base_scale.max(0.75), guide_color),
    );

    // 2. Anchor Origin Pin (Golden circle with center dot)
    let pin_radius = 4.5 * base_scale.max(0.8);
    painter.circle_filled(
        screen_anchor_pos,
        pin_radius,
        egui::Color32::from_rgb(255, 185, 30),
    );
    painter.circle_stroke(
        screen_anchor_pos,
        pin_radius,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 30, 10)),
    );

    // 3. Anchor Type Micro Label
    let anchor_name = match elem.anchor {
        ae_core::ecs::UiAnchor::TopLeft => "TopLeft",
        ae_core::ecs::UiAnchor::TopCenter => "TopCenter",
        ae_core::ecs::UiAnchor::TopRight => "TopRight",
        ae_core::ecs::UiAnchor::CenterLeft => "CenterLeft",
        ae_core::ecs::UiAnchor::Center => "Center",
        ae_core::ecs::UiAnchor::CenterRight => "CenterRight",
        ae_core::ecs::UiAnchor::BottomLeft => "BottomLeft",
        ae_core::ecs::UiAnchor::BottomCenter => "BottomCenter",
        ae_core::ecs::UiAnchor::BottomRight => "BottomRight",
    };

    painter.text(
        egui::pos2(screen_anchor_pos.x + 6.0, screen_anchor_pos.y - 6.0),
        egui::Align2::LEFT_BOTTOM,
        format!("⚓ {}", anchor_name),
        egui::FontId::proportional(9.0 * base_scale.max(0.75)),
        egui::Color32::from_rgb(255, 215, 80),
    );
}