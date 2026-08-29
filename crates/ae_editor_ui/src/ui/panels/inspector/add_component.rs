// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Add Component Dropdown
//!
//! Categorized menu button for attaching new ECS components to the selected entity.

use crate::ui::EngineUiAction;

/// Renders dynamic "Add Component" dropdown menu automatically categorized via InspectorUiRegistry & ComponentRegistry.
pub fn draw_add_component_button(
    ui: &mut egui::Ui,
    world: &hecs::World,
    entity: hecs::Entity,
    ui_actions: &mut Vec<EngineUiAction>,
) {
    let ui_registry = super::registry::InspectorUiRegistry::global();
    let grouped = ui_registry.grouped_by_category();
    let mut handled_names = std::collections::HashSet::new();

    ui.menu_button("➕ Add Component", |ui| {
        for (category, handlers) in grouped {
            for h in &handlers {
                handled_names.insert(h.component_name());
            }

            let available: Vec<_> = handlers
                .into_iter()
                .filter(|h| !h.has_component(world, entity))
                .collect();

            if !available.is_empty() {
                ui.menu_button(category, |ui| {
                    for handler in available {
                        let (_, display_name) = handler.menu_category();
                        let (_, icon, _) = handler.card_header();
                        if ui.button(format!("{} {}", icon, display_name)).clicked() {
                            handler.add_default_to_entity(world, entity, ui_actions);
                            ui.close();
                        }
                    }
                });
            }
        }

        // Fallback for custom / dynamically registered components from ComponentRegistry
        let comp_registry = ae_core::registry::ComponentRegistry::global();
        let dynamic_available: Vec<_> = comp_registry
            .handlers()
            .iter()
            .filter(|h| {
                let name = h.type_name();
                !handled_names.contains(name)
                    && !super::dynamic_reflection::is_internal_or_specialized(name)
                    && !h.has_component(world, entity)
            })
            .collect();

        if !dynamic_available.is_empty() {
            ui.menu_button("Custom / Dynamic", |ui| {
                for handler in dynamic_available {
                    let name = handler.type_name();
                    if ui.button(format!("🧩 {}", name)).clicked() {
                        ui_actions.push(EngineUiAction::AddComponent(entity, name));
                        ui.close();
                    }
                }
            });
        }
    });
}