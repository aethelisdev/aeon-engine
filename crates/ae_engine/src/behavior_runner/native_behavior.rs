// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native Pure Rust Behavior Execution Subsystem.
//!
//! Iterates through entities with attached `NativeBehavior` components, executing
//! their standard lifecycle hooks (`on_start`, `on_update`, `on_fixed_update`, `on_destroy`)
//! with zero `unsafe` blocks and complete borrow safety.
//!

use ae_core::behavior::{BehaviorContext, NativeBehavior};
use ae_core::commands::EntityCommandBuffer;
use ae_core::events::DynamicEventBus;
use hecs::World;

/// Executes the frame update lifecycle for all active `NativeBehavior` components.
/// Ensures borrow safety by temporarily extracting the component from the entity,
/// executing lifecycle methods against `BehaviorContext`, and restoring the component
/// if the entity remains valid.
pub fn update_native_behaviors(
    world: &mut World,
    event_bus: &mut DynamicEventBus,
    commands: &mut EntityCommandBuffer,
    camera_forward: cgmath::Vector3<f32>,
    dt: f32,
) {
    let mut behavior_entities = Vec::new();
    for (ent, _) in world.query::<(hecs::Entity, &NativeBehavior)>().iter() {
        behavior_entities.push(ent);
    }

    let mut behaviors_to_restore = Vec::new();

    for ent in behavior_entities {
        if let Ok(mut behavior) = world.remove_one::<NativeBehavior>(ent) {
            let mut ctx = BehaviorContext {
                world,
                event_bus,
                commands,
                camera_forward: [camera_forward.x, camera_forward.y, camera_forward.z],
                delta_time: dt,
            };

            if !behavior.started {
                behavior.inner.on_start(ent, &mut ctx);
                behavior.started = true;
            }

            behavior.inner.on_update(ent, &mut ctx, dt);
            behaviors_to_restore.push((ent, behavior));
        }
    }

    for (ent, behavior) in behaviors_to_restore {
        if world.contains(ent) {
            let _ = world.insert_one(ent, behavior);
        }
    }
}

/// Executes the fixed physics step lifecycle for all active `NativeBehavior` components.
pub fn fixed_update_native_behaviors(
    world: &mut World,
    event_bus: &mut DynamicEventBus,
    commands: &mut EntityCommandBuffer,
    camera_forward: cgmath::Vector3<f32>,
    fixed_dt: f32,
) {
    let mut behavior_entities = Vec::new();
    for (ent, _) in world.query::<(hecs::Entity, &NativeBehavior)>().iter() {
        behavior_entities.push(ent);
    }

    let mut behaviors_to_restore = Vec::new();

    for ent in behavior_entities {
        if let Ok(mut behavior) = world.remove_one::<NativeBehavior>(ent) {
            let mut ctx = BehaviorContext {
                world,
                event_bus,
                commands,
                camera_forward: [camera_forward.x, camera_forward.y, camera_forward.z],
                delta_time: fixed_dt,
            };

            behavior.inner.on_fixed_update(ent, &mut ctx, fixed_dt);
            behaviors_to_restore.push((ent, behavior));
        }
    }

    for (ent, behavior) in behaviors_to_restore {
        if world.contains(ent) {
            let _ = world.insert_one(ent, behavior);
        }
    }
}