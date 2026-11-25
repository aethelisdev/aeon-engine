// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Authoritative ECS world, shared resources, main-thread non-send resources, event bus,
//! frame clock, and physical input ownership.

use ae_core::Resources;
use ae_core::events::DynamicEventBus;
use ae_ecs::World;
use ae_input::InputManager;
use ae_time::Time;
use ae_world::{EntityGuid, GuidGenerator};
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Type-safe storage for main-thread-pinned (`!Send` / `!Sync`) subsystem resources such as
/// hardware audio output handles and loaded dynamic script backends.
#[derive(Default)]
pub struct NonSendResources {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl NonSendResources {
    /// Creates an empty [`NonSendResources`] container.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces a main-thread typed resource `T`.
    pub fn insert<T: Any>(&mut self, resource: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(resource));
    }

    /// Returns an immutable reference to resource `T` when present.
    #[must_use]
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|any| any.downcast_ref::<T>())
    }

    /// Returns a mutable reference to resource `T` when present.
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|any| any.downcast_mut::<T>())
    }

    /// Removes and returns resource `T` when present.
    pub fn remove<T: Any>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|any| any.downcast::<T>().ok().map(|boxed| *boxed))
    }
}

/// Core simulation data container shared cleanly across editor and standalone applications.
///
/// Isolates ECS entities, typed resources, events, frame timing, and input state away from
/// GPU device handles, OS windowing, and editor-only UI state.
pub struct CoreWorld {
    /// Entity Component System storage.
    pub world: World,
    /// Thread-safe shared engine and gameplay resources.
    pub resources: Resources,
    /// Main-thread-pinned (`!Send`) subsystem resources.
    pub local_resources: NonSendResources,
    /// Dynamic multi-channel event bus and module enablement flags.
    pub event_bus: DynamicEventBus,
    /// Monotonic frame clock and fixed-step physics accumulator.
    pub time: Time,
    /// Single-source physical keyboard/mouse/scroll input state.
    pub input: InputManager,
    /// Persistent world document identifier for save-slot and partitioned world tracking.
    pub active_world_guid: EntityGuid,
}

impl CoreWorld {
    /// Creates an empty [`CoreWorld`] with fresh ECS storage, resources, event bus, and clock.
    #[must_use]
    pub fn new() -> Self {
        Self {
            world: World::new(),
            resources: Resources::new(),
            local_resources: NonSendResources::new(),
            event_bus: DynamicEventBus::new(),
            time: Time::new(),
            input: InputManager::new(),
            active_world_guid: GuidGenerator::new().next_entity_guid(),
        }
    }

    /// Creates a [`CoreWorld`] using preloaded `world`, `input`, and `active_world_guid`.
    #[must_use]
    pub fn with_loaded(world: World, input: InputManager, active_world_guid: EntityGuid) -> Self {
        Self {
            world,
            resources: Resources::new(),
            local_resources: NonSendResources::new(),
            event_bus: DynamicEventBus::new(),
            time: Time::new(),
            input,
            active_world_guid,
        }
    }

    /// Clears transient pointer/key states when entering or resuming Play pause overlays.
    pub fn clear_inputs_for_pause(&mut self) {
        self.input.clear_mouse_delta();
        self.input.clear_pressed_keys();
    }
}

impl Default for CoreWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::marker::PhantomData;

    #[derive(Debug, PartialEq, Eq)]
    struct LocalToken {
        value: u32,
        _not_send_sync: PhantomData<*const ()>,
    }

    #[test]
    fn test_core_world_initializes_non_nil_guid_and_non_send_storage() {
        let mut core = CoreWorld::new();
        assert_eq!(core.world.len(), 0);
        assert!(!core.active_world_guid.is_nil());

        core.local_resources.insert(LocalToken {
            value: 42,
            _not_send_sync: PhantomData,
        });
        assert_eq!(
            core.local_resources.get::<LocalToken>().map(|v| v.value),
            Some(42)
        );
        assert_eq!(
            core.local_resources.remove::<LocalToken>().map(|v| v.value),
            Some(42)
        );
        assert!(core.local_resources.get::<LocalToken>().is_none());
    }
}