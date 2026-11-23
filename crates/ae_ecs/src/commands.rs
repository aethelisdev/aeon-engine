// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Deferred ECS command queue executed at safe synchronization boundaries.

use crate::component::Component;
use crate::entity::Entity;
use crate::world::World;

/// Deferred closure command executed against mutable [`World`] storage.
pub type CommandFn = Box<dyn FnOnce(&mut World) + Send + Sync>;

/// Deferred entity command buffer queueing structural mutations for end-of-phase application.
///
/// Captures spawn, despawn, and component insertion/removal operations safely while query
/// borrows over [`World`] are active.
#[derive(Default)]
pub struct EntityCommandBuffer {
    commands: Vec<CommandFn>,
}

impl EntityCommandBuffer {
    /// Creates an empty [`EntityCommandBuffer`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Returns `true` if no deferred commands are queued.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Returns the number of queued deferred commands.
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Discards all queued commands without executing them.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Queues an arbitrary world mutation closure for deferred execution.
    pub fn add_command<F>(&mut self, command: F)
    where
        F: FnOnce(&mut World) + Send + Sync + 'static,
    {
        self.commands.push(Box::new(command));
    }

    /// Queues despawning `entity` when [`Self::apply`] runs.
    pub fn despawn(&mut self, entity: Entity) {
        self.add_command(move |world| {
            let _ = world.despawn(entity);
        });
    }

    /// Queues attaching `component` to `entity` when [`Self::apply`] runs.
    pub fn insert_one<T: Component>(&mut self, entity: Entity, component: T) {
        self.add_command(move |world| {
            let _ = world.insert_one(entity, component);
        });
    }

    /// Queues detaching component type `T` from `entity` when [`Self::apply`] runs.
    pub fn remove_one<T: Component>(&mut self, entity: Entity) {
        self.add_command(move |world| {
            let _ = world.remove_one::<T>(entity);
        });
    }

    /// Queues executing `factory` to create a new entity inside [`Self::apply`].
    pub fn spawn_with<F>(&mut self, factory: F)
    where
        F: FnOnce(&mut World) -> Entity + Send + Sync + 'static,
    {
        self.add_command(move |world| {
            let _ = factory(world);
        });
    }

    /// Executes all queued commands sequentially against `world` and clears the buffer.
    pub fn apply(&mut self, world: &mut World) {
        if self.commands.is_empty() {
            return;
        }
        let pending = std::mem::take(&mut self.commands);
        for command in pending {
            command(world);
        }
    }
}