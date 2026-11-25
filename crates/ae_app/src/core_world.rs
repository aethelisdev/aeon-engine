// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Authoritative ECS world, shared resources, event bus, clock, and physical input ownership.

use ae_core::Resources;
use ae_core::events::DynamicEventBus;
use ae_ecs::World;
use ae_input::InputManager;
use ae_time::Time;
use ae_world::{EntityGuid, GuidGenerator};

/// Core simulation data container shared cleanly across editor and standalone applications.
///
/// Isolates ECS entities, typed resources, events, frame timing, and input state away from
/// GPU device handles, OS windowing, and editor-only UI state.
pub struct CoreWorld {
    /// Entity Component System storage.
    pub world: World,
    /// Type-erased shared engine/gameplay resources.
    pub resources: Resources,
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

    #[test]
    fn test_core_world_initializes_non_nil_guid_and_empty_ecs() {
        let core = CoreWorld::new();
        assert_eq!(core.world.len(), 0);
        assert!(!core.active_world_guid.is_nil());
    }
}