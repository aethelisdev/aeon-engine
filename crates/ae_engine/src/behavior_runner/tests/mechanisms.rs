// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for rotating actors, moving platforms, and trigger mechanisms.
//!

use crate::behavior_runner::{BehaviorRunnerParams, update_gameplay_behaviors};
use ae_core::ecs::{Color, MovingPlatform, Position, Rotation, Rotator, Scale, TriggerZone};
use ae_core::events::DynamicEventBus;
use ae_editor::input::InputManager;
use ae_physics::world::PhysicsWorld;
use hecs::World;

#[test]
fn test_rotator_behavior_rotation_progression() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let ent = world.spawn((
        Position::new(0.0, 0.0, 0.0),
        Rotation::identity(),
        Rotator::new(2.0, [0.0, 1.0, 0.0]),
    ));

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.5,
    });

    let rot = *world.get::<&Rotation>(ent).unwrap();
    assert!(
        rot.y.abs() > 0.01 || rot.w.abs() < 0.99,
        "Rotator must advance rotation quaternion"
    );
}

#[test]
fn test_moving_platform_waypoint_interpolation() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let ent = world.spawn((
        Position::new(0.0, 0.0, 0.0),
        Rotation::identity(),
        MovingPlatform::new(5.0, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
    ));

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 1.0,
    });

    let pos = *world.get::<&Position>(ent).unwrap();
    assert!(
        (pos.x - 5.0).abs() < 0.1,
        "Platform should move 5.0 units in 1 second at 5.0 m/s"
    );
}

#[test]
fn test_trigger_zone_activation_and_elevation() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let trigger_ent = world.spawn((
        Position::new(0.0, 0.0, 0.0),
        Color::soft_blue(),
        TriggerZone {
            is_triggered: false,
            speed: 4.0,
            axis: [0.0, 1.0, 0.0],
            original_position: [0.0, 0.0, 0.0],
            target_position: [0.0, 4.0, 0.0],
            ping_pong_forward: true,
        },
    ));

    let other_ent = world.spawn((Position::new(0.0, 0.0, 0.0),));

    // Send TriggerEnter event
    event_bus.send(ae_core::events::TriggerEnter {
        entity_a: trigger_ent,
        entity_b: other_ent,
    });

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.5,
    });

    let zone = world.get::<&TriggerZone>(trigger_ent).unwrap();
    assert!(zone.is_triggered, "Trigger zone should be marked triggered");

    let pos = world.get::<&Position>(trigger_ent).unwrap();
    assert!(
        pos.y > 0.5,
        "Trigger zone should begin moving up towards target Y"
    );
}

#[test]
fn test_trigger_zone_step_on_and_step_off_elevation_cycle() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    // 1. Sensor Pad
    let sensor_pad = world.spawn((
        Position::new(10.0, 0.55, -4.0),
        Scale::new(4.0, 0.1, 4.0),
        ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 10.0, 0.5],
            },
            friction: 0.0,
            restitution: 0.0,
            is_sensor: true,
        },
        TriggerZone::new(),
    ));

    // 2. Sliding Door
    let door = world.spawn((
        Position::new(10.0, 2.0, -7.0),
        ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.5,
            restitution: 0.0,
            is_sensor: false,
        },
        TriggerZone {
            is_triggered: false,
            speed: 10.0,
            axis: [0.0, 1.0, 0.0],
            original_position: [10.0, 2.0, -7.0],
            target_position: [10.0, 6.0, -7.0],
            ping_pong_forward: true,
        },
    ));

    // 3. Player entity stepping on sensor pad (10.0, 0.55, -4.0)
    let player = world.spawn((Position::new(10.0, 1.0, -4.0), ae_core::ecs::PlayerTag));

    // Frame 1: Player is on the pad -> Door raises up
    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.5,
    });

    let is_triggered = world.get::<&TriggerZone>(sensor_pad).unwrap().is_triggered;
    assert!(is_triggered, "Pad must be triggered when player is on it");
    let door_pos = *world.get::<&Position>(door).unwrap();
    assert!(
        door_pos.y > 2.0,
        "Door must move up towards 6.0 when player is on pad"
    );

    // Frame 2: Player steps OFF the pad to (50.0, 1.0, 50.0) -> Door lowers back down
    if let Ok(mut pos) = world.get::<&mut Position>(player) {
        pos.x = 50.0;
        pos.z = 50.0;
    }

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 1.0,
    });

    let is_triggered = world.get::<&TriggerZone>(sensor_pad).unwrap().is_triggered;
    assert!(
        !is_triggered,
        "Pad must NOT be triggered when player steps off"
    );
    let door_pos = *world.get::<&Position>(door).unwrap();
    assert!(
        door_pos.y <= 2.05,
        "Door must lower back down to original position (2.0) after player steps off"
    );
}