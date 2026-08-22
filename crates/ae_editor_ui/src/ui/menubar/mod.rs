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
                            for &panel in PanelId::all_tool_panels() {
                                let is_open = layout_state.is_panel_visible(panel);
                                let label = format!("{} {}", panel.icon(), panel.title());
                                if ui.selectable_label(is_open, label).clicked() {
                                    layout_state.activate_or_open(panel);
                                }
                            }
                            ui.separator();
                            if ui.button("🔄 Reset Layout to Default").clicked() {
                                layout_state.reset_to_default();
                            }
                        });

                        // Help Menu
                        ui.menu_button("Help", |ui| {
                            if ui.button("About Aeon Engine").clicked() {
                                *show_about = true;
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