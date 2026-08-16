// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::panel_layout::PanelLayoutState;

/// Renders the "View" menu category under the top menu bar.
/// Exposes toggles for screen layout adjustments and docking zone controls.
pub(super) fn draw_view_menu(ui: &mut egui::Ui, layout_state: &mut PanelLayoutState) {
    ui.menu_button("View", |ui| {
        let _ = ui.button("Toggle Fullscreen");
        ui.separator();
        ui.checkbox(&mut layout_state.show_left_panel, "◀ Left Dock Panel");
        ui.checkbox(&mut layout_state.show_right_panel, "▶ Right Dock Panel");
        ui.checkbox(&mut layout_state.show_bottom_panel, "▼ Bottom Dock Panel");
        ui.separator();
        if ui.button("🔄 Reset Layout to Default").clicked() {
            layout_state.reset_to_default();
        }
    });
}