// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dynamic Undo/Redo and Entity Snapshot architecture for the editor.
//!
//! Integrates with the central `ComponentRegistry` to provide 100% automated,
//! extensible entity state capture, restoration, serialization, and undo history.
//!

use ae_core::ecs::{Color, ComponentRegistry, Light, Name, Position, Rotation, Scale};
use serde::{Deserialize, Serialize};

/// Serialized component entry stored within an `EntitySnapshot`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedComponent {
    /// Human-readable type identifier corresponding to a registered `ComponentHandler`.
    pub type_name: String,
    /// Serialized JSON component payload.
    pub data: Vec<u8>,
}

/// Complete dynamic snapshot of an entity's components at a point in time.
/// Automatically captures all registered components via `ComponentRegistry` without
/// requiring hardcoded struct fields. Fully extensible for new components and custom plugins.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySnapshot {
    /// Dynamic list of captured component payloads.
    pub components: Vec<SerializedComponent>,
}

impl EntitySnapshot {
    /// Creates an empty entity snapshot.
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    /// Captures the current state of all registered components on `entity` using the global registry.
    pub fn capture(world: &hecs::World, entity: hecs::Entity) -> Self {
        Self::capture_with_registry(world, entity, ComponentRegistry::global())
    }

    /// Captures all components present on `entity` registered in the provided `registry`.
    pub fn capture_with_registry(
        world: &hecs::World,
        entity: hecs::Entity,
        registry: &ComponentRegistry,
    ) -> Self {
        let mut components = Vec::new();
        for handler in registry.handlers() {
            if let Some(data) = handler.capture(world, entity) {
                components.push(SerializedComponent {
                    type_name: handler.type_name().to_string(),
                    data,
                });
            }
        }
        Self { components }
    }

    /// Restores all captured components onto an existing entity using the global registry.
    pub fn apply(&self, world: &mut hecs::World, entity: hecs::Entity) {
        self.apply_with_registry(world, entity, ComponentRegistry::global());
    }

    /// Restores all captured components onto an existing entity using the provided `registry`.
    pub fn apply_with_registry(
        &self,
        world: &mut hecs::World,
        entity: hecs::Entity,
        registry: &ComponentRegistry,
    ) {
        for comp in &self.components {
            if let Some(handler) = registry.get_by_name(&comp.type_name) {
                let _ = handler.apply(world, entity, &comp.data);
            }
        }
    }

    /// Spawns a new entity in the world and applies this snapshot's components.
    /// Returns the new entity handle. Used by undo-delete to re-create
    /// a previously destroyed entity with all its original components.
    pub fn spawn(&self, world: &mut hecs::World) -> hecs::Entity {
        self.spawn_with_registry(world, ComponentRegistry::global())
    }

    /// Spawns a new entity in `world` and applies components using the provided `registry`.
    pub fn spawn_with_registry(
        &self,
        world: &mut hecs::World,
        registry: &ComponentRegistry,
    ) -> hecs::Entity {
        let entity = world.spawn(());
        self.apply_with_registry(world, entity, registry);
        entity
    }

    /// Deserializes and returns a specific component `T` if present in the snapshot.
    pub fn get<T: hecs::Component + serde::de::DeserializeOwned>(&self) -> Option<T> {
        let type_id = std::any::TypeId::of::<T>();
        let handler = ComponentRegistry::global().get_by_type_id(type_id)?;
        let name = handler.type_name();
        for comp in &self.components {
            if comp.type_name == name {
                return serde_json::from_slice(&comp.data).ok();
            }
        }
        None
    }

    /// Returns true if a component of type `T` is stored in the snapshot.
    pub fn has<T: hecs::Component>(&self) -> bool {
        let type_id = std::any::TypeId::of::<T>();
        if let Some(handler) = ComponentRegistry::global().get_by_type_id(type_id) {
            let name = handler.type_name();
            self.components.iter().any(|c| c.type_name == name)
        } else {
            false
        }
    }

    /// Returns the total number of components captured in the snapshot.
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Returns true if no components are stored in the snapshot.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

/// Undoable/redoable command representing a single atomic operation.
/// Four variants:
/// - `Spawn`: entity creation (undo = despawn, redo = re-spawn)
/// - `Delete`: entity destruction (undo = re-spawn, redo = despawn)
/// - `Modify`: single property change (undo = restore old, redo = apply new)
/// - `Batch`: groups multiple commands for atomic multi-entity operations
/// Entity ID remapping is handled via `remap_entity()` when undo/redo
/// produces a different entity handle (since `hecs` may reuse or assign new IDs).
#[derive(Clone, Debug)]
pub enum Command {
    Spawn(hecs::Entity, EntitySnapshot),
    Delete(hecs::Entity, EntitySnapshot),
    Modify(hecs::Entity, Property),
    Batch(Vec<Command>),
}

impl Command {
    /// Reverses this command. Returns `Some((old_id, new_id))` if the operation
    /// changed an entity's ID (e.g., re-spawning a deleted entity produces a new handle).
    pub fn undo(&mut self, world: &mut hecs::World) -> Option<(hecs::Entity, hecs::Entity)> {
        match self {
            Command::Spawn(entity, _snapshot) => {
                let _ = world.despawn(*entity);
                None
            }
            Command::Delete(entity, snapshot) => {
                let old_entity = *entity;
                let new_entity = snapshot.spawn(world);
                *entity = new_entity;
                if old_entity != new_entity {
                    Some((old_entity, new_entity))
                } else {
                    None
                }
            }
            Command::Modify(entity, property) => {
                match property {
                    Property::Component {
                        type_name,
                        old_data,
                        ..
                    } => {
                        if let Some(handler) = ComponentRegistry::global().get_by_name(type_name) {
                            let _ = handler.apply(world, *entity, old_data);
                        }
                        let _ = world.insert_one(*entity, ae_core::ecs::TransformDirty);
                    }
                    Property::Position(old, _) => {
                        let _ = world.insert_one(*entity, *old);
                        let _ = world.insert_one(*entity, ae_core::ecs::TransformDirty);
                    }
                    Property::Rotation(old, _) => {
                        let _ = world.insert_one(*entity, *old);
                        let _ = world.insert_one(*entity, ae_core::ecs::TransformDirty);
                    }
                    Property::Scale(old, _) => {
                        let _ = world.insert_one(*entity, *old);
                        let _ = world.insert_one(*entity, ae_core::ecs::TransformDirty);
                    }
                    Property::Name(old, _) => {
                        let _ = world.insert_one(*entity, Name(old.clone()));
                    }
                    Property::Light(old, _) => {
                        let _ = world.insert_one(*entity, *old);
                    }
                    Property::Color(old, _) => {
                        let _ = world.insert_one(*entity, *old);
                    }
                }
                None
            }
            Command::Batch(cmds) => {
                let mut remap = None;
                for cmd in cmds.iter_mut().rev() {
                    if let Some(r) = cmd.undo(world) {
                        remap = Some(r);
                    }
                }
                remap
            }
        }
    }

    /// Executes the redo command. If an Entity ID change occurred, returns `Some((old_id, new_id))`.
    pub fn redo(&mut self, world: &mut hecs::World) -> Option<(hecs::Entity, hecs::Entity)> {
        match self {
            Command::Spawn(entity, snapshot) => {
                if !world.contains(*entity) {
                    let old_entity = *entity;
                    let new_entity = snapshot.spawn(world);
                    *entity = new_entity;
                    if old_entity != new_entity {
                        return Some((old_entity, new_entity));
                    }
                }
                None
            }
            Command::Delete(entity, _snapshot) => {
                let _ = world.despawn(*entity);
                None
            }
            Command::Modify(entity, property) => {
                match property {
                    Property::Component {
                        type_name,
                        new_data,
                        ..
                    } => {
                        if let Some(handler) = ComponentRegistry::global().get_by_name(type_name) {
                            let _ = handler.apply(world, *entity, new_data);
                        }
                        let _ = world.insert_one(*entity, ae_core::ecs::TransformDirty);
                    }
                    Property::Position(_, new) => {
                        let _ = world.insert_one(*entity, *new);
                        let _ = world.insert_one(*entity, ae_core::ecs::TransformDirty);
                    }
                    Property::Rotation(_, new) => {
                        let _ = world.insert_one(*entity, *new);
                        let _ = world.insert_one(*entity, ae_core::ecs::TransformDirty);
                    }
                    Property::Scale(_, new) => {
                        let _ = world.insert_one(*entity, *new);
                        let _ = world.insert_one(*entity, ae_core::ecs::TransformDirty);
                    }
                    Property::Name(_, new) => {
                        let _ = world.insert_one(*entity, Name(new.clone()));
                    }
                    Property::Light(_, new) => {
                        let _ = world.insert_one(*entity, *new);
                    }
                    Property::Color(_, new) => {
                        let _ = world.insert_one(*entity, *new);
                    }
                }
                None
            }
            Command::Batch(cmds) => {
                let mut remap = None;
                for cmd in cmds.iter_mut() {
                    if let Some(r) = cmd.redo(world) {
                        remap = Some(r);
                    }
                }
                remap
            }
        }
    }

    /// Updates the entity reference within the command (old → new ID).
    /// Applied to all commands in the stack when an entity ID changes during undo/redo.
    pub fn remap_entity(&mut self, old: hecs::Entity, new: hecs::Entity) {
        match self {
            Command::Spawn(entity, _) | Command::Delete(entity, _) | Command::Modify(entity, _) => {
                if *entity == old {
                    *entity = new;
                }
            }
            Command::Batch(cmds) => {
                for cmd in cmds {
                    cmd.remap_entity(old, new);
                }
            }
        }
    }
}

/// Trackable entity property for single-field undo/redo.
/// Each variant stores `(old_value, new_value)` to enable bidirectional
/// restore. Used by `Command::Modify` for inspector-driven property edits.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Property {
    /// Generic component modification for any registered component type
    Component {
        type_name: String,
        old_data: Vec<u8>,
        new_data: Vec<u8>,
    },
    Position(Position, Position),
    Rotation(Rotation, Rotation),
    Scale(Scale, Scale),
    Name(String, String),
    Light(Light, Light),
    Color(Color, Color),
}

impl Property {
    /// Creates a generic component modification property for any registered component type.
    pub fn from_component<T: hecs::Component + serde::Serialize>(
        old: &T,
        new: &T,
    ) -> Result<Self, String> {
        let full_name = std::any::type_name::<T>();
        let type_name = full_name
            .rsplit("::")
            .next()
            .unwrap_or(full_name)
            .to_string();
        let old_data = serde_json::to_vec(old).map_err(|e| e.to_string())?;
        let new_data = serde_json::to_vec(new).map_err(|e| e.to_string())?;
        Ok(Self::Component {
            type_name,
            old_data,
            new_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::ecs::{
        CharacterController, Collider, ColliderShape, Parent, RigidBody, RigidBodyType,
    };

    /// Tests that EntitySnapshot captures and restores CharacterController correctly.
    #[test]
    fn test_entity_snapshot_character_controller_retention() {
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Position::new(1.0, 2.0, 3.0),
            CharacterController {
                radius: 0.4,
                height: 1.8,
                center_y: 0.0,
                max_slope_climb_angle: 45.0,
                step_height: 0.3,
                is_grounded: true,
            },
        ));

        let snapshot = EntitySnapshot::capture(&world, entity);
        assert!(snapshot.has::<CharacterController>());
        assert_eq!(snapshot.component_count(), 2);

        let restored_ent = snapshot.spawn(&mut world);
        let cc = world.get::<&CharacterController>(restored_ent);
        assert!(cc.is_ok());
        let cc = cc.unwrap();
        assert_eq!(cc.radius, 0.4);
        assert_eq!(cc.height, 1.8);
    }

    /// Tests that EntitySnapshot captures complex hierarchies and physics components dynamically.
    #[test]
    fn test_entity_snapshot_dynamic_hierarchy_and_physics() {
        let mut world = hecs::World::new();
        let parent_entity = world.spawn((
            Name("ParentEntity".to_string()),
            Position::new(0.0, 5.0, 0.0),
        ));

        let child_entity = world.spawn((
            Name("ChildEntity".to_string()),
            Position::new(1.0, 2.0, 3.0),
            Rotation::identity(),
            Scale::new(2.0, 2.0, 2.0),
            Parent(parent_entity),
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 15.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [1.0, 1.0, 1.0],
                },
                friction: 0.7,
                restitution: 0.3,
                is_sensor: false,
            },
        ));

        let snapshot = EntitySnapshot::capture(&world, child_entity);
        assert_eq!(snapshot.component_count(), 7);
        assert!(snapshot.has::<Name>());
        assert!(snapshot.has::<Position>());
        assert!(snapshot.has::<Rotation>());
        assert!(snapshot.has::<Scale>());
        assert!(snapshot.has::<Parent>());
        assert!(snapshot.has::<RigidBody>());
        assert!(snapshot.has::<Collider>());

        let restored_ent = snapshot.spawn(&mut world);
        assert_eq!(world.get::<&Name>(restored_ent).unwrap().0, "ChildEntity");
        assert_eq!(world.get::<&Parent>(restored_ent).unwrap().0, parent_entity);
        assert_eq!(world.get::<&RigidBody>(restored_ent).unwrap().mass, 15.0);
    }

    /// Tests that Property::from_component enables generic single-field undo and redo.
    #[test]
    fn test_generic_property_component_undo_redo() {
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Position::new(0.0, 0.0, 0.0),
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 10.0,
                gravity_scale: 0.0,
            },
        ));

        let old_rb = RigidBody {
            body_type: RigidBodyType::Static,
            mass: 10.0,
            gravity_scale: 0.0,
        };
        let new_rb = RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 50.0,
            gravity_scale: 2.0,
        };

        let prop = Property::from_component(&old_rb, &new_rb).unwrap();
        let mut cmd = Command::Modify(entity, prop);

        // Apply change (redo)
        cmd.redo(&mut world);
        {
            let current_rb = world.get::<&RigidBody>(entity).unwrap();
            assert_eq!(current_rb.mass, 50.0);
            assert_eq!(current_rb.body_type, RigidBodyType::Dynamic);
        }

        // Revert change (undo)
        cmd.undo(&mut world);
        {
            let current_rb = world.get::<&RigidBody>(entity).unwrap();
            assert_eq!(current_rb.mass, 10.0);
            assert_eq!(current_rb.body_type, RigidBodyType::Static);
        }
    }

    /// Tests that Property::Component for Velocity correctly undos, redos, and marks TransformDirty.
    #[test]
    fn test_generic_velocity_component_undo_redo() {
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Position::new(0.0, 0.0, 0.0),
            ae_core::ecs::Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        ));

        let old_vel = ae_core::ecs::Velocity {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let new_vel = ae_core::ecs::Velocity {
            x: 5.0,
            y: 10.0,
            z: -2.5,
        };

        let prop = Property::from_component(&old_vel, &new_vel).unwrap();
        let mut cmd = Command::Modify(entity, prop);

        // Apply change (redo)
        cmd.redo(&mut world);
        {
            let current_vel = world.get::<&ae_core::ecs::Velocity>(entity).unwrap();
            assert_eq!(current_vel.x, 5.0);
            assert_eq!(current_vel.y, 10.0);
            assert_eq!(current_vel.z, -2.5);
            assert!(world.get::<&ae_core::ecs::TransformDirty>(entity).is_ok());
        }

        // Revert change (undo)
        let _ = world.remove_one::<ae_core::ecs::TransformDirty>(entity);
        cmd.undo(&mut world);
        {
            let current_vel = world.get::<&ae_core::ecs::Velocity>(entity).unwrap();
            assert_eq!(current_vel.x, 0.0);
            assert_eq!(current_vel.y, 0.0);
            assert_eq!(current_vel.z, 0.0);
            assert!(world.get::<&ae_core::ecs::TransformDirty>(entity).is_ok());
        }
    }
}