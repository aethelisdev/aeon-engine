// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Built-in Component Handlers Registration
//!
//! Registers all standard ECS component cards into the `InspectorRegistry`.

pub mod character;
pub mod gameplay;
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

    // 3. Rendering & Illumination Components
    registry.register(rendering::LightHandler);
    registry.register(rendering::ModelMeshHandler);
    registry.register(rendering::ShapeHandler);

    // 4. In-Game 2D UI Components
    registry.register(ui_canvas::UiElementHandler);
    registry.register(ui_canvas::UiProgressBarHandler);
    registry.register(ui_canvas::UiButtonHandler);
}