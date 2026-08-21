// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dynamic Gameplay Behavior Execution Pipeline.
//!
//! Provides a decoupled `BehaviorHandler` and `BehaviorRunnerRegistry` architecture
//! for extensible, data-driven entity execution.
//!

pub mod combat;
pub mod destructible;
pub mod moving_platform;
pub mod rotator;
pub mod trigger_zone;

#[cfg(test)]
mod tests;

use ae_core::ecs::{BehaviorComponent, BehaviorType, Position};
use ae_core::events::DynamicEventBus;
use ae_editor::input::InputManager;
use ae_physics::world::PhysicsWorld;
use hecs::World;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Parameters bundle for executing the behavior runner pipeline.
pub struct BehaviorRunnerParams<'a> {
    pub world: &'a mut World,
    pub physics_world: &'a mut PhysicsWorld,
    pub input: &'a InputManager,
    pub event_bus: &'a mut DynamicEventBus,
    pub camera_forward: cgmath::Vector3<f32>,
    pub delta_time: f32,
}

/// Execution context passed to each behavior handler during update.
pub struct BehaviorUpdateContext<'a> {
    pub world: &'a mut World,
    pub physics_world: &'a mut PhysicsWorld,
    pub input: &'a InputManager,
    pub event_bus: &'a mut DynamicEventBus,
    pub camera_forward: cgmath::Vector3<f32>,
    pub delta_time: f32,
    pub player_entity_and_pos: Option<(hecs::Entity, [f32; 3])>,
    pub dirty_entities: &'a mut Vec<hecs::Entity>,
}

/// Interface for custom gameplay behavior update logic.
pub trait BehaviorHandler: Send + Sync {
    /// Associated behavior type matching the ECS `BehaviorComponent`.
    fn behavior_type(&self) -> BehaviorType;

    /// Executes the gameplay tick for all entities holding this behavior.
    fn update(&self, ctx: &mut BehaviorUpdateContext, entities: &[hecs::Entity]);
}

/// Continuous Rotator behavior handler.
pub struct RotatorBehaviorHandler;
impl BehaviorHandler for RotatorBehaviorHandler {
    fn behavior_type(&self) -> BehaviorType {
        BehaviorType::Rotator
    }
    fn update(&self, ctx: &mut BehaviorUpdateContext, entities: &[hecs::Entity]) {
        rotator::update_rotators(
            ctx.world,
            ctx.physics_world,
            entities,
            ctx.delta_time,
            ctx.dirty_entities,
        );
    }
}

/// Moving Platform waypoint interpolation behavior handler.
pub struct MovingPlatformBehaviorHandler;
impl BehaviorHandler for MovingPlatformBehaviorHandler {
    fn behavior_type(&self) -> BehaviorType {
        BehaviorType::MovingPlatform
    }
    fn update(&self, ctx: &mut BehaviorUpdateContext, entities: &[hecs::Entity]) {
        moving_platform::update_moving_platforms(
            ctx.world,
            entities,
            ctx.player_entity_and_pos,
            ctx.delta_time,
            ctx.dirty_entities,
        );
    }
}

/// Proximity Trigger Zone and elevator/door mechanism behavior handler.
pub struct TriggerZoneBehaviorHandler;
impl BehaviorHandler for TriggerZoneBehaviorHandler {
    fn behavior_type(&self) -> BehaviorType {
        BehaviorType::TriggerZone
    }
    fn update(&self, ctx: &mut BehaviorUpdateContext, entities: &[hecs::Entity]) {
        trigger_zone::update_trigger_zone_mechanisms(
            ctx.world,
            entities,
            ctx.delta_time,
            ctx.dirty_entities,
        );
    }
}

/// Destructible target damage and hit flash behavior handler.
pub struct DestructibleTargetBehaviorHandler;
impl BehaviorHandler for DestructibleTargetBehaviorHandler {
    fn behavior_type(&self) -> BehaviorType {
        BehaviorType::DestructibleTarget
    }
    fn update(&self, ctx: &mut BehaviorUpdateContext, entities: &[hecs::Entity]) {
        destructible::update_destructible_visuals(ctx.world, entities, ctx.delta_time);
    }
}

/// Character Action weapon shooting, raycasts and impulses behavior handler.
pub struct CharacterActionBehaviorHandler;
impl BehaviorHandler for CharacterActionBehaviorHandler {
    fn behavior_type(&self) -> BehaviorType {
        BehaviorType::CharacterAction
    }
    fn update(&self, ctx: &mut BehaviorUpdateContext, entities: &[hecs::Entity]) {
        combat::update_character_actions(
            ctx.world,
            ctx.physics_world,
            ctx.input,
            ctx.event_bus,
            entities,
            ctx.camera_forward,
        );
    }
}

/// Central registry managing all gameplay behavior handlers.
#[derive(Default)]
pub struct BehaviorRunnerRegistry {
    handlers: Vec<Box<dyn BehaviorHandler>>,
}

impl BehaviorRunnerRegistry {
    /// Creates a new empty behavior runner registry.
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Registers a new behavior handler.
    pub fn register<H: BehaviorHandler + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }

    /// Returns all registered behavior handlers.
    pub fn handlers(&self) -> &[Box<dyn BehaviorHandler>] {
        &self.handlers
    }

    /// Builds the default engine registry with all built-in behavior handlers.
    pub fn default_registry() -> Self {
        let mut registry = Self::new();
        registry.register(RotatorBehaviorHandler);
        registry.register(MovingPlatformBehaviorHandler);
        registry.register(TriggerZoneBehaviorHandler);
        registry.register(DestructibleTargetBehaviorHandler);
        registry.register(CharacterActionBehaviorHandler);
        registry
    }

    /// Returns a reference to the global engine behavior runner registry singleton.
    pub fn global() -> &'static BehaviorRunnerRegistry {
        static REGISTRY: OnceLock<BehaviorRunnerRegistry> = OnceLock::new();
        REGISTRY.get_or_init(Self::default_registry)
    }
}

/// Executes all active entity behaviors during Play mode.
pub fn update_gameplay_behaviors(params: BehaviorRunnerParams<'_>) {
    let world = params.world;
    let physics_world = params.physics_world;
    let input = params.input;
    let event_bus = params.event_bus;
    let camera_forward = params.camera_forward;
    let dt = params.delta_time;

    // 1. Identify player entity and position for trigger and elevator interactions
    let mut player_entity_and_pos = None;
    for (ent, _) in world
        .query::<(hecs::Entity, &ae_core::ecs::PlayerTag)>()
        .iter()
    {
        if let Ok(pos) = world.get::<&Position>(ent) {
            player_entity_and_pos = Some((ent, [pos.x, pos.y, pos.z]));
            break;
        }
    }
    if player_entity_and_pos.is_none() {
        for (ent, _) in world
            .query::<(hecs::Entity, &ae_core::ecs::CharacterController)>()
            .iter()
        {
            if let Ok(pos) = world.get::<&Position>(ent) {
                player_entity_and_pos = Some((ent, [pos.x, pos.y, pos.z]));
                break;
            }
        }
    }

    // 2. Process physics trigger events and spatial sensor volume overlaps
    trigger_zone::process_trigger_events(world, event_bus);
    trigger_zone::test_spatial_sensor_overlaps(world, player_entity_and_pos);

    // 3. Process raycast damage on destructible targets
    destructible::process_destructible_hits(world, event_bus);

    // 4. Collect active entities dynamically by BehaviorType
    let mut grouped_entities: HashMap<BehaviorType, Vec<hecs::Entity>> = HashMap::new();
    for (entity, behavior) in world.query::<(hecs::Entity, &BehaviorComponent)>().iter() {
        grouped_entities
            .entry(behavior.behavior_type)
            .or_default()
            .push(entity);
    }

    let mut dirty_entities = Vec::new();

    // 5. Execute registered behavior handlers dynamically
    let registry = BehaviorRunnerRegistry::global();
    {
        let mut ctx = BehaviorUpdateContext {
            world,
            physics_world,
            input,
            event_bus,
            camera_forward,
            delta_time: dt,
            player_entity_and_pos,
            dirty_entities: &mut dirty_entities,
        };

        for handler in registry.handlers() {
            if let Some(entities) = grouped_entities.get(&handler.behavior_type())
                && !entities.is_empty()
            {
                handler.update(&mut ctx, entities);
            }
        }

        // 6. Update Ephemeral projectile despawning
        destructible::update_ephemeral_projectiles(ctx.world, dt);
    }

    // 7. Mark dirty entities for transform hierarchy sync
    for ent in dirty_entities {
        let _ = world.insert_one(ent, ae_core::ecs::TransformDirty);
    }
}