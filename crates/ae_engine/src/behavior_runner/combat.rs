// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Character combat actions, weapon raycast shooting, kinetic impulses, and projectile spawning.
//!

use ae_core::ecs::{
    CharacterAction, Collider, ColliderShape, Color, EphemeralProjectile, Name, Position,
    RigidBody, RigidBodyType, Rotation, Scale, TransformDirty, Velocity,
};
use ae_core::events::{DynamicEventBus, RaycastHitEvent};
use ae_editor::input::InputManager;
use ae_physics::world::PhysicsWorld;
use hecs::{Entity, World};

/// Updates weapon shooting for character actions, queries filtered physics raycasts, applies impulses, and spawns projectiles.
pub fn update_character_actions(
    world: &mut World,
    physics_world: &mut PhysicsWorld,
    input: &InputManager,
    event_bus: &mut DynamicEventBus,
    camera_forward: cgmath::Vector3<f32>,
    dt: f32,
) {
    // 1. Tick cooldown timers for all active CharacterAction components
    for (_ent, action) in world.query::<(Entity, &mut CharacterAction)>().iter() {
        action.timer = (action.timer - dt).max(0.0);
    }

    let fire_pressed = input.is_action_just_pressed("Fire")
        || input.is_mouse_button_just_pressed(winit::event::MouseButton::Left)
        || input.is_key_just_pressed(ae_editor::input::KeyCode::KeyF);

    if fire_pressed {
        let mut projectiles_to_spawn = Vec::new();

        // 2. Filter shooters whose cooldown timer has elapsed
        let mut ready_shooters = Vec::new();
        for (ent, action, pos) in world
            .query::<(Entity, &mut CharacterAction, &Position)>()
            .iter()
        {
            if action.timer <= 0.0 {
                action.timer = action.cooldown;
                let speed = if action.speed > 0.0 {
                    action.speed
                } else {
                    50.0
                };
                ready_shooters.push((ent, *pos, speed));
            }
        }

        for (shooter_ent, pos, speed) in ready_shooters {
            shooters_action(
                shooter_ent,
                &pos,
                speed,
                camera_forward,
                physics_world,
                event_bus,
                &mut projectiles_to_spawn,
            );
        }

        for (b_pos, b_vel) in projectiles_to_spawn {
            let _ = world.spawn((
                Name("Laser Bolt".to_string()),
                Position {
                    x: b_pos.x,
                    y: b_pos.y,
                    z: b_pos.z,
                },
                Velocity {
                    x: b_vel.x,
                    y: b_vel.y,
                    z: b_vel.z,
                },
                Rotation {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                Scale {
                    x: 0.2,
                    y: 0.2,
                    z: 0.2,
                },
                Color::new(1.0, 0.85, 0.15, 1.0),
                ae_core::ecs::Shape::Sphere,
                RigidBody {
                    body_type: RigidBodyType::Dynamic,
                    mass: 0.05,
                    gravity_scale: 0.0, // Straight ballistic trajectory without gravity droop
                },
                Collider {
                    shape: ColliderShape::Sphere { radius: 0.2 },
                    friction: 0.0,
                    restitution: 0.0,
                    is_sensor: true, // Sensor prevents slow-mo floor bouncing
                },
                EphemeralProjectile::new(1.5),
                TransformDirty,
            ));
        }
    }
}

/// Helper function executing weapon raycasting and physical impulse application for a shooter.
fn shooters_action(
    shooter_ent: Entity,
    pos: &Position,
    speed: f32,
    camera_forward: cgmath::Vector3<f32>,
    physics_world: &mut PhysicsWorld,
    event_bus: &mut DynamicEventBus,
    projectiles_to_spawn: &mut Vec<(ae_physics::glam::Vec3, ae_physics::glam::Vec3)>,
) {
    let ray_origin = ae_physics::glam::Vec3::new(pos.x, pos.y + 1.5, pos.z);
    let ray_dir = ae_physics::glam::Vec3::new(camera_forward.x, camera_forward.y, camera_forward.z)
        .normalize();

    log::info!(
        "🔫 [Weapon Fired] Shooter {:?} fired laser bolt with speed: {:.1} m/s!",
        shooter_ent,
        speed
    );

    // Queue projectile with dynamic speed configured from CharacterAction
    let bolt_pos = ray_origin + ray_dir * 0.8;
    let bolt_vel = ray_dir * speed;
    projectiles_to_spawn.push((bolt_pos, bolt_vel));

    // Cast ray with range proportional to weapon speed
    let ray_distance = (speed * 2.0).clamp(50.0, 500.0);
    if let Some(hit) =
        physics_world.raycast_filtered(ray_origin, ray_dir, ray_distance, Some(shooter_ent), true)
    {
        log::info!(
            "⚡ [Weapon Raycast] Hit Entity {:?} at dist: {:.2}m",
            hit.entity,
            hit.distance
        );

        event_bus.send(RaycastHitEvent {
            shooter: Some(shooter_ent),
            target: hit.entity,
            hit_point: hit.point,
            hit_normal: hit.normal,
            damage: 25.0,
        });

        // Physical kinetic impulse scaled by weapon velocity
        let impulse_mag = (speed * 0.2).clamp(5.0, 50.0);
        let impulse = ray_dir * impulse_mag + ae_physics::glam::Vec3::new(0.0, 3.5, 0.0);
        physics_world.apply_impulse(hit.entity, impulse);
    } else {
        log::info!("💨 [Weapon Raycast] Shot missed (no target in line of fire)");
    }
}