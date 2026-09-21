// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Number Input Value Handlers
//!
//! Maps numeric scrub and drag values directly to underlying ECS component data fields.

use crate::ui::iris_bridge::inspector::InspectorNumberInputId;

/// Mutates numeric input fields across all supported ECS components.
///
/// # Arguments
/// * `world` - Hecs ECS world reference.
/// * `entity` - Target entity being modified.
/// * `num_id` - Identifier of the modified numeric property.
/// * `val` - Input floating-point value.
/// * `inspector_euler` - Cached Euler rotation angles in degrees.
pub(crate) fn handle_set_number_value(
    world: &hecs::World,
    entity: hecs::Entity,
    num_id: InspectorNumberInputId,
    val: f32,
    inspector_euler: &mut [f32; 3],
) {
    let val = num_id.clamp_value(val);
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
            let mut euler = world
                .get::<&ae_core::ecs::Rotation>(entity)
                .map(|r| crate::ui::iris_bridge::inspector::quaternion_to_euler_deg(&r))
                .unwrap_or([0.0, 0.0, 0.0]);
            euler[0] = val;
            *inspector_euler = euler;
            let quat = crate::ui::iris_bridge::inspector::euler_deg_to_quaternion(
                euler[0], euler[1], euler[2],
            );
            if let Ok(mut r) = world.get::<&mut ae_core::ecs::Rotation>(entity) {
                *r = quat;
            }
        }
        InspectorNumberInputId::RotY => {
            let mut euler = world
                .get::<&ae_core::ecs::Rotation>(entity)
                .map(|r| crate::ui::iris_bridge::inspector::quaternion_to_euler_deg(&r))
                .unwrap_or([0.0, 0.0, 0.0]);
            euler[1] = val;
            *inspector_euler = euler;
            let quat = crate::ui::iris_bridge::inspector::euler_deg_to_quaternion(
                euler[0], euler[1], euler[2],
            );
            if let Ok(mut r) = world.get::<&mut ae_core::ecs::Rotation>(entity) {
                *r = quat;
            }
        }
        InspectorNumberInputId::RotZ => {
            let mut euler = world
                .get::<&ae_core::ecs::Rotation>(entity)
                .map(|r| crate::ui::iris_bridge::inspector::quaternion_to_euler_deg(&r))
                .unwrap_or([0.0, 0.0, 0.0]);
            euler[2] = val;
            *inspector_euler = euler;
            let quat = crate::ui::iris_bridge::inspector::euler_deg_to_quaternion(
                euler[0], euler[1], euler[2],
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
        InspectorNumberInputId::AudioVolume => {
            if let Ok(mut a) = world.get::<&mut ae_audio::AudioSource>(entity) {
                a.volume = val.clamp(0.0, 2.0);
            }
        }
        InspectorNumberInputId::AudioPitch => {
            if let Ok(mut a) = world.get::<&mut ae_audio::AudioSource>(entity) {
                a.pitch = val.clamp(0.1, 3.0);
            }
        }
        InspectorNumberInputId::UiOffsetX => {
            if let Ok(mut u) = world.get::<&mut ae_core::ecs::UiElement>(entity) {
                u.offset[0] = val;
            }
        }
        InspectorNumberInputId::UiOffsetY => {
            if let Ok(mut u) = world.get::<&mut ae_core::ecs::UiElement>(entity) {
                u.offset[1] = val;
            }
        }
        InspectorNumberInputId::UiSizeW => {
            if let Ok(mut u) = world.get::<&mut ae_core::ecs::UiElement>(entity) {
                u.size[0] = val.max(1.0);
            }
        }
        InspectorNumberInputId::UiSizeH => {
            if let Ok(mut u) = world.get::<&mut ae_core::ecs::UiElement>(entity) {
                u.size[1] = val.max(1.0);
            }
        }
        InspectorNumberInputId::UiPivotX => {
            if let Ok(mut u) = world.get::<&mut ae_core::ecs::UiElement>(entity) {
                u.pivot[0] = val.clamp(0.0, 1.0);
            }
        }
        InspectorNumberInputId::UiPivotY => {
            if let Ok(mut u) = world.get::<&mut ae_core::ecs::UiElement>(entity) {
                u.pivot[1] = val.clamp(0.0, 1.0);
            }
        }
        InspectorNumberInputId::UiZIndex => {
            if let Ok(mut u) = world.get::<&mut ae_core::ecs::UiElement>(entity) {
                u.z_index = val as i32;
            }
        }
        InspectorNumberInputId::UiAlpha => {
            if let Ok(mut u) = world.get::<&mut ae_core::ecs::UiElement>(entity) {
                u.alpha = val.clamp(0.0, 1.0);
            }
        }
        InspectorNumberInputId::UiFontSize => {
            if let Ok(mut t) = world.get::<&mut ae_core::ecs::UiText>(entity) {
                t.font_size = val.clamp(6.0, 256.0);
            }
        }
        InspectorNumberInputId::UiBorderWidth => {
            if let Ok(mut p) = world.get::<&mut ae_core::ecs::UiPanel>(entity) {
                p.border_width = val.max(0.0);
            }
        }
        InspectorNumberInputId::UiCornerRadius => {
            if let Ok(mut p) = world.get::<&mut ae_core::ecs::UiPanel>(entity) {
                p.corner_radius = val.max(0.0);
            }
        }
        InspectorNumberInputId::UiProgressMin => {
            if let Ok(mut pb) = world.get::<&mut ae_core::ecs::UiProgressBar>(entity) {
                pb.min = val;
            }
        }
        InspectorNumberInputId::UiProgressMax => {
            if let Ok(mut pb) = world.get::<&mut ae_core::ecs::UiProgressBar>(entity) {
                pb.max = val;
            }
        }
        InspectorNumberInputId::UiProgressVal => {
            if let Ok(mut pb) = world.get::<&mut ae_core::ecs::UiProgressBar>(entity) {
                pb.value = val;
            }
        }
        _ => {}
    }
}