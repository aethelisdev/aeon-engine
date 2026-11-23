// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dynamic Event Bus and standard core engine event definitions.

use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, VecDeque};

/// Core engine modules that can be dynamically enabled/disabled at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineModule {
    /// Rapier3D & Rapier2D physics simulation, character controller, and collision detection.
    Physics,
    /// Spatial and ambient hardware audio playback system.
    Audio,
    /// 3D graphics rendering pipeline including PBR, shadows, skybox, and post-processing.
    Render,
    /// Hardware-accelerated 2D sprite batching, sorting, and tile rendering pipeline.
    Render2D,
}

/// The mode in which the engine runs: Edit or Play.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum EngineMode {
    Edit,
    Play,
}

/// Empty trait representing any event within the engine.
pub trait Event: Any + Send + Sync {}

/// Type-agnostic dynamic event bus.
pub struct DynamicEventBus {
    queues: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    pub enabled_modules: HashSet<EngineModule>,
}

impl DynamicEventBus {
    /// Creates a new DynamicEventBus with all modules enabled.
    pub fn new() -> Self {
        let mut enabled_modules = HashSet::new();
        enabled_modules.insert(EngineModule::Physics);
        enabled_modules.insert(EngineModule::Audio);
        enabled_modules.insert(EngineModule::Render);
        enabled_modules.insert(EngineModule::Render2D);
        Self {
            queues: HashMap::new(),
            enabled_modules,
        }
    }

    /// Enqueues an event onto its specific type queue.
    pub fn send<E: Event>(&mut self, event: E) {
        let queue = self
            .queues
            .entry(TypeId::of::<E>())
            .or_insert_with(|| Box::new(VecDeque::<E>::new()));
        if let Some(q) = queue.downcast_mut::<VecDeque<E>>() {
            q.push_back(event);
        }
    }

    /// Takes and returns all queued events of the specified type.
    pub fn receive<E: Event>(&mut self) -> Option<VecDeque<E>> {
        let queue = self.queues.get_mut(&TypeId::of::<E>())?;
        if let Some(q) = queue.downcast_mut::<VecDeque<E>>() {
            Some(std::mem::take(q))
        } else {
            None
        }
    }

    /// Checks if there are any pending events for the specified type.
    pub fn has_events<E: Event>(&self) -> bool {
        self.queues
            .get(&TypeId::of::<E>())
            .and_then(|any| any.downcast_ref::<VecDeque<E>>())
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }

    /// Clears all event queues across the bus.
    pub fn clear(&mut self) {
        self.queues.clear();
    }

    /// Checks whether a specific engine subsystem module is enabled.
    pub fn is_module_enabled(&self, module: EngineModule) -> bool {
        self.enabled_modules.contains(&module)
    }

    /// Enables or disables a specific engine subsystem module.
    pub fn set_module_enabled(&mut self, module: EngineModule, enabled: bool) {
        if enabled {
            self.enabled_modules.insert(module);
        } else {
            self.enabled_modules.remove(&module);
        }
    }
}

impl Default for DynamicEventBus {
    fn default() -> Self {
        Self::new()
    }
}

// --- Standard Core Event Definitions ---

/// Event fired when two solid physics bodies begin contact.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionEnter {
    /// First entity involved in the physical contact.
    pub entity_a: ae_ecs::Entity,
    /// Second entity involved in the physical contact.
    pub entity_b: ae_ecs::Entity,
    /// World-space contact point where physical impact occurred, if available.
    pub contact_point: Option<[f32; 3]>,
    /// Contact surface normal vector, if available.
    pub normal: Option<[f32; 3]>,
    /// Magnitude of physical impulse/force exchanged during collision.
    pub impulse: f32,
}
impl Event for CollisionEnter {}

impl CollisionEnter {
    /// Creates a new `CollisionEnter` event with entity IDs and default contact info.
    pub fn new(entity_a: ae_ecs::Entity, entity_b: ae_ecs::Entity) -> Self {
        Self {
            entity_a,
            entity_b,
            contact_point: None,
            normal: None,
            impulse: 0.0,
        }
    }

    /// Creates a new `CollisionEnter` event with full contact and impulse parameters.
    pub fn with_details(
        entity_a: ae_ecs::Entity,
        entity_b: ae_ecs::Entity,
        contact_point: Option<[f32; 3]>,
        normal: Option<[f32; 3]>,
        impulse: f32,
    ) -> Self {
        Self {
            entity_a,
            entity_b,
            contact_point,
            normal,
            impulse,
        }
    }
}

/// Event fired when two solid physics bodies end contact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionExit {
    pub entity_a: ae_ecs::Entity,
    pub entity_b: ae_ecs::Entity,
}
impl Event for CollisionExit {}

/// Event fired when an entity enters a sensor/trigger volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerEnter {
    pub entity_a: ae_ecs::Entity,
    pub entity_b: ae_ecs::Entity,
}
impl Event for TriggerEnter {}

/// Event fired when an entity leaves a sensor/trigger volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerExit {
    pub entity_a: ae_ecs::Entity,
    pub entity_b: ae_ecs::Entity,
}
impl Event for TriggerExit {}

/// Event fired when an entity is spawned into the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntitySpawned(pub ae_ecs::Entity);
impl Event for EntitySpawned {}

/// Event fired when an entity is destroyed from the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityDestroyed(pub ae_ecs::Entity);
impl Event for EntityDestroyed {}

/// Physics simulation tick marker event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicsTick;
impl Event for PhysicsTick {}

/// Physics state update marker event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicsUpdate;
impl Event for PhysicsUpdate {}

/// Audio playback command event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaySound(pub &'static str);
impl Event for PlaySound {}

/// Audio stop command event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopSound(pub &'static str);
impl Event for StopSound {}

/// Audio mute toggle event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioMute;
impl Event for AudioMute {}

/// Render frame marker event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderFrame;
impl Event for RenderFrame {}

/// Viewport resize notification event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportResize;
impl Event for ViewportResize {}

/// Material modification event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialChanged;
impl Event for MaterialChanged {}

/// Event broadcast when a raycast or projectile strikes a target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastHitEvent {
    pub shooter: Option<ae_ecs::Entity>,
    pub target: ae_ecs::Entity,
    pub hit_point: [f32; 3],
    pub hit_normal: [f32; 3],
    pub damage: f32,
}
impl Event for RaycastHitEvent {}

/// Event broadcast when a destructible target runs out of health.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetDestroyedEvent {
    pub target: ae_ecs::Entity,
}
impl Event for TargetDestroyedEvent {}

/// Command event emitted by gameplay behaviors or plugins to persist runtime delta state to disk.
///
/// Consumed on the host runtime tick to serialize a [`ae_world::WorldDeltaSnapshot`] into
/// `Saved/SaveGames/slot_<slot_index>.aesave` atomically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveGameRequest {
    /// Save slot number (`0` autosave/quicksave, `1..N` manual slots).
    pub slot_index: u16,
    /// Human-readable save title displayed in slot menus.
    pub label: String,
}
impl Event for SaveGameRequest {}

/// Command event emitted by gameplay behaviors or plugins to restore runtime state from a slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadGameRequest {
    /// Save slot number to load from `Saved/SaveGames/slot_<slot_index>.aesave`.
    pub slot_index: u16,
}
impl Event for LoadGameRequest {}

/// Completion notification broadcast after servicing a [`SaveGameRequest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveGameCompleted {
    /// Target save slot index.
    pub slot_index: u16,
    /// `true` when atomic write completed cleanly.
    pub success: bool,
    /// Diagnostic error message when `success` is `false`.
    pub error: Option<String>,
}
impl Event for SaveGameCompleted {}

/// Completion notification broadcast after servicing a [`LoadGameRequest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadGameCompleted {
    /// Target save slot index.
    pub slot_index: u16,
    /// `true` when savegame validation and restoration succeeded.
    pub success: bool,
    /// Diagnostic error message when `success` is `false`.
    pub error: Option<String>,
}
impl Event for LoadGameCompleted {}

impl DynamicEventBus {
    /// Enqueues a [`SaveGameRequest`] asking the runtime to persist slot `slot_id`.
    pub fn save_slot(&mut self, slot_id: u16, label: impl Into<String>) {
        self.send(SaveGameRequest {
            slot_index: slot_id,
            label: label.into(),
        });
    }

    /// Enqueues a [`LoadGameRequest`] asking the runtime to restore slot `slot_id`.
    pub fn load_slot(&mut self, slot_id: u16) {
        self.send(LoadGameRequest {
            slot_index: slot_id,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_default_modules_contain_render2d() {
        let bus = DynamicEventBus::new();
        assert!(bus.is_module_enabled(EngineModule::Physics));
        assert!(bus.is_module_enabled(EngineModule::Audio));
        assert!(bus.is_module_enabled(EngineModule::Render));
        assert!(bus.is_module_enabled(EngineModule::Render2D));
    }

    #[test]
    fn test_event_bus_toggle_render2d_module() {
        let mut bus = DynamicEventBus::new();
        assert!(bus.is_module_enabled(EngineModule::Render2D));

        bus.set_module_enabled(EngineModule::Render2D, false);
        assert!(!bus.is_module_enabled(EngineModule::Render2D));

        bus.set_module_enabled(EngineModule::Render2D, true);
        assert!(bus.is_module_enabled(EngineModule::Render2D));
    }

    #[test]
    fn test_event_bus_save_and_load_slot_commands() {
        let mut bus = DynamicEventBus::new();
        bus.save_slot(2, "Foundry Gate");
        bus.load_slot(2);

        let saves = bus.receive::<SaveGameRequest>().expect("save queue");
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].slot_index, 2);
        assert_eq!(saves[0].label, "Foundry Gate");

        let loads = bus.receive::<LoadGameRequest>().expect("load queue");
        assert_eq!(loads.len(), 1);
        assert_eq!(loads[0].slot_index, 2);
    }
}