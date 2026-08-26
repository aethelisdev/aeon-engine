// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dynamic Inspector UI Component Registry (`Inspectable` architecture).
//!
//! Provides a decoupled, extensible component inspection and "Add Component" menu
//! system for dynamic entity property visualization and management. Eliminates hardcoded
//! `if let Ok(...)` chains and allows modular component UI definitions.
//!

use crate::ui::EngineUiAction;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Execution context passed to each component UI handler during inspector rendering.
pub struct InspectorContext<'a> {
    pub world: &'a hecs::World,
    pub entity: hecs::Entity,
    pub ui_actions: &'a mut Vec<EngineUiAction>,
    pub editor_state: &'a ae_editor::editor_state::EditorState,
    pub camera: &'a ae_renderer::camera::Camera,
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    pub inspector_color_hex: &'a mut String,
    pub saved_swatches: &'a mut Vec<[f32; 4]>,
}

/// Interface for custom component UI rendering and lifecycle in the Inspector.
pub trait ComponentUiHandler: Send + Sync {
    /// Canonical component name matching ECS registration (e.g. "RigidBody", "Light").
    fn component_name(&self) -> &'static str;

    /// Card display title, icon, and header accent color.
    fn card_header(&self) -> (&'static str, &'static str, egui::Color32);

    /// Category and display label for the "Add Component" menu (e.g. ("Physics", "RigidBody")).
    fn menu_category(&self) -> (&'static str, &'static str);

    /// Checks if this component is attached to the entity in the ECS world.
    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool;

    /// Whether this component can be removed via the trash icon button.
    fn is_removable(&self) -> bool {
        true
    }

    /// Renders the component's inner UI inside the styled inspector card frame.
    fn render_ui(&self, ui: &mut egui::Ui, ctx: &mut InspectorContext);

    /// Adds the default instance of this component to the entity via generic `EngineUiAction::AddComponent`.
    fn add_default_to_entity(
        &self,
        _world: &hecs::World,
        entity: hecs::Entity,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        ui_actions.push(EngineUiAction::AddComponent(entity, self.component_name()));
    }

    /// Removes this component from the entity via generic `EngineUiAction::RemoveComponent`.
    fn remove_from_entity(&self, entity: hecs::Entity, ui_actions: &mut Vec<EngineUiAction>) {
        ui_actions.push(EngineUiAction::RemoveComponent(
            entity,
            self.component_name(),
        ));
    }
}

/// Central registry managing all component UI inspector handlers.
#[derive(Default)]
pub struct InspectorUiRegistry {
    handlers: Vec<Box<dyn ComponentUiHandler>>,
}

impl InspectorUiRegistry {
    /// Creates a new empty inspector registry.
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Registers a new component UI handler.
    pub fn register<H: ComponentUiHandler + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }

    /// Returns all registered component UI handlers.
    pub fn handlers(&self) -> &[Box<dyn ComponentUiHandler>] {
        &self.handlers
    }

    /// Returns registered components grouped by their menu category.
    pub fn grouped_by_category(&self) -> BTreeMap<&'static str, Vec<&dyn ComponentUiHandler>> {
        let mut map: BTreeMap<&'static str, Vec<&dyn ComponentUiHandler>> = BTreeMap::new();
        for handler in &self.handlers {
            let (category, _) = handler.menu_category();
            map.entry(category).or_default().push(&**handler);
        }
        map
    }

    /// Builds the default engine UI inspector registry with all built-in components.
    pub fn default_engine_registry() -> Self {
        let mut registry = Self::new();

        // 1. Rendering
        registry.register(super::appearance::AppearanceUiHandler);
        registry.register(super::appearance::LightUiHandler);
        registry.register(super::lod::LodGroupUiHandler);

        // 2. Physics
        registry.register(super::physics::RigidBodyUiHandler);
        registry.register(super::physics::ColliderUiHandler);
        registry.register(super::physics::PhysicsMaterialUiHandler);
        registry.register(super::physics::CharacterControllerUiHandler);

        // 3. Audio
        registry.register(super::audio::AudioSourceUiHandler);
        registry.register(super::audio::AudioListenerUiHandler);

        // 4. Gameplay
        registry.register(super::behavior::RotatorUiHandler);
        registry.register(super::behavior::MovingPlatformUiHandler);
        registry.register(super::behavior::TriggerZoneUiHandler);
        registry.register(super::behavior::DestructibleTargetUiHandler);
        registry.register(super::behavior::CharacterActionUiHandler);
        registry.register(super::behavior::PlayerTagUiHandler);

        // 5. Animation
        registry.register(super::animation::AnimationUiHandler);

        // 6. Hierarchy
        registry.register(super::parenting::ParentingUiHandler);

        registry
    }

    /// Returns a reference to the global engine inspector UI registry singleton.
    pub fn global() -> &'static InspectorUiRegistry {
        static REGISTRY: OnceLock<InspectorUiRegistry> = OnceLock::new();
        REGISTRY.get_or_init(Self::default_engine_registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_registry_global_initialization() {
        let registry = InspectorUiRegistry::global();
        assert!(
            registry.handlers().len() >= 10,
            "Builtin inspector registry should register all standard engine components"
        );
    }

    #[test]
    fn test_inspector_registry_grouped_by_category() {
        let registry = InspectorUiRegistry::global();
        let grouped = registry.grouped_by_category();

        assert!(grouped.contains_key("Rendering"));
        assert!(grouped.contains_key("Physics"));
        assert!(grouped.contains_key("Audio"));
        assert!(grouped.contains_key("Gameplay"));
        assert!(grouped.contains_key("Animation"));
        assert!(grouped.contains_key("Hierarchy"));

        let physics_handlers = grouped.get("Physics").expect("Physics category present");
        let names: Vec<&str> = physics_handlers
            .iter()
            .map(|h| h.component_name())
            .collect();
        assert!(names.contains(&"RigidBody"));
        assert!(names.contains(&"Collider"));
        assert!(names.contains(&"CharacterController"));
    }

    #[test]
    fn test_component_ui_handler_lifecycle() {
        let registry = InspectorUiRegistry::global();
        let mut world = hecs::World::new();
        let entity = world.spawn((
            ae_core::ecs::Name("TestEntity".to_string()),
            ae_core::ecs::Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        ));

        // Find RigidBody handler
        let rb_handler = registry
            .handlers()
            .iter()
            .find(|h| h.component_name() == "RigidBody")
            .expect("RigidBody handler registered");

        assert!(!rb_handler.has_component(&world, entity));

        let mut actions = Vec::new();
        rb_handler.add_default_to_entity(&world, entity, &mut actions);
        assert_eq!(actions.len(), 2); // AddRigidBody + AddCollider default pair

        // Add component to world directly
        world
            .insert_one(
                entity,
                ae_core::ecs::RigidBody {
                    body_type: ae_core::ecs::RigidBodyType::Dynamic,
                    mass: 1.0,
                    gravity_scale: 1.0,
                },
            )
            .expect("Inserted RigidBody");

        assert!(rb_handler.has_component(&world, entity));

        actions.clear();
        rb_handler.remove_from_entity(entity, &mut actions);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            EngineUiAction::RemoveComponent(ent, name) => {
                assert_eq!(*ent, entity);
                assert_eq!(*name, "RigidBody");
            }
            _ => panic!("Expected RemoveComponent action"),
        }
    }

    #[test]
    fn test_all_inspector_ui_handlers_produce_valid_actions() {
        let registry = InspectorUiRegistry::global();
        let world = hecs::World::new();
        let entity = hecs::Entity::DANGLING;

        for handler in registry.handlers() {
            let mut add_actions = Vec::new();
            handler.add_default_to_entity(&world, entity, &mut add_actions);
            assert!(
                !add_actions.is_empty(),
                "Handler '{}' must produce at least one action on add_default_to_entity",
                handler.component_name()
            );

            if handler.is_removable() {
                let mut remove_actions = Vec::new();
                handler.remove_from_entity(entity, &mut remove_actions);
                assert!(
                    !remove_actions.is_empty(),
                    "Handler '{}' must produce at least one action on remove_from_entity",
                    handler.component_name()
                );
            }
        }
    }
}