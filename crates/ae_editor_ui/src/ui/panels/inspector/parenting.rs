// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::EngineUi;
use crate::ui::EngineUiAction;

impl EngineUi {
    /// Draws the Hierarchy / Parenting relationship editor panel inside the Inspector.
    /// Evaluates parenting compatibility dynamically inside the combo box's lazy show closure
    /// to avoid O(N) traversal overhead on regular frame drawing when the popup is closed.
    pub(super) fn draw_parenting_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        super::widgets::draw_inspector_card(
            ui,
            "Parent / Child Relationship",
            "🔗",
            egui::Color32::WHITE,
            false,
            |ui| {
                let parent_entity = world.get::<&ae_core::ecs::Parent>(entity).ok().map(|p| p.0);

                if let Some(parent) = parent_entity {
                    let parent_name = world
                        .get::<&ae_core::ecs::Name>(parent)
                        .map(|n| n.0.clone())
                        .unwrap_or_else(|_| format!("Entity {:?}", parent));

                    ui.horizontal(|ui| {
                        ui.label(format!("Parent: {}", parent_name));
                        if ui.button("❌ Unparent").clicked() {
                            ui_actions.push(EngineUiAction::UnparentEntity(entity));
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Parent: None (Root)");

                        // Helper function to check if candidate is a descendant of potential_ancestor to prevent cycles
                        fn is_descendant(
                            world: &hecs::World,
                            candidate: hecs::Entity,
                            potential_ancestor: hecs::Entity,
                        ) -> bool {
                            let mut curr = candidate;
                            while let Ok(parent_ref) = world.get::<&ae_core::ecs::Parent>(curr) {
                                if parent_ref.0 == potential_ancestor {
                                    return true;
                                }
                                curr = parent_ref.0;
                            }
                            false
                        }

                        #[allow(deprecated)]
                        egui::ComboBox::from_id_salt(egui::Id::new(("parent_combo", entity)))
                            .selected_text("Set Parent...")
                            .show_ui(ui, |ui| {
                                let mut count = 0;
                                for ent_ref in world.iter() {
                                    if count >= 50 {
                                        break;
                                    }
                                    let candidate = ent_ref.entity();
                                    if candidate != entity
                                        && !is_descendant(world, candidate, entity)
                                        && let Ok(name) =
                                            world.get::<&ae_core::ecs::Name>(candidate)
                                    {
                                        let name_str = name.0.clone();
                                        count += 1;
                                        if ui.button(name_str).clicked() {
                                            ui_actions.push(EngineUiAction::ParentEntity(
                                                entity, candidate,
                                            ));
                                            ui.close();
                                        }
                                    }
                                }
                                if count == 0 {
                                    ui.label("No candidate parents.");
                                }
                            });
                    });
                }
            },
        );
    }
}

pub struct ParentingUiHandler;

impl super::registry::ComponentUiHandler for ParentingUiHandler {
    fn component_name(&self) -> &'static str {
        "Parenting"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        ("Parent / Child Relationship", "🔗", egui::Color32::WHITE)
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Hierarchy", "Parenting")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Parent>(entity).is_ok()
            || world.get::<&ae_core::ecs::Children>(entity).is_ok()
    }

    fn is_removable(&self) -> bool {
        false
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        EngineUi::draw_parenting_section(ui, ctx.world, ctx.entity, ctx.ui_actions);
    }

    fn add_default_to_entity(
        &self,
        _world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        ui_actions.push(EngineUiAction::AddComponent(entity, "Children"));
    }

    fn remove_from_entity(&self, entity: hecs::Entity, ui_actions: &mut Vec<EngineUiAction>) {
        ui_actions.push(EngineUiAction::UnparentEntity(entity));
    }
}