// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dynamic ECS Component Registry & Reflection subsystem.
//!
//! Provides an industry-standard component registry and dynamic serialization
//! pipeline for `hecs::World`. Enables automated snapshot capturing, state
//! restoration, entity cloning, and serialization without hardcoded component structs.
//!

use std::any::{TypeId, type_name};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Interface for type-erased component operations, serialization, and lifecycle hooks.
pub trait ComponentHandler: Send + Sync {
    /// Human-readable type identifier (e.g. "Position", "Rotation", "Name").
    fn type_name(&self) -> &'static str;

    /// TypeId of the underlying component.
    fn type_id(&self) -> TypeId;

    /// Captures component state from `world` on `entity` as serialized JSON bytes.
    fn capture(&self, world: &hecs::World, entity: hecs::Entity) -> Option<Vec<u8>>;

    /// Restores serialized component data onto `entity` in `world`.
    fn apply(
        &self,
        world: &mut hecs::World,
        entity: hecs::Entity,
        data: &[u8],
    ) -> Result<(), String>;

    /// Clones the component directly from `source` entity to `target` entity without serialization overhead.
    fn clone_component(
        &self,
        world: &hecs::World,
        source: hecs::Entity,
        target: hecs::Entity,
        target_world: &mut hecs::World,
    );

    /// Checks if the component exists on the given entity in `world`.
    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool;

    /// Removes the component from the given entity in `world`.
    fn remove_component(&self, world: &mut hecs::World, entity: hecs::Entity) -> bool;

    /// Instantiates and attaches a default instance of this component onto `entity` in `world`.
    fn add_default(&self, world: &mut hecs::World, entity: hecs::Entity) -> Result<(), String>;
}

/// Generic handler implementation for any component implementing `Serialize + DeserializeOwned + Clone`.
pub struct TypedComponentHandler<T> {
    name: &'static str,
    default_fn: Option<fn() -> T>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> TypedComponentHandler<T> {
    /// Creates a new typed component handler with the given human-readable name.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            default_fn: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a new typed component handler with a default value constructor.
    pub fn with_default(name: &'static str, default_fn: fn() -> T) -> Self {
        Self {
            name,
            default_fn: Some(default_fn),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: hecs::Component + serde::Serialize + serde::de::DeserializeOwned + Clone> ComponentHandler
    for TypedComponentHandler<T>
{
    fn type_name(&self) -> &'static str {
        self.name
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn capture(&self, world: &hecs::World, entity: hecs::Entity) -> Option<Vec<u8>> {
        if let Ok(comp) = world.get::<&T>(entity) {
            serde_json::to_vec(&*comp).ok()
        } else {
            None
        }
    }

    fn apply(
        &self,
        world: &mut hecs::World,
        entity: hecs::Entity,
        data: &[u8],
    ) -> Result<(), String> {
        let comp: T = serde_json::from_slice(data).map_err(|e| e.to_string())?;
        world
            .insert_one(entity, comp)
            .map_err(|e| format!("{:?}", e))?;
        Ok(())
    }

    fn clone_component(
        &self,
        world: &hecs::World,
        source: hecs::Entity,
        target: hecs::Entity,
        target_world: &mut hecs::World,
    ) {
        if let Ok(comp) = world.get::<&T>(source) {
            let cloned = (*comp).clone();
            let _ = target_world.insert_one(target, cloned);
        }
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&T>(entity).is_ok()
    }

    fn remove_component(&self, world: &mut hecs::World, entity: hecs::Entity) -> bool {
        world.remove_one::<T>(entity).is_ok()
    }

    fn add_default(&self, world: &mut hecs::World, entity: hecs::Entity) -> Result<(), String> {
        if let Some(factory) = self.default_fn {
            let instance = factory();
            world
                .insert_one(entity, instance)
                .map_err(|e| format!("{:?}", e))?;
            Ok(())
        } else if let Ok(comp) = serde_json::from_str::<T>("{}") {
            world
                .insert_one(entity, comp)
                .map_err(|e| format!("{:?}", e))?;
            Ok(())
        } else {
            Err(format!(
                "Component '{}' has no default constructor registered",
                self.name
            ))
        }
    }
}

/// Central registry storing all registered component handlers for dynamic ECS operations.
pub struct ComponentRegistry {
    handlers: Vec<Box<dyn ComponentHandler>>,
    type_id_map: HashMap<TypeId, usize>,
    name_map: HashMap<String, usize>,
}

impl ComponentRegistry {
    /// Creates an empty component registry.
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            type_id_map: HashMap::new(),
            name_map: HashMap::new(),
        }
    }

    /// Registers a component type with its default type name.
    pub fn register<
        T: hecs::Component + serde::Serialize + serde::de::DeserializeOwned + Clone + Default,
    >(
        &mut self,
    ) {
        let full_name = type_name::<T>();
        let short_name = full_name.rsplit("::").next().unwrap_or(full_name);
        self.register_with_default::<T>(short_name, T::default);
    }

    /// Registers a component type with a custom identifier name.
    pub fn register_with_name<
        T: hecs::Component + serde::Serialize + serde::de::DeserializeOwned + Clone,
    >(
        &mut self,
        name: &'static str,
    ) {
        let type_id = TypeId::of::<T>();
        let idx = self.handlers.len();
        self.handlers
            .push(Box::new(TypedComponentHandler::<T>::new(name)));
        self.type_id_map.insert(type_id, idx);
        self.name_map.insert(name.to_string(), idx);
    }

    /// Registers a component type with a custom identifier name and a default constructor.
    pub fn register_with_default<
        T: hecs::Component + serde::Serialize + serde::de::DeserializeOwned + Clone,
    >(
        &mut self,
        name: &'static str,
        default_fn: fn() -> T,
    ) {
        let type_id = TypeId::of::<T>();
        let idx = self.handlers.len();
        self.handlers
            .push(Box::new(TypedComponentHandler::<T>::with_default(
                name, default_fn,
            )));
        self.type_id_map.insert(type_id, idx);
        self.name_map.insert(name.to_string(), idx);
    }

    /// Retrieves a component handler by `TypeId`.
    pub fn get_by_type_id(&self, type_id: TypeId) -> Option<&dyn ComponentHandler> {
        self.type_id_map
            .get(&type_id)
            .map(|&idx| &*self.handlers[idx])
    }

    /// Retrieves a component handler by component name string.
    pub fn get_by_name(&self, name: &str) -> Option<&dyn ComponentHandler> {
        self.name_map.get(name).map(|&idx| &*self.handlers[idx])
    }

    /// Returns a slice of all registered component handlers.
    pub fn handlers(&self) -> &[Box<dyn ComponentHandler>] {
        &self.handlers
    }

    /// Returns the total number of registered component handlers.
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Returns true if no component handlers are registered.
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    /// Clones all registered components from `source` entity to `target` entity.
    pub fn clone_entity(
        &self,
        world: &hecs::World,
        source: hecs::Entity,
        target: hecs::Entity,
        target_world: &mut hecs::World,
    ) {
        for handler in &self.handlers {
            handler.clone_component(world, source, target, target_world);
        }
    }

    /// Builds a default component registry pre-populated with all core engine components.
    pub fn default_engine_registry() -> Self {
        let mut registry = Self::new();
        registry.register_with_default::<ae_plugin_api::Position>(
            "Position",
            ae_plugin_api::Position::default,
        );
        registry.register_with_default::<ae_plugin_api::Rotation>(
            "Rotation",
            ae_plugin_api::Rotation::default,
        );
        registry
            .register_with_default::<ae_plugin_api::Scale>("Scale", ae_plugin_api::Scale::default);
        registry.register_with_default::<ae_plugin_api::Velocity>(
            "Velocity",
            ae_plugin_api::Velocity::default,
        );
        registry.register_with_default::<ae_plugin_api::Name>("Name", ae_plugin_api::Name::default);
        registry
            .register_with_default::<ae_plugin_api::Shape>("Shape", ae_plugin_api::Shape::default);
        registry
            .register_with_default::<ae_plugin_api::Color>("Color", ae_plugin_api::Color::default);
        registry
            .register_with_default::<ae_plugin_api::Light>("Light", ae_plugin_api::Light::default);
        registry.register_with_default::<ae_plugin_api::SpriteId>(
            "SpriteId",
            ae_plugin_api::SpriteId::default,
        );
        registry.register_with_default::<ae_plugin_api::ModelId>(
            "ModelId",
            ae_plugin_api::ModelId::default,
        );
        registry.register_with_default::<ae_plugin_api::RigidBody>(
            "RigidBody",
            ae_plugin_api::RigidBody::default,
        );
        registry.register_with_default::<ae_plugin_api::Collider>(
            "Collider",
            ae_plugin_api::Collider::default,
        );
        registry.register_with_default::<ae_plugin_api::Rotator>(
            "Rotator",
            ae_plugin_api::Rotator::default,
        );
        registry.register_with_default::<ae_plugin_api::MovingPlatform>(
            "MovingPlatform",
            ae_plugin_api::MovingPlatform::default,
        );
        registry.register_with_default::<ae_plugin_api::TriggerZone>(
            "TriggerZone",
            ae_plugin_api::TriggerZone::default,
        );
        registry.register_with_default::<ae_plugin_api::DestructibleTarget>(
            "DestructibleTarget",
            ae_plugin_api::DestructibleTarget::default,
        );
        registry.register_with_default::<ae_plugin_api::CharacterAction>(
            "CharacterAction",
            ae_plugin_api::CharacterAction::default,
        );
        registry.register_with_default::<ae_plugin_api::EphemeralProjectile>(
            "EphemeralProjectile",
            ae_plugin_api::EphemeralProjectile::default,
        );
        registry.register_with_default::<ae_plugin_api::CharacterController>(
            "CharacterController",
            ae_plugin_api::CharacterController::default,
        );
        registry.register_with_default::<ae_plugin_api::PlayerTag>(
            "PlayerTag",
            ae_plugin_api::PlayerTag::default,
        );
        registry.register_with_default::<ae_plugin_api::BoundingRadius>(
            "BoundingRadius",
            ae_plugin_api::BoundingRadius::default,
        );
        registry.register_with_default::<ae_plugin_api::BoundingBox>(
            "BoundingBox",
            ae_plugin_api::BoundingBox::default,
        );
        registry.register_with_default::<ae_plugin_api::Hidden>(
            "Hidden",
            ae_plugin_api::Hidden::default,
        );
        registry.register_with_default::<ae_plugin_api::Parent>(
            "Parent",
            ae_plugin_api::Parent::default,
        );
        registry.register_with_default::<ae_plugin_api::Children>(
            "Children",
            ae_plugin_api::Children::default,
        );
        registry.register_with_default::<crate::ecs::LodGroup>(
            "LodGroup",
            crate::ecs::LodGroup::default,
        );
        registry.register_with_default::<ae_plugin_api::PhysicsMaterialHandle>(
            "PhysicsMaterialHandle",
            ae_plugin_api::PhysicsMaterialHandle::default,
        );
        registry.register_with_default::<ae_plugin_api::PhysicsMaterial>(
            "PhysicsMaterial",
            ae_plugin_api::PhysicsMaterial::default,
        );
        registry.register_with_default::<ae_plugin_api::TransformDirty>(
            "TransformDirty",
            ae_plugin_api::TransformDirty::default,
        );
        registry.register_with_default::<ae_plugin_api::UiElement>(
            "UiElement",
            ae_plugin_api::UiElement::default,
        );
        registry.register_with_default::<ae_plugin_api::UiPanel>(
            "UiPanel",
            ae_plugin_api::UiPanel::default,
        );
        registry.register_with_default::<ae_plugin_api::UiText>(
            "UiText",
            ae_plugin_api::UiText::default,
        );
        registry.register_with_default::<ae_plugin_api::UiProgressBar>(
            "UiProgressBar",
            ae_plugin_api::UiProgressBar::default,
        );
        registry.register_with_default::<ae_plugin_api::UiButton>(
            "UiButton",
            ae_plugin_api::UiButton::default,
        );
        registry.register_with_default::<ae_plugin_api::UiImage>(
            "UiImage",
            ae_plugin_api::UiImage::default,
        );
        registry.register_with_default::<ae_plugin_api::UiSlider>(
            "UiSlider",
            ae_plugin_api::UiSlider::default,
        );
        registry.register_with_default::<ae_plugin_api::UiCheckbox>(
            "UiCheckbox",
            ae_plugin_api::UiCheckbox::default,
        );
        registry.register_with_default::<ae_plugin_api::UiTextInput>(
            "UiTextInput",
            ae_plugin_api::UiTextInput::default,
        );
        registry.register_with_default::<ae_plugin_api::UiLayoutGroup>(
            "UiLayoutGroup",
            ae_plugin_api::UiLayoutGroup::default,
        );
        registry.register_with_default::<ae_plugin_api::PlayerHealthBarTag>(
            "PlayerHealthBarTag",
            || ae_plugin_api::PlayerHealthBarTag,
        );
        registry.register_with_default::<ae_plugin_api::ScoreDisplayTag>("ScoreDisplayTag", || {
            ae_plugin_api::ScoreDisplayTag
        });
        registry.register_with_default::<ae_plugin_api::ReticleTag>("ReticleTag", || {
            ae_plugin_api::ReticleTag
        });

        registry
    }

    /// Returns a reference to the global engine component registry singleton.
    pub fn global() -> &'static ComponentRegistry {
        static REGISTRY: OnceLock<ComponentRegistry> = OnceLock::new();
        REGISTRY.get_or_init(Self::default_engine_registry)
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::default_engine_registry()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    #[test]
    fn test_component_registry_add_default_and_remove() {
        let registry = ComponentRegistry::global();
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

        let removed = rotator_handler.remove_component(&mut world, entity);
        assert!(removed);
        assert!(!rotator_handler.has_component(&world, entity));
    }

    #[test]
    fn test_component_registry_all_registered_components_support_add_default() {
        let registry = ComponentRegistry::global();
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
}