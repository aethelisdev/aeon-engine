// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Number Getter Subsystem
//!
//! Reads scalar numeric property values directly from underlying ECS components without string parsing.

use crate::ui::iris_bridge::inspector::InspectorNumberInputId;

/// Reads the current numeric property value directly from the underlying ECS component on an entity.
///
/// Bypasses UI text parsing entirely, ensuring zero allocations and robust data integrity
/// regardless of string localization or display formatting.
#[must_use]
pub fn read_inspector_number_value(
    world: &hecs::World,
    entity: hecs::Entity,
    num_id: InspectorNumberInputId,
) -> f32 {
    match num_id {
        InspectorNumberInputId::PosX => world
            .get::<&ae_core::ecs::Position>(entity)
            .map(|p| p.x)
            .unwrap_or(0.0),
        InspectorNumberInputId::PosY => world
            .get::<&ae_core::ecs::Position>(entity)
            .map(|p| p.y)
            .unwrap_or(0.0),
        InspectorNumberInputId::PosZ => world
            .get::<&ae_core::ecs::Position>(entity)
            .map(|p| p.z)
            .unwrap_or(0.0),
        InspectorNumberInputId::RotX => world
            .get::<&ae_core::ecs::Rotation>(entity)
            .map(|r| crate::ui::iris_bridge::inspector::quaternion_to_euler_deg(&r)[0])
            .unwrap_or(0.0),
        InspectorNumberInputId::RotY => world
            .get::<&ae_core::ecs::Rotation>(entity)
            .map(|r| crate::ui::iris_bridge::inspector::quaternion_to_euler_deg(&r)[1])
            .unwrap_or(0.0),
        InspectorNumberInputId::RotZ => world
            .get::<&ae_core::ecs::Rotation>(entity)
            .map(|r| crate::ui::iris_bridge::inspector::quaternion_to_euler_deg(&r)[2])
            .unwrap_or(0.0),
        InspectorNumberInputId::ScaleX => world
            .get::<&ae_core::ecs::Scale>(entity)
            .map(|s| s.x)
            .unwrap_or(1.0),
        InspectorNumberInputId::ScaleY => world
            .get::<&ae_core::ecs::Scale>(entity)
            .map(|s| s.y)
            .unwrap_or(1.0),
        InspectorNumberInputId::ScaleZ => world
            .get::<&ae_core::ecs::Scale>(entity)
            .map(|s| s.z)
            .unwrap_or(1.0),
        InspectorNumberInputId::ColliderHalfHeight => world
            .get::<&ae_core::ecs::Collider>(entity)
            .map(|c| match c.shape {
                ae_core::ecs::ColliderShape::Capsule { half_height, .. } => half_height,
                _ => 0.0,
            })
            .unwrap_or(0.0),
        InspectorNumberInputId::ColliderRadius => world
            .get::<&ae_core::ecs::Collider>(entity)
            .map(|c| match c.shape {
                ae_core::ecs::ColliderShape::Capsule { radius, .. }
                | ae_core::ecs::ColliderShape::Sphere { radius } => radius,
                _ => 0.0,
            })
            .unwrap_or(0.0),
        InspectorNumberInputId::ColliderCenterY => world
            .get::<&ae_core::ecs::Collider>(entity)
            .map(|c| match c.shape {
                ae_core::ecs::ColliderShape::Capsule { center_y, .. } => center_y,
                _ => 0.0,
            })
            .unwrap_or(0.0),
        InspectorNumberInputId::ColliderBoxX => world
            .get::<&ae_core::ecs::Collider>(entity)
            .map(|c| match c.shape {
                ae_core::ecs::ColliderShape::Box { half_extents } => half_extents[0],
                _ => 0.0,
            })
            .unwrap_or(0.0),
        InspectorNumberInputId::ColliderBoxY => world
            .get::<&ae_core::ecs::Collider>(entity)
            .map(|c| match c.shape {
                ae_core::ecs::ColliderShape::Box { half_extents } => half_extents[1],
                _ => 0.0,
            })
            .unwrap_or(0.0),
        InspectorNumberInputId::ColliderBoxZ => world
            .get::<&ae_core::ecs::Collider>(entity)
            .map(|c| match c.shape {
                ae_core::ecs::ColliderShape::Box { half_extents } => half_extents[2],
                _ => 0.0,
            })
            .unwrap_or(0.0),
        InspectorNumberInputId::ColliderFriction => world
            .get::<&ae_core::ecs::Collider>(entity)
            .map(|c| c.friction)
            .unwrap_or(0.0),
        InspectorNumberInputId::ColliderRestitution => world
            .get::<&ae_core::ecs::Collider>(entity)
            .map(|c| c.restitution)
            .unwrap_or(0.0),
        InspectorNumberInputId::PhysMatFriction => world
            .get::<&ae_core::ecs::PhysicsMaterial>(entity)
            .map(|m| m.friction)
            .unwrap_or(0.0),
        InspectorNumberInputId::PhysMatRestitution => world
            .get::<&ae_core::ecs::PhysicsMaterial>(entity)
            .map(|m| m.restitution)
            .unwrap_or(0.0),
        InspectorNumberInputId::CharacterHeight => world
            .get::<&ae_core::ecs::CharacterController>(entity)
            .map(|c| c.height)
            .unwrap_or(0.0),
        InspectorNumberInputId::CharacterRadius => world
            .get::<&ae_core::ecs::CharacterController>(entity)
            .map(|c| c.radius)
            .unwrap_or(0.0),
        InspectorNumberInputId::CharacterCenterY => world
            .get::<&ae_core::ecs::CharacterController>(entity)
            .map(|c| c.center_y)
            .unwrap_or(0.0),
        InspectorNumberInputId::CharacterMaxSlope => world
            .get::<&ae_core::ecs::CharacterController>(entity)
            .map(|c| c.max_slope_climb_angle)
            .unwrap_or(0.0),
        InspectorNumberInputId::CharacterStepHeight => world
            .get::<&ae_core::ecs::CharacterController>(entity)
            .map(|c| c.step_height)
            .unwrap_or(0.0),
        InspectorNumberInputId::ActionSpeedRange => world
            .get::<&ae_core::ecs::CharacterAction>(entity)
            .map(|a| a.speed)
            .unwrap_or(0.0),
        InspectorNumberInputId::ActionCooldown => world
            .get::<&ae_core::ecs::CharacterAction>(entity)
            .map(|a| a.cooldown)
            .unwrap_or(0.0),
        InspectorNumberInputId::VelocityX => world
            .get::<&ae_core::ecs::Velocity>(entity)
            .map(|v| v.x)
            .unwrap_or(0.0),
        InspectorNumberInputId::VelocityY => world
            .get::<&ae_core::ecs::Velocity>(entity)
            .map(|v| v.y)
            .unwrap_or(0.0),
        InspectorNumberInputId::VelocityZ => world
            .get::<&ae_core::ecs::Velocity>(entity)
            .map(|v| v.z)
            .unwrap_or(0.0),
        InspectorNumberInputId::RotatorSpeed => world
            .get::<&ae_core::ecs::Rotator>(entity)
            .map(|r| r.speed)
            .unwrap_or(0.0),
        InspectorNumberInputId::RotatorAxisX => world
            .get::<&ae_core::ecs::Rotator>(entity)
            .map(|r| r.axis[0])
            .unwrap_or(0.0),
        InspectorNumberInputId::RotatorAxisY => world
            .get::<&ae_core::ecs::Rotator>(entity)
            .map(|r| r.axis[1])
            .unwrap_or(0.0),
        InspectorNumberInputId::RotatorAxisZ => world
            .get::<&ae_core::ecs::Rotator>(entity)
            .map(|r| r.axis[2])
            .unwrap_or(0.0),
        InspectorNumberInputId::RigidBodyMass => world
            .get::<&ae_core::ecs::RigidBody>(entity)
            .map(|rb| rb.mass)
            .unwrap_or(1.0),
        InspectorNumberInputId::RigidBodyGravity => world
            .get::<&ae_core::ecs::RigidBody>(entity)
            .map(|rb| rb.gravity_scale)
            .unwrap_or(1.0),
        InspectorNumberInputId::LightOffsetY => world
            .get::<&ae_core::ecs::Light>(entity)
            .map(|l| l.position[1])
            .unwrap_or(0.0),
        InspectorNumberInputId::LightColorR => world
            .get::<&ae_core::ecs::Light>(entity)
            .map(|l| l.color[0])
            .unwrap_or(1.0),
        InspectorNumberInputId::LightColorG => world
            .get::<&ae_core::ecs::Light>(entity)
            .map(|l| l.color[1])
            .unwrap_or(1.0),
        InspectorNumberInputId::AudioVolume => world
            .get::<&ae_audio::AudioSource>(entity)
            .map(|a| a.volume)
            .unwrap_or(1.0),
        InspectorNumberInputId::AudioPitch => world
            .get::<&ae_audio::AudioSource>(entity)
            .map(|a| a.pitch)
            .unwrap_or(1.0),
        InspectorNumberInputId::UiOffsetX => world
            .get::<&ae_core::ecs::UiElement>(entity)
            .map(|u| u.offset[0])
            .unwrap_or(0.0),
        InspectorNumberInputId::UiOffsetY => world
            .get::<&ae_core::ecs::UiElement>(entity)
            .map(|u| u.offset[1])
            .unwrap_or(0.0),
        InspectorNumberInputId::UiSizeW => world
            .get::<&ae_core::ecs::UiElement>(entity)
            .map(|u| u.size[0])
            .unwrap_or(100.0),
        InspectorNumberInputId::UiSizeH => world
            .get::<&ae_core::ecs::UiElement>(entity)
            .map(|u| u.size[1])
            .unwrap_or(100.0),
        InspectorNumberInputId::UiPivotX => world
            .get::<&ae_core::ecs::UiElement>(entity)
            .map(|u| u.pivot[0])
            .unwrap_or(0.0),
        InspectorNumberInputId::UiPivotY => world
            .get::<&ae_core::ecs::UiElement>(entity)
            .map(|u| u.pivot[1])
            .unwrap_or(0.0),
        InspectorNumberInputId::UiZIndex => world
            .get::<&ae_core::ecs::UiElement>(entity)
            .map(|u| u.z_index as f32)
            .unwrap_or(0.0),
        InspectorNumberInputId::UiAlpha => world
            .get::<&ae_core::ecs::UiElement>(entity)
            .map(|u| u.alpha)
            .unwrap_or(1.0),
        InspectorNumberInputId::UiFontSize => world
            .get::<&ae_core::ecs::UiText>(entity)
            .map(|t| t.font_size)
            .unwrap_or(14.0),
        InspectorNumberInputId::UiBorderWidth => world
            .get::<&ae_core::ecs::UiPanel>(entity)
            .map(|p| p.border_width)
            .unwrap_or(0.0),
        InspectorNumberInputId::UiCornerRadius => world
            .get::<&ae_core::ecs::UiPanel>(entity)
            .map(|p| p.corner_radius)
            .unwrap_or(0.0),
        InspectorNumberInputId::UiProgressMin => world
            .get::<&ae_core::ecs::UiProgressBar>(entity)
            .map(|pb| pb.min)
            .unwrap_or(0.0),
        InspectorNumberInputId::UiProgressMax => world
            .get::<&ae_core::ecs::UiProgressBar>(entity)
            .map(|pb| pb.max)
            .unwrap_or(1.0),
        InspectorNumberInputId::UiProgressVal => world
            .get::<&ae_core::ecs::UiProgressBar>(entity)
            .map(|pb| pb.value)
            .unwrap_or(0.0),
        _ => 0.0,
    }
}