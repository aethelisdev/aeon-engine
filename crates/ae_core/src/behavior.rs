// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Pure Safe Rust Gameplay Lifecycle & Behavior Infrastructure.
//!
//! Defines standard lifecycle traits and execution contexts for gameplay actors,
//! interactive entities, and procedural game mechanics.
//!

use crate::commands::EntityCommandBuffer;
use ae_plugin_api::DynamicEventBus;

/// Execution context provided to `Behavior` lifecycle callbacks during active simulation.
/// Encapsulates references to ECS storage, event dispatcher, deferred entity mutation
/// queue, frame delta time, and spatial camera vectors.
pub struct BehaviorContext<'a> {
    /// Mutable reference to the ECS world for querying or reading entity state.
    pub world: &'a mut hecs::World,
    /// Shared reference to dynamic event bus for dispatching and reading gameplay events.
    pub event_bus: &'a mut DynamicEventBus,
    /// Deferred entity command buffer for queueing safe spawning, despawning, and component mutations.
    pub commands: &'a mut EntityCommandBuffer,
    /// Camera forward direction vector in world space for aim and directional queries.
    pub camera_forward: [f32; 3],
    /// Elapsed frame delta time in seconds.
    pub delta_time: f32,
}

/// Standard trait interface for pure Rust entity behaviors and gameplay actors.
/// Implementors define logic hooks that execute automatically during Play mode simulation.
pub trait Behavior: Send + Sync + 'static {
    /// Invoked once when Play mode initializes or when the component is spawned during runtime.
    fn on_start(&mut self, _entity: hecs::Entity, _ctx: &mut BehaviorContext<'_>) {}

    /// Invoked every frame during standard gameplay update.
    fn on_update(&mut self, _entity: hecs::Entity, _ctx: &mut BehaviorContext<'_>, _dt: f32) {}

    /// Invoked at fixed time intervals during physics integration ticks.
    fn on_fixed_update(
        &mut self,
        _entity: hecs::Entity,
        _ctx: &mut BehaviorContext<'_>,
        _fixed_dt: f32,
    ) {
    }

    /// Invoked when the entity is despawned or when Play mode transitions back to Edit mode.
    fn on_destroy(&mut self, _entity: hecs::Entity, _ctx: &mut BehaviorContext<'_>) {}

    /// Invoked when this entity begins physical contact with another solid physics body.
    fn on_collision_enter(
        &mut self,
        _entity: hecs::Entity,
        _other: hecs::Entity,
        _ctx: &mut BehaviorContext<'_>,
    ) {
    }

    /// Invoked when this entity ends physical contact with another solid physics body.
    fn on_collision_exit(
        &mut self,
        _entity: hecs::Entity,
        _other: hecs::Entity,
        _ctx: &mut BehaviorContext<'_>,
    ) {
    }

    /// Invoked when this entity enters a volumetric sensor / trigger zone.
    fn on_trigger_enter(
        &mut self,
        _entity: hecs::Entity,
        _other: hecs::Entity,
        _ctx: &mut BehaviorContext<'_>,
    ) {
    }

    /// Invoked when this entity exits a volumetric sensor / trigger zone.
    fn on_trigger_exit(
        &mut self,
        _entity: hecs::Entity,
        _other: hecs::Entity,
        _ctx: &mut BehaviorContext<'_>,
    ) {
    }
}

/// ECS wrapper component for attaching dynamic pure Rust `Behavior` instances to entities.
/// Enables flexible, polymorphism-friendly entity scripting while preserving strict
/// type-safety and 100% safe memory guarantees.
pub struct NativeBehavior {
    /// The boxed trait object managing entity lifecycle hooks.
    pub inner: Box<dyn Behavior>,
    /// Flag tracking whether `on_start` has been invoked for this behavior instance.
    pub started: bool,
}

impl NativeBehavior {
    /// Creates a new `NativeBehavior` component wrapping the given behavior instance.
    pub fn new<B: Behavior>(behavior: B) -> Self {
        Self {
            inner: Box::new(behavior),
            started: false,
        }
    }
}