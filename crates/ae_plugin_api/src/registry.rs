// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Standard engine component registrations into [`ae_ecs::ComponentRegistry`].
//!
//! Binds all shared engine POD components (transform, physics, rendering, gameplay, and UI)
//! into a single process-wide [`ae_ecs::ComponentRegistry`] instance without duplicating ECS
//! serialization or reflection machinery outside `ae_ecs`.

use ae_ecs::ComponentRegistry;
use std::sync::OnceLock;

/// Registers all core built-in engine components into `registry`.
pub fn register_engine_components(registry: &mut ComponentRegistry) {
    use crate::{components::*, identity::EntityGuid};

    registry.register_with_default::<EntityGuid>("EntityGuid", EntityGuid::default);
    registry.register_with_default::<Position>("Position", Position::default);
    registry.register_with_default::<Rotation>("Rotation", Rotation::default);
    registry.register_with_default::<Scale>("Scale", Scale::default);
    registry.register_with_default::<Velocity>("Velocity", Velocity::default);
    registry.register_with_default::<Name>("Name", Name::default);
    registry.register_with_default::<Shape>("Shape", Shape::default);
    registry.register_with_default::<Color>("Color", Color::default);
    registry.register_with_default::<Light>("Light", Light::default);
    registry.register_with_default::<SpriteId>("SpriteId", SpriteId::default);
    registry.register_with_default::<ModelId>("ModelId", ModelId::default);
    registry.register_with_default::<RigidBody>("RigidBody", RigidBody::default);
    registry.register_with_default::<Collider>("Collider", Collider::default);
    registry.register_with_default::<Rotator>("Rotator", Rotator::default);
    registry.register_with_default::<MovingPlatform>("MovingPlatform", MovingPlatform::default);
    registry.register_with_default::<TriggerZone>("TriggerZone", TriggerZone::default);
    registry.register_with_default::<DestructibleTarget>(
        "DestructibleTarget",
        DestructibleTarget::default,
    );
    registry.register_with_default::<CharacterAction>("CharacterAction", CharacterAction::default);
    registry.register_with_default::<EphemeralProjectile>(
        "EphemeralProjectile",
        EphemeralProjectile::default,
    );
    registry.register_with_default::<CharacterController>(
        "CharacterController",
        CharacterController::default,
    );
    registry.register_with_default::<PlayerTag>("PlayerTag", PlayerTag::default);
    registry.register_with_default::<BoundingRadius>("BoundingRadius", BoundingRadius::default);
    registry.register_with_default::<BoundingBox>("BoundingBox", BoundingBox::default);
    registry.register_with_default::<Hidden>("Hidden", Hidden::default);
    registry.register_with_default::<Parent>("Parent", Parent::default);
    registry.register_with_default::<Children>("Children", Children::default);
    registry.register_with_default::<LodGroup>("LodGroup", LodGroup::default);
    registry.register_with_default::<PhysicsMaterialHandle>(
        "PhysicsMaterialHandle",
        PhysicsMaterialHandle::default,
    );
    registry.register_with_default::<PhysicsMaterial>("PhysicsMaterial", PhysicsMaterial::default);
    registry.register_with_default::<TransformDirty>("TransformDirty", TransformDirty::default);
    registry.register_with_default::<UiElement>("UiElement", UiElement::default);
    registry.register_with_default::<UiPanel>("UiPanel", UiPanel::default);
    registry.register_with_default::<UiText>("UiText", UiText::default);
    registry.register_with_default::<UiProgressBar>("UiProgressBar", UiProgressBar::default);
    registry.register_with_default::<UiButton>("UiButton", UiButton::default);
    registry.register_with_default::<UiImage>("UiImage", UiImage::default);
    registry.register_with_default::<UiSlider>("UiSlider", UiSlider::default);
    registry.register_with_default::<UiCheckbox>("UiCheckbox", UiCheckbox::default);
    registry.register_with_default::<UiTextInput>("UiTextInput", UiTextInput::default);
    registry.register_with_default::<UiLayoutGroup>("UiLayoutGroup", UiLayoutGroup::default);
    registry
        .register_with_default::<PlayerHealthBarTag>("PlayerHealthBarTag", || PlayerHealthBarTag);
    registry.register_with_default::<ScoreDisplayTag>("ScoreDisplayTag", || ScoreDisplayTag);
    registry.register_with_default::<ReticleTag>("ReticleTag", || ReticleTag);
    registry.register_with_default::<SpriteRenderer>("SpriteRenderer", SpriteRenderer::default);
}

/// Returns the shared process-wide [`ComponentRegistry`] populated with all core engine components.
pub fn engine_component_registry() -> &'static ComponentRegistry {
    static REGISTRY: OnceLock<ComponentRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = ComponentRegistry::new();
        register_engine_components(&mut registry);
        registry
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_ecs::World;
    use std::collections::HashSet;

    #[test]
    fn test_component_registry_add_default_and_remove() {
        let registry = engine_component_registry();
        let mut world = World::new();
        let entity = world.spawn(());

        let rotator_handler = registry
            .get_by_name("Rotator")
            .expect("Rotator must be registered");
        assert!(!rotator_handler.has_component(&world, entity));

        rotator_handler
            .add_default(&mut world, entity)
            .expect("add_default must succeed");
        assert!(rotator_handler.has_component(&world, entity));

        assert!(rotator_handler.remove_component(&mut world, entity));
        assert!(!rotator_handler.has_component(&world, entity));
    }

    #[test]
    fn test_component_registry_all_registered_components_support_add_default() {
        let registry = engine_component_registry();
        for handler in registry.handlers() {
            let mut world = World::new();
            let entity = world.spawn(());
            let res = handler.add_default(&mut world, entity);
            assert!(
                res.is_ok(),
                "Component '{}' must support add_default: {:?}",
                handler.type_name(),
                res.err()
            );
            assert!(handler.has_component(&world, entity));
        }
    }

    #[test]
    fn test_component_registry_sprite_renderer_roundtrip() {
        let registry = engine_component_registry();
        let mut world = World::new();
        let entity = world.spawn(());

        let handler = registry
            .get_by_name("SpriteRenderer")
            .expect("SpriteRenderer must be registered in engine registry");

        let sprite = crate::SpriteRenderer {
            tint: [0.2, 0.4, 0.8, 1.0],
            sorting_layer: 10,
            order_in_layer: 5,
            flip_x: true,
            flip_y: false,
            uv_rect: [0.1, 0.2, 0.8, 0.9],
            ..Default::default()
        };
        world
            .insert_one(entity, sprite.clone())
            .expect("insert_one sprite must succeed in test");

        let data = handler
            .capture(&world, entity)
            .expect("capture must succeed for SpriteRenderer");

        let mut target_world = World::new();
        let target_entity = target_world.spawn(());
        handler
            .apply(&mut target_world, target_entity, &data)
            .expect("apply must succeed for SpriteRenderer");

        let restored = target_world
            .get_cloned::<crate::SpriteRenderer>(target_entity)
            .expect("restored SpriteRenderer must exist");

        assert_eq!(restored, sprite);
    }

    #[test]
    fn test_component_registry_type_hashes_are_collision_free() {
        let registry = engine_component_registry();
        let mut seen = HashSet::with_capacity(registry.len());
        for handler in registry.handlers() {
            let hash = ae_ecs::component_type_hash(handler.type_name());
            assert!(
                seen.insert(hash),
                "Duplicate component_type_hash detected for '{}'",
                handler.type_name()
            );
        }
        assert!(registry.get_by_name("EntityGuid").is_some());
        assert!(registry.get_by_name("LodGroup").is_some());
    }
}