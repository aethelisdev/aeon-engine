// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Widget Palette, Dropdown Menus, and Toolbar Controls.
//!

use crate::state::UiDesignerContext;
use crate::types::{CanvasAspectRatio, UiDesignerAction, UiElementType};

/// Renders a neatly formatted, column-aligned dropdown menu item with a fixed-width icon column.
/// Ensures all menu items have identical 18px icon centering and text starting at an exact
/// horizontal coordinate regardless of varying Unicode emoji glyph widths.
pub fn dropdown_item(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    let padding_x = 6.0;
    let icon_width = 18.0;
    let height = 22.0;
    let item_width = 180.0;

    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(item_width, height), egui::Sense::click());

    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(4),
            egui::Color32::from_rgb(38, 44, 58),
        );
    }

    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x + padding_x, rect.min.y),
        egui::vec2(icon_width, height),
    );
    let text_pos = egui::pos2(
        rect.min.x + padding_x + icon_width + 6.0,
        rect.min.y + (height - 12.0) * 0.5 - 1.0,
    );

    let (icon_color, text_color) = if response.hovered() {
        (egui::Color32::WHITE, egui::Color32::WHITE)
    } else {
        (egui::Color32::from_gray(180), egui::Color32::from_gray(215))
    };

    ui.painter().text(
        icon_rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(12.0),
        icon_color,
    );

    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(12.0),
        text_color,
    );

    response
}

/// Renders the top toolbar of the UI Designer canvas.
pub fn draw_designer_toolbar(ui: &mut egui::Ui, ctx: &mut UiDesignerContext<'_>) {
    ui.horizontal(|ui| {
        ui.add_space(4.0);

        // Aspect Ratio Dropdown
        ui.label(
            egui::RichText::new("Aspect:")
                .size(11.0)
                .color(egui::Color32::from_gray(160)),
        );
        egui::ComboBox::from_id_salt("ui_designer_aspect_combo")
            .selected_text(ctx.state.aspect_ratio.label())
            .width(160.0)
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
        ui.label(
            egui::RichText::new("Zoom:")
                .size(11.0)
                .color(egui::Color32::from_gray(160)),
        );
        if ui.button("➖").on_hover_text("Zoom Out (25%)").clicked() {
            ctx.state.zoom = (ctx.state.zoom - 0.25).max(0.25);
        }
        ui.label(
            egui::RichText::new(format!("{:.0}%", ctx.state.zoom * 100.0))
                .strong()
                .size(11.5)
                .color(egui::Color32::from_rgb(0, 210, 255)),
        );
        if ui.button("➕").on_hover_text("Zoom In (25%)").clicked() {
            ctx.state.zoom = (ctx.state.zoom + 0.25).min(4.0);
        }
        if ui.button("100%").on_hover_text("Reset Zoom").clicked() {
            ctx.state.zoom = 1.0;
            ctx.state.pan_offset = egui::Vec2::ZERO;
        }

        ui.separator();

        // Snap Grid Dropdown
        ui.label(
            egui::RichText::new("Snap:")
                .size(11.0)
                .color(egui::Color32::from_gray(160)),
        );
        let snap_label = match ctx.state.snap_grid {
            None => "Free",
            Some(8.0) => "8 px",
            Some(16.0) => "16 px",
            Some(32.0) => "32 px",
            Some(_v) => "Custom",
        };
        egui::ComboBox::from_id_salt("ui_designer_snap_combo")
            .selected_text(snap_label)
            .width(70.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut ctx.state.snap_grid, None, "Free");
                ui.selectable_value(&mut ctx.state.snap_grid, Some(8.0), "8 px");
                ui.selectable_value(&mut ctx.state.snap_grid, Some(16.0), "16 px");
                ui.selectable_value(&mut ctx.state.snap_grid, Some(32.0), "32 px");
            });

        ui.separator();

        // Visual Toggles
        ui.toggle_value(&mut ctx.state.show_anchor_guides, "⚓ Anchor Guides");
        ui.toggle_value(&mut ctx.state.show_grid, "Grid");

        ui.separator();

        // Quick Spawn Menu (Palette)
        ui.menu_button("➕ Add Element", |ui| {
            ui.set_min_width(180.0);
            if dropdown_item(ui, "🟩", "Panel / Canvas Box").clicked() {
                ctx.actions
                    .push(UiDesignerAction::SpawnElement(UiElementType::Panel));
                ui.close();
            }
            if dropdown_item(ui, "🔤", "Text Label").clicked() {
                ctx.actions
                    .push(UiDesignerAction::SpawnElement(UiElementType::Text));
                ui.close();
            }
            if dropdown_item(ui, "🖼️", "Image / Icon").clicked() {
                ctx.actions
                    .push(UiDesignerAction::SpawnElement(UiElementType::Image));
                ui.close();
            }
            if dropdown_item(ui, "🔘", "Interactive Button").clicked() {
                ctx.actions
                    .push(UiDesignerAction::SpawnElement(UiElementType::Button));
                ui.close();
            }
            if dropdown_item(ui, "📊", "Progress Bar").clicked() {
                ctx.actions
                    .push(UiDesignerAction::SpawnElement(UiElementType::ProgressBar));
                ui.close();
            }
            if dropdown_item(ui, "🎚️", "Numeric Slider").clicked() {
                ctx.actions
                    .push(UiDesignerAction::SpawnElement(UiElementType::Slider));
                ui.close();
            }
            if dropdown_item(ui, "☑️", "Toggle Checkbox").clicked() {
                ctx.actions
                    .push(UiDesignerAction::SpawnElement(UiElementType::Checkbox));
                ui.close();
            }
            if dropdown_item(ui, "📝", "Text Input Field").clicked() {
                ctx.actions
                    .push(UiDesignerAction::SpawnElement(UiElementType::TextInput));
                ui.close();
            }
            ui.separator();
            ui.menu_button("🎮 HUD Presets", |ui| {
                ui.set_min_width(180.0);
                if dropdown_item(ui, "❤️", "Health Bar (Player Tag)").clicked() {
                    ctx.actions
                        .push(UiDesignerAction::SpawnElement(UiElementType::HealthBar));
                    ui.close();
                }
                if dropdown_item(ui, "⭐", "Score Display (Score Tag)").clicked() {
                    ctx.actions
                        .push(UiDesignerAction::SpawnElement(UiElementType::ScoreDisplay));
                    ui.close();
                }
            });
        });
    });
}