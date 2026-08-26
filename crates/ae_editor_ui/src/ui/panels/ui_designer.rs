// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 2D Visual UI Designer & Canvas Editor Subsystem.
//!
//! Provides a dedicated, resolution-independent WYSIWYG 2D canvas editor for designing
//! in-game HUDs, health bars, interactive buttons, menus, and typography layouts with
//! visual anchor guides, grid snapping, and interactive drag-and-drop.
//!

use crate::ui::types::{EngineUiAction, UiElementType};
use ae_core::ecs::UiElement;

use ae_core::ui::{UiDrawCommand, UiLayoutResolver};

/// Predefined standard screen aspect ratio presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasAspectRatio {
    #[default]
    Ratio16x9,
    Ratio16x10,
    Ratio4x3,
    Ratio21x9,
}

impl CanvasAspectRatio {
    /// Returns the virtual reference resolution `[width, height]` in pixels.
    pub fn resolution(&self) -> [f32; 2] {
        match self {
            Self::Ratio16x9 => [1920.0, 1080.0],
            Self::Ratio16x10 => [1920.0, 1200.0],
            Self::Ratio4x3 => [1440.0, 1080.0],
            Self::Ratio21x9 => [2560.0, 1080.0],
        }
    }

    /// Returns display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ratio16x9 => "16:9 (1080p Full HD)",
            Self::Ratio16x10 => "16:10 (WUXGA)",
            Self::Ratio4x3 => "4:3 (Classic)",
            Self::Ratio21x9 => "21:9 (Ultrawide)",
        }
    }
}

/// Active drag interaction state for moving UI elements in the 2D canvas.
#[derive(Debug, Clone, Copy)]
pub struct UiDragState {
    pub entity: hecs::Entity,
    pub anchor_origin: [f32; 2],
    pub drag_start_mouse_canvas: [f32; 2],
    pub initial_offset: [f32; 2],
}

/// Persistent editor state for the UI Designer canvas panel.
#[derive(Debug, Clone)]
pub struct UiDesignerState {
    pub aspect_ratio: CanvasAspectRatio,
    pub zoom: f32,
    pub pan_offset: egui::Vec2,
    pub snap_grid: Option<f32>,
    pub show_anchor_guides: bool,
    pub show_grid: bool,
    pub drag_state: Option<UiDragState>,
}

impl Default for UiDesignerState {
    fn default() -> Self {
        Self {
            aspect_ratio: CanvasAspectRatio::Ratio16x9,
            zoom: 1.0,
            pan_offset: egui::Vec2::ZERO,
            snap_grid: Some(8.0),
            show_anchor_guides: true,
            show_grid: true,
            drag_state: None,
        }
    }
}

/// Context parameters passed into the UI Designer panel renderer.
pub struct UiDesignerContext<'a> {
    pub world: &'a hecs::World,
    pub selected_entity: Option<hecs::Entity>,
    pub ui_actions: &'a mut Vec<EngineUiAction>,
    pub state: &'a mut UiDesignerState,
}

/// Renders the 2D UI Designer panel frame, canvas, anchor lines, and widget overlays.
pub fn draw_ui_designer_panel(ui: &mut egui::Ui, ctx: &mut UiDesignerContext<'_>) {
    // 1. Top Toolbar
    draw_designer_toolbar(ui, ctx);

    ui.separator();

    // 2. Interactive 2D Canvas Area
    let available_rect = ui.available_rect_before_wrap();
    let (response, painter) =
        ui.allocate_painter(available_rect.size(), egui::Sense::click_and_drag());

    let [screen_w, screen_h] = ctx.state.aspect_ratio.resolution();

    // Calculate fitted canvas dimensions centered in available space
    let margin = 24.0;
    let max_w = (available_rect.width() - margin * 2.0).max(100.0);
    let max_h = (available_rect.height() - margin * 2.0).max(100.0);

    let scale_w = max_w / screen_w;
    let scale_h = max_h / screen_h;
    let base_scale = scale_w.min(scale_h) * ctx.state.zoom;

    let canvas_w = screen_w * base_scale;
    let canvas_h = screen_h * base_scale;

    let center_x = available_rect.center().x + ctx.state.pan_offset.x;
    let center_y = available_rect.center().y + ctx.state.pan_offset.y;

    let canvas_rect = egui::Rect::from_center_size(
        egui::pos2(center_x, center_y),
        egui::vec2(canvas_w, canvas_h),
    );

    // Canvas Background (Outer Letterbox)
    painter.rect_filled(
        available_rect,
        egui::CornerRadius::ZERO,
        egui::Color32::from_rgb(14, 16, 22),
    );

    // Canvas Virtual Screen Frame
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

    // Subtle 2D Grid
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

    // 3. Render In-Game UI Elements on the Canvas
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

    // 4. Interactive Selection & Anchor Pin Visualizer
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

        // Hit testing
        if let Some(mouse_p) = response.hover_pos()
            && screen_elem_rect.contains(mouse_p)
        {
            hovered_entity = Some(ent);
        }

        let is_selected = ctx.selected_entity == Some(ent);

        // Draw Selection Outline & Resize Handles
        if is_selected {
            painter.rect_stroke(
                screen_elem_rect,
                egui::CornerRadius::ZERO,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 230, 255)),
                egui::StrokeKind::Outside,
            );

            // Draw Corner Handles
            let handle_size = 6.0;
            for corner in [
                screen_elem_rect.left_top(),
                screen_elem_rect.right_top(),
                screen_elem_rect.left_bottom(),
                screen_elem_rect.right_bottom(),
            ] {
                painter.rect_filled(
                    egui::Rect::from_center_size(corner, egui::vec2(handle_size, handle_size)),
                    egui::CornerRadius::ZERO,
                    egui::Color32::from_rgb(0, 230, 255),
                );
            }
        }

        // ⚓ Anchor Pin & Dotted Guide Line
        if ctx.state.show_anchor_guides && (is_selected || hovered_entity == Some(ent)) {
            let anchor_origin = elem.anchor.compute_origin(screen_w, screen_h);
            let screen_anchor_origin = to_screen_pos(anchor_origin[0], anchor_origin[1]);
            let screen_elem_center = screen_elem_rect.center();

            // Dotted Guide Line from Anchor Pin to Element Center
            painter.line_segment(
                [screen_anchor_origin, screen_elem_center],
                egui::Stroke::new(1.2, egui::Color32::from_rgb(255, 200, 40)),
            );

            // Draw Anchor Pin Glyph (⚓)
            painter.circle_filled(
                screen_anchor_origin,
                5.0,
                egui::Color32::from_rgb(255, 200, 40),
            );
            painter.circle_stroke(
                screen_anchor_origin,
                7.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 50)),
            );

            painter.text(
                egui::pos2(screen_anchor_origin.x, screen_anchor_origin.y - 14.0),
                egui::Align2::CENTER_CENTER,
                format!("⚓ {:?}", elem.anchor),
                egui::FontId::proportional(11.0),
                egui::Color32::from_rgb(255, 220, 80),
            );
        }
    }

    // 5. Handle Click & Drag Logic
    if response.clicked() {
        ctx.ui_actions
            .push(EngineUiAction::SelectEntity(hovered_entity));
    }

    if response.drag_started()
        && let Some(sel) = ctx.selected_entity
        && let Ok(elem) = ctx.world.get::<&UiElement>(sel)
        && let Some(mouse_p) = response.hover_pos()
    {
        let mouse_canvas = to_canvas_pos(mouse_p);
        let anchor_orig = elem.anchor.compute_origin(screen_w, screen_h);
        ctx.state.drag_state = Some(UiDragState {
            entity: sel,
            anchor_origin: anchor_orig,
            drag_start_mouse_canvas: mouse_canvas,
            initial_offset: elem.offset,
        });
    }

    if response.dragged()
        && let Some(drag) = ctx.state.drag_state
        && let Some(mouse_p) = response.hover_pos()
        && let Ok(elem) = ctx.world.get::<&UiElement>(drag.entity)
    {
        let current_mouse_canvas = to_canvas_pos(mouse_p);
        let delta_x = current_mouse_canvas[0] - drag.drag_start_mouse_canvas[0];
        let delta_y = current_mouse_canvas[1] - drag.drag_start_mouse_canvas[1];

        let mut new_offset = [
            drag.initial_offset[0] + delta_x,
            drag.initial_offset[1] + delta_y,
        ];

        // Apply grid snap
        if let Some(snap) = ctx.state.snap_grid {
            new_offset[0] = (new_offset[0] / snap).round() * snap;
            new_offset[1] = (new_offset[1] / snap).round() * snap;
        }

        if new_offset != elem.offset {
            let mut updated_elem = *elem;
            updated_elem.offset = new_offset;
            ctx.ui_actions.push(EngineUiAction::modify_component(
                drag.entity,
                "UiElement",
                &updated_elem,
            ));
        }
    }

    if response.drag_stopped() {
        ctx.state.drag_state = None;
    }

    // Canvas Info overlay (Bottom-Right)
    let info_text = format!(
        "Canvas: {:.0}x{:.0} | Zoom: {:.0}%",
        screen_w,
        screen_h,
        ctx.state.zoom * 100.0
    );
    painter.text(
        egui::pos2(canvas_rect.right() - 8.0, canvas_rect.bottom() - 14.0),
        egui::Align2::RIGHT_CENTER,
        info_text,
        egui::FontId::proportional(11.0),
        egui::Color32::from_gray(140),
    );
}

/// Draws subtle dotted grid lines over the 2D virtual canvas.
fn draw_canvas_grid(painter: &egui::Painter, canvas_rect: egui::Rect, scale: f32) {
    let grid_step = 64.0 * scale;
    if grid_step < 12.0 {
        return;
    }

    let mut x = canvas_rect.left() + grid_step;
    while x < canvas_rect.right() {
        painter.line_segment(
            [
                egui::pos2(x, canvas_rect.top()),
                egui::pos2(x, canvas_rect.bottom()),
            ],
            egui::Stroke::new(0.5, egui::Color32::from_rgb(30, 36, 48)),
        );
        x += grid_step;
    }

    let mut y = canvas_rect.top() + grid_step;
    while y < canvas_rect.bottom() {
        painter.line_segment(
            [
                egui::pos2(canvas_rect.left(), y),
                egui::pos2(canvas_rect.right(), y),
            ],
            egui::Stroke::new(0.5, egui::Color32::from_rgb(30, 36, 48)),
        );
        y += grid_step;
    }
}

/// Renders the top control toolbar for aspect ratio, zoom, grid snapping, and quick spawner.
fn draw_designer_toolbar(ui: &mut egui::Ui, ctx: &mut UiDesignerContext<'_>) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("📐 UI Designer")
                .strong()
                .color(egui::Color32::from_rgb(0, 200, 255)),
        );

        ui.separator();

        // Aspect Ratio Selector
        ui.label("Aspect:");
        egui::ComboBox::from_id_salt("ui_designer_aspect_ratio")
            .selected_text(ctx.state.aspect_ratio.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut ctx.state.aspect_ratio,
                    CanvasAspectRatio::Ratio16x9,
                    CanvasAspectRatio::Ratio16x9.label(),
                );
                ui.selectable_value(
                    &mut ctx.state.aspect_ratio,
                    CanvasAspectRatio::Ratio16x10,
                    CanvasAspectRatio::Ratio16x10.label(),
                );
                ui.selectable_value(
                    &mut ctx.state.aspect_ratio,
                    CanvasAspectRatio::Ratio4x3,
                    CanvasAspectRatio::Ratio4x3.label(),
                );
                ui.selectable_value(
                    &mut ctx.state.aspect_ratio,
                    CanvasAspectRatio::Ratio21x9,
                    CanvasAspectRatio::Ratio21x9.label(),
                );
            });

        ui.separator();

        // Zoom Controls
        ui.label("Zoom:");
        if ui.button("➖").clicked() {
            ctx.state.zoom = (ctx.state.zoom - 0.1).max(0.25);
        }
        ui.label(format!("{:.0}%", ctx.state.zoom * 100.0));
        if ui.button("➕").clicked() {
            ctx.state.zoom = (ctx.state.zoom + 0.1).min(2.5);
        }
        if ui.button("100%").clicked() {
            ctx.state.zoom = 1.0;
            ctx.state.pan_offset = egui::Vec2::ZERO;
        }

        ui.separator();

        // Grid Snapping
        let snap_labels = ["Off", "8 px", "16 px", "32 px"];
        let current_snap_idx = match ctx.state.snap_grid {
            None => 0,
            Some(s) if (s - 8.0).abs() < 1e-4 => 1,
            Some(s) if (s - 16.0).abs() < 1e-4 => 2,
            _ => 3,
        };
        let mut selected_snap = current_snap_idx;

        ui.label("Snap:");
        egui::ComboBox::from_id_salt("ui_designer_snap")
            .selected_text(snap_labels[selected_snap])
            .show_ui(ui, |ui| {
                for (i, label) in snap_labels.iter().enumerate() {
                    ui.selectable_value(&mut selected_snap, i, *label);
                }
            });

        if selected_snap != current_snap_idx {
            ctx.state.snap_grid = match selected_snap {
                0 => None,
                1 => Some(8.0),
                2 => Some(16.0),
                _ => Some(32.0),
            };
        }

        ui.separator();

        // Anchor Guides Toggle
        ui.toggle_value(&mut ctx.state.show_anchor_guides, "⚓ Anchor Guides");
        ui.toggle_value(&mut ctx.state.show_grid, "Grid");

        ui.separator();

        // Quick Spawn Menu
        ui.menu_button("➕ Add Element", |ui| {
            if ui.button("🟩 Panel / Canvas Box").clicked() {
                ctx.ui_actions
                    .push(EngineUiAction::SpawnUiElement(UiElementType::Panel));
                ui.close();
            }
            if ui.button("📊 Progress Bar").clicked() {
                ctx.ui_actions
                    .push(EngineUiAction::SpawnUiElement(UiElementType::ProgressBar));
                ui.close();
            }
            if ui.button("❤️ Health Bar (Player Tag)").clicked() {
                ctx.ui_actions
                    .push(EngineUiAction::SpawnUiElement(UiElementType::HealthBar));
                ui.close();
            }
            if ui.button("🔤 Text Label").clicked() {
                ctx.ui_actions
                    .push(EngineUiAction::SpawnUiElement(UiElementType::Text));
                ui.close();
            }
            if ui.button("⭐ Score Display (Score Tag)").clicked() {
                ctx.ui_actions
                    .push(EngineUiAction::SpawnUiElement(UiElementType::ScoreDisplay));
                ui.close();
            }
            if ui.button("🔘 Button").clicked() {
                ctx.ui_actions
                    .push(EngineUiAction::SpawnUiElement(UiElementType::Button));
                ui.close();
            }
            if ui.button("🖼️ Image").clicked() {
                ctx.ui_actions
                    .push(EngineUiAction::SpawnUiElement(UiElementType::Image));
                ui.close();
            }
            if ui.button("🎚️ Slider").clicked() {
                ctx.ui_actions
                    .push(EngineUiAction::SpawnUiElement(UiElementType::Slider));
                ui.close();
            }
            if ui.button("☑️ Checkbox").clicked() {
                ctx.ui_actions
                    .push(EngineUiAction::SpawnUiElement(UiElementType::Checkbox));
                ui.close();
            }
            if ui.button("📝 Text Input").clicked() {
                ctx.ui_actions
                    .push(EngineUiAction::SpawnUiElement(UiElementType::TextInput));
                ui.close();
            }
        });
    });
}