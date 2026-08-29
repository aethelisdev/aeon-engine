// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::panel_layout::{PanelId, PanelLayoutState};

/// Renders the "View" menu category under the top menu bar.
/// Exposes toggles for screen layout adjustments, tool panel visibility, and layout reset.
pub fn draw_view_menu(ui: &mut egui::Ui, layout_state: &mut PanelLayoutState) {
    ui.menu_button("View", |ui| {
        ui.set_width(super::MENU_ITEM_WIDTH);

        if super::menu_item(ui, "⛶", "Toggle Fullscreen", Some("F11"), true).clicked() {
            // Fullscreen toggle
            ui.close();
        }
        ui.separator();
        for &panel in PanelId::all_tool_panels() {
            let is_open = layout_state.is_panel_visible(panel);
            if super::selectable_menu_item(ui, panel.icon(), panel.title(), is_open).clicked() {
                layout_state.activate_or_open(panel);
                ui.close();
            }
        }
        ui.separator();
        if super::menu_item(ui, "🔄", "Reset Layout to Default", None, true).clicked() {
            layout_state.reset_to_default();
            ui.close();
        }
    });
}