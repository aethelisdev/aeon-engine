// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Renders the "Edit" menu category under the top menu bar.
/// Exposes historical commands (Undo/Redo command stacks) and triggers the
/// global preferences modal dialog visibility state.
pub fn draw_edit_menu(
    ui: &mut egui::Ui,
    undo_stack: &[ae_editor::undo_redo::Command],
    redo_stack: &[ae_editor::undo_redo::Command],
    show_preferences: &mut bool,
    ui_actions: &mut Vec<crate::ui::EngineUiAction>,
) {
    ui.menu_button("Edit", |ui| {
        ui.set_width(super::MENU_ITEM_WIDTH);

        if super::menu_item(ui, "↩", "Undo", Some("Ctrl Z"), !undo_stack.is_empty()).clicked() {
            ui_actions.push(crate::ui::EngineUiAction::Undo);
            ui.close();
        }
        if super::menu_item(ui, "↪", "Redo", Some("Ctrl Y"), !redo_stack.is_empty()).clicked() {
            ui_actions.push(crate::ui::EngineUiAction::Redo);
            ui.close();
        }
        ui.separator();
        if super::menu_item(ui, "⚙", "Preferences", Some("Ctrl ,"), true).clicked() {
            *show_preferences = true;
            ui.close();
        }
    });
}