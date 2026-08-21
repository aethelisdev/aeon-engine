// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Gameplay behavior execution pipeline for interactive gameplay systems.
//!

use ae_core::ecs::{BehaviorComponent, BehaviorType, Color, Position, Rotation, Scale};
use ae_core::events::{DynamicEventBus, RaycastHitEvent, TargetDestroyedEvent};
use ae_editor::input::InputManager;
use ae_physics::world::PhysicsWorld;
use cgmath::{InnerSpace, Rotation3};
use hecs::World;

/// Parameters bundle for executing the behavior runner pipeline.
pub struct BehaviorRunnerParams<'a> {
    pub world: &'a mut World,
    pub physics_world: &'a mut PhysicsWorld,
    pub input: &'a InputManager,
    pub event_bus: &'a mut DynamicEventBus,
    pub camera_forward: cgmath::Vector3<f32>,
    pub delta_time: f32,
}

/// Executes all active entity behaviors during Play mode.
pub fn update_gameplay_behaviors(params: BehaviorRunnerParams<'_>) {
    let world = params.world;
    let physics_world = params.physics_world;
    let input = params.input;
    let event_bus = params.event_bus;
    let camera_forward = params.camera_forward;
    let dt = params.delta_time;

    // 1. Process physics trigger events dispatched by Rapier
    let mut player_entity_and_pos = None;

    for (ent, _) in world
        .query::<(hecs::Entity, &ae_core::ecs::PlayerTag)>()
        .iter()
    {
        if let Ok(pos) = world.get::<&Position>(ent) {
            player_entity_and_pos = Some((ent, [pos.x, pos.y, pos.z]));
            break;
        }
    }
    if player_entity_and_pos.is_none() {
        for (ent, _) in world
            .query::<(hecs::Entity, &ae_core::ecs::CharacterController)>()
            .iter()
        {
            if let Ok(pos) = world.get::<&Position>(ent) {
                player_entity_and_pos = Some((ent, [pos.x, pos.y, pos.z]));
                break;
            }
        }
    }

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

    // 2. Process raycast hit events applied to destructible targets
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

    // 3. Collect behaviors to evaluate
    let mut rotators = Vec::new();
    let mut moving_platforms = Vec::new();
    let mut trigger_zones = Vec::new();
    let mut destructible_targets = Vec::new();
    let mut character_actions = Vec::new();

    for (entity, behavior) in world.query::<(hecs::Entity, &BehaviorComponent)>().iter() {
        match behavior.behavior_type {
            BehaviorType::Rotator => rotators.push(entity),
            BehaviorType::MovingPlatform => moving_platforms.push(entity),
            BehaviorType::TriggerZone => trigger_zones.push(entity),
            BehaviorType::DestructibleTarget => destructible_targets.push(entity),
            BehaviorType::CharacterAction => character_actions.push(entity),
            BehaviorType::Custom => {}
        }
    }

    let mut dirty_entities = Vec::new();

    // 4. Update Rotators (Smooth Euler/Quaternion rotation)
    for ent in rotators {
        if let Ok(mut behavior) = world.get::<&mut BehaviorComponent>(ent) {
            let speed = behavior.speed;
            let axis = behavior.axis;
            behavior.timer += dt;

            if let Ok(mut rot) = world.get::<&mut Rotation>(ent) {
                let angle_rad = speed * dt;
                let axis_vec = cgmath::Vector3::new(axis[0], axis[1], axis[2]);
                let norm_axis = if axis_vec.magnitude2() > 0.001 {
                    axis_vec.normalize()
                } else {
                    cgmath::Vector3::unit_y()
                };

                let delta_q =
                    cgmath::Quaternion::from_axis_angle(norm_axis, cgmath::Rad(angle_rad));
                let cur_q = cgmath::Quaternion::new(rot.w, rot.x, rot.y, rot.z);
                let new_q = cur_q * delta_q;

                rot.x = new_q.v.x;
                rot.y = new_q.v.y;
                rot.z = new_q.v.z;
                rot.w = new_q.s;
                dirty_entities.push(ent);
            }
        }
    }

    // 5. Update Moving Platforms (Waypoint interpolation & passenger translation)
    for ent in moving_platforms {
        if let Ok(mut behavior) = world.get::<&mut BehaviorComponent>(ent)
            && let Ok(mut pos) = world.get::<&mut Position>(ent)
        {
            let target = if behavior.ping_pong_forward {
                behavior.target_position
            } else {
                behavior.original_position
            };
            let speed = behavior.speed;

            let dx = target[0] - pos.x;
            let dy = target[1] - pos.y;
            let dz = target[2] - pos.z;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();

            if dist < 0.05 {
                // Reached waypoint: flip direction
                behavior.ping_pong_forward = !behavior.ping_pong_forward;
            } else {
                let step = (speed * dt).min(dist);
                let inv_dist = 1.0 / dist;
                let step_x = dx * inv_dist * step;
                let step_y = dy * inv_dist * step;
                let step_z = dz * inv_dist * step;

                // If player is standing on the elevator/platform, carry player
                if let Some((p_ent, p_pos)) = player_entity_and_pos {
                    let on_platform_xz =
                        (p_pos[0] - pos.x).abs() <= 2.3 && (p_pos[2] - pos.z).abs() <= 2.3;
                    let on_platform_y = p_pos[1] >= pos.y - 0.2 && p_pos[1] <= pos.y + 1.8;
                    if on_platform_xz
                        && on_platform_y
                        && let Ok(mut player_pos_comp) = world.get::<&mut Position>(p_ent)
                    {
                        player_pos_comp.x += step_x;
                        player_pos_comp.y += step_y;
                        player_pos_comp.z += step_z;
                        dirty_entities.push(p_ent);
                    }
                }

                pos.x += step_x;
                pos.y += step_y;
                pos.z += step_z;
                dirty_entities.push(ent);
            }
        }
    }

    // 6. Update Trigger Zones (Physics Trigger driven motion & visual response)
    let any_sensor_triggered = world
        .query::<&BehaviorComponent>()
        .iter()
        .any(|b| b.behavior_type == BehaviorType::TriggerZone && b.is_triggered);

    for ent in trigger_zones {
        if let Ok(behavior) = world.get::<&BehaviorComponent>(ent) {
            let is_elevating_mechanism =
                (behavior.original_position[1] - behavior.target_position[1]).abs() > 0.05;

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

    for ent in dirty_entities {
        let _ = world.insert_one(ent, ae_core::ecs::TransformDirty);
    }

    // 7. Update Destructible Targets (Hit flash timer decay)
    for ent in destructible_targets {
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

    // 8. Update Ephemeral Projectiles (Lifetime decay and auto-despawn)
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

    // 9. Update Character Actions (Raycast Weapon Shooting & Interactions)
    let fire_pressed = input.is_action_just_pressed("Fire")
        || input.is_mouse_button_just_pressed(winit::event::MouseButton::Left)
        || input.is_key_just_pressed(ae_editor::input::KeyCode::KeyF);

    if fire_pressed && !character_actions.is_empty() {
        let mut projectiles_to_spawn = Vec::new();

        for shooter_ent in character_actions {
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
                ae_core::ecs::Name("Laser Bolt".to_string()),
                ae_core::ecs::Position {
                    x: b_pos.x,
                    y: b_pos.y,
                    z: b_pos.z,
                },
                ae_core::ecs::Velocity {
                    x: b_vel.x,
                    y: b_vel.y,
                    z: b_vel.z,
                },
                ae_core::ecs::Rotation {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                ae_core::ecs::Scale {
                    x: 0.2,
                    y: 0.2,
                    z: 0.2,
                },
                ae_core::ecs::Color::new(1.0, 0.85, 0.15, 1.0),
                ae_core::ecs::Shape::Sphere,
                ae_core::ecs::RigidBody {
                    body_type: ae_core::ecs::RigidBodyType::Dynamic,
                    mass: 0.05,
                    gravity_scale: 0.0, // Straight ballistic trajectory without gravity droop
                },
                ae_core::ecs::Collider {
                    shape: ae_core::ecs::ColliderShape::Sphere { radius: 0.2 },
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
                ae_core::ecs::TransformDirty,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::ecs::{BehaviorComponent, BehaviorType, Position, Rotation};
    use ae_editor::input::InputManager;
    use ae_physics::world::PhysicsWorld;
    use hecs::World;

    #[test]
    fn test_rotator_behavior_rotation_progression() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();
        let input = InputManager::new();
        let mut event_bus = DynamicEventBus::new();

        let ent = world.spawn((
            Position::new(0.0, 0.0, 0.0),
            Rotation::identity(),
            BehaviorComponent::rotator(2.0, [0.0, 1.0, 0.0]),
        ));

        update_gameplay_behaviors(BehaviorRunnerParams {
            world: &mut world,
            physics_world: &mut physics,
            input: &input,
            event_bus: &mut event_bus,
            camera_forward: cgmath::Vector3::unit_z(),
            delta_time: 0.5,
        });

        let rot = *world.get::<&Rotation>(ent).unwrap();
        assert!(
            rot.y.abs() > 0.01 || rot.w.abs() < 0.99,
            "Rotator must advance rotation quaternion"
        );
    }

    #[test]
    fn test_moving_platform_waypoint_interpolation() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();
        let input = InputManager::new();
        let mut event_bus = DynamicEventBus::new();

        let ent = world.spawn((
            Position::new(0.0, 0.0, 0.0),
            Rotation::identity(),
            BehaviorComponent::moving_platform(5.0, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
        ));

        update_gameplay_behaviors(BehaviorRunnerParams {
            world: &mut world,
            physics_world: &mut physics,
            input: &input,
            event_bus: &mut event_bus,
            camera_forward: cgmath::Vector3::unit_z(),
            delta_time: 1.0,
        });

        let pos = *world.get::<&Position>(ent).unwrap();
        assert!(
            (pos.x - 5.0).abs() < 0.1,
            "Platform should move 5.0 units in 1 second at 5.0 m/s"
        );
    }

    #[test]
    fn test_trigger_zone_activation_and_elevation() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();
        let input = InputManager::new();
        let mut event_bus = DynamicEventBus::new();

        let trigger_ent = world.spawn((
            Position::new(0.0, 0.0, 0.0),
            Color::soft_blue(),
            BehaviorComponent {
                behavior_type: BehaviorType::TriggerZone,
                speed: 4.0,
                axis: [0.0, 1.0, 0.0],
                health: 100.0,
                max_health: 100.0,
                is_triggered: false,
                original_position: [0.0, 0.0, 0.0],
                target_position: [0.0, 4.0, 0.0],
                ping_pong_forward: true,
                timer: 0.0,
                hit_flash_timer: 0.0,
            },
        ));

        let other_ent = world.spawn((Position::new(0.0, 0.0, 0.0),));

        // Send TriggerEnter event
        event_bus.send(ae_core::events::TriggerEnter {
            entity_a: trigger_ent,
            entity_b: other_ent,
        });

        update_gameplay_behaviors(BehaviorRunnerParams {
            world: &mut world,
            physics_world: &mut physics,
            input: &input,
            event_bus: &mut event_bus,
            camera_forward: cgmath::Vector3::unit_z(),
            delta_time: 0.5,
        });

        let behavior = world.get::<&BehaviorComponent>(trigger_ent).unwrap();
        assert!(
            behavior.is_triggered,
            "Trigger zone should be marked triggered"
        );

        let pos = world.get::<&Position>(trigger_ent).unwrap();
        assert!(
            pos.y > 0.5,
            "Trigger zone should begin moving up towards target Y"
        );
    }

    #[test]
    fn test_destructible_target_damage_and_destruction_event() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();
        let input = InputManager::new();
        let mut event_bus = DynamicEventBus::new();

        let target_ent = world.spawn((
            Position::new(0.0, 0.0, 5.0),
            Color::red(),
            BehaviorComponent::destructible_target(50.0),
        ));

        // Send 30 damage hit
        event_bus.send(RaycastHitEvent {
            shooter: None,
            target: target_ent,
            hit_point: [0.0, 0.0, 4.5],
            hit_normal: [0.0, 0.0, -1.0],
            damage: 30.0,
        });

        update_gameplay_behaviors(BehaviorRunnerParams {
            world: &mut world,
            physics_world: &mut physics,
            input: &input,
            event_bus: &mut event_bus,
            camera_forward: cgmath::Vector3::unit_z(),
            delta_time: 0.016,
        });

        {
            let behavior = world.get::<&BehaviorComponent>(target_ent).unwrap();
            assert_eq!(
                behavior.health, 20.0,
                "Health should drop to 20.0 after 30 damage"
            );
            assert!(
                behavior.hit_flash_timer > 0.0,
                "Hit flash timer should be active"
            );
        }

        // Send fatal 25 damage hit
        event_bus.send(RaycastHitEvent {
            shooter: None,
            target: target_ent,
            hit_point: [0.0, 0.0, 4.5],
            hit_normal: [0.0, 0.0, -1.0],
            damage: 25.0,
        });

        update_gameplay_behaviors(BehaviorRunnerParams {
            world: &mut world,
            physics_world: &mut physics,
            input: &input,
            event_bus: &mut event_bus,
            camera_forward: cgmath::Vector3::unit_z(),
            delta_time: 0.016,
        });

        let behavior = world.get::<&BehaviorComponent>(target_ent).unwrap();
        assert_eq!(behavior.health, 0.0, "Health should clamp to 0.0");

        let destruction_events = event_bus.receive::<TargetDestroyedEvent>();
        assert!(
            destruction_events.is_some(),
            "TargetDestroyedEvent should be broadcast"
        );
        let events = destruction_events.unwrap();
        assert_eq!(events[0].target, target_ent);
    }
}