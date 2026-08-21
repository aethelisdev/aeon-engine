// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::EngineUi;
use crate::ui::EngineUiAction;

impl EngineUi {
    /// Draws the RigidBody component section if the entity has one.
    pub(super) fn draw_rigidbody_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        if let Ok(rb) = world.get::<&ae_core::ecs::RigidBody>(entity) {
            let mut body_type = rb.body_type;
            let mut mass = rb.mass;
            let mut gravity_scale = rb.gravity_scale;

            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "RigidBody",
                "⚙",
                egui::Color32::from_rgb(100, 200, 255),
                true,
                |ui| {
                    let type_labels = ["Static", "Dynamic", "Kinematic"];
                    let type_idx = match body_type {
                        ae_core::ecs::RigidBodyType::Static => 0,
                        ae_core::ecs::RigidBodyType::Dynamic => 1,
                        ae_core::ecs::RigidBodyType::Kinematic => 2,
                    };
                    let mut selected = type_idx;
                    #[allow(deprecated)]
                    egui::ComboBox::from_id_salt("rb_type_combo")
                        .selected_text(type_labels[selected])
                        .show_ui(ui, |ui| {
                            for (i, label) in type_labels.iter().enumerate() {
                                ui.selectable_value(&mut selected, i, *label);
                            }
                        });
                    if selected != type_idx {
                        body_type = match selected {
                            0 => ae_core::ecs::RigidBodyType::Static,
                            1 => ae_core::ecs::RigidBodyType::Dynamic,
                            _ => ae_core::ecs::RigidBodyType::Kinematic,
                        };
                    }

                    ui.horizontal(|ui| {
                        ui.label("Mass:");
                        ui.add(
                            egui::DragValue::new(&mut mass)
                                .speed(0.1)
                                .range(0.001..=10000.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Gravity:");
                        ui.add(
                            egui::DragValue::new(&mut gravity_scale)
                                .speed(0.05)
                                .range(-10.0..=10.0),
                        );
                    });
                },
            );

            if remove_clicked {
                ui_actions.push(EngineUiAction::RemoveRigidBody(entity));
            }

            let new_rb = ae_core::ecs::RigidBody {
                body_type,
                mass,
                gravity_scale,
            };
            if new_rb != *rb {
                ui_actions.push(EngineUiAction::ModifyRigidBody(entity, new_rb));
            }
        }
    }

    /// Draws the Collider component section if the entity has one.
    pub(super) fn draw_collider_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        if let Ok(col) = world.get::<&ae_core::ecs::Collider>(entity) {
            let mut shape = col.shape;
            let mut friction = col.friction;
            let mut restitution = col.restitution;
            let mut is_sensor = col.is_sensor;

            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "Collider",
                "🛡",
                egui::Color32::from_rgb(100, 255, 150),
                true,
                |ui| {
                    ui.checkbox(&mut is_sensor, "Is Trigger (Sensor)").on_hover_text(
                        "If checked, this collider detects overlaps without physical collision response",
                    );
                    ui.separator();

                    let shape_name = match shape {
                        ae_core::ecs::ColliderShape::Box { .. } => "Box",
                        ae_core::ecs::ColliderShape::Sphere { .. } => "Sphere",
                        ae_core::ecs::ColliderShape::Capsule { .. } => "Capsule",
                        ae_core::ecs::ColliderShape::Trimesh => "Trimesh",
                        ae_core::ecs::ColliderShape::ConvexHull => "ConvexHull",
                    };
                    let mut selected_idx = match shape {
                        ae_core::ecs::ColliderShape::Box { .. } => 0,
                        ae_core::ecs::ColliderShape::Sphere { .. } => 1,
                        ae_core::ecs::ColliderShape::Capsule { .. } => 2,
                        ae_core::ecs::ColliderShape::Trimesh => 3,
                        ae_core::ecs::ColliderShape::ConvexHull => 4,
                    };
                    let shape_labels = ["Box", "Sphere", "Capsule", "Trimesh", "ConvexHull"];
                    #[allow(deprecated)]
                    egui::ComboBox::from_id_salt("col_shape_combo")
                        .selected_text(shape_name)
                        .show_ui(ui, |ui| {
                            for (i, label) in shape_labels.iter().enumerate() {
                                ui.selectable_value(&mut selected_idx, i, *label);
                            }
                        });

                    match selected_idx {
                        0 => {
                            let mut he = match shape {
                                ae_core::ecs::ColliderShape::Box { half_extents } => half_extents,
                                _ => [0.5, 0.5, 0.5],
                            };
                            ui.horizontal(|ui| {
                                ui.label("Half Extents:");
                                ui.add(
                                    egui::DragValue::new(&mut he[0])
                                        .prefix("X:")
                                        .speed(0.02)
                                        .range(0.01..=100.0),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut he[1])
                                        .prefix("Y:")
                                        .speed(0.02)
                                        .range(0.01..=100.0),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut he[2])
                                        .prefix("Z:")
                                        .speed(0.02)
                                        .range(0.01..=100.0),
                                );
                            });
                            shape = ae_core::ecs::ColliderShape::Box { half_extents: he };
                        }
                        1 => {
                            let mut r = 0.5;
                            if let ae_core::ecs::ColliderShape::Sphere { radius } = shape {
                                r = radius;
                            }
                            ui.horizontal(|ui| {
                                ui.label("Radius:");
                                ui.add(
                                    egui::DragValue::new(&mut r).speed(0.02).range(0.01..=100.0),
                                );
                            });
                            shape = ae_core::ecs::ColliderShape::Sphere { radius: r };
                        }
                        2 => {
                            let mut hh = 0.5;
                            let mut r = 0.5;
                            let mut cy = 0.0;
                            if let ae_core::ecs::ColliderShape::Capsule {
                                half_height,
                                radius,
                                center_y,
                            } = shape
                            {
                                hh = half_height;
                                r = radius;
                                cy = center_y;
                            }
                            ui.horizontal(|ui| {
                                ui.label("Half Height:");
                                ui.add(
                                    egui::DragValue::new(&mut hh)
                                        .speed(0.02)
                                        .range(0.01..=100.0),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Radius:");
                                ui.add(
                                    egui::DragValue::new(&mut r).speed(0.02).range(0.01..=100.0),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Center Y:");
                                ui.add(
                                    egui::DragValue::new(&mut cy)
                                        .speed(0.02)
                                        .range(-10.0..=10.0),
                                );
                            });
                            shape = ae_core::ecs::ColliderShape::Capsule {
                                half_height: hh,
                                radius: r,
                                center_y: cy,
                            };
                        }
                        3 => {
                            ui.label("Trimesh (Static Mesh Collider)");
                            shape = ae_core::ecs::ColliderShape::Trimesh;
                        }
                        _ => {
                            ui.label("Convex Hull (Convex Dynamic/Kinematic)");
                            shape = ae_core::ecs::ColliderShape::ConvexHull;
                        }
                    }

                    ui.horizontal(|ui| {
                        ui.label("Friction:");
                        ui.add(
                            egui::DragValue::new(&mut friction)
                                .speed(0.01)
                                .range(0.0..=2.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Restitution:");
                        ui.add(
                            egui::DragValue::new(&mut restitution)
                                .speed(0.01)
                                .range(0.0..=2.0),
                        );
                    });
                },
            );

            if remove_clicked {
                ui_actions.push(EngineUiAction::RemoveCollider(entity));
            }

            let new_col = ae_core::ecs::Collider {
                shape,
                friction,
                restitution,
                is_sensor,
            };
            if new_col != *col {
                ui_actions.push(EngineUiAction::ModifyCollider(entity, new_col));
            }
        }
    }

    /// Draws the CharacterController component section if the entity has one.
    pub(super) fn draw_character_controller_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        if let Ok(ctrl) = world.get::<&ae_core::ecs::CharacterController>(entity) {
            let mut height = ctrl.height;
            let mut radius = ctrl.radius;
            let mut center_y = ctrl.center_y;
            let mut max_slope = ctrl.max_slope_climb_angle;
            let mut step_height = ctrl.step_height;

            let (_, remove_clicked) = super::widgets::draw_inspector_card(
                ui,
                "CharacterController",
                "👤",
                egui::Color32::from_rgb(255, 200, 100),
                true,
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        ui.add(
                            egui::DragValue::new(&mut height)
                                .speed(0.05)
                                .range(0.1..=10.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.add(
                            egui::DragValue::new(&mut radius)
                                .speed(0.02)
                                .range(0.05..=5.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Center Y:");
                        ui.add(
                            egui::DragValue::new(&mut center_y)
                                .speed(0.02)
                                .range(-10.0..=10.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Step Height:");
                        ui.add(
                            egui::DragValue::new(&mut step_height)
                                .speed(0.02)
                                .range(0.0..=2.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Max Slope (°):");
                        ui.add(
                            egui::DragValue::new(&mut max_slope)
                                .speed(0.5)
                                .range(0.0..=89.0),
                        );
                    });

                    ui.separator();
                    let status_color = if ctrl.is_grounded {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::LIGHT_GRAY
                    };
                    let status_text = if ctrl.is_grounded {
                        "🟢 Grounded"
                    } else {
                        "⚪ In Air"
                    };
                    ui.label(egui::RichText::new(status_text).color(status_color).small());
                },
            );

            if remove_clicked {
                ui_actions.push(EngineUiAction::RemoveCharacterController(entity));
            }

            let new_ctrl = ae_core::ecs::CharacterController {
                height,
                radius,
                center_y,
                max_slope_climb_angle: max_slope,
                step_height,
                is_grounded: ctrl.is_grounded,
            };
            if new_ctrl != *ctrl {
                ui_actions.push(EngineUiAction::ModifyCharacterController(entity, new_ctrl));
            }
        }
    }
}

pub struct RigidBodyUiHandler;

impl super::registry::ComponentUiHandler for RigidBodyUiHandler {
    fn component_name(&self) -> &'static str {
        "RigidBody"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        ("RigidBody", "⚙", egui::Color32::from_rgb(100, 200, 255))
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Physics", "RigidBody")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::RigidBody>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        EngineUi::draw_rigidbody_section(ui, ctx.world, ctx.entity, ctx.ui_actions);
    }

    fn add_default_to_entity(
        &self,
        world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        let default_rb = ae_core::ecs::RigidBody {
            body_type: ae_core::ecs::RigidBodyType::Dynamic,
            mass: 1.0,
            gravity_scale: 1.0,
        };
        ui_actions.push(EngineUiAction::AddRigidBody(entity, default_rb));
        if world.get::<&ae_core::ecs::Collider>(entity).is_err() {
            let default_col = ae_core::ecs::Collider {
                shape: ae_core::ecs::ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: false,
            };
            ui_actions.push(EngineUiAction::AddCollider(entity, default_col));
        }
    }

    fn remove_from_entity(&self, entity: hecs::Entity, ui_actions: &mut Vec<EngineUiAction>) {
        ui_actions.push(EngineUiAction::RemoveRigidBody(entity));
    }
}

pub struct ColliderUiHandler;

impl super::registry::ComponentUiHandler for ColliderUiHandler {
    fn component_name(&self) -> &'static str {
        "Collider"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        ("Collider", "🛡", egui::Color32::from_rgb(120, 255, 120))
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Physics", "Collider")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Collider>(entity).is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        EngineUi::draw_collider_section(ui, ctx.world, ctx.entity, ctx.ui_actions);
    }

    fn add_default_to_entity(
        &self,
        _world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        let default_col = ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.7,
            restitution: 0.0,
            is_sensor: false,
        };
        ui_actions.push(EngineUiAction::AddCollider(entity, default_col));
    }

    fn remove_from_entity(&self, entity: hecs::Entity, ui_actions: &mut Vec<EngineUiAction>) {
        ui_actions.push(EngineUiAction::RemoveCollider(entity));
    }
}

pub struct CharacterControllerUiHandler;

impl super::registry::ComponentUiHandler for CharacterControllerUiHandler {
    fn component_name(&self) -> &'static str {
        "CharacterController"
    }

    fn card_header(&self) -> (&'static str, &'static str, egui::Color32) {
        (
            "Kinematic Character Controller",
            "🚶",
            egui::Color32::from_rgb(255, 120, 200),
        )
    }

    fn menu_category(&self) -> (&'static str, &'static str) {
        ("Physics", "Character Controller")
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world
            .get::<&ae_core::ecs::CharacterController>(entity)
            .is_ok()
    }

    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut super::registry::InspectorContext) {
        EngineUi::draw_character_controller_section(ui, ctx.world, ctx.entity, ctx.ui_actions);
    }

    fn add_default_to_entity(
        &self,
        _world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        let default_cc = ae_core::ecs::CharacterController::default();
        ui_actions.push(EngineUiAction::AddCharacterController(entity, default_cc));
    }

    fn remove_from_entity(&self, entity: hecs::Entity, ui_actions: &mut Vec<EngineUiAction>) {
        ui_actions.push(EngineUiAction::RemoveCharacterController(entity));
    }
}