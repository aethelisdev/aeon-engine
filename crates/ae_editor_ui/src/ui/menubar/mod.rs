// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

pub mod edit;
pub mod file;
pub mod help;
pub mod view;

pub use edit::*;
pub use file::*;
pub use view::*;

/// Standard menu item width for all top menu bar dropdowns.
pub const MENU_ITEM_WIDTH: f32 = 205.0;

/// Renders a neatly formatted menu bar item with an icon column, text label, and right-aligned shortcut hint.
pub fn menu_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    shortcut: Option<&str>,
    enabled: bool,
) -> egui::Response {
    let font_size = 12.0;
    let height = 24.0;
    let icon_width = 24.0;

    let available_width = ui.available_width().max(MENU_ITEM_WIDTH);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(available_width, height), egui::Sense::click());

    if !enabled {
        // Disabled state: draw faded text without hover response
        let text_color = egui::Color32::from_rgb(100, 100, 110);
        let icon_rect = egui::Rect::from_min_size(rect.min, egui::vec2(icon_width, height));
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(font_size),
            text_color,
        );

        let label_pos = rect.min + egui::vec2(icon_width + 4.0, (height - font_size) * 0.5);
        ui.painter().text(
            label_pos,
            egui::Align2::LEFT_TOP,
            label,
            egui::FontId::proportional(font_size),
            text_color,
        );

        if let Some(sc) = shortcut {
            let sc_pos = egui::pos2(
                rect.max.x - 8.0,
                rect.min.y + (height - (font_size - 1.0)) * 0.5,
            );
            ui.painter().text(
                sc_pos,
                egui::Align2::RIGHT_TOP,
                sc,
                egui::FontId::proportional(font_size - 1.0),
                egui::Color32::from_rgb(70, 70, 80),
            );
        }

        return response;
    }

    let is_hovered = response.hovered();

    if is_hovered {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(3),
            egui::Color32::from_rgb(40, 44, 56),
        );
    }

    let icon_color = if is_hovered {
        egui::Color32::from_rgb(255, 255, 255)
    } else {
        egui::Color32::from_rgb(180, 180, 190)
    };

    let label_color = if is_hovered {
        egui::Color32::from_rgb(255, 255, 255)
    } else {
        egui::Color32::from_rgb(220, 220, 230)
    };

    let shortcut_color = if is_hovered {
        egui::Color32::from_rgb(160, 160, 175)
    } else {
        egui::Color32::from_rgb(120, 120, 135)
    };

    let icon_rect = egui::Rect::from_min_size(rect.min, egui::vec2(icon_width, height));
    ui.painter().text(
        icon_rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(font_size),
        icon_color,
    );

    let label_pos = rect.min + egui::vec2(icon_width + 4.0, (height - font_size) * 0.5);
    ui.painter().text(
        label_pos,
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(font_size),
        label_color,
    );

    if let Some(sc) = shortcut {
        let sc_pos = egui::pos2(
            rect.max.x - 8.0,
            rect.min.y + (height - (font_size - 1.0)) * 0.5,
        );
        ui.painter().text(
            sc_pos,
            egui::Align2::RIGHT_TOP,
            sc,
            egui::FontId::proportional(font_size - 1.0),
            shortcut_color,
        );
    }

    response
}

/// Renders a selectable menu bar item with an active indicator tick or checkmark.
pub fn selectable_menu_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    is_active: bool,
) -> egui::Response {
    let padding_x = 6.0;
    let icon_width = 18.0;
    let height = 22.0;

    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(MENU_ITEM_WIDTH, height), egui::Sense::click());

    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(4),
            egui::Color32::from_rgb(38, 44, 58),
        );
    }

    let (icon_color, text_color) = if is_active {
        (egui::Color32::from_rgb(0, 229, 255), egui::Color32::WHITE)
    } else if response.hovered() {
        (egui::Color32::WHITE, egui::Color32::WHITE)
    } else {
        (egui::Color32::from_gray(180), egui::Color32::from_gray(215))
    };

    // 1. Icon Column
    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x + padding_x, rect.min.y),
        egui::vec2(icon_width, height),
    );
    ui.painter().text(
        icon_rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(12.0),
        icon_color,
    );

    // 2. Text Label
    let text_pos = egui::pos2(
        rect.min.x + padding_x + icon_width + 6.0,
        rect.min.y + (height - 12.0) * 0.5 - 1.0,
    );
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(12.0),
        text_color,
    );

    // 3. Right Checkmark
    if is_active {
        let check_pos = egui::pos2(
            rect.max.x - padding_x - 2.0,
            rect.min.y + (height - 12.0) * 0.5 - 1.0,
        );
        ui.painter().text(
            check_pos,
            egui::Align2::RIGHT_TOP,
            "✓",
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgb(0, 229, 255),
        );
    }

    response
}