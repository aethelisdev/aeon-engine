// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Camera Info HUD Builder
//!
//! Renders the bottom-right camera position and orientation angles overlay badge.

use super::types::ViewportHudParams;
use irisui::prelude::*;

/// Builds the camera position & rotation angles HUD badge at the bottom-right of the viewport.
pub fn build_camera_hud(tree: &mut UiTree, parent_id: WidgetId, params: &ViewportHudParams<'_>) {
    let p = params.camera.position;
    let pos_text = format!("Pos: {:.1}, {:.1}, {:.1}", p.x, p.y, p.z);
    let pitch_deg = params.camera.pitch.0.to_degrees();
    let yaw_deg = params.camera.yaw.0.to_degrees();
    let rot_text = format!("Rot: {:.0}°, {:.0}°", pitch_deg, yaw_deg);
    let full_text = format!("{}   |   {}", pos_text, rot_text);

    let hud_w = 210.0;
    let hud_h = 22.0;
    let hud_x = params.viewport_rect.x + params.viewport_rect.width - hud_w - 8.0;
    let hud_y = params.viewport_rect.y + params.viewport_rect.height - hud_h - 8.0;

    let hud_rect = Rect::new(hud_x, hud_y, hud_w, hud_h);

    let hud_id = tree.create_node();
    if let Some(node) = tree.get_mut(hud_id) {
        node.set_name("CameraHudPill");
        node.computed_rect = hud_rect;
        node.style = Style::new()
            .background(Color::rgba(0.07, 0.08, 0.11, 0.75))
            .border(1.0, Color::rgba(0.24, 0.28, 0.38, 0.50))
            .border_radius(4.0);
    }
    let _ = tree.add_child(parent_id, hud_id);

    let txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(txt_id) {
        node.set_name("CameraHudText");
        node.set_text(full_text);
        node.font_size = 10.0;
        node.line_height = hud_h;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.80, 0.83, 0.90, 0.90);
        node.computed_rect = hud_rect;
    }
    let _ = tree.add_child(hud_id, txt_id);
}