// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Renders the "File" menu category under the top menu bar.
/// Exposes commands for starting new projects, importing/saving scenes, and initiating
/// safe engine shutdown via the Event Bus UI actions pipeline.
pub fn draw_file_menu(ui: &mut egui::Ui, ui_actions: &mut Vec<crate::ui::EngineUiAction>) {
    ui.menu_button("File", |ui| {
        ui.set_width(super::MENU_ITEM_WIDTH);

        if super::menu_item(ui, "📄", "New Project", Some("Ctrl N"), true).clicked() {
            // New project logic placeholder
            ui.close();
        }
        if super::menu_item(ui, "📂", "Load Scene", Some("Ctrl O"), true).clicked() {
            ui_actions.push(crate::ui::EngineUiAction::OpenLoadSceneDialog);
            ui.close();
        }
        ui.separator();
        if super::menu_item(ui, "💾", "Save Scene", Some("Ctrl S"), true).clicked() {
            ui_actions.push(crate::ui::EngineUiAction::SaveScene);
            ui.close();
        }
        if super::menu_item(ui, "💾", "Save Scene As", Some("Ctrl Shift S"), true).clicked() {
            ui_actions.push(crate::ui::EngineUiAction::OpenSaveSceneDialog);
            ui.close();
        }
        ui.separator();
        if super::menu_item(ui, "⏻", "Exit", Some("Alt F4"), true).clicked() {
            ui_actions.push(crate::ui::EngineUiAction::Exit);
            ui.close();
        }
    });
}