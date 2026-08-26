// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Physics and collision event bridge subsystem.
//!
//! Inspects contact points, normal vectors, and physical material surfaces
//! (`PhysicsMaterial` / `SurfaceType`) to dispatch standardized `SurfaceImpactEvent`
//! and audio cues across the `DynamicEventBus`.
//!

use ae_core::ecs::{PhysicsMaterial, Position, SurfaceType};
use ae_core::events::{CollisionEnter, DynamicEventBus, SurfaceImpactEvent};
use hecs::World;

/// Processes physical collision enter events and generates surface impact events.
pub fn process_collision_surface_impacts(world: &World, event_bus: &mut DynamicEventBus) {
    if let Some(events) = event_bus.receive::<CollisionEnter>() {
        let mut re_emit_collisions = Vec::new();

        for col in events {
            // Determine surface types for both entities
            let surface_a = world
                .get::<&PhysicsMaterial>(col.entity_a)
                .map(|m| m.surface_type)
                .unwrap_or(SurfaceType::Default);
            let surface_b = world
                .get::<&PhysicsMaterial>(col.entity_b)
                .map(|m| m.surface_type)
                .unwrap_or(SurfaceType::Default);

            let hit_point = col.contact_point.unwrap_or_else(|| {
                world
                    .get::<&Position>(col.entity_a)
                    .map(|p| [p.x, p.y, p.z])
                    .unwrap_or([0.0, 0.0, 0.0])
            });

            let hit_normal = col.normal.unwrap_or([0.0, 1.0, 0.0]);

            // Dispatch surface impact event for entity A
            event_bus.send(SurfaceImpactEvent {
                entity: col.entity_a,
                surface_type: surface_a,
                hit_point,
                hit_normal,
                energy: col.impulse,
            });

            // Dispatch surface impact event for entity B
            event_bus.send(SurfaceImpactEvent {
                entity: col.entity_b,
                surface_type: surface_b,
                hit_point,
                hit_normal: [-hit_normal[0], -hit_normal[1], -hit_normal[2]],
                energy: col.impulse,
            });

            re_emit_collisions.push(col);
        }

        // Re-emit CollisionEnter events back onto the bus so NativeBehavior and other listeners receive them
        for col in re_emit_collisions {
            event_bus.send(col);
        }
    }
}