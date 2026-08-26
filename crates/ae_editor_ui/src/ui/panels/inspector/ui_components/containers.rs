// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Inspector UI Handlers for Panel and Auto-Layout Containers (`UiPanel`, `UiLayoutGroup`).
//!

use crate::ui::panels::inspector::registry::{ComponentUiHandler, InspectorContext};
use crate::ui::types::EngineUiAction;
use ae_core::ecs::{UiLayoutGroup, UiLayoutType, UiPanel};

/// UI Handler for `UiPanel` styling and border settings.
pub struct UiPanelUiHandler;

impl ComponentUiHandler for UiPanelUiHandler {
    fn component_name(&self) -> &'static str {
        "UiPanel"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Panel Container",
            "🟩",
            egui::Color32::from_rgb(60, 160, 90),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "Panel Container")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&UiPanel>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut InspectorContext) {
        if let Ok(panel) = ctx.world.get::<&UiPanel>(ctx.entity) {
            let mut new_panel = *panel;
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Background:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_panel.background_color)
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Border Color:");
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut new_panel.border_color)
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Border Width:");
                if ui
                    .add(
                        egui::DragValue::new(&mut new_panel.border_width)
                            .speed(0.1)
                            .range(0.0..=20.0),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Corner Radius:");
                if ui
                    .add(
                        egui::DragValue::new(&mut new_panel.corner_radius)
                            .speed(0.5)
                            .range(0.0..=50.0),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            if changed && new_panel != *panel {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity, "UiPanel", &new_panel,
                ));
            }
        }
    }
}

/// UI Handler for `UiLayoutGroup` auto-layout containers.
pub struct UiLayoutGroupUiHandler;

impl ComponentUiHandler for UiLayoutGroupUiHandler {
    fn component_name(&self) -> &'static str {
        "UiLayoutGroup"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Auto Layout Group",
            "📐",
            egui::Color32::from_rgb(140, 100, 220),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "Auto Layout Group")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&UiLayoutGroup>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut InspectorContext) {
        if let Ok(layout) = ctx.world.get::<&UiLayoutGroup>(ctx.entity) {
            let mut new_layout = *layout;
            let mut changed = false;

            let layout_labels = ["Horizontal", "Vertical", "Grid"];
            let current_idx = match new_layout.layout_type {
                UiLayoutType::Horizontal => 0,
                UiLayoutType::Vertical => 1,
                UiLayoutType::Grid => 2,
            };
            let mut selected = current_idx;

            ui.horizontal(|ui| {
                ui.label("Flow Type:");
                egui::ComboBox::from_id_salt("ui_layout_type")
                    .selected_text(layout_labels[selected])
                    .show_ui(ui, |ui| {
                        for (i, label) in layout_labels.iter().enumerate() {
                            if ui.selectable_value(&mut selected, i, *label).changed() {
                                changed = true;
                            }
                        }
                    });
            });

            if selected != current_idx {
                new_layout.layout_type = match selected {
                    0 => UiLayoutType::Horizontal,
                    1 => UiLayoutType::Vertical,
                    _ => UiLayoutType::Grid,
                };
                changed = true;
            }

            ui.horizontal(|ui| {
                ui.label("Spacing:");
                if ui
                    .add(
                        egui::DragValue::new(&mut new_layout.spacing)
                            .speed(0.5)
                            .range(0.0..=100.0),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            if changed && new_layout != *layout {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity,
                    "UiLayoutGroup",
                    &new_layout,
                ));
            }
        }
    }
}