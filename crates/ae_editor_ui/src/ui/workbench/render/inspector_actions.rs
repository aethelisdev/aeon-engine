// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Panel Action Handlers
//!
//! Dispatches ECS component mutations, numeric input edits, color adjustments,
//! transform resets, and combobox selections triggered by the Iris UI Inspector panel.

use crate::ui::iris_bridge::inspector::{
    ComponentCheckboxId, InspectorAction, InspectorDropdownId, InspectorNumberInputId,
    TransformAxisType,
};
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::state::EngineUi;

impl EngineUi {
    /// Dispatches all pending Inspector panel actions to ECS entities and UI state.
    pub fn process_inspector_actions(
        &mut self,
        world: &hecs::World,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        for action in self.iris_overlay.take_inspector_actions() {
            match action {
                InspectorAction::RenameEntity(name) => {
                    if let Some(entity) = self.selected_entity {
                        let old_name = world
                            .get::<&ae_core::ecs::Name>(entity)
                            .map(|n| n.0.clone())
                            .unwrap_or_default();
                        ui_actions.push(EngineUiAction::ModifyName(entity, old_name, name));
                    }
                }
                InspectorAction::ResetTransform(axis) => {
                    self.handle_reset_transform(world, ui_actions, axis);
                }
                InspectorAction::SetObjectColor(col) => {
                    self.handle_set_object_color(world, ui_actions, col);
                }
                InspectorAction::AddColorToPalette(col) => {
                    if let Some(entity) = self.selected_entity {
                        let arr = if col == irisui::prelude::Color::TRANSPARENT {
                            world
                                .get::<&ae_core::ecs::Color>(entity)
                                .map(|c| [c.r, c.g, c.b, c.a])
                                .unwrap_or([0.60, 0.75, 0.95, 1.0])
                        } else {
                            [col.r, col.g, col.b, col.a]
                        };
                        if !self.saved_swatches.contains(&arr) && self.saved_swatches.len() < 28 {
                            self.saved_swatches.push(arr);
                        }
                    }
                }
                InspectorAction::ClearCustomPalette => {
                    self.saved_swatches.clear();
                }
                InspectorAction::RemoveColorFromPalette(idx) => {
                    if idx < self.saved_swatches.len() {
                        self.saved_swatches.remove(idx);
                    }
                }
                InspectorAction::RemoveComponent(comp_name) => {
                    if let Some(entity) = self.selected_entity {
                        ui_actions.push(EngineUiAction::RemoveComponent(entity, comp_name));
                    }
                }
                InspectorAction::AddComponent(comp_name) => {
                    if let Some(entity) = self.selected_entity {
                        ui_actions.push(EngineUiAction::AddComponent(entity, comp_name));
                    }
                }
                InspectorAction::SaveAsPrefab => {
                    if let Some(entity) = self.selected_entity {
                        ui_actions.push(EngineUiAction::SaveEntityAsPrefab(
                            entity,
                            std::path::PathBuf::from("assets/prefabs/prefab.json"),
                        ));
                    }
                }
                InspectorAction::StartNumberEdit(num_id) => {
                    if let Some(entity) = self.selected_entity {
                        let comp_name = num_id.component_name();
                        let registry = ae_core::registry::ComponentRegistry::global();
                        if let Some(handler) = registry.get_by_name(comp_name)
                            && let Some(old_bytes) = handler.capture(world, entity)
                        {
                            self.iris_overlay.inspector_edit_start_snapshot =
                                Some((entity, comp_name, old_bytes));
                        }
                    }
                }
                InspectorAction::SetNumberValue(num_id, val) => {
                    if let Some(entity) = self.selected_entity {
                        if self.iris_overlay.inspector_edit_start_snapshot.is_none() {
                            let comp_name = num_id.component_name();
                            let registry = ae_core::registry::ComponentRegistry::global();
                            if let Some(handler) = registry.get_by_name(comp_name)
                                && let Some(old_bytes) = handler.capture(world, entity)
                            {
                                self.iris_overlay.inspector_edit_start_snapshot =
                                    Some((entity, comp_name, old_bytes));
                            }
                        }
                        handle_set_number_value(
                            world,
                            entity,
                            num_id,
                            val,
                            &mut self.inspector_euler,
                        );
                    }
                }
                InspectorAction::CommitNumberEdit(num_id) => {
                    if let Some(entity) = self.selected_entity {
                        let comp_name = num_id.component_name();
                        if let Some((snap_entity, snap_comp_name, old_bytes)) =
                            self.iris_overlay.inspector_edit_start_snapshot.take()
                            && snap_entity == entity
                            && snap_comp_name == comp_name
                        {
                            let registry = ae_core::registry::ComponentRegistry::global();
                            if let Some(handler) = registry.get_by_name(comp_name)
                                && let Some(new_bytes) = handler.capture(world, entity)
                                && old_bytes != new_bytes
                            {
                                ui_actions.push(EngineUiAction::CommitComponentModify(
                                    entity, comp_name, old_bytes, new_bytes,
                                ));
                            }
                        }
                    }
                }
                InspectorAction::SelectDropdown(dd_id, opt_idx) => {
                    if let Some(entity) = self.selected_entity {
                        let comp_name = dd_id.component_name();
                        let registry = ae_core::registry::ComponentRegistry::global();
                        let old_bytes = registry
                            .get_by_name(comp_name)
                            .and_then(|h| h.capture(world, entity));
                        handle_select_dropdown(world, entity, dd_id, opt_idx);
                        let new_bytes = registry
                            .get_by_name(comp_name)
                            .and_then(|h| h.capture(world, entity));
                        if let (Some(old), Some(new)) = (old_bytes, new_bytes)
                            && old != new
                        {
                            ui_actions.push(EngineUiAction::CommitComponentModify(
                                entity, comp_name, old, new,
                            ));
                        }
                    }
                }
                InspectorAction::ToggleCheckbox(cb_id) => {
                    if let Some(entity) = self.selected_entity {
                        let comp_name = cb_id.component_name();
                        let registry = ae_core::registry::ComponentRegistry::global();
                        let old_bytes = registry
                            .get_by_name(comp_name)
                            .and_then(|h| h.capture(world, entity));
                        if let ComponentCheckboxId::ColliderIsSensor = cb_id
                            && let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity)
                        {
                            c.is_sensor = !c.is_sensor;
                        }
                        let new_bytes = registry
                            .get_by_name(comp_name)
                            .and_then(|h| h.capture(world, entity));
                        if let (Some(old), Some(new)) = (old_bytes, new_bytes)
                            && old != new
                        {
                            ui_actions.push(EngineUiAction::CommitComponentModify(
                                entity, comp_name, old, new,
                            ));
                        }
                    }
                }
                InspectorAction::ResetPhysMatPreset => {
                    if let Some(entity) = self.selected_entity {
                        let comp_name = "PhysicsMaterial";
                        let registry = ae_core::registry::ComponentRegistry::global();
                        let old_bytes = registry
                            .get_by_name(comp_name)
                            .and_then(|h| h.capture(world, entity));
                        let surf = world
                            .get::<&ae_core::ecs::PhysicsMaterial>(entity)
                            .map(|m| m.surface_type)
                            .unwrap_or(ae_core::ecs::SurfaceType::Default);
                        if let Ok(mut m) = world.get::<&mut ae_core::ecs::PhysicsMaterial>(entity) {
                            *m = ae_core::ecs::PhysicsMaterial::from_preset(surf);
                        }
                        let new_bytes = registry
                            .get_by_name(comp_name)
                            .and_then(|h| h.capture(world, entity));
                        if let (Some(old), Some(new)) = (old_bytes, new_bytes)
                            && old != new
                        {
                            ui_actions.push(EngineUiAction::CommitComponentModify(
                                entity, comp_name, old, new,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Helper function resetting Position, Rotation, or Scale transform components.
    fn handle_reset_transform(
        &mut self,
        world: &hecs::World,
        ui_actions: &mut Vec<EngineUiAction>,
        axis: TransformAxisType,
    ) {
        if let Some(entity) = self.selected_entity {
            match axis {
                TransformAxisType::Position => {
                    let old_pos = world
                        .get::<&ae_core::ecs::Position>(entity)
                        .map(|p| *p)
                        .unwrap_or(ae_core::ecs::Position {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        });
                    let new_pos = ae_core::ecs::Position {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    };
                    ui_actions.push(EngineUiAction::ModifyPosition(entity, old_pos, new_pos));
                }
                TransformAxisType::Rotation => {
                    let old_rot = world
                        .get::<&ae_core::ecs::Rotation>(entity)
                        .map(|r| *r)
                        .unwrap_or_else(|_| ae_core::ecs::Rotation::identity());
                    let new_rot = ae_core::ecs::Rotation::identity();
                    self.inspector_euler = [0.0, 0.0, 0.0];
                    ui_actions.push(EngineUiAction::ModifyRotation(entity, old_rot, new_rot));
                }
                TransformAxisType::Scale => {
                    let old_scale = world
                        .get::<&ae_core::ecs::Scale>(entity)
                        .map(|s| *s)
                        .unwrap_or(ae_core::ecs::Scale {
                            x: 1.0,
                            y: 1.0,
                            z: 1.0,
                        });
                    let new_scale = ae_core::ecs::Scale {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    };
                    ui_actions.push(EngineUiAction::ModifyScale(entity, old_scale, new_scale));
                }
            }
        }
    }

    /// Helper function updating object color and synchronizing hex string state.
    fn handle_set_object_color(
        &mut self,
        world: &hecs::World,
        ui_actions: &mut Vec<EngineUiAction>,
        col: irisui::prelude::Color,
    ) {
        if let Some(entity) = self.selected_entity {
            let old_col = world
                .get::<&ae_core::ecs::Color>(entity)
                .map(|c| *c)
                .unwrap_or(ae_core::ecs::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                });
            let new_col = ae_core::ecs::Color {
                r: col.r,
                g: col.g,
                b: col.b,
                a: col.a,
            };
            let r = (col.r.clamp(0.0, 1.0) * 255.0) as u8;
            let g = (col.g.clamp(0.0, 1.0) * 255.0) as u8;
            let b = (col.b.clamp(0.0, 1.0) * 255.0) as u8;
            self.inspector_color_hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
            let (h, s, v) = irisui::prelude::rgb_to_hsv(col.r, col.g, col.b);
            self.iris_overlay.inspector_hsv = [h, s, v];
            ui_actions.push(EngineUiAction::ModifyColor(entity, old_col, new_col));
        }
    }
}

/// Helper function mutating numeric input fields across all ECS components.
fn handle_set_number_value(
    world: &hecs::World,
    entity: hecs::Entity,
    num_id: InspectorNumberInputId,
    val: f32,
    inspector_euler: &mut [f32; 3],
) {
    match num_id {
        InspectorNumberInputId::PosX => {
            if let Ok(mut p) = world.get::<&mut ae_core::ecs::Position>(entity) {
                p.x = val;
            }
        }
        InspectorNumberInputId::PosY => {
            if let Ok(mut p) = world.get::<&mut ae_core::ecs::Position>(entity) {
                p.y = val;
            }
        }
        InspectorNumberInputId::PosZ => {
            if let Ok(mut p) = world.get::<&mut ae_core::ecs::Position>(entity) {
                p.z = val;
            }
        }
        InspectorNumberInputId::RotX => {
            inspector_euler[0] = val;
            let quat = crate::ui::panels::inspector::widgets::euler_deg_to_quaternion(
                inspector_euler[0],
                inspector_euler[1],
                inspector_euler[2],
            );
            if let Ok(mut r) = world.get::<&mut ae_core::ecs::Rotation>(entity) {
                *r = quat;
            }
        }
        InspectorNumberInputId::RotY => {
            inspector_euler[1] = val;
            let quat = crate::ui::panels::inspector::widgets::euler_deg_to_quaternion(
                inspector_euler[0],
                inspector_euler[1],
                inspector_euler[2],
            );
            if let Ok(mut r) = world.get::<&mut ae_core::ecs::Rotation>(entity) {
                *r = quat;
            }
        }
        InspectorNumberInputId::RotZ => {
            inspector_euler[2] = val;
            let quat = crate::ui::panels::inspector::widgets::euler_deg_to_quaternion(
                inspector_euler[0],
                inspector_euler[1],
                inspector_euler[2],
            );
            if let Ok(mut r) = world.get::<&mut ae_core::ecs::Rotation>(entity) {
                *r = quat;
            }
        }
        InspectorNumberInputId::ScaleX => {
            if let Ok(mut s) = world.get::<&mut ae_core::ecs::Scale>(entity) {
                s.x = val;
            }
        }
        InspectorNumberInputId::ScaleY => {
            if let Ok(mut s) = world.get::<&mut ae_core::ecs::Scale>(entity) {
                s.y = val;
            }
        }
        InspectorNumberInputId::ScaleZ => {
            if let Ok(mut s) = world.get::<&mut ae_core::ecs::Scale>(entity) {
                s.z = val;
            }
        }
        InspectorNumberInputId::ColliderHalfHeight => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity)
                && let ae_core::ecs::ColliderShape::Capsule {
                    ref mut half_height,
                    ..
                } = c.shape
            {
                *half_height = val;
            }
        }
        InspectorNumberInputId::ColliderRadius => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity) {
                match c.shape {
                    ae_core::ecs::ColliderShape::Capsule { ref mut radius, .. } => *radius = val,
                    ae_core::ecs::ColliderShape::Sphere { ref mut radius } => *radius = val,
                    _ => {}
                }
            }
        }
        InspectorNumberInputId::ColliderCenterY => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity)
                && let ae_core::ecs::ColliderShape::Capsule {
                    ref mut center_y, ..
                } = c.shape
            {
                *center_y = val;
            }
        }
        InspectorNumberInputId::ColliderBoxX => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity)
                && let ae_core::ecs::ColliderShape::Box {
                    ref mut half_extents,
                } = c.shape
            {
                half_extents[0] = val;
            }
        }
        InspectorNumberInputId::ColliderBoxY => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity)
                && let ae_core::ecs::ColliderShape::Box {
                    ref mut half_extents,
                } = c.shape
            {
                half_extents[1] = val;
            }
        }
        InspectorNumberInputId::ColliderBoxZ => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity)
                && let ae_core::ecs::ColliderShape::Box {
                    ref mut half_extents,
                } = c.shape
            {
                half_extents[2] = val;
            }
        }
        InspectorNumberInputId::ColliderFriction => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity) {
                c.friction = val;
            }
        }
        InspectorNumberInputId::ColliderRestitution => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity) {
                c.restitution = val;
            }
        }
        InspectorNumberInputId::PhysMatFriction => {
            if let Ok(mut m) = world.get::<&mut ae_core::ecs::PhysicsMaterial>(entity) {
                m.friction = val;
            }
        }
        InspectorNumberInputId::PhysMatRestitution => {
            if let Ok(mut m) = world.get::<&mut ae_core::ecs::PhysicsMaterial>(entity) {
                m.restitution = val;
            }
        }
        InspectorNumberInputId::CharacterHeight => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::CharacterController>(entity) {
                c.height = val;
            }
        }
        InspectorNumberInputId::CharacterRadius => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::CharacterController>(entity) {
                c.radius = val;
            }
        }
        InspectorNumberInputId::CharacterCenterY => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::CharacterController>(entity) {
                c.center_y = val;
            }
        }
        InspectorNumberInputId::CharacterMaxSlope => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::CharacterController>(entity) {
                c.max_slope_climb_angle = val;
            }
        }
        InspectorNumberInputId::CharacterStepHeight => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::CharacterController>(entity) {
                c.step_height = val;
            }
        }
        InspectorNumberInputId::ActionSpeedRange => {
            if let Ok(mut a) = world.get::<&mut ae_core::ecs::CharacterAction>(entity) {
                a.speed = val;
            }
        }
        InspectorNumberInputId::ActionCooldown => {
            if let Ok(mut a) = world.get::<&mut ae_core::ecs::CharacterAction>(entity) {
                a.cooldown = val;
            }
        }
        InspectorNumberInputId::VelocityX => {
            if let Ok(mut v) = world.get::<&mut ae_core::ecs::Velocity>(entity) {
                v.x = val;
            }
        }
        InspectorNumberInputId::VelocityY => {
            if let Ok(mut v) = world.get::<&mut ae_core::ecs::Velocity>(entity) {
                v.y = val;
            }
        }
        InspectorNumberInputId::VelocityZ => {
            if let Ok(mut v) = world.get::<&mut ae_core::ecs::Velocity>(entity) {
                v.z = val;
            }
        }
        InspectorNumberInputId::RigidBodyMass => {
            if let Ok(mut rb) = world.get::<&mut ae_core::ecs::RigidBody>(entity) {
                rb.mass = val;
            }
        }
        InspectorNumberInputId::RigidBodyGravity => {
            if let Ok(mut rb) = world.get::<&mut ae_core::ecs::RigidBody>(entity) {
                rb.gravity_scale = val;
            }
        }
        _ => {}
    }
}

/// Helper function mutating ECS components from Inspector dropdown selection.
fn handle_select_dropdown(
    world: &hecs::World,
    entity: hecs::Entity,
    dd_id: InspectorDropdownId,
    opt_idx: usize,
) {
    match dd_id {
        InspectorDropdownId::RigidBodyType => {
            let bt = match opt_idx {
                0 => ae_core::ecs::RigidBodyType::Dynamic,
                1 => ae_core::ecs::RigidBodyType::Kinematic,
                2 => ae_core::ecs::RigidBodyType::Static,
                _ => ae_core::ecs::RigidBodyType::Kinematic,
            };
            if let Ok(mut rb) = world.get::<&mut ae_core::ecs::RigidBody>(entity) {
                rb.body_type = bt;
            }
        }
        InspectorDropdownId::ColliderShape => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity) {
                c.shape = match opt_idx {
                    0 => ae_core::ecs::ColliderShape::Capsule {
                        half_height: 0.5,
                        radius: 0.4,
                        center_y: 0.0,
                    },
                    1 => ae_core::ecs::ColliderShape::Box {
                        half_extents: [0.5, 0.5, 0.5],
                    },
                    2 => ae_core::ecs::ColliderShape::Sphere { radius: 0.5 },
                    3 => ae_core::ecs::ColliderShape::Trimesh,
                    4 => ae_core::ecs::ColliderShape::ConvexHull,
                    _ => c.shape,
                };
            }
        }
        InspectorDropdownId::SurfaceType => {
            let surf = match opt_idx {
                0 => ae_core::ecs::SurfaceType::Flesh,
                1 => ae_core::ecs::SurfaceType::Default,
                2 => ae_core::ecs::SurfaceType::Metal,
                3 => ae_core::ecs::SurfaceType::Wood,
                4 => ae_core::ecs::SurfaceType::Stone,
                5 => ae_core::ecs::SurfaceType::Dirt,
                6 => ae_core::ecs::SurfaceType::Glass,
                7 => ae_core::ecs::SurfaceType::Rubber,
                _ => ae_core::ecs::SurfaceType::Default,
            };
            if let Ok(mut m) = world.get::<&mut ae_core::ecs::PhysicsMaterial>(entity) {
                *m = ae_core::ecs::PhysicsMaterial::from_preset(surf);
            }
        }
        InspectorDropdownId::ShapeType => {
            let shp = match opt_idx {
                0 => ae_core::ecs::Shape::Cube,
                1 => ae_core::ecs::Shape::Sphere,
                2 => ae_core::ecs::Shape::Cylinder,
                3 => ae_core::ecs::Shape::Capsule,
                4 => ae_core::ecs::Shape::Torus,
                5 => ae_core::ecs::Shape::Triangle,
                _ => ae_core::ecs::Shape::Cube,
            };
            if let Ok(mut s) = world.get::<&mut ae_core::ecs::Shape>(entity) {
                *s = shp;
            }
        }
        _ => {}
    }
}