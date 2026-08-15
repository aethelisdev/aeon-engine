// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::EngineUi;

mod edit;
mod file;
pub(crate) mod help;
mod view;

impl EngineUi {
    /// Renders the clean top menu bar panel of the engine editor.
    /// Manages file options, history/preferences configuration, view settings,
    /// modular panel anchors, and fast engine edit/play status toggling.
    pub(super) fn draw_menu_bar(
        show_preferences: &mut bool,
        show_about: &mut bool,
        _should_save_scene: &mut bool,
        _should_load_scene: &mut bool,
        show_workspace: &mut bool,
        workspace_tab: &mut usize,
        show_left_panel: &mut bool,
        left_panel_tab: &mut usize,
        ui: &mut egui::Ui,
        _world: &hecs::World,
        _mode: &ae_core::modules::EngineMode,
        undo_stack: &[ae_editor::undo_redo::Command],
        redo_stack: &[ae_editor::undo_redo::Command],
        is_editing: bool,
        ui_actions: &mut Vec<crate::ui::EngineUiAction>,
    ) {
        egui::Panel::top("top_menu_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    // Draw modular submenus
                    file::draw_file_menu(ui, ui_actions);
                    edit::draw_edit_menu(ui, undo_stack, redo_stack, show_preferences, ui_actions);
                    view::draw_view_menu(ui, show_workspace);

                    // Window Menu
                    ui.menu_button("Window", |ui| {
                        if ui.button("🏗️ Hierarchy").clicked() {
                            *show_left_panel = true;
                            *left_panel_tab = 0;
                        }
                        if ui.button("📊 Stats & Profiler").clicked() {
                            *show_left_panel = true;
                            *left_panel_tab = 1;
                        }
                        ui.separator();
                        let _ = ui.button("⚙️ Inspector");
                        ui.separator();
                        if ui.button("📂 Assets").clicked() {
                            *show_workspace = true;
                            *workspace_tab = 0;
                        }
                        if ui.button("📜 Console").clicked() {
                            *show_workspace = true;
                            *workspace_tab = 1;
                        }
                        if ui.button("🎬 Timeline").clicked() {
                            *show_workspace = true;
                            *workspace_tab = 2;
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