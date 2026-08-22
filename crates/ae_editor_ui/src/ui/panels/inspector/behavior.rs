// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular Inspector UI handlers for gameplay behavior components.
//!
//! Provides decoupled inspector cards and live property editing for:
//! - `Rotator` (speed, axis)
//! - `MovingPlatform` (speed, waypoints)
//! - `TriggerZone` (status, speed, target elevation)
//! - `DestructibleTarget` (health, max health, hit reaction)
//! - `CharacterAction` (weapon parameters)
//! - `PlayerTag` (player target marker)
//!

use crate::ui::EngineUiAction;
use ae_core::ecs::{
    CharacterAction, DestructibleTarget, MovingPlatform, PlayerTag, Rotator, TriggerZone,
};

/// UI handler for `Rotator` continuous angular rotation component.
pub struct RotatorUiHandler;

impl super::registry::ComponentUiHandler for RotatorUiHandler {
    fn component_name(&self) -> &'static str {
        "Rotator"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        ("Rotator", "🔄", egui::Color32::from_rgb(180, 130, 255))
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Gameplay", "Rotator")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&Rotator>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        if let Ok(rotator) = ctx.world.get::<&Rotator>(ctx.entity) {
            let mut updated = *rotator;
            let mut changed = false;

            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "Rotator",
                "🔄",
                egui::Color32::from_rgb(180, 130, 255),
                true,
                |ui| {
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
                },
            );

            if changed {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity, "Rotator", &updated,
                ));
            }
            if remove_clicked {
                self.remove_from_entity(ctx.entity, ctx.ui_actions);
            }
        }
    }
}

/// UI handler for `MovingPlatform` waypoint translation component.
pub struct MovingPlatformUiHandler;

impl super::registry::ComponentUiHandler for MovingPlatformUiHandler {
    fn component_name(&self) -> &'static str {
        "MovingPlatform"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "MovingPlatform",
            "🚡",
            egui::Color32::from_rgb(180, 130, 255),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Gameplay", "Moving Platform")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&MovingPlatform>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        if let Ok(platform) = ctx.world.get::<&MovingPlatform>(ctx.entity) {
            let mut updated = *platform;
            let mut changed = false;

            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "MovingPlatform",
                "🚡",
                egui::Color32::from_rgb(180, 130, 255),
                true,
                |ui| {
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
                        ui.label("Target Pos:");
                        ui.label("X:");
                        if ui
                            .add(egui::DragValue::new(&mut updated.target_position[0]).speed(0.1))
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label("Y:");
                        if ui
                            .add(egui::DragValue::new(&mut updated.target_position[1]).speed(0.1))
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label("Z:");
                        if ui
                            .add(egui::DragValue::new(&mut updated.target_position[2]).speed(0.1))
                            .changed()
                        {
                            changed = true;
                        }
                    });
                },
            );

            if changed {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity,
                    "MovingPlatform",
                    &updated,
                ));
            }
            if remove_clicked {
                self.remove_from_entity(ctx.entity, ctx.ui_actions);
            }
        }
    }
}

/// UI handler for `TriggerZone` proximity sensor and mechanism component.
pub struct TriggerZoneUiHandler;

impl super::registry::ComponentUiHandler for TriggerZoneUiHandler {
    fn component_name(&self) -> &'static str {
        "TriggerZone"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        ("TriggerZone", "⚡", egui::Color32::from_rgb(180, 130, 255))
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Gameplay", "Trigger Zone")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&TriggerZone>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        if let Ok(zone) = ctx.world.get::<&TriggerZone>(ctx.entity) {
            let mut updated = *zone;
            let mut changed = false;

            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "TriggerZone",
                "⚡",
                egui::Color32::from_rgb(180, 130, 255),
                true,
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Status:");
                        if updated.is_triggered {
                            ui.colored_label(egui::Color32::from_rgb(50, 230, 80), "● ACTIVATED");
                        } else {
                            ui.colored_label(egui::Color32::from_rgb(120, 130, 150), "○ IDLE");
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
                            .add(egui::DragValue::new(&mut updated.target_position[1]).speed(0.1))
                            .changed()
                        {
                            changed = true;
                        }
                    });
                },
            );

            if changed {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity,
                    "TriggerZone",
                    &updated,
                ));
            }
            if remove_clicked {
                self.remove_from_entity(ctx.entity, ctx.ui_actions);
            }
        }
    }
}

/// UI handler for `DestructibleTarget` health and damage response component.
pub struct DestructibleTargetUiHandler;

impl super::registry::ComponentUiHandler for DestructibleTargetUiHandler {
    fn component_name(&self) -> &'static str {
        "DestructibleTarget"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "DestructibleTarget",
            "🎯",
            egui::Color32::from_rgb(180, 130, 255),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Gameplay", "Destructible Target")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&DestructibleTarget>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        if let Ok(target) = ctx.world.get::<&DestructibleTarget>(ctx.entity) {
            let mut updated = *target;
            let mut changed = false;

            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "DestructibleTarget",
                "🎯",
                egui::Color32::from_rgb(180, 130, 255),
                true,
                |ui| {
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

                    let health_frac =
                        (updated.health / updated.max_health.max(1.0)).clamp(0.0, 1.0);
                    let bar_color = if health_frac > 0.5 {
                        egui::Color32::from_rgb(50, 200, 80)
                    } else if health_frac > 0.25 {
                        egui::Color32::from_rgb(230, 180, 30)
                    } else {
                        egui::Color32::from_rgb(230, 50, 50)
                    };

                    ui.horizontal(|ui| {
                        ui.label("Status:");
                        let bar_width = (ui.available_width() - 80.0).clamp(60.0, 240.0);
                        let text_color = if health_frac > 0.3 {
                            egui::Color32::BLACK
                        } else {
                            egui::Color32::WHITE
                        };
                        let progress_bar =
                            egui::ProgressBar::new(health_frac).fill(bar_color).text(
                                egui::RichText::new(format!("{:.0}%", health_frac * 100.0))
                                    .color(text_color)
                                    .strong(),
                            );
                        ui.add_sized([bar_width, 16.0], progress_bar);
                    });

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
                            updated.health = updated.health.min(updated.max_health);
                            changed = true;
                        }
                    });
                },
            );

            if changed {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity,
                    "DestructibleTarget",
                    &updated,
                ));
            }
            if remove_clicked {
                self.remove_from_entity(ctx.entity, ctx.ui_actions);
            }
        }
    }
}

/// UI handler for `CharacterAction` weapon raycast shooting component.
pub struct CharacterActionUiHandler;

impl super::registry::ComponentUiHandler for CharacterActionUiHandler {
    fn component_name(&self) -> &'static str {
        "CharacterAction"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "CharacterAction",
            "🔫",
            egui::Color32::from_rgb(180, 130, 255),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Gameplay", "Character Action")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&CharacterAction>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        if let Ok(action) = ctx.world.get::<&CharacterAction>(ctx.entity) {
            let mut updated = *action;
            let mut changed = false;

            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "CharacterAction",
                "🔫",
                egui::Color32::from_rgb(180, 130, 255),
                true,
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Speed / Range:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut updated.speed)
                                    .speed(1.0)
                                    .range(1.0..=500.0),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label("m/s");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Cooldown:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut updated.cooldown)
                                    .speed(0.05)
                                    .range(0.05..=5.0),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label("s");
                    });
                },
            );

            if changed {
                ctx.ui_actions.push(EngineUiAction::modify_component(
                    ctx.entity,
                    "CharacterAction",
                    &updated,
                ));
            }
            if remove_clicked {
                self.remove_from_entity(ctx.entity, ctx.ui_actions);
            }
        }
    }
}

/// UI handler for `PlayerTag` marker component.
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
        world.get::<&PlayerTag>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        if ctx.world.get::<&PlayerTag>(ctx.entity).is_ok() {
            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "PlayerTag",
                "🎮",
                egui::Color32::from_rgb(255, 180, 80),
                true,
                |ui| {
                    ui.label(
                        egui::RichText::new(
                            "Designates this entity as the active Player target for gameplay logic and camera tracking.",
                        )
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
}