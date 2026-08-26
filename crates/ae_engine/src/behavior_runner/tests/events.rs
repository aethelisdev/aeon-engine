// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for standard pub/sub gameplay event channels (Damage, Kill, Trigger, Score, Custom).
//!

use crate::behavior_runner::{destructible, trigger_zone};
use ae_core::events::{DynamicEventBus, RaycastHitEvent};
use hecs::World;

#[test]
fn test_gameplay_events_damage_and_actor_killed_dispatch() {
    let mut world = World::new();
    let mut event_bus = DynamicEventBus::new();

    let target_ent = world.spawn((
        ae_core::ecs::Position::new(0.0, 1.0, 5.0),
        ae_core::ecs::DestructibleTarget::new(50.0),
        ae_core::ecs::Color::red(),
    ));

    let shooter_ent = world.spawn((ae_core::ecs::Position::new(0.0, 1.0, 0.0),));

    // Send RaycastHitEvent with 25 damage
    event_bus.send(RaycastHitEvent {
        shooter: Some(shooter_ent),
        target: target_ent,
        hit_point: [0.0, 1.0, 5.0],
        hit_normal: [0.0, 0.0, -1.0],
        damage: 25.0,
    });

    destructible::process_destructible_hits(&mut world, &mut event_bus);

    // Verify DamageEvent was broadcast on event bus
    let damage_events = event_bus.receive::<ae_core::events::DamageEvent>().unwrap();
    assert_eq!(damage_events.len(), 1);
    assert_eq!(damage_events[0].target, target_ent);
    assert_eq!(damage_events[0].amount, 25.0);
    assert_eq!(damage_events[0].source, Some(shooter_ent));

    // Send fatal damage (25 remaining)
    event_bus.send(RaycastHitEvent {
        shooter: Some(shooter_ent),
        target: target_ent,
        hit_point: [0.0, 1.0, 5.0],
        hit_normal: [0.0, 0.0, -1.0],
        damage: 25.0,
    });

    destructible::process_destructible_hits(&mut world, &mut event_bus);

    // Verify ActorKilledEvent was broadcast
    let killed_events = event_bus
        .receive::<ae_core::events::ActorKilledEvent>()
        .unwrap();
    assert_eq!(killed_events.len(), 1);
    assert_eq!(killed_events[0].victim, target_ent);
    assert_eq!(killed_events[0].killer, Some(shooter_ent));
}

#[test]
fn test_gameplay_events_trigger_channel_dispatch() {
    let mut world = World::new();
    let mut event_bus = DynamicEventBus::new();

    let trigger_ent = world.spawn((
        ae_core::ecs::Position::new(0.0, 0.0, 0.0),
        ae_core::ecs::TriggerZone::new(),
    ));
    let player_ent = world.spawn((ae_core::ecs::Position::new(0.0, 0.0, 0.0),));

    // Simulate TriggerEnter
    event_bus.send(ae_core::events::TriggerEnter {
        entity_a: trigger_ent,
        entity_b: player_ent,
    });

    trigger_zone::process_trigger_events(&mut world, &mut event_bus);

    let trigger_events = event_bus
        .receive::<ae_core::events::TriggerEvent>()
        .unwrap();
    assert_eq!(trigger_events.len(), 1);
    assert_eq!(trigger_events[0].trigger, trigger_ent);
    assert_eq!(trigger_events[0].activator, player_ent);
    assert!(trigger_events[0].is_enter);
}

#[test]
fn test_custom_gameplay_events_and_score_events() {
    let mut event_bus = DynamicEventBus::new();

    // Send ScoreEvent
    event_bus.send(ae_core::events::ScoreEvent {
        delta: 100,
        new_total: 500,
    });

    let score_events = event_bus.receive::<ae_core::events::ScoreEvent>().unwrap();
    assert_eq!(score_events.len(), 1);
    assert_eq!(score_events[0].delta, 100);
    assert_eq!(score_events[0].new_total, 500);

    // Send CustomGameplayEvent
    event_bus.send(ae_core::events::CustomGameplayEvent {
        name: "BossDefeated".to_string(),
        value: 1.0,
    });

    let custom_events = event_bus
        .receive::<ae_core::events::CustomGameplayEvent>()
        .unwrap();
    assert_eq!(custom_events.len(), 1);
    assert_eq!(custom_events[0].name, "BossDefeated");
    assert_eq!(custom_events[0].value, 1.0);
}