// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Trigger zone, sensor pad, and elevating door mechanism subsystem.
//!

use ae_core::ecs::{BehaviorComponent, BehaviorType, Color, Position, Scale};
use ae_core::events::DynamicEventBus;
use hecs::{Entity, World};

/// Processes incoming trigger enter/exit events dispatched by Rapier.
pub fn process_trigger_events(world: &mut World, event_bus: &mut DynamicEventBus) {
    if let Some(events) = event_bus.receive::<ae_core::events::TriggerEnter>() {
        for ev in events {
            if let Ok(mut behavior) = world.get::<&mut BehaviorComponent>(ev.entity_a)
                && behavior.behavior_type == BehaviorType::TriggerZone
            {
                behavior.is_triggered = true;
            }
            if let Ok(mut behavior) = world.get::<&mut BehaviorComponent>(ev.entity_b)
                && behavior.behavior_type == BehaviorType::TriggerZone
            {
                behavior.is_triggered = true;
            }
        }
    }

    if let Some(events) = event_bus.receive::<ae_core::events::TriggerExit>() {
        for ev in events {
            if let Ok(mut behavior) = world.get::<&mut BehaviorComponent>(ev.entity_a)
                && behavior.behavior_type == BehaviorType::TriggerZone
            {
                behavior.is_triggered = false;
            }
            if let Ok(mut behavior) = world.get::<&mut BehaviorComponent>(ev.entity_b)
                && behavior.behavior_type == BehaviorType::TriggerZone
            {
                behavior.is_triggered = false;
            }
        }
    }
}

/// Performs continuous spatial AABB volume tests between the player and sensor pads.
pub fn test_spatial_sensor_overlaps(
    world: &mut World,
    player_entity_and_pos: Option<(Entity, [f32; 3])>,
) {
    if let Some((_p_ent, p_pos)) = player_entity_and_pos {
        for (behavior, pos, scale_opt, col_opt) in world
            .query_mut::<(
                &mut BehaviorComponent,
                &Position,
                Option<&Scale>,
                Option<&ae_core::ecs::Collider>,
            )>()
            .into_iter()
        {
            if behavior.behavior_type == BehaviorType::TriggerZone {
                let is_sensor = col_opt.map(|c| c.is_sensor).unwrap_or(false);
                if is_sensor {
                    let sx = scale_opt.map(|s| s.x.abs()).unwrap_or(1.0);
                    let sy = scale_opt.map(|s| s.y.abs()).unwrap_or(1.0);
                    let sz = scale_opt.map(|s| s.z.abs()).unwrap_or(1.0);

                    let (hx, hy, hz) = if let Some(col) = col_opt {
                        match col.shape {
                            ae_core::ecs::ColliderShape::Box { half_extents } => (
                                half_extents[0] * sx,
                                (half_extents[1] * sy).max(1.5),
                                half_extents[2] * sz,
                            ),
                            ae_core::ecs::ColliderShape::Sphere { radius } => (
                                radius * sx.max(sy).max(sz),
                                (radius * sx.max(sy).max(sz)).max(1.5),
                                radius * sx.max(sy).max(sz),
                            ),
                            _ => (sx * 0.5, (sy * 0.5).max(1.5), sz * 0.5),
                        }
                    } else {
                        (sx * 0.5, 1.5, sz * 0.5)
                    };

                    let inside_x = (p_pos[0] - pos.x).abs() <= hx + 0.5;
                    let inside_z = (p_pos[2] - pos.z).abs() <= hz + 0.5;
                    let inside_y = p_pos[1] >= pos.y - 0.5 && p_pos[1] <= pos.y + hy + 2.5;

                    if inside_x && inside_z && inside_y {
                        behavior.is_triggered = true;
                    }
                }
            }
        }
    }
}

/// Updates visual feedback or vertical movement for trigger zones and linked mechanisms.
pub fn update_trigger_zone_mechanisms(
    world: &mut World,
    trigger_zone_entities: &[Entity],
    dt: f32,
    dirty_entities: &mut Vec<Entity>,
) {
    let any_sensor_triggered = world
        .query::<&BehaviorComponent>()
        .iter()
        .any(|b| b.behavior_type == BehaviorType::TriggerZone && b.is_triggered);

    for &ent in trigger_zone_entities {
        if let Ok(behavior) = world.get::<&BehaviorComponent>(ent) {
            let is_sensor = world
                .get::<&ae_core::ecs::Collider>(ent)
                .map(|c| c.is_sensor)
                .unwrap_or(false);

            let is_elevating_mechanism = !is_sensor
                && (behavior.original_position[1] - behavior.target_position[1]).abs() > 0.05;

            let is_active = if is_elevating_mechanism {
                any_sensor_triggered || behavior.is_triggered
            } else {
                behavior.is_triggered
            };

            if is_elevating_mechanism {
                let target_y = if is_active {
                    behavior.target_position[1]
                } else {
                    behavior.original_position[1]
                };

                if let Ok(mut pos) = world.get::<&mut Position>(ent) {
                    let diff = target_y - pos.y;
                    if diff.abs() > 0.01 {
                        pos.y += diff.signum() * (behavior.speed * dt).min(diff.abs());
                        dirty_entities.push(ent);
                    }
                }
            } else if let Ok(mut col) = world.get::<&mut Color>(ent) {
                // Stationary trigger zone pad visual feedback (Component-driven)
                if is_active {
                    col.r = 0.2;
                    col.g = 1.0;
                    col.b = 0.3;
                } else {
                    col.r = 0.1;
                    col.g = 0.7;
                    col.b = 0.2;
                }
            }
        }
    }
}