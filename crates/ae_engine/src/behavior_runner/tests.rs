// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for gameplay behaviors and subsystem pipelines.
//!

use super::*;
use ae_core::ecs::{BehaviorComponent, BehaviorType, Color, Position, Rotation};
use ae_core::events::{DynamicEventBus, RaycastHitEvent, TargetDestroyedEvent};
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
        BehaviorComponent::rotator(2.0, [0.0, 1.0, 0.0]),
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
        BehaviorComponent::moving_platform(5.0, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
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
        BehaviorComponent {
            behavior_type: BehaviorType::TriggerZone,
            speed: 4.0,
            axis: [0.0, 1.0, 0.0],
            health: 100.0,
            max_health: 100.0,
            is_triggered: false,
            original_position: [0.0, 0.0, 0.0],
            target_position: [0.0, 4.0, 0.0],
            ping_pong_forward: true,
            timer: 0.0,
            hit_flash_timer: 0.0,
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

    let behavior = world.get::<&BehaviorComponent>(trigger_ent).unwrap();
    assert!(
        behavior.is_triggered,
        "Trigger zone should be marked triggered"
    );

    let pos = world.get::<&Position>(trigger_ent).unwrap();
    assert!(
        pos.y > 0.5,
        "Trigger zone should begin moving up towards target Y"
    );
}

#[test]
fn test_destructible_target_damage_and_destruction_event() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let target_ent = world.spawn((
        Position::new(0.0, 0.0, 5.0),
        Color::red(),
        BehaviorComponent::destructible_target(50.0),
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
        let behavior = world.get::<&BehaviorComponent>(target_ent).unwrap();
        assert_eq!(
            behavior.health, 20.0,
            "Health should drop to 20.0 after 30 damage"
        );
        assert!(
            behavior.hit_flash_timer > 0.0,
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

    let behavior = world.get::<&BehaviorComponent>(target_ent).unwrap();
    assert_eq!(behavior.health, 0.0, "Health should clamp to 0.0");

    let destruction_events = event_bus.receive::<TargetDestroyedEvent>();
    assert!(
        destruction_events.is_some(),
        "TargetDestroyedEvent should be broadcast"
    );
    let events = destruction_events.unwrap();
    assert_eq!(events[0].target, target_ent);
}