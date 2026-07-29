// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use ae_core::ecs::{
    BoundingRadius, Collider, Color, Light, ModelId, Name, PlayerTag, Position, RigidBody,
    Rotation, Scale, Shape, SpriteId, Velocity,
};

use serde::{Deserialize, Serialize};

/// Complete snapshot of an entity's components at a point in time.
/// Captures all serializable components (Position, Rotation, Scale, Color, Light,
/// Velocity, Name, Shape, SpriteId, BoundingRadius, PlayerTag, RigidBody, Collider, ModelId)
/// for use in undo/redo operations. Supports `capture()`, `apply()`, and `spawn()`
/// to enable full entity state restoration and re-creation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub name: Option<Name>,
    pub pos: Option<Position>,
    pub rot: Option<Rotation>,
    pub scale: Option<Scale>,
    pub shape: Option<Shape>,
    pub sprite_id: Option<SpriteId>,
    pub light: Option<Light>,
    pub color: Option<Color>,
    pub vel: Option<Velocity>,
    pub radius: Option<BoundingRadius>,
    pub is_player: bool,
    pub rigid_body: Option<RigidBody>,
    pub collider: Option<Collider>,
    pub model_id: Option<ModelId>,
}

impl EntitySnapshot {
    /// Captures the current state of all known components on the given entity.
    /// Returns a snapshot containing `Some(component)` for each component
    /// present on the entity, `None` for absent components.
    pub fn capture(world: &hecs::World, entity: hecs::Entity) -> Self {
        Self {
            name: world.get::<&Name>(entity).ok().map(|n| (*n).clone()),
            pos: world.get::<&Position>(entity).ok().map(|p| *p),
            rot: world.get::<&Rotation>(entity).ok().map(|r| *r),
            scale: world.get::<&Scale>(entity).ok().map(|s| *s),
            shape: world.get::<&Shape>(entity).ok().map(|s| *s),
            sprite_id: world.get::<&SpriteId>(entity).ok().map(|s| *s),
            light: world.get::<&Light>(entity).ok().map(|l| *l),
            color: world.get::<&Color>(entity).ok().map(|c| *c),
            vel: world.get::<&Velocity>(entity).ok().map(|v| *v),
            radius: world.get::<&BoundingRadius>(entity).ok().map(|r| *r),
            is_player: world.get::<&PlayerTag>(entity).is_ok(),
            rigid_body: world.get::<&RigidBody>(entity).ok().map(|r| *r),
            collider: world.get::<&Collider>(entity).ok().map(|c| *c),
            model_id: world.get::<&ModelId>(entity).ok().map(|m| *m),
        }
    }

    /// Restores all captured components onto an existing entity.
    /// Overwrites any existing components with the snapshot values.
    /// Used by `Command::Delete::undo()` to restore a deleted entity's state.
    pub fn apply(&self, world: &mut hecs::World, entity: hecs::Entity) {
        if let Some(n) = &self.name {
            let _ = world.insert_one(entity, n.clone());
        }
        if let Some(p) = self.pos {
            let _ = world.insert_one(entity, p);
        }
        if let Some(r) = self.rot {
            let _ = world.insert_one(entity, r);
        }
        if let Some(s) = self.scale {
            let _ = world.insert_one(entity, s);
        }
        if let Some(s) = self.shape {
            let _ = world.insert_one(entity, s);
        }
        if let Some(s) = self.sprite_id {
            let _ = world.insert_one(entity, s);
        }
        if let Some(l) = self.light {
            let _ = world.insert_one(entity, l);
        }
        if let Some(c) = self.color {
            let _ = world.insert_one(entity, c);
        }
        if let Some(v) = self.vel {
            let _ = world.insert_one(entity, v);
        }
        if let Some(r) = self.radius {
            let _ = world.insert_one(entity, r);
        }
        if self.is_player {
            let _ = world.insert_one(entity, PlayerTag);
        }
        if let Some(rb) = self.rigid_body {
            let _ = world.insert_one(entity, rb);
        }
        if let Some(col) = self.collider {
            let _ = world.insert_one(entity, col);
        }
        if let Some(m) = self.model_id {
            let _ = world.insert_one(entity, m);
        }
    }

    /// Spawns a new entity in the world and applies this snapshot's components.
    /// Returns the new entity handle. Used by undo-delete to re-create
    /// a previously destroyed entity with all its original components.
    pub fn spawn(&self, world: &mut hecs::World) -> hecs::Entity {
        let entity = world.spawn(());
        self.apply(world, entity);
        entity
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
                    Property::Position(old, _) => {
                        let _ = world.insert_one(*entity, *old);
                    }
                    Property::Rotation(old, _) => {
                        let _ = world.insert_one(*entity, *old);
                    }
                    Property::Scale(old, _) => {
                        let _ = world.insert_one(*entity, *old);
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
                    Property::Position(_, new) => {
                        let _ = world.insert_one(*entity, *new);
                    }
                    Property::Rotation(_, new) => {
                        let _ = world.insert_one(*entity, *new);
                    }
                    Property::Scale(_, new) => {
                        let _ = world.insert_one(*entity, *new);
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
#[derive(Clone, Debug)]
pub enum Property {
    Position(Position, Position),
    Rotation(Rotation, Rotation),
    Scale(Scale, Scale),
    Name(String, String),
    Light(Light, Light),
    Color(Color, Color),
}