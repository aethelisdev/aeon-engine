// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Destructible targets, hit reactions, health tracking, and projectile lifecycle subsystem.
//!

use ae_core::ecs::{BehaviorComponent, BehaviorType, Color, Scale};
use ae_core::events::{DynamicEventBus, RaycastHitEvent, TargetDestroyedEvent};
use hecs::{Entity, World};

/// Processes raycast damage events, decrements health, triggers hit flashes, and broadcasts destruction.
pub fn process_destructible_hits(world: &mut World, event_bus: &mut DynamicEventBus) {
    let mut targets_to_destroy = Vec::new();
    if let Some(hits) = event_bus.receive::<RaycastHitEvent>() {
        for hit in hits {
            if let Ok(mut target_behavior) = world.get::<&mut BehaviorComponent>(hit.target)
                && target_behavior.behavior_type == BehaviorType::DestructibleTarget
            {
                target_behavior.health = (target_behavior.health - hit.damage).max(0.0);
                target_behavior.hit_flash_timer = 0.35; // 350ms bright flash

                log::info!(
                    "🎯 [RaycastHit] Target {:?} took {} damage! Health: {}/{}",
                    hit.target,
                    hit.damage,
                    target_behavior.health,
                    target_behavior.max_health
                );

                if target_behavior.health <= 0.0 {
                    event_bus.send(TargetDestroyedEvent { target: hit.target });
                    targets_to_destroy.push(hit.target);
                }
            }
        }
    }

    for target in targets_to_destroy {
        log::info!("💥 [Target Destroyed] Entity {:?} was destroyed!", target);
        if let Ok(mut col) = world.get::<&mut Color>(target) {
            col.r = 0.2;
            col.g = 0.2;
            col.b = 0.2;
            col.a = 0.6;
        }
        if let Ok(mut scale) = world.get::<&mut Scale>(target) {
            scale.y *= 0.35; // Visually flatten / destroy the dummy
        }
        let _ = world.insert_one(target, ae_core::ecs::TransformDirty);
    }
}

/// Updates visual hit flash timers and restores dynamic health colors.
pub fn update_destructible_visuals(world: &mut World, destructible_entities: &[Entity], dt: f32) {
    for &ent in destructible_entities {
        if let Ok(mut behavior) = world.get::<&mut BehaviorComponent>(ent)
            && behavior.hit_flash_timer > 0.0
        {
            behavior.hit_flash_timer = (behavior.hit_flash_timer - dt).max(0.0);
            if let Ok(mut col) = world.get::<&mut Color>(ent) {
                if behavior.hit_flash_timer > 0.0 {
                    // Flash bright orange/red
                    col.r = 1.0;
                    col.g = 0.3;
                    col.b = 0.1;
                } else {
                    // Restore healthy / damage ratio color
                    let health_ratio = behavior.health / behavior.max_health.max(1.0);
                    col.r = 1.0 - health_ratio * 0.6;
                    col.g = health_ratio * 0.8;
                    col.b = 0.2;
                }
            }
        }
    }
}

/// Updates projectile lifetime timers and cleans up expired entities.
pub fn update_ephemeral_projectiles(world: &mut World, dt: f32) {
    let mut projectiles_to_despawn = Vec::new();
    for (ent, behavior) in world
        .query::<(hecs::Entity, &mut BehaviorComponent)>()
        .iter()
    {
        if behavior.behavior_type == BehaviorType::Custom {
            behavior.timer -= dt;
            if behavior.timer <= 0.0 {
                projectiles_to_despawn.push(ent);
            }
        }
    }
    for ent in projectiles_to_despawn {
        let _ = world.despawn(ent);
    }
}