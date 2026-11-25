// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Destructible targets, hit reactions, health tracking, and projectile lifecycle subsystem.

use ae_core::events::{DynamicEventBus, RaycastHitEvent, TargetDestroyedEvent};
use ae_ecs::World;
use ae_plugin_api::{Color, DestructibleTarget, EphemeralProjectile, Scale};

/// Processes raycast damage events, decrements health, triggers hit flashes, and broadcasts destruction.
pub fn process_destructible_hits(world: &mut World, event_bus: &mut DynamicEventBus) {
    let mut targets_to_destroy = Vec::new();
    if let Some(hits) = event_bus.receive::<RaycastHitEvent>() {
        for hit in hits {
            let needs_orig = world
                .inspect::<DestructibleTarget, _>(hit.target, |t| {
                    t.health > 0.0 && t.hit_flash_timer <= 0.0
                })
                .unwrap_or(false);
            let orig_color = if needs_orig {
                world
                    .get_copied::<Color>(hit.target)
                    .ok()
                    .map(|col| [col.r, col.g, col.b, col.a])
            } else {
                None
            };

            let mut flash_applied = false;
            if let Ok(target) = world.get_mut::<DestructibleTarget>(hit.target) {
                // If target is already dead (health <= 0.0), ignore subsequent hits!
                if target.health <= 0.0 {
                    continue;
                }

                target.health = (target.health - hit.damage).max(0.0);

                // Save original color before applying flash if not currently flashing
                if let Some(orig) = orig_color {
                    target.original_color = orig;
                }
                target.hit_flash_timer = 0.20; // 200ms impact flash
                flash_applied = true;

                log::info!(
                    "🎯 [RaycastHit] Target {:?} took {} damage! Health: {}/{}",
                    hit.target,
                    hit.damage,
                    target.health,
                    target.max_health
                );

                event_bus.send(ae_core::events::DamageEvent {
                    source: hit.shooter,
                    target: hit.target,
                    amount: hit.damage,
                    hit_point: Some(hit.hit_point),
                    hit_normal: Some(hit.hit_normal),
                });

                if target.health <= 0.0 {
                    event_bus.send(TargetDestroyedEvent { target: hit.target });
                    event_bus.send(ae_core::events::ActorKilledEvent {
                        killer: hit.shooter,
                        victim: hit.target,
                    });
                    event_bus.send(ae_core::events::ScoreEvent {
                        delta: 100,
                        new_total: 100,
                    });
                    targets_to_destroy.push(hit.target);
                }
            }

            // Apply bright impact flash color
            if flash_applied && let Ok(col) = world.get_mut::<Color>(hit.target) {
                col.r = 1.0;
                col.g = 0.9;
                col.b = 0.4;
                col.a = 1.0;
            }
        }
    }

    for target in targets_to_destroy {
        log::info!("💥 [Target Destroyed] Entity {:?} was destroyed!", target);
        if let Ok(col) = world.get_mut::<Color>(target) {
            col.r = 0.2;
            col.g = 0.2;
            col.b = 0.2;
            col.a = 0.6;
        }
        if let Ok(scale) = world.get_mut::<Scale>(target) {
            scale.y *= 0.35; // Visually flatten / destroy the dummy
        }
        let _ = world.insert_one(target, ae_plugin_api::TransformDirty);
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
    for (ent, projectile) in world.query_mut::<(ae_ecs::Entity, &mut EphemeralProjectile)>() {
        projectile.lifetime_remaining -= dt;
        if projectile.lifetime_remaining <= 0.0 {
            projectiles_to_despawn.push(ent);
        }
    }
    for ent in projectiles_to_despawn {
        let _ = world.despawn(ent);
    }
}