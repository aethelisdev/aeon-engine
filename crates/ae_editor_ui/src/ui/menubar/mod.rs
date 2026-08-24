// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::EngineUi;
use crate::ui::panel_layout::{PanelId, PanelLayoutState};

mod edit;
mod file;
pub(crate) mod help;
mod view;

/// Parameters for drawing the top menu bar.
pub struct MenuBarDrawParams<'a> {
    pub show_preferences: &'a mut bool,
    pub show_about: &'a mut bool,
    pub layout_state: &'a mut PanelLayoutState,
    pub undo_stack: &'a [ae_editor::undo_redo::Command],
    pub redo_stack: &'a [ae_editor::undo_redo::Command],
    pub is_editing: bool,
    pub ui_actions: &'a mut Vec<crate::ui::EngineUiAction>,
}

impl EngineUi {
    /// Renders the clean top menu bar panel of the engine editor.
    /// Manages file options, history/preferences configuration, view settings,
    /// modular panel anchors, and fast engine edit/play status toggling.
    pub(super) fn draw_menu_bar(ui: &mut egui::Ui, params: MenuBarDrawParams<'_>) {
        let show_preferences = params.show_preferences;
        let show_about = params.show_about;
        let layout_state = params.layout_state;
        let undo_stack = params.undo_stack;
        let redo_stack = params.redo_stack;
        let is_editing = params.is_editing;
        let ui_actions = params.ui_actions;
        egui::Panel::top("top_menu_bar")
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(15, 15, 20))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 48, 60)))
                    .inner_margin(egui::Margin::symmetric(8, 4)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    egui::MenuBar::new().ui(ui, |ui| {
                        // Draw modular submenus
                        file::draw_file_menu(ui, ui_actions);
                        edit::draw_edit_menu(
                            ui,
                            undo_stack,
                            redo_stack,
                            show_preferences,
                            ui_actions,
                        );
                        view::draw_view_menu(ui, layout_state);

                        // Window Menu (Modular Panels & Reordering)
                        ui.menu_button("Window", |ui| {
                            ui.set_width(MENU_ITEM_WIDTH);
                            for &panel in PanelId::all_tool_panels() {
                                let is_open = layout_state.is_panel_visible(panel);
                                if selectable_menu_item(ui, panel.icon(), panel.title(), is_open)
                                    .clicked()
                                {
                                    layout_state.activate_or_open(panel);
                                    ui.close();
                                }
                            }
                            ui.separator();
                            if menu_item(ui, "🔄", "Reset Layout to Default", None, true).clicked()
                            {
                                layout_state.reset_to_default();
                                ui.close();
                            }
                        });

                        // Help Menu
                        ui.menu_button("Help", |ui| {
                            ui.set_width(MENU_ITEM_WIDTH);
                            if menu_item(ui, "ℹ", "About Aeon Engine", Some("F1"), true).clicked()
                            {
                                *show_about = true;
                                ui.close();
                            }
                        });
                    });

                    // Engine Mode Controls (Play/Stop button on the right)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_editing {
                            let play_btn = egui::Button::new(
                                egui::RichText::new("▶ Play")
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            )
                            .fill(egui::Color32::from_rgb(34, 139, 34))
                            .small();

                            if ui.add(play_btn).clicked() {
                                ui_actions.push(crate::ui::EngineUiAction::ChangeMode(
                                    ae_core::modules::EngineMode::Play,
                                ));
                            }
                        } else {
                            let stop_btn = egui::Button::new(
                                egui::RichText::new("⏹ Stop")
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            )
                            .fill(egui::Color32::from_rgb(180, 40, 40))
                            .small();

                            if ui.add(stop_btn).clicked() {
                                ui_actions.push(crate::ui::EngineUiAction::ChangeMode(
                                    ae_core::modules::EngineMode::Edit,
                                ));
                            }
                        }
                    });
                });
            });
    }
}

/// Standard menu item width for all top menu bar dropdowns.
pub const MENU_ITEM_WIDTH: f32 = 205.0;

/// Renders a neatly formatted menu bar item with an icon column, text label, and right-aligned shortcut hint.
pub(super) fn menu_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    shortcut: Option<&str>,
    enabled: bool,
) -> egui::Response {
    let padding_x = 6.0;
    let icon_width = 18.0;
    let height = 22.0;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(MENU_ITEM_WIDTH, height),
        if enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );

    if enabled && response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(4),
            egui::Color32::from_rgb(38, 44, 58),
        );
    }

    let (icon_color, text_color, shortcut_color) = if !enabled {
        (
            egui::Color32::from_gray(90),
            egui::Color32::from_gray(100),
            egui::Color32::from_gray(75),
        )
    } else if response.hovered() {
        (
            egui::Color32::WHITE,
            egui::Color32::WHITE,
            egui::Color32::from_gray(190),
        )
    } else {
        (
            egui::Color32::from_gray(180),
            egui::Color32::from_gray(215),
            egui::Color32::from_gray(130),
        )
    };

    // 1. Fixed Icon Column (18px)
    if !icon.is_empty() {
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
    }

    // 2. Text Label (Starts at X = min.x + padding + icon_width + 6)
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

    // 3. Right-Aligned Shortcut Hint
    if let Some(sc) = shortcut {
        let sc_pos = egui::pos2(
            rect.max.x - padding_x - 2.0,
            rect.min.y + (height - 12.0) * 0.5 - 1.0,
        );
        ui.painter().text(
            sc_pos,
            egui::Align2::RIGHT_TOP,
            sc,
            egui::FontId::proportional(11.0),
            shortcut_color,
        );
    }

    response
}

/// Renders a toggleable menu bar item with an active checkmark indicator on the right.
pub(super) fn selectable_menu_item(
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