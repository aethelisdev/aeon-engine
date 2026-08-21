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
}

/// Generic handler implementation for any component implementing `Serialize + DeserializeOwned + Clone`.
pub struct TypedComponentHandler<T> {
    name: &'static str,
    _marker: std::marker::PhantomData<T>,
}

impl<T> TypedComponentHandler<T> {
    /// Creates a new typed component handler with the given human-readable name.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
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
    pub fn register<T: hecs::Component + serde::Serialize + serde::de::DeserializeOwned + Clone>(
        &mut self,
    ) {
        let full_name = type_name::<T>();
        let short_name = full_name.rsplit("::").next().unwrap_or(full_name);
        self.register_with_name::<T>(short_name);
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
        registry.register_with_name::<ae_plugin_api::Position>("Position");
        registry.register_with_name::<ae_plugin_api::Rotation>("Rotation");
        registry.register_with_name::<ae_plugin_api::Scale>("Scale");
        registry.register_with_name::<ae_plugin_api::Velocity>("Velocity");
        registry.register_with_name::<ae_plugin_api::Name>("Name");
        registry.register_with_name::<ae_plugin_api::Shape>("Shape");
        registry.register_with_name::<ae_plugin_api::Color>("Color");
        registry.register_with_name::<ae_plugin_api::Light>("Light");
        registry.register_with_name::<ae_plugin_api::SpriteId>("SpriteId");
        registry.register_with_name::<ae_plugin_api::ModelId>("ModelId");
        registry.register_with_name::<ae_plugin_api::RigidBody>("RigidBody");
        registry.register_with_name::<ae_plugin_api::Collider>("Collider");
        registry.register_with_name::<ae_plugin_api::BehaviorComponent>("BehaviorComponent");
        registry.register_with_name::<ae_plugin_api::CharacterController>("CharacterController");
        registry.register_with_name::<ae_plugin_api::PlayerTag>("PlayerTag");
        registry.register_with_name::<ae_plugin_api::BoundingRadius>("BoundingRadius");
        registry.register_with_name::<ae_plugin_api::BoundingBox>("BoundingBox");
        registry.register_with_name::<ae_plugin_api::Hidden>("Hidden");
        registry.register_with_name::<ae_plugin_api::Parent>("Parent");
        registry.register_with_name::<ae_plugin_api::Children>("Children");
        registry
            .register_with_name::<ae_plugin_api::PhysicsMaterialHandle>("PhysicsMaterialHandle");
        registry.register_with_name::<ae_plugin_api::TransformDirty>("TransformDirty");
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