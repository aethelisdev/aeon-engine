// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # In-Game UI Draw Command Rasterizer
//!
//! Renders backend-agnostic in-game UI draw commands onto the egui canvas viewport.

/// Renders backend-agnostic `UiDrawCommand` batches into the viewport.
pub fn render_ui_draw_commands(
    painter: &egui::Painter,
    viewport_rect: egui::Rect,
    commands: &[ae_core::ui::UiDrawCommand],
) {
    for cmd in commands {
        match cmd {
            ae_core::ui::UiDrawCommand::Rect {
                rect,
                fill_color,
                border_color,
                border_width,
                border_radius,
                ..
            } => {
                let egui_rect = egui::Rect::from_min_max(
                    egui::pos2(
                        viewport_rect.left() + rect.min_x,
                        viewport_rect.top() + rect.min_y,
                    ),
                    egui::pos2(
                        viewport_rect.left() + rect.max_x,
                        viewport_rect.top() + rect.max_y,
                    ),
                );
                let fill = egui::Color32::from_rgba_unmultiplied(
                    (fill_color[0] * 255.0) as u8,
                    (fill_color[1] * 255.0) as u8,
                    (fill_color[2] * 255.0) as u8,
                    (fill_color[3] * 255.0) as u8,
                );
                let stroke_col = egui::Color32::from_rgba_unmultiplied(
                    (border_color[0] * 255.0) as u8,
                    (border_color[1] * 255.0) as u8,
                    (border_color[2] * 255.0) as u8,
                    (border_color[3] * 255.0) as u8,
                );
                let actual_stroke_width = if border_color[3] > 0.01 {
                    *border_width
                } else {
                    0.0
                };
                painter.rect(
                    egui_rect,
                    egui::CornerRadius::same(*border_radius as u8),
                    fill,
                    egui::Stroke::new(actual_stroke_width, stroke_col),
                    egui::StrokeKind::Outside,
                );
            }
            ae_core::ui::UiDrawCommand::Text {
                pos,
                text,
                font_size,
                color,
                alignment,
                shadow_color,
                ..
            } => {
                let egui_pos =
                    egui::pos2(viewport_rect.left() + pos[0], viewport_rect.top() + pos[1]);
                let align = match alignment {
                    ae_core::ui::UiTextAlignment::Left => egui::Align2::LEFT_CENTER,
                    ae_core::ui::UiTextAlignment::Center => egui::Align2::CENTER_CENTER,
                    ae_core::ui::UiTextAlignment::Right => egui::Align2::RIGHT_CENTER,
                };
                if let Some(shadow) = shadow_color {
                    let shadow_col = egui::Color32::from_rgba_unmultiplied(
                        (shadow[0] * 255.0) as u8,
                        (shadow[1] * 255.0) as u8,
                        (shadow[2] * 255.0) as u8,
                        (shadow[3] * 255.0) as u8,
                    );
                    painter.text(
                        egui::pos2(egui_pos.x + 1.0, egui_pos.y + 1.0),
                        align,
                        text,
                        egui::FontId::proportional(*font_size),
                        shadow_col,
                    );
                }
                let text_col = egui::Color32::from_rgba_unmultiplied(
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                    (color[3] * 255.0) as u8,
                );
                painter.text(
                    egui_pos,
                    align,
                    text,
                    egui::FontId::proportional(*font_size),
                    text_col,
                );
            }

            ae_core::ui::UiDrawCommand::Image { .. } => {}
        }
    }
}