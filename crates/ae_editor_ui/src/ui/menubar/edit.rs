// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Renders the "Edit" menu category under the top menu bar.
/// Exposes historical commands (Undo/Redo command stacks) and triggers the
/// global preferences modal dialog visibility state.
pub(super) fn draw_edit_menu(
    ui: &mut egui::Ui,
    undo_stack: &[ae_editor::undo_redo::Command],
    redo_stack: &[ae_editor::undo_redo::Command],
    show_preferences: &mut bool,
    ui_actions: &mut Vec<crate::ui::EngineUiAction>,
) {
    ui.menu_button("Edit", |ui| {
        if ui
            .add_enabled(!undo_stack.is_empty(), egui::Button::new("Undo"))
            .clicked()
        {
            ui_actions.push(crate::ui::EngineUiAction::Undo);
        }
        if ui
            .add_enabled(!redo_stack.is_empty(), egui::Button::new("Redo"))
            .clicked()
        {
            ui_actions.push(crate::ui::EngineUiAction::Redo);
        }
        ui.separator();
        if ui.button("Preferences").clicked() {
            *show_preferences = true;
        }
    });
}