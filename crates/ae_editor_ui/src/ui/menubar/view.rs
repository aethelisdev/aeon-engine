// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::panel_layout::{PanelId, PanelLayoutState};

/// Renders the "View" menu category under the top menu bar.
/// Exposes toggles for screen layout adjustments, tool panel visibility, and layout reset.
pub(super) fn draw_view_menu(ui: &mut egui::Ui, layout_state: &mut PanelLayoutState) {
    ui.menu_button("View", |ui| {
        let _ = ui.button("Toggle Fullscreen");
        ui.separator();
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
}