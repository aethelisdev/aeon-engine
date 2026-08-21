// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Character combat actions, weapon raycast shooting, kinetic impulses, and projectile spawning.
//!

use ae_core::ecs::{
    BehaviorComponent, BehaviorType, Collider, ColliderShape, Color, Name, Position, RigidBody,
    RigidBodyType, Rotation, Scale, TransformDirty, Velocity,
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
    character_action_entities: &[Entity],
    camera_forward: cgmath::Vector3<f32>,
) {
    let fire_pressed = input.is_action_just_pressed("Fire")
        || input.is_mouse_button_just_pressed(winit::event::MouseButton::Left)
        || input.is_key_just_pressed(ae_editor::input::KeyCode::KeyF);

    if fire_pressed && !character_action_entities.is_empty() {
        let mut projectiles_to_spawn = Vec::new();

        for &shooter_ent in character_action_entities {
            let shooter_pos = world
                .get::<&Position>(shooter_ent)
                .map(|p| [p.x, p.y, p.z])
                .ok();

            if let Some(pos) = shooter_pos {
                let ray_origin = ae_physics::glam::Vec3::new(pos[0], pos[1] + 1.5, pos[2]);
                let ray_dir = ae_physics::glam::Vec3::new(
                    camera_forward.x,
                    camera_forward.y,
                    camera_forward.z,
                )
                .normalize();

                log::info!(
                    "🔫 [Weapon Fired] Shooter {:?} fired laser bolt!",
                    shooter_ent
                );

                // Queue fast, straight-line laser projectile in 3D world
                let bolt_pos = ray_origin + ray_dir * 0.8;
                let bolt_vel = ray_dir * 60.0;
                projectiles_to_spawn.push((bolt_pos, bolt_vel));

                // Cast ray excluding the shooter's own collider
                if let Some(hit) = physics_world.raycast_filtered(
                    ray_origin,
                    ray_dir,
                    100.0,
                    Some(shooter_ent),
                    true,
                ) {
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

                    // Physical kinetic impulse to dynamic physics bodies (e.g. bouncing cubes)
                    let impulse = ray_dir * 10.0 + ae_physics::glam::Vec3::new(0.0, 3.5, 0.0);
                    physics_world.apply_impulse(hit.entity, impulse);
                } else {
                    log::info!("💨 [Weapon Raycast] Shot missed (no target in line of fire)");
                }
            }
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
                BehaviorComponent {
                    behavior_type: BehaviorType::Custom,
                    speed: 0.0,
                    axis: [0.0, 0.0, 0.0],
                    timer: 0.7, // 700ms lifetime then auto-despawn
                    is_triggered: false,
                    target_position: [0.0, 0.0, 0.0],
                    original_position: [0.0, 0.0, 0.0],
                    ping_pong_forward: true,
                    health: 1.0,
                    max_health: 1.0,
                    hit_flash_timer: 0.0,
                },
                TransformDirty,
            ));
        }
    }
}