// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::super::EngineUi;
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

            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("⚙ RigidBody")
                            .strong()
                            .color(egui::Color32::from_rgb(100, 200, 255)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑").on_hover_text("Remove RigidBody").clicked() {
                            ui_actions.push(EngineUiAction::RemoveRigidBody(entity));
                        }
                    });
                });
                ui.separator();

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
            });

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

            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🛡 Collider").strong().color(egui::Color32::from_rgb(100, 255, 150)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑").on_hover_text("Remove Collider").clicked() {
                            ui_actions.push(EngineUiAction::RemoveCollider(entity));
                        }
                    });
                });
                ui.separator();

                ui.checkbox(&mut is_sensor, "Is Trigger (Sensor)").on_hover_text("If checked, this collider detects overlaps without physical collision response");
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
                            ui.add(egui::DragValue::new(&mut he[0]).prefix("X:").speed(0.05));
                            ui.add(egui::DragValue::new(&mut he[1]).prefix("Y:").speed(0.05));
                            ui.add(egui::DragValue::new(&mut he[2]).prefix("Z:").speed(0.05));
                        });
                        shape = ae_core::ecs::ColliderShape::Box { half_extents: he };
                    }
                    1 => {
                        let mut r = match shape {
                            ae_core::ecs::ColliderShape::Sphere { radius } => radius,
                            _ => 0.5,
                        };
                        ui.horizontal(|ui| {
                            ui.label("Radius:");
                            ui.add(egui::DragValue::new(&mut r).speed(0.05).range(0.001..=1000.0));
                        });
                        shape = ae_core::ecs::ColliderShape::Sphere { radius: r };
                    }
                    2 => {
                        let (mut hh, mut r) = match shape {
                            ae_core::ecs::ColliderShape::Capsule { half_height, radius } => (half_height, radius),
                            _ => (0.5, 0.25),
                        };
                        ui.horizontal(|ui| {
                            ui.label("Half H:");
                            ui.add(egui::DragValue::new(&mut hh).speed(0.05).range(0.001..=1000.0));
                            ui.label("R:");
                            ui.add(egui::DragValue::new(&mut r).speed(0.05).range(0.001..=1000.0));
                        });
                        shape = ae_core::ecs::ColliderShape::Capsule { half_height: hh, radius: r };
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
                    ui.add(egui::DragValue::new(&mut friction).speed(0.01).range(0.0..=2.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Restitution:");
                    ui.add(egui::DragValue::new(&mut restitution).speed(0.01).range(0.0..=2.0));
                });
            });

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
            let mut max_slope = ctrl.max_slope_climb_angle;
            let mut step_height = ctrl.step_height;

            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("👤 CharacterController")
                            .strong()
                            .color(egui::Color32::from_rgb(255, 200, 100)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button("🗑")
                            .on_hover_text("Remove CharacterController")
                            .clicked()
                        {
                            ui_actions.push(EngineUiAction::RemoveCharacterController(entity));
                        }
                    });
                });
                ui.separator();

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
            });

            let new_ctrl = ae_core::ecs::CharacterController {
                height,
                radius,
                max_slope_climb_angle: max_slope,
                step_height,
                is_grounded: ctrl.is_grounded,
            };
            if new_ctrl != *ctrl {
                ui_actions.push(EngineUiAction::ModifyCharacterController(entity, new_ctrl));
            }
        }
    }

    /// Renders a button dropdown that allows attaching physics components (RigidBody, Collider, CharacterController) to an entity.
    pub(super) fn draw_add_component_button(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        let has_rb = world.get::<&ae_core::ecs::RigidBody>(entity).is_ok();
        let has_col = world.get::<&ae_core::ecs::Collider>(entity).is_ok();
        let has_ctrl = world
            .get::<&ae_core::ecs::CharacterController>(entity)
            .is_ok();
        let has_lod = world.get::<&ae_core::ecs::LodGroup>(entity).is_ok();

        if has_rb && has_col && has_ctrl && has_lod {
            return; // Nothing to add
        }

        ui.horizontal(|ui| {
            ui.menu_button("➕ Add Component", |ui| {
                if !has_rb {
                    if ui.button("⚙ RigidBody").clicked() {
                        let default_rb = ae_core::ecs::RigidBody {
                            body_type: ae_core::ecs::RigidBodyType::Dynamic,
                            mass: 1.0,
                            gravity_scale: 1.0,
                        };
                        ui_actions.push(EngineUiAction::AddRigidBody(entity, default_rb));
                        // Automatically attach matching Collider if not present so dynamic objects collide with ground
                        if !has_col {
                            let default_col = if let Ok(shape) =
                                world.get::<&ae_core::ecs::Shape>(entity)
                            {
                                match *shape {
                                    ae_core::ecs::Shape::Sphere => ae_core::ecs::Collider {
                                        shape: ae_core::ecs::ColliderShape::Sphere { radius: 0.5 },
                                        friction: 0.7,
                                        restitution: 0.0,
                                        is_sensor: false,
                                    },
                                    ae_core::ecs::Shape::Cylinder
                                    | ae_core::ecs::Shape::Capsule => ae_core::ecs::Collider {
                                        shape: ae_core::ecs::ColliderShape::Capsule {
                                            half_height: 0.15,
                                            radius: 0.35,
                                        },
                                        friction: 0.7,
                                        restitution: 0.0,
                                        is_sensor: false,
                                    },
                                    _ => ae_core::ecs::Collider {
                                        shape: ae_core::ecs::ColliderShape::Box {
                                            half_extents: [0.5, 0.5, 0.5],
                                        },
                                        friction: 0.7,
                                        restitution: 0.0,
                                        is_sensor: false,
                                    },
                                }
                            } else if world.get::<&ae_core::ecs::ModelId>(entity).is_ok() {
                                ae_core::ecs::Collider {
                                    shape: ae_core::ecs::ColliderShape::Trimesh,
                                    friction: 0.7,
                                    restitution: 0.0,
                                    is_sensor: false,
                                }
                            } else {
                                ae_core::ecs::Collider {
                                    shape: ae_core::ecs::ColliderShape::Box {
                                        half_extents: [0.5, 0.5, 0.5],
                                    },
                                    friction: 0.7,
                                    restitution: 0.0,
                                    is_sensor: false,
                                }
                            };
                            ui_actions.push(EngineUiAction::AddCollider(entity, default_col));
                        }
                        ui.close();
                    }
                }
                if !has_col {
                    if ui.button("🛡 Collider").clicked() {
                        let default_col = ae_core::ecs::Collider {
                            shape: ae_core::ecs::ColliderShape::Box {
                                half_extents: [0.5, 0.5, 0.5],
                            },
                            friction: 0.7,
                            restitution: 0.0,
                            is_sensor: false,
                        };
                        ui_actions.push(EngineUiAction::AddCollider(entity, default_col));
                        ui.close();
                    }
                }
                if !has_ctrl {
                    if ui.button("👤 CharacterController").clicked() {
                        let default_ctrl = ae_core::ecs::CharacterController::default();
                        ui_actions
                            .push(EngineUiAction::AddCharacterController(entity, default_ctrl));
                        // Automatically attach Kinematic rigid body and Capsule collider for character physics if not present
                        if !has_rb {
                            let rb = ae_core::ecs::RigidBody {
                                body_type: ae_core::ecs::RigidBodyType::Kinematic,
                                mass: 1.0,
                                gravity_scale: 1.0,
                            };
                            ui_actions.push(EngineUiAction::AddRigidBody(entity, rb));
                        }
                        if !has_col {
                            let col = ae_core::ecs::Collider {
                                shape: ae_core::ecs::ColliderShape::Capsule {
                                    half_height: 0.15,
                                    radius: 0.35,
                                },
                                friction: 0.5,
                                restitution: 0.0,
                                is_sensor: false,
                            };
                            ui_actions.push(EngineUiAction::AddCollider(entity, col));
                        }
                        ui.close();
                    }
                }
                if !has_lod {
                    if ui.button("📊 LodGroup").clicked() {
                        ui_actions.push(EngineUiAction::AddLodGroup(entity));
                        ui.close();
                    }
                }
                if world.get::<&ae_audio::AudioSource>(entity).is_err() {
                    if ui.button("🔊 AudioSource").clicked() {
                        ui_actions.push(EngineUiAction::AddAudioSource(entity));
                        ui.close();
                    }
                }
                if world.get::<&ae_audio::AudioListener>(entity).is_err() {
                    if ui.button("👂 AudioListener").clicked() {
                        ui_actions.push(EngineUiAction::AddAudioListener(entity));
                        ui.close();
                    }
                }
                if world.get::<&ae_core::ecs::PlayerTag>(entity).is_err() {
                    if ui.button("🎮 PlayerTag").clicked() {
                        ui_actions.push(EngineUiAction::AddPlayerTag(entity));
                        ui.close();
                    }
                }
            });
        });
    }

    /// Draws the PlayerTag component section if the entity has one.
    pub(super) fn draw_player_tag_section(
        ui: &mut egui::Ui,
        world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        if world.get::<&ae_core::ecs::PlayerTag>(entity).is_ok() {
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("🎮 PlayerTag")
                            .strong()
                            .color(egui::Color32::from_rgb(255, 180, 80)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑").on_hover_text("Remove PlayerTag").clicked() {
                            ui_actions.push(EngineUiAction::RemovePlayerTag(entity));
                        }
                    });
                });
                ui.label(
                    egui::RichText::new("Designates this entity as the active Player target for gameplay logic and camera tracking.")
                        .small()
                        .weak(),
                );
            });
        }
    }
}