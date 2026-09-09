// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for character combat actions, weapon cooldowns, projectile speed, and destructibles.
//!

use crate::behavior_runner::{BehaviorRunnerParams, update_gameplay_behaviors};
use ae_core::ecs::{
    Collider, ColliderShape, Color, DestructibleTarget, Position, RigidBody, RigidBodyType,
    Rotation, Velocity,
};
use ae_core::events::{DynamicEventBus, RaycastHitEvent, TargetDestroyedEvent};
use ae_editor::input::InputManager;
use ae_physics::world::PhysicsWorld;
use hecs::World;

#[test]
fn test_destructible_target_damage_and_destruction_event() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let target_ent = world.spawn((
        Position::new(0.0, 0.0, 5.0),
        Color::red(),
        DestructibleTarget::new(50.0),
    ));

    // Send 30 damage hit
    event_bus.send(RaycastHitEvent {
        shooter: None,
        target: target_ent,
        hit_point: [0.0, 0.0, 4.5],
        hit_normal: [0.0, 0.0, -1.0],
        damage: 30.0,
    });

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.016,
    });

    {
        let target = world.get::<&DestructibleTarget>(target_ent).unwrap();
        assert_eq!(
            target.health, 20.0,
            "Health should drop to 20.0 after 30 damage"
        );
        assert!(
            target.hit_flash_timer > 0.0,
            "Hit flash timer should be active"
        );
    }

    // Send fatal 25 damage hit
    event_bus.send(RaycastHitEvent {
        shooter: None,
        target: target_ent,
        hit_point: [0.0, 0.0, 4.5],
        hit_normal: [0.0, 0.0, -1.0],
        damage: 25.0,
    });

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.016,
    });

    let target = world.get::<&DestructibleTarget>(target_ent).unwrap();
    assert_eq!(target.health, 0.0, "Health should clamp to 0.0");

    let destruction_events = event_bus.receive::<TargetDestroyedEvent>();
    assert!(
        destruction_events.is_some(),
        "TargetDestroyedEvent should be broadcast"
    );
    let events = destruction_events.unwrap();
    assert_eq!(events[0].target, target_ent);
}

#[test]
fn test_destructible_target_hit_flash_restores_original_color() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let target_ent = world.spawn((
        Position::new(0.0, 0.0, 5.0),
        Color::red(), // Original color is Red (1.0, 0.2, 0.2, 1.0)
        DestructibleTarget::new(100.0),
    ));

    // Send 10 damage hit
    event_bus.send(RaycastHitEvent {
        shooter: None,
        target: target_ent,
        hit_point: [0.0, 0.0, 4.5],
        hit_normal: [0.0, 0.0, -1.0],
        damage: 10.0,
    });

    // Step 1: Hit applied -> flash active
    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.016,
    });

    {
        let col = world.get::<&Color>(target_ent).unwrap();
        assert_eq!(col.r, 1.0);
        assert_eq!(col.g, 0.9);
    }

    // Step 2: Simulate 300ms passing -> flash timer expires
    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.30,
    });

    {
        let col = world.get::<&Color>(target_ent).unwrap();
        assert_eq!(col.r, 1.0, "Red channel must be 1.0");
        assert_eq!(col.g, 0.2, "Green channel must be 0.2 (Color::red)");
        assert_eq!(col.b, 0.2, "Blue channel must be 0.2 (Color::red)");
    }
}

#[test]
fn test_entities_without_character_action_do_not_shoot() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let mut input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    // Spawn player entity with ONLY PlayerTag (NO CharacterAction)
    let _player_ent = world.spawn((
        Position::new(0.0, 1.0, 0.0),
        Rotation::identity(),
        ae_core::ecs::PlayerTag,
    ));

    // Simulate Fire button press
    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Pressed,
    );

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.016,
    });

    // Verify NO EphemeralProjectile was spawned
    let projectile_count = world
        .query::<&ae_core::ecs::EphemeralProjectile>()
        .into_iter()
        .count();
    assert_eq!(
        projectile_count, 0,
        "Entities without CharacterAction must NOT shoot projectiles"
    );

    // Now attach CharacterAction and verify it CAN shoot
    let shooter_ent = world.spawn((
        Position::new(0.0, 1.0, 0.0),
        Rotation::identity(),
        ae_core::ecs::CharacterAction::default(),
    ));

    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Pressed,
    );

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.016,
    });

    let projectile_count_after = world
        .query::<&ae_core::ecs::EphemeralProjectile>()
        .into_iter()
        .count();
    assert_eq!(
        projectile_count_after, 1,
        "Entity with CharacterAction MUST spawn projectile on fire"
    );
    let _ = shooter_ent;
}

#[test]
fn test_character_action_cooldown_and_speed() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let mut input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let _shooter = world.spawn((
        Position::new(0.0, 1.0, 0.0),
        Rotation::identity(),
        ae_core::ecs::CharacterAction {
            speed: 80.0,
            cooldown: 1.0,
            timer: 0.0,
            axis: [0.0, 0.0, -1.0],
        },
    ));

    // First shot (timer = 0.0, ready to fire)
    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Pressed,
    );

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.1,
    });

    let mut bolt_velocity = None;
    for (vel, _) in world
        .query::<(&Velocity, &ae_core::ecs::EphemeralProjectile)>()
        .iter()
    {
        bolt_velocity = Some(*vel);
    }
    assert!(
        bolt_velocity.is_some(),
        "Projectile must be spawned on first shot"
    );
    let vel = bolt_velocity.unwrap();
    assert!(
        (vel.z - 80.0).abs() < 0.1,
        "Projectile speed must match configured speed 80.0 m/s"
    );

    // Second shot immediately (cooldown active: 1.0 - 0.1 = 0.9s remaining)
    input.end_frame();
    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Pressed,
    );

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.1,
    });

    let projectile_count = world
        .query::<&ae_core::ecs::EphemeralProjectile>()
        .into_iter()
        .count();
    assert_eq!(
        projectile_count, 1,
        "Second shot must be blocked by active cooldown"
    );

    // Advance time by 1.0s and release key so next press is registered
    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Released,
    );
    input.end_frame();
    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 1.0,
    });

    // Third shot after cooldown has elapsed
    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Pressed,
    );

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.1,
    });

    let projectile_count_after = world
        .query::<&ae_core::ecs::EphemeralProjectile>()
        .into_iter()
        .count();
    assert_eq!(
        projectile_count_after, 2,
        "Third shot must succeed after cooldown expiration"
    );
}

#[test]
fn test_character_action_shoots_and_pushes_dynamic_cube() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let mut input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    // 1. Shooter character at (0, 0, 0)
    let shooter = world.spawn((
        Position::new(0.0, 0.0, 0.0),
        Rotation::identity(),
        ae_core::ecs::CharacterAction {
            speed: 50.0,
            cooldown: 0.5,
            timer: 0.0,
            axis: [0.0, 0.0, 1.0],
        },
    ));

    // 2. Dynamic cube target at (0, 1.5, 6.0)
    let cube = world.spawn((
        Position::new(0.0, 1.5, 6.0),
        Rotation::identity(),
        Velocity {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 2.0,
            gravity_scale: 0.0,
        },
        Collider {
            shape: ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.5,
            restitution: 0.0,
            is_sensor: false,
        },
    ));

    // Initial step to register cube in physics world
    physics.step(&mut world, |_| None, 0.016, &mut event_bus);

    // Fire weapon along +Z towards the cube
    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Pressed,
    );

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.016,
    });

    // Verify RaycastHitEvent was sent targeting the cube
    let hit_events = event_bus.receive::<RaycastHitEvent>().unwrap();
    assert_eq!(hit_events.len(), 1, "Must generate exactly one hit event");
    assert_eq!(hit_events[0].target, cube);
    assert_eq!(hit_events[0].shooter, Some(shooter));

    // Verify spawned projectile has impact lifetime clamped to hit distance / speed
    let mut projectile_lifetime = None;
    for projectile in world.query::<&ae_core::ecs::EphemeralProjectile>().iter() {
        projectile_lifetime = Some(projectile.lifetime_remaining);
    }
    assert!(
        projectile_lifetime.is_some(),
        "Visual laser projectile must be spawned"
    );
    let lt = projectile_lifetime.unwrap();
    // Distance from gun muzzle (0.8m) to cube face (5.5m) is ~4.7m.
    // At speed 50 m/s, time to impact should be ~0.09s, much less than 1.5s!
    assert!(
        lt < 0.25,
        "Projectile lifetime must be clamped to time-to-impact (expected < 0.25s, got {})",
        lt
    );

    // Step physics: dynamic cube must retain impulse and accelerate along +Z
    physics.step(&mut world, |_| None, 0.1, &mut event_bus);

    let cube_vel = world.get::<&Velocity>(cube).unwrap();
    assert!(
        cube_vel.z > 2.0,
        "Dynamic cube velocity along +Z must accelerate from weapon impulse, got {}",
        cube_vel.z
    );

    let cube_pos = world.get::<&Position>(cube).unwrap();
    assert!(
        cube_pos.z > 6.1,
        "Dynamic cube position must advance along +Z after being shot, got {}",
        cube_pos.z
    );
}