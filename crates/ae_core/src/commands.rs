// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Deferred Entity Command Buffer Infrastructure.
//!
//! Provides a safe, decoupled mechanism for scheduling entity creation, deletion,
//! and component mutations during gameplay loops and system iterations without
//! causing borrow checker or iterator invalidation errors in `hecs::World`.
//!

/// Deferred closure command type executed against mutable ECS world storage.
pub type CommandFn = Box<dyn FnOnce(&mut hecs::World) + Send + Sync>;

/// Deferred entity command buffer queueing mutations to be applied at frame boundaries.
/// In ECS systems, mutating the entity topology (spawning new entities, despawning dead
/// actors, or inserting components) while actively querying `hecs::World` leads to aliasing
/// borrow conflicts or iterator invalidation. The `EntityCommandBuffer` captures these mutation
/// intents and executes them sequentially in a single consolidated mutation phase.
#[derive(Default)]
pub struct EntityCommandBuffer {
    commands: Vec<CommandFn>,
}

impl EntityCommandBuffer {
    /// Creates a new empty `EntityCommandBuffer`.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Returns `true` if there are no pending commands in the queue.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Returns the number of scheduled commands currently in the queue.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Clears all scheduled commands from the queue without executing them.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Schedules an arbitrary closure mutation to be executed against `hecs::World`.
    pub fn add_command<F>(&mut self, command: F)
    where
        F: FnOnce(&mut hecs::World) + Send + Sync + 'static,
    {
        self.commands.push(Box::new(command));
    }

    /// Schedules the deletion (despawning) of the target entity.
    /// If the entity does not exist when the command is processed, the operation is safely ignored.
    pub fn despawn(&mut self, entity: hecs::Entity) {
        self.add_command(move |world| {
            let _ = world.despawn(entity);
        });
    }

    /// Schedules attaching a component instance onto the target entity.
    pub fn insert_one<T: hecs::Component>(&mut self, entity: hecs::Entity, component: T) {
        self.add_command(move |world| {
            let _ = world.insert_one(entity, component);
        });
    }

    /// Schedules removing a component of type `T` from the target entity.
    pub fn remove_one<T: hecs::Component>(&mut self, entity: hecs::Entity) {
        self.add_command(move |world| {
            let _ = world.remove_one::<T>(entity);
        });
    }

    /// Schedules entity creation with a custom factory closure.
    pub fn spawn_with<F>(&mut self, factory: F)
    where
        F: FnOnce(&mut hecs::World) -> hecs::Entity + Send + Sync + 'static,
    {
        self.add_command(move |world| {
            let _ = factory(world);
        });
    }

    /// Consumes and executes all queued commands sequentially against `world`.
    /// Employs move semantics (`std::mem::take`) to ensure zero redundant cloning
    /// and resets the internal buffer capacity for reuse in subsequent frames.
    pub fn apply(&mut self, world: &mut hecs::World) {
        if self.commands.is_empty() {
            return;
        }

        let pending_commands = std::mem::take(&mut self.commands);
        for command in pending_commands {
            command(world);
        }
    }
}