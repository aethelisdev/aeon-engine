// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Renders the "View" menu category under the top menu bar.
/// Exposes toggles for screen layout adjustments, specifically controls
/// visibility of the bottom workspace panel.
pub(super) fn draw_view_menu(ui: &mut egui::Ui, show_workspace: &mut bool) {
    ui.menu_button("View", |ui| {
        let _ = ui.button("Toggle Fullscreen");
        ui.checkbox(show_workspace, "🖥 Workspace Panel");
    });
}