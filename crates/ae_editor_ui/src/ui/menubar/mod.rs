// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::EngineUi;
use crate::ui::panel_layout::{PanelId, PanelLayoutState};

mod edit;
mod file;
pub(crate) mod help;
mod view;

impl EngineUi {
    /// Renders the clean top menu bar panel of the engine editor.
    /// Manages file options, history/preferences configuration, view settings,
    /// modular panel anchors, and fast engine edit/play status toggling.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_menu_bar(
        show_preferences: &mut bool,
        show_about: &mut bool,
        _should_save_scene: &mut bool,
        _should_load_scene: &mut bool,
        layout_state: &mut PanelLayoutState,
        ui: &mut egui::Ui,
        _world: &hecs::World,
        _mode: &ae_core::modules::EngineMode,
        undo_stack: &[ae_editor::undo_redo::Command],
        redo_stack: &[ae_editor::undo_redo::Command],
        is_editing: bool,
        ui_actions: &mut Vec<crate::ui::EngineUiAction>,
    ) {
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