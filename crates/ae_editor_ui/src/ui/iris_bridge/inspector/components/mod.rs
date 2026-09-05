// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Built-in Component Handlers Registration
//!
//! Registers all standard ECS component cards into the `InspectorRegistry`.

pub mod animation;
pub mod audio;
pub mod character;
pub mod gameplay;
pub mod hierarchy;
pub mod lod;
pub mod physics;
pub mod rendering;
pub mod ui_canvas;

use super::registry::InspectorRegistry;

/// Registers all built-in component inspector handlers in deterministic visual order.
pub fn register_all_components(registry: &mut InspectorRegistry) {
    // 1. Physics Components
    registry.register(physics::RigidBodyHandler);
    registry.register(physics::ColliderHandler);
    registry.register(physics::PhysicsMaterialHandler);

    // 2. Character & Gameplay Components
    registry.register(character::CharacterControllerHandler);
    registry.register(gameplay::CharacterActionHandler);
    registry.register(character::PlayerTagHandler);
    registry.register(gameplay::VelocityHandler);
    registry.register(gameplay::RotatorHandler);
    registry.register(gameplay::MovingPlatformHandler);
    registry.register(gameplay::TriggerZoneHandler);
    registry.register(gameplay::DestructibleTargetHandler);

    // 3. Rendering, LOD & Illumination Components
    registry.register(rendering::LightHandler);
    registry.register(rendering::ModelMeshHandler);
    registry.register(rendering::ShapeHandler);
    registry.register(lod::LodGroupHandler);

    // 4. Skeletal Animation Components
    registry.register(animation::AnimationPlayerHandler);

    // 5. Audio Components
    registry.register(audio::AudioSourceHandler);
    registry.register(audio::AudioListenerHandler);

    // 6. In-Game 2D UI & HUD Components
    registry.register(ui_canvas::UiElementHandler);
    registry.register(ui_canvas::UiPanelHandler);
    registry.register(ui_canvas::UiTextHandler);
    registry.register(ui_canvas::UiProgressBarHandler);
    registry.register(ui_canvas::UiButtonHandler);
    registry.register(ui_canvas::UiImageHandler);
    registry.register(ui_canvas::UiSliderHandler);
    registry.register(ui_canvas::UiCheckboxHandler);
    registry.register(ui_canvas::UiTextInputHandler);
    registry.register(ui_canvas::UiLayoutGroupHandler);
    registry.register(ui_canvas::PlayerHealthBarTagHandler);
    registry.register(ui_canvas::ScoreDisplayTagHandler);
    registry.register(ui_canvas::ReticleTagHandler);

    // 7. Hierarchy Components
    registry.register(hierarchy::ParentHandler);
}