// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Gameplay behavior execution pipeline for interactive gameplay systems.
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

/// Parameters bundle for executing the behavior runner pipeline.
pub struct BehaviorRunnerParams<'a> {
    pub world: &'a mut World,
    pub physics_world: &'a mut PhysicsWorld,
    pub input: &'a InputManager,
    pub event_bus: &'a mut DynamicEventBus,
    pub camera_forward: cgmath::Vector3<f32>,
    pub delta_time: f32,
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

    // 4. Collect active entities by behavior type
    let mut rotators = Vec::new();
    let mut moving_platforms = Vec::new();
    let mut trigger_zones = Vec::new();
    let mut destructible_targets = Vec::new();
    let mut character_actions = Vec::new();

    for (entity, behavior) in world.query::<(hecs::Entity, &BehaviorComponent)>().iter() {
        match behavior.behavior_type {
            BehaviorType::Rotator => rotators.push(entity),
            BehaviorType::MovingPlatform => moving_platforms.push(entity),
            BehaviorType::TriggerZone => trigger_zones.push(entity),
            BehaviorType::DestructibleTarget => destructible_targets.push(entity),
            BehaviorType::CharacterAction => character_actions.push(entity),
            BehaviorType::Custom => {}
        }
    }

    let mut dirty_entities = Vec::new();

    // 5. Update Rotators and Player continuous angular rotations
    rotator::update_rotators(world, physics_world, &rotators, dt, &mut dirty_entities);
    rotator::update_player_rotations(world, physics_world, dt, &mut dirty_entities);

    // 6. Update Moving Platforms and passenger translation
    moving_platform::update_moving_platforms(
        world,
        &moving_platforms,
        player_entity_and_pos,
        dt,
        &mut dirty_entities,
    );

    // 7. Update Trigger Zones and linked elevator/door mechanisms
    trigger_zone::update_trigger_zone_mechanisms(world, &trigger_zones, dt, &mut dirty_entities);

    // Mark dirty entities for transform hierarchy sync
    for ent in dirty_entities {
        let _ = world.insert_one(ent, ae_core::ecs::TransformDirty);
    }

    // 8. Update Destructible Targets visual hit flashes
    destructible::update_destructible_visuals(world, &destructible_targets, dt);

    // 9. Update Ephemeral Projectiles lifetime and auto-despawn
    destructible::update_ephemeral_projectiles(world, dt);

    // 10. Update Character Actions (Weapon shooting, raycasts, impulses & projectiles)
    combat::update_character_actions(
        world,
        physics_world,
        input,
        event_bus,
        &character_actions,
        camera_forward,
    );
}