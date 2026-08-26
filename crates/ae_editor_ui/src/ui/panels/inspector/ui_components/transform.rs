// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Inspector UI Handler for 2D Screen Transform and Anchoring (`UiElement`).
//!

use crate::ui::panels::inspector::registry::{ComponentUiHandler, InspectorContext};
use crate::ui::types::EngineUiAction;
use ae_core::ecs::{UiAnchor, UiElement};

/// UI Handler for `UiElement` 2D anchoring and screen layout.
pub struct UiElementUiHandler;

impl ComponentUiHandler for UiElementUiHandler {
    fn component_name(&self) -> &'static str {
        "UiElement"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "2D Screen Transform",
            "📐",
            egui::Color32::from_rgb(45, 120, 220),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("UI & HUD", "2D Screen Transform")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&UiElement>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut InspectorContext) {
        if let Ok(elem) = ctx.world.get::<&UiElement>(ctx.entity) {
            let mut new_elem = *elem;
            let mut changed = false;

            // Anchor Dropdown
            let anchor_labels = [
                "Top-Left",
                "Top-Center",
                "Top-Right",
                "Center-Left",
                "Center",
                "Center-Right",
                "Bottom-Left",
                "Bottom-Center",
                "Bottom-Right",
            ];
            let current_anchor_idx = match new_elem.anchor {
                UiAnchor::TopLeft => 0,
                UiAnchor::TopCenter => 1,
                UiAnchor::TopRight => 2,
                UiAnchor::CenterLeft => 3,
                UiAnchor::Center => 4,
                UiAnchor::CenterRight => 5,
                UiAnchor::BottomLeft => 6,
                UiAnchor::BottomCenter => 7,
                UiAnchor::BottomRight => 8,
            };
            let mut selected_anchor = current_anchor_idx;

            ui.horizontal(|ui| {
                ui.label("Anchor:");
                egui::ComboBox::from_id_salt("ui_element_anchor")
                    .selected_text(anchor_labels[selected_anchor])
                    .show_ui(ui, |ui| {
                        for (i, label) in anchor_labels.iter().enumerate() {
                            if ui
                                .selectable_value(&mut selected_anchor, i, *label)
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
            });

            if selected_anchor != current_anchor_idx {
                new_elem.anchor = match selected_anchor {
                    0 => UiAnchor::TopLeft,
                    1 => UiAnchor::TopCenter,
                    2 => UiAnchor::TopRight,
                    3 => UiAnchor::CenterLeft,
                    4 => UiAnchor::Center,
                    5 => UiAnchor::CenterRight,
                    6 => UiAnchor::BottomLeft,
                    7 => UiAnchor::BottomCenter,
                    _ => UiAnchor::BottomRight,
                };
                changed = true;
            }

            // Offset
            ui.horizontal(|ui| {
                ui.label("Offset:");
                ui.label("X:");
                if ui
                    .add(egui::DragValue::new(&mut new_elem.offset[0]).speed(1.0))
                    .changed()
                {
                    changed = true;
                }
                ui.label("Y:");
                if ui
                    .add(egui::DragValue::new(&mut new_elem.offset[1]).speed(1.0))
                    .changed()
                {
                    changed = true;
                }
            });

            // Size
            ui.horizontal(|ui| {
                ui.label("Size:");
                ui.label("W:");
                if ui
                    .add(
                        egui::DragValue::new(&mut new_elem.size[0])
                            .speed(1.0)
                            .range(0.0..=4096.0),
                    )
                    .changed()
                {
                    changed = true;
                }
                ui.label("H:");
                if ui
                    .add(
                        egui::DragValue::new(&mut new_elem.size[1])
                            .speed(1.0)
                            .range(0.0..=4096.0),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            // Layer / Z-Index & Alpha
            ui.horizontal(|ui| {
                ui.label("Z-Index:");
                if ui
                    .add(egui::DragValue::new(&mut new_elem.z_index).speed(1))
                    .changed()
                {
                    changed = true;
                }
                ui.label("Alpha:");
                if ui
                    .add(
                        egui::DragValue::new(&mut new_elem.alpha)
                            .speed(0.02)
                            .range(0.0..=1.0),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            // Visibility
            if ui.checkbox(&mut new_elem.visible, "Visible").changed() {
                changed = true;
            }

            if changed && new_elem != *elem {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity,
                    "UiElement",
                    &new_elem,
                ));
            }
        }
    }
}