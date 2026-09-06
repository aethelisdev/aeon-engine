// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 2D Virtual Canvas, Coordinate Projection, Grid, and Interactive WYSIWYG Rendering.
//!

use crate::anchors::draw_anchor_pin_and_guide;
use crate::state::UiDesignerContext;
use crate::types::{UiDesignerAction, UiDragState};
use ae_core::ecs::UiElement;
use ae_core::ui::{UiDrawCommand, UiLayoutResolver};

/// Renders the virtual canvas and all active UI elements with selection and drag interactions.
pub fn draw_canvas_area(ui: &mut egui::Ui, ctx: &mut UiDesignerContext<'_>) {
    let available_rect = ui.available_rect_before_wrap();
    let (response, painter) =
        ui.allocate_painter(available_rect.size(), egui::Sense::click_and_drag());

    let [screen_w, screen_h] = ctx.state.aspect_ratio.resolution();

    // 1. Compute fitted virtual canvas boundaries centered in view
    let margin = 24.0;
    let max_w = (available_rect.width() - margin * 2.0).max(100.0);
    let max_h = (available_rect.height() - margin * 2.0).max(100.0);

    let scale_w = max_w / screen_w;
    let scale_h = max_h / screen_h;
    let base_scale = scale_w.min(scale_h) * ctx.state.zoom;

    let canvas_w = screen_w * base_scale;
    let canvas_h = screen_h * base_scale;

    let center_x = available_rect.center().x + ctx.state.pan_offset[0];
    let center_y = available_rect.center().y + ctx.state.pan_offset[1];

    let canvas_rect = egui::Rect::from_center_size(
        egui::pos2(center_x, center_y),
        egui::vec2(canvas_w, canvas_h),
    );

    // 2. Letterbox Outer Background & Screen Frame
    painter.rect_filled(
        available_rect,
        egui::CornerRadius::ZERO,
        egui::Color32::from_rgb(14, 16, 22),
    );
    painter.rect_filled(
        canvas_rect,
        egui::CornerRadius::ZERO,
        egui::Color32::from_rgb(22, 26, 35),
    );
    painter.rect_stroke(
        canvas_rect,
        egui::CornerRadius::ZERO,
        egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 180, 240)),
        egui::StrokeKind::Outside,
    );

    // 3. Optional Background Grid
    if ctx.state.show_grid {
        draw_canvas_grid(&painter, canvas_rect, base_scale);
    }

    // Coordinate conversion closures
    let to_screen_pos = |canvas_x: f32, canvas_y: f32| -> egui::Pos2 {
        egui::pos2(
            canvas_rect.left() + (canvas_x / screen_w) * canvas_w,
            canvas_rect.top() + (canvas_y / screen_h) * canvas_h,
        )
    };

    let to_canvas_pos = |screen_pos: egui::Pos2| -> [f32; 2] {
        let rel_x = (screen_pos.x - canvas_rect.left()) / canvas_w;
        let rel_y = (screen_pos.y - canvas_rect.top()) / canvas_h;
        [rel_x * screen_w, rel_y * screen_h]
    };

    // 4. Resolve and Render In-Game UI Elements
    let mouse_canvas_pos = response.hover_pos().map(&to_canvas_pos);
    let draw_commands = UiLayoutResolver::resolve_draw_commands(
        ctx.world,
        screen_w,
        screen_h,
        mouse_canvas_pos,
        false,
    );

    for cmd in &draw_commands {
        match cmd {
            UiDrawCommand::Rect {
                rect,
                fill_color,
                border_color,
                border_width,
                border_radius,
                ..
            } => {
                let min_pos = to_screen_pos(rect.min_x, rect.min_y);
                let max_pos = to_screen_pos(rect.max_x, rect.max_y);
                let draw_rect = egui::Rect::from_min_max(min_pos, max_pos);

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
                let actual_stroke = if border_color[3] > 0.01 {
                    *border_width * base_scale
                } else {
                    0.0
                };

                painter.rect(
                    draw_rect,
                    egui::CornerRadius::same((*border_radius * base_scale) as u8),
                    fill,
                    egui::Stroke::new(actual_stroke.max(1.0), stroke_col),
                    egui::StrokeKind::Outside,
                );
            }
            UiDrawCommand::Text {
                pos,
                text,
                font_size,
                color,
                alignment,
                shadow_color,
                ..
            } => {
                let egui_pos = to_screen_pos(pos[0], pos[1]);
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
                        egui::FontId::proportional(*font_size * base_scale),
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
                    egui::FontId::proportional(*font_size * base_scale),
                    text_col,
                );
            }
            UiDrawCommand::Image { .. } => {}
        }
    }

    // 5. Interactive Selection, Anchor Pins, and Drag Hit-Testing
    let mut hovered_entity = None;

    for (ent, elem) in ctx.world.query::<(hecs::Entity, &UiElement)>().iter() {
        if !elem.visible {
            continue;
        }

        let elem_rect = elem.compute_rect(screen_w, screen_h);
        let screen_elem_rect = egui::Rect::from_min_max(
            to_screen_pos(elem_rect.min_x, elem_rect.min_y),
            to_screen_pos(elem_rect.max_x, elem_rect.max_y),
        );

        if let Some(mouse_p) = response.hover_pos()
            && screen_elem_rect.contains(mouse_p)
        {
            hovered_entity = Some(ent);
        }

        let is_selected = ctx.selected_entity == Some(ent);

        // Draw Selection Outline & Resize Handles
        if is_selected {
            painter.rect_stroke(
                screen_elem_rect.expand(2.0),
                egui::CornerRadius::same(2),
                egui::Stroke::new(1.8, egui::Color32::from_rgb(0, 210, 255)),
                egui::StrokeKind::Outside,
            );

            let handle_size = 6.0;
            let corners = [
                screen_elem_rect.left_top(),
                screen_elem_rect.right_top(),
                screen_elem_rect.left_bottom(),
                screen_elem_rect.right_bottom(),
            ];
            for corner in corners {
                painter.rect_filled(
                    egui::Rect::from_center_size(corner, egui::vec2(handle_size, handle_size)),
                    egui::CornerRadius::same(1),
                    egui::Color32::WHITE,
                );
            }

            if ctx.state.show_anchor_guides {
                draw_anchor_pin_and_guide(
                    &painter,
                    elem,
                    screen_elem_rect,
                    screen_w,
                    screen_h,
                    base_scale,
                    &to_screen_pos,
                );
            }
        } else if hovered_entity == Some(ent) {
            painter.rect_stroke(
                screen_elem_rect.expand(1.0),
                egui::CornerRadius::same(2),
                egui::Stroke::new(1.2, egui::Color32::from_rgba_unmultiplied(0, 210, 255, 120)),
                egui::StrokeKind::Outside,
            );
        }
    }

    // 6. Handle Mouse Dragging & Canvas Interactions
    if response.drag_started() {
        if let Some(ent) = hovered_entity {
            ctx.actions.push(UiDesignerAction::SelectEntity(Some(ent)));
            if let Ok(elem) = ctx.world.get::<&UiElement>(ent) {
                let origin = elem.anchor.compute_origin(screen_w, screen_h);
                if let Some(mouse_screen) = response.interact_pointer_pos() {
                    let mouse_canvas = to_canvas_pos(mouse_screen);
                    ctx.state.drag_state = Some(UiDragState {
                        entity: ent,
                        anchor_origin: origin,
                        drag_start_mouse_canvas: mouse_canvas,
                        initial_offset: elem.offset,
                    });
                }
            }
        } else if response.hover_pos().is_some() {
            ctx.actions.push(UiDesignerAction::SelectEntity(None));
        }
    }

    if response.dragged() {
        if let Some(drag) = ctx.state.drag_state {
            if let Some(current_mouse_screen) = response.interact_pointer_pos() {
                let current_mouse_canvas = to_canvas_pos(current_mouse_screen);
                let delta_x = current_mouse_canvas[0] - drag.drag_start_mouse_canvas[0];
                let delta_y = current_mouse_canvas[1] - drag.drag_start_mouse_canvas[1];

                let mut new_offset_x = drag.initial_offset[0] + delta_x;
                let mut new_offset_y = drag.initial_offset[1] + delta_y;

                // Snap to Grid
                if let Some(snap) = ctx.state.snap_grid {
                    new_offset_x = (new_offset_x / snap).round() * snap;
                    new_offset_y = (new_offset_y / snap).round() * snap;
                }

                ctx.actions.push(UiDesignerAction::UpdateElementOffset {
                    entity: drag.entity,
                    offset: [new_offset_x, new_offset_y],
                });
            }
        } else {
            // Pan virtual canvas
            let delta = response.drag_delta();
            ctx.state.pan_offset[0] += delta.x;
            ctx.state.pan_offset[1] += delta.y;
        }
    }

    if response.drag_stopped() {
        ctx.state.drag_state = None;
    }

    // Canvas footer indicator
    painter.text(
        egui::pos2(canvas_rect.right() - 8.0, canvas_rect.bottom() - 8.0),
        egui::Align2::RIGHT_BOTTOM,
        format!(
            "Canvas: {:.0}x{:.0} | Zoom: {:.0}%",
            screen_w,
            screen_h,
            ctx.state.zoom * 100.0
        ),
        egui::FontId::proportional(10.0),
        egui::Color32::from_gray(100),
    );
}

/// Draws an adaptive background grid on the virtual canvas.
pub fn draw_canvas_grid(painter: &egui::Painter, canvas_rect: egui::Rect, base_scale: f32) {
    let grid_step_pixels = 64.0 * base_scale;
    if grid_step_pixels < 8.0 {
        return;
    }

    let grid_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 12);
    let major_grid_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25);

    // Vertical lines
    let mut x = canvas_rect.left();
    let mut col_idx = 0;
    while x <= canvas_rect.right() {
        let col = if col_idx % 4 == 0 {
            major_grid_color
        } else {
            grid_color
        };
        painter.line_segment(
            [
                egui::pos2(x, canvas_rect.top()),
                egui::pos2(x, canvas_rect.bottom()),
            ],
            egui::Stroke::new(1.0, col),
        );
        x += grid_step_pixels;
        col_idx += 1;
    }

    // Horizontal lines
    let mut y = canvas_rect.top();
    let mut row_idx = 0;
    while y <= canvas_rect.bottom() {
        let col = if row_idx % 4 == 0 {
            major_grid_color
        } else {
            grid_color
        };
        painter.line_segment(
            [
                egui::pos2(canvas_rect.left(), y),
                egui::pos2(canvas_rect.right(), y),
            ],
            egui::Stroke::new(1.0, col),
        );
        y += grid_step_pixels;
        row_idx += 1;
    }
}