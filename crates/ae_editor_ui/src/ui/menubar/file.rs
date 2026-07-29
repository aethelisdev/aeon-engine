// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Renders the "File" menu category under the top menu bar.
/// Exposes commands for starting new projects, importing/saving scenes, and initiating
/// safe engine shutdown via the Event Bus UI actions pipeline.
pub(super) fn draw_file_menu(ui: &mut egui::Ui, ui_actions: &mut Vec<crate::ui::EngineUiAction>) {
    ui.menu_button("File", |ui| {
        if ui.button("New Project").clicked() {
            // New project logic placeholder
        }
        if ui.button("Load Scene").clicked() {
            ui_actions.push(crate::ui::EngineUiAction::OpenLoadSceneDialog);
        }
        ui.separator();
        if ui.button("Save Scene").clicked() {
            ui_actions.push(crate::ui::EngineUiAction::SaveScene);
        }
        if ui.button("Save Scene As...").clicked() {
            ui_actions.push(crate::ui::EngineUiAction::OpenSaveSceneDialog);
        }
        ui.separator();
        if ui.button("Exit").clicked() {
            ui_actions.push(crate::ui::EngineUiAction::Exit);
        }
    });
}