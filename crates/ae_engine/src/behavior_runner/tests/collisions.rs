// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for physics collision, trigger lifecycle hooks, and surface impact bridge.
//!

use std::sync::{Arc, Mutex};

use ae_core::behavior::{Behavior, BehaviorContext, NativeBehavior};
use ae_core::commands::EntityCommandBuffer;
use ae_core::ecs::{PhysicsMaterial, SurfaceType};
use ae_core::events::{
    CollisionEnter, CollisionExit, DynamicEventBus, SurfaceImpactEvent, TriggerEnter, TriggerExit,
};
use hecs::World;

struct TestCollisionActor {
    collided_with: Arc<Mutex<Vec<hecs::Entity>>>,
    exited_with: Arc<Mutex<Vec<hecs::Entity>>>,
    triggered_with: Arc<Mutex<Vec<hecs::Entity>>>,
    trigger_exited_with: Arc<Mutex<Vec<hecs::Entity>>>,
}

impl Behavior for TestCollisionActor {
    fn on_collision_enter(
        &mut self,
        _entity: hecs::Entity,
        other: hecs::Entity,
        _ctx: &mut BehaviorContext<'_>,
    ) {
        if let Ok(mut lock) = self.collided_with.lock() {
            lock.push(other);
        }
    }

    fn on_collision_exit(
        &mut self,
        _entity: hecs::Entity,
        other: hecs::Entity,
        _ctx: &mut BehaviorContext<'_>,
    ) {
        if let Ok(mut lock) = self.exited_with.lock() {
            lock.push(other);
        }
    }

    fn on_trigger_enter(
        &mut self,
        _entity: hecs::Entity,
        other: hecs::Entity,
        _ctx: &mut BehaviorContext<'_>,
    ) {
        if let Ok(mut lock) = self.triggered_with.lock() {
            lock.push(other);
        }
    }

    fn on_trigger_exit(
        &mut self,
        _entity: hecs::Entity,
        other: hecs::Entity,
        _ctx: &mut BehaviorContext<'_>,
    ) {
        if let Ok(mut lock) = self.trigger_exited_with.lock() {
            lock.push(other);
        }
    }
}

#[test]
fn test_behavior_collision_and_trigger_lifecycle_hooks() {
    let mut world = World::new();
    let mut event_bus = DynamicEventBus::new();
    let mut cmd = EntityCommandBuffer::new();

    let collided = Arc::new(Mutex::new(Vec::new()));
    let exited = Arc::new(Mutex::new(Vec::new()));
    let triggered = Arc::new(Mutex::new(Vec::new()));
    let trig_exited = Arc::new(Mutex::new(Vec::new()));

    let actor = TestCollisionActor {
        collided_with: Arc::clone(&collided),
        exited_with: Arc::clone(&exited),
        triggered_with: Arc::clone(&triggered),
        trigger_exited_with: Arc::clone(&trig_exited),
    };

    let ent_a = world.spawn((NativeBehavior::new(actor),));
    let ent_b = world.spawn(());

    // 1. Send CollisionEnter & CollisionExit
    event_bus.send(CollisionEnter::with_details(
        ent_a,
        ent_b,
        Some([0.0, 1.0, 0.0]),
        Some([0.0, 1.0, 0.0]),
        10.0,
    ));
    event_bus.send(CollisionExit {
        entity_a: ent_a,
        entity_b: ent_b,
    });

    // 2. Send TriggerEnter & TriggerExit
    event_bus.send(TriggerEnter {
        entity_a: ent_a,
        entity_b: ent_b,
    });
    event_bus.send(TriggerExit {
        entity_a: ent_a,
        entity_b: ent_b,
    });

    // Dispatch events
    let cam_fwd = cgmath::Vector3::new(0.0, 0.0, -1.0);
    crate::behavior_runner::native_behavior::dispatch_collision_and_trigger_behaviors(
        &mut world,
        &mut event_bus,
        &mut cmd,
        cam_fwd,
        0.016,
    );

    // Verify hooks were invoked
    assert_eq!(*collided.lock().unwrap(), vec![ent_b]);
    assert_eq!(*exited.lock().unwrap(), vec![ent_b]);
    assert_eq!(*triggered.lock().unwrap(), vec![ent_b]);
    assert_eq!(*trig_exited.lock().unwrap(), vec![ent_b]);
}

#[test]
fn test_surface_impact_bridge_with_materials() {
    let mut world = World::new();
    let mut event_bus = DynamicEventBus::new();

    let ent_a = world.spawn((PhysicsMaterial::new(SurfaceType::Metal, 0.8, 0.1),));
    let ent_b = world.spawn((PhysicsMaterial::new(SurfaceType::Stone, 0.6, 0.0),));

    event_bus.send(CollisionEnter::with_details(
        ent_a,
        ent_b,
        Some([5.0, 0.0, -2.0]),
        Some([0.0, 1.0, 0.0]),
        25.0,
    ));

    crate::behavior_runner::collision_bridge::process_collision_surface_impacts(
        &world,
        &mut event_bus,
    );

    assert!(event_bus.has_events::<SurfaceImpactEvent>());
    let impacts = event_bus.receive::<SurfaceImpactEvent>().unwrap();
    assert_eq!(impacts.len(), 2);

    let impact_a = impacts.iter().find(|i| i.entity == ent_a).unwrap();
    assert_eq!(impact_a.surface_type, SurfaceType::Metal);
    assert_eq!(impact_a.hit_point, [5.0, 0.0, -2.0]);
    assert_eq!(impact_a.energy, 25.0);

    let impact_b = impacts.iter().find(|i| i.entity == ent_b).unwrap();
    assert_eq!(impact_b.surface_type, SurfaceType::Stone);
    assert_eq!(impact_b.energy, 25.0);
}