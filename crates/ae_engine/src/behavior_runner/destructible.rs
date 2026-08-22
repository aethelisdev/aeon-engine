// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Destructible targets, hit reactions, health tracking, and projectile lifecycle subsystem.
//!

use ae_core::ecs::{Color, DestructibleTarget, EphemeralProjectile, Scale};
use ae_core::events::{DynamicEventBus, RaycastHitEvent, TargetDestroyedEvent};
use hecs::World;

/// Processes raycast damage events, decrements health, triggers hit flashes, and broadcasts destruction.
pub fn process_destructible_hits(world: &mut World, event_bus: &mut DynamicEventBus) {
    let mut targets_to_destroy = Vec::new();
    if let Some(hits) = event_bus.receive::<RaycastHitEvent>() {
        for hit in hits {
            if let Ok(mut target) = world.get::<&mut DestructibleTarget>(hit.target) {
                target.health = (target.health - hit.damage).max(0.0);

                // Save original color before applying flash if not currently flashing
                if target.hit_flash_timer <= 0.0
                    && let Ok(col) = world.get::<&Color>(hit.target)
                {
                    target.original_color = [col.r, col.g, col.b, col.a];
                }
                target.hit_flash_timer = 0.20; // 200ms impact flash

                // Apply bright impact flash color
                if let Ok(mut col) = world.get::<&mut Color>(hit.target) {
                    col.r = 1.0;
                    col.g = 0.9;
                    col.b = 0.4;
                    col.a = 1.0;
                }

                log::info!(
                    "🎯 [RaycastHit] Target {:?} took {} damage! Health: {}/{}",
                    hit.target,
                    hit.damage,
                    target.health,
                    target.max_health
                );

                if target.health <= 0.0 {
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
pub fn update_destructible_visuals(world: &mut World, dt: f32) {
    for (target, color_opt) in world.query_mut::<(&mut DestructibleTarget, Option<&mut Color>)>() {
        if target.hit_flash_timer > 0.0 {
            target.hit_flash_timer = (target.hit_flash_timer - dt).max(0.0);
            if target.hit_flash_timer == 0.0 {
                // Restore the entity's exact original color
                if let Some(col) = color_opt {
                    col.r = target.original_color[0];
                    col.g = target.original_color[1];
                    col.b = target.original_color[2];
                    col.a = target.original_color[3];
                }
            }
        }
    }
}

/// Updates projectile lifetime timers and cleans up expired entities.
pub fn update_ephemeral_projectiles(world: &mut World, dt: f32) {
    let mut projectiles_to_despawn = Vec::new();
    for (ent, projectile) in world.query_mut::<(hecs::Entity, &mut EphemeralProjectile)>() {
        projectile.lifetime_remaining -= dt;
        if projectile.lifetime_remaining <= 0.0 {
            projectiles_to_despawn.push(ent);
        }
    }
    for ent in projectiles_to_despawn {
        let _ = world.despawn(ent);
    }
}