// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Inspector section card for inspecting and live-editing gameplay BehaviorComponent properties.
//!

use crate::ui::{EngineUi, EngineUiAction};
use ae_core::ecs::{BehaviorComponent, BehaviorType};

impl EngineUi {
    /// Renders the BehaviorComponent inspector panel section.
    pub(super) fn draw_behavior_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        if let Ok(behavior) = world.get::<&BehaviorComponent>(entity) {
            let mut updated = (*behavior).clone();
            let mut changed = false;

            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "Behavior",
                "🧠",
                egui::Color32::from_rgb(180, 130, 255),
                true,
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        let prev_type = updated.behavior_type;
                        egui::ComboBox::from_id_salt(("behavior_type_combo", entity))
                            .selected_text(match updated.behavior_type {
                                BehaviorType::Rotator => "Rotator (Spin)",
                                BehaviorType::TriggerZone => "Trigger Zone (Proximity)",
                                BehaviorType::DestructibleTarget => "Destructible Target (Health)",
                                BehaviorType::MovingPlatform => "Moving Platform (Waypoints)",
                                BehaviorType::CharacterAction => "Character Action (Weapon)",
                                BehaviorType::Custom => "Custom Script",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut updated.behavior_type,
                                    BehaviorType::Rotator,
                                    "Rotator (Spin)",
                                );
                                ui.selectable_value(
                                    &mut updated.behavior_type,
                                    BehaviorType::TriggerZone,
                                    "Trigger Zone (Proximity)",
                                );
                                ui.selectable_value(
                                    &mut updated.behavior_type,
                                    BehaviorType::DestructibleTarget,
                                    "Destructible Target (Health)",
                                );
                                ui.selectable_value(
                                    &mut updated.behavior_type,
                                    BehaviorType::MovingPlatform,
                                    "Moving Platform (Waypoints)",
                                );
                                ui.selectable_value(
                                    &mut updated.behavior_type,
                                    BehaviorType::CharacterAction,
                                    "Character Action (Weapon)",
                                );
                                ui.selectable_value(
                                    &mut updated.behavior_type,
                                    BehaviorType::Custom,
                                    "Custom Script",
                                );
                            });

                        if updated.behavior_type != prev_type {
                            changed = true;
                        }
                    });

                    ui.separator();

                    match updated.behavior_type {
                        BehaviorType::Rotator => {
                            ui.horizontal(|ui| {
                                ui.label("Speed:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut updated.speed)
                                            .speed(0.1)
                                            .range(-20.0..=20.0),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                                ui.label("rad/s");
                            });

                            ui.horizontal(|ui| {
                                ui.label("Axis:");
                                ui.label("X:");
                                if ui
                                    .add(egui::DragValue::new(&mut updated.axis[0]).speed(0.05))
                                    .changed()
                                {
                                    changed = true;
                                }
                                ui.label("Y:");
                                if ui
                                    .add(egui::DragValue::new(&mut updated.axis[1]).speed(0.05))
                                    .changed()
                                {
                                    changed = true;
                                }
                                ui.label("Z:");
                                if ui
                                    .add(egui::DragValue::new(&mut updated.axis[2]).speed(0.05))
                                    .changed()
                                {
                                    changed = true;
                                }
                            });
                        }
                        BehaviorType::TriggerZone => {
                            ui.horizontal(|ui| {
                                ui.label("Status:");
                                if updated.is_triggered {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(50, 230, 80),
                                        "● ACTIVATED",
                                    );
                                } else {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(120, 130, 150),
                                        "○ IDLE",
                                    );
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Speed:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut updated.speed)
                                            .speed(0.1)
                                            .range(0.1..=50.0),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Target Y:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut updated.target_position[1])
                                            .speed(0.1),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            });
                        }
                        BehaviorType::DestructibleTarget => {
                            ui.horizontal(|ui| {
                                ui.label("Health:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut updated.health)
                                            .speed(1.0)
                                            .range(0.0..=updated.max_health),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                                ui.label(format!("/ {:.0}", updated.max_health));
                            });

                            let health_fraction =
                                (updated.health / updated.max_health.max(1.0)).clamp(0.0, 1.0);
                            let health_bar = egui::ProgressBar::new(health_fraction)
                                .desired_width(ui.available_width().min(240.0))
                                .show_percentage();
                            ui.add(health_bar);

                            ui.horizontal(|ui| {
                                ui.label("Max HP:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut updated.max_health)
                                            .speed(5.0)
                                            .range(1.0..=10000.0),
                                    )
                                    .changed()
                                {
                                    if updated.health > updated.max_health {
                                        updated.health = updated.max_health;
                                    }
                                    changed = true;
                                }
                            });
                        }
                        BehaviorType::MovingPlatform => {
                            ui.horizontal(|ui| {
                                ui.label("Speed:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut updated.speed)
                                            .speed(0.1)
                                            .range(0.1..=50.0),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                                ui.label("m/s");
                            });

                            ui.horizontal(|ui| {
                                ui.label("Direction:");
                                if updated.ping_pong_forward {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(100, 200, 255),
                                        "➔ Moving to Target",
                                    );
                                } else {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(255, 180, 80),
                                        "⬅ Returning to Origin",
                                    );
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Target Pos:");
                                ui.label("X:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut updated.target_position[0])
                                            .speed(0.2),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                                ui.label("Y:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut updated.target_position[1])
                                            .speed(0.2),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                                ui.label("Z:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut updated.target_position[2])
                                            .speed(0.2),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            });
                        }
                        BehaviorType::CharacterAction => {
                            ui.label("🎯 Action: WASD Move, Shift Sprint, Space Jump");
                            ui.label("🔫 Weapon: Left Mouse / Key 'F' Raycast Shoot");
                            ui.label("🚪 Interaction: Key 'E' Trigger Proximity");
                        }
                        BehaviorType::Custom => {
                            ui.label("Custom dynamic gameplay plugin behavior.");
                        }
                    }
                },
            );

            if remove_clicked {
                ui_actions.push(EngineUiAction::RemoveBehavior(entity));
            }

            if changed {
                ui_actions.push(EngineUiAction::ModifyBehavior(entity, updated));
            }
        }
    }
}

pub struct BehaviorUiHandler;

impl super::registry::ComponentUiHandler for BehaviorUiHandler {
    fn component_name(&self) -> &'static str {
        "Behavior"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        ("Behavior", "🧠", egui::Color32::from_rgb(180, 130, 255))
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Gameplay", "Behavior")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&BehaviorComponent>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        EngineUi::draw_behavior_section(ui, ctx.world, ctx.entity, ctx.ui_actions);
    }

    fn add_default_to_entity(
        &self,
        _world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        ui_actions.push(EngineUiAction::AddBehavior(
            entity,
            BehaviorComponent::default(),
        ));
    }

    fn remove_from_entity(&self, entity: hecs::Entity, ui_actions: &mut Vec<EngineUiAction>) {
        ui_actions.push(EngineUiAction::RemoveBehavior(entity));
    }
}

pub struct PlayerTagUiHandler;

impl super::registry::ComponentUiHandler for PlayerTagUiHandler {
    fn component_name(&self) -> &'static str {
        "PlayerTag"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        ("PlayerTag", "🎮", egui::Color32::from_rgb(255, 180, 80))
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Gameplay", "Player Tag")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::PlayerTag>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        if ctx
            .world
            .get::<&ae_core::ecs::PlayerTag>(ctx.entity)
            .is_ok()
        {
            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "PlayerTag",
                "🎮",
                egui::Color32::from_rgb(255, 180, 80),
                true,
                |ui| {
                    ui.label(
                        egui::RichText::new("Designates this entity as the active Player target for gameplay logic and camera tracking.")
                            .small()
                            .color(egui::Color32::from_gray(170)),
                    );
                },
            );
            if remove_clicked {
                self.remove_from_entity(ctx.entity, ctx.ui_actions);
            }
        }
    }

    fn add_default_to_entity(
        &self,
        _world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        ui_actions.push(EngineUiAction::AddPlayerTag(entity));
    }

    fn remove_from_entity(&self, entity: hecs::Entity, ui_actions: &mut Vec<EngineUiAction>) {
        ui_actions.push(EngineUiAction::RemovePlayerTag(entity));
    }
}