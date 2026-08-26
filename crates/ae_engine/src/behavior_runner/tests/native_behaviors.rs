// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for NativeBehavior lifecycle and EntityCommandBuffer operations.
//!

use crate::behavior_runner::{BehaviorRunnerParams, update_gameplay_behaviors};
use ae_core::ecs::{Position, Rotation};
use ae_core::events::DynamicEventBus;
use ae_editor::input::InputManager;
use ae_physics::world::PhysicsWorld;
use hecs::World;

#[derive(Default)]
struct MockActorBehavior {
    start_invoked: bool,
    update_count: u32,
}

impl ae_core::behavior::Behavior for MockActorBehavior {
    fn on_start(
        &mut self,
        _entity: hecs::Entity,
        ctx: &mut ae_core::behavior::BehaviorContext<'_>,
    ) {
        self.start_invoked = true;
        // Schedule spawning a companion entity via deferred command buffer
        ctx.commands.spawn_with(|w| {
            w.spawn((
                Position::new(10.0, 20.0, 30.0),
                ae_core::ecs::Name("CompanionEntity".to_string()),
            ))
        });
    }

    fn on_update(
        &mut self,
        entity: hecs::Entity,
        ctx: &mut ae_core::behavior::BehaviorContext<'_>,
        dt: f32,
    ) {
        self.update_count += 1;
        if let Ok(mut pos) = ctx.world.get::<&mut Position>(entity) {
            pos.y += 10.0 * dt;
        }
    }
}

#[test]
fn test_native_behavior_lifecycle_and_deferred_commands() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let actor_ent = world.spawn((
        Position::new(0.0, 0.0, 0.0),
        Rotation::identity(),
        ae_core::behavior::NativeBehavior::new(MockActorBehavior::default()),
    ));

    // Execute first frame
    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.1,
    });

    // 1. Verify position was updated by on_update
    let actor_pos = *world.get::<&Position>(actor_ent).unwrap();
    assert!(
        (actor_pos.y - 1.0).abs() < 0.01,
        "Actor Y should increase by 1.0 unit (10.0 * 0.1)"
    );

    // 2. Verify companion entity was spawned by on_start via CommandBuffer
    let mut companion_found = false;
    for (pos, name) in world.query::<(&Position, &ae_core::ecs::Name)>().iter() {
        if name.0 == "CompanionEntity" {
            assert_eq!(pos.x, 10.0);
            assert_eq!(pos.y, 20.0);
            assert_eq!(pos.z, 30.0);
            companion_found = true;
        }
    }

    assert!(
        companion_found,
        "Companion entity spawned via command buffer must exist in World"
    );

    // Execute second frame to verify on_start is NOT called again
    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.1,
    });

    let actor_pos_frame2 = *world.get::<&Position>(actor_ent).unwrap();
    assert!(
        (actor_pos_frame2.y - 2.0).abs() < 0.01,
        "Actor Y should now be 2.0"
    );
}

#[test]
fn test_entity_command_buffer_direct_operations() {
    let mut world = World::new();
    let mut cmd = ae_core::commands::EntityCommandBuffer::new();

    let ent1 = world.spawn((Position::new(0.0, 0.0, 0.0),));
    let ent2 = world.spawn((Position::new(5.0, 5.0, 5.0),));

    // Queue insert_one, remove_one, despawn, and spawn_with
    cmd.insert_one(ent1, ae_core::ecs::Color::red());
    cmd.remove_one::<Position>(ent2);
    cmd.despawn(ent1);
    cmd.spawn_with(|w| w.spawn((Position::new(100.0, 100.0, 100.0), ae_core::ecs::PlayerTag)));

    assert_eq!(cmd.len(), 4);
    assert!(!cmd.is_empty());

    // Apply commands to world
    cmd.apply(&mut world);
    assert!(cmd.is_empty());

    // ent1 must be despawned
    assert!(!world.contains(ent1));

    // ent2 must no longer have Position
    assert!(world.get::<&Position>(ent2).is_err());

    // New spawned entity with PlayerTag must exist at 100,100,100
    let mut player_found = false;
    for (pos, _) in world
        .query::<(&Position, &ae_core::ecs::PlayerTag)>()
        .iter()
    {
        if pos.x == 100.0 && pos.y == 100.0 && pos.z == 100.0 {
            player_found = true;
        }
    }
    assert!(player_found, "Deferred spawned player entity must exist");
}