// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

/// AE Physics — Physics simulation unit tests.

#[cfg(test)]
mod tests {
    use super::super::*;
    use ae_core::ecs::{
        AssetHandle, CharacterController, Collider, ColliderShape, ModelId, Position, RigidBody,
        RigidBodyType, Rotation, Scale, Velocity,
    };
    use glam::Vec3;
    use hecs::World;

    #[test]
    fn test_physics_gravity_falling() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        // Spawn a dynamic rigid body entity at y = 10.0 with a collider
        let entity = world.spawn((
            Position {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            },
            Rotation::identity(),
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 1.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Sphere { radius: 0.5 },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // Sync initially
        physics.sync_ecs_to_physics(&mut world, |_| None);
        assert!(physics.entity_to_body.contains_key(&entity));

        let mut event_bus = ae_core::events::DynamicEventBus::new();
        // Step physics multiple times
        for _ in 0..10 {
            physics.step(&mut world, |_| None, 0.1, &mut event_bus);
        }

        // Verify the entity fell down (y < 10.0)
        let pos = world.get::<&Position>(entity).unwrap();
        assert!(
            pos.y < 10.0,
            "Entity should have fallen under gravity, got y = {}",
            pos.y
        );
    }

    #[test]
    fn test_trimesh_collider_creation() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0, 1, 2];
        let asset_handle = AssetHandle::default();

        let entity = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            ModelId(asset_handle),
            Collider {
                shape: ColliderShape::Trimesh,
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // Sync and resolve mesh data via closure
        physics.sync_ecs_to_physics(&mut world, |handle| {
            if handle == asset_handle {
                Some((&vertices, &indices))
            } else {
                None
            }
        });

        assert!(physics.entity_to_body.contains_key(&entity));
        let body_handle = physics.entity_to_body[&entity];
        let body = physics.rigid_body_set.get(body_handle).unwrap();
        assert!(!body.is_dynamic());

        assert_eq!(body.colliders().len(), 1);
        let col_handle = body.colliders()[0];
        let col = physics.collider_set.get(col_handle).unwrap();

        assert!(
            col.shape().as_trimesh().is_some(),
            "Collider shape should be a trimesh"
        );
    }

    #[test]
    fn test_trigger_sensor_events() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        // Sensor volume entity at origin
        let trigger_entity = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 1.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [1.0, 1.0, 1.0],
                },
                friction: 0.0,
                restitution: 0.0,
                is_sensor: true,
            },
        ));

        // Moving dynamic entity entering the sensor volume
        let dynamic_entity = world.spawn((
            Position {
                x: 0.0,
                y: 0.5,
                z: 0.0,
            },
            Rotation::identity(),
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 1.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Sphere { radius: 0.2 },
                friction: 0.0,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        let mut event_bus = ae_core::events::DynamicEventBus::new();
        physics.step(&mut world, |_| None, 0.016, &mut event_bus);

        assert!(physics.entity_to_body.contains_key(&trigger_entity));
        assert!(physics.entity_to_body.contains_key(&dynamic_entity));
        assert!(event_bus.has_events::<ae_core::events::TriggerEnter>());
    }

    #[test]
    fn test_scaled_ground_collision() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        // Scaled ground platform at y = 0.0, extending from x = -25 to x = +25
        let _ground = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 50.0,
                y: 1.0,
                z: 50.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 1.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // Dynamic cube positioned at x = 10.0 (far outside default 0.5 extent), y = 5.0
        let cube = world.spawn((
            Position {
                x: 10.0,
                y: 5.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 1.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        let mut event_bus = ae_core::events::DynamicEventBus::new();
        // Step physics for 60 steps (1 second)
        for _ in 0..60 {
            physics.step(&mut world, |_| None, 0.016, &mut event_bus);
        }

        let cube_pos = world.get::<&Position>(cube).unwrap();
        // Cube should land on top of the ground platform surface (y >= 0.8) and NOT fall through to y < 0
        assert!(
            cube_pos.y >= 0.8,
            "Cube should land on scaled ground platform, but got y = {}",
            cube_pos.y
        );
    }

    #[test]
    fn test_trigger_toggle_reactivity() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        let entity = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 1.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // Initial sync -> non-sensor
        physics.sync_ecs_to_physics(&mut world, |_| None);
        let body_handle = physics.entity_to_body[&entity];
        let body = physics.rigid_body_set.get(body_handle).unwrap();
        let col = physics.collider_set.get(body.colliders()[0]).unwrap();
        assert!(
            !col.is_sensor(),
            "Collider should initially be solid (not a sensor)"
        );

        // Toggle to sensor = true in ECS
        if let Ok(mut c) = world.get::<&mut Collider>(entity) {
            c.is_sensor = true;
        }

        // Sync again -> Rapier collider must immediately update to is_sensor: true
        physics.sync_ecs_to_physics(&mut world, |_| None);
        let body = physics.rigid_body_set.get(body_handle).unwrap();
        let col = physics.collider_set.get(body.colliders()[0]).unwrap();
        assert!(
            col.is_sensor(),
            "Collider should update to sensor after ECS toggle"
        );

        // Toggle back to sensor = false in ECS
        if let Ok(mut c) = world.get::<&mut Collider>(entity) {
            c.is_sensor = false;
        }

        // Sync again -> Rapier collider must immediately update to is_sensor: false
        physics.sync_ecs_to_physics(&mut world, |_| None);
        let body = physics.rigid_body_set.get(body_handle).unwrap();
        let col = physics.collider_set.get(body.colliders()[0]).unwrap();
        assert!(
            !col.is_sensor(),
            "Collider should update back to solid after ECS toggle"
        );
    }

    #[test]
    fn test_physics_raycast() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        let target_entity = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 10.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 1.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        let mut event_bus = ae_core::events::DynamicEventBus::new();
        physics.step(&mut world, |_| None, 0.016, &mut event_bus);

        let hit = physics.raycast(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 20.0);
        assert!(
            hit.is_some(),
            "Raycast should hit the target entity at z = 10.0"
        );
        let hit = hit.unwrap();
        assert_eq!(hit.entity, target_entity);
        assert!(
            (hit.distance - 9.5).abs() < 0.1,
            "Raycast distance should be approx 9.5 units"
        );
    }

    #[test]
    fn test_character_controller_movement() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        // Create ground plane at y = 0
        let _ground = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 20.0,
                y: 1.0,
                z: 20.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 1.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // Create player entity with CharacterController at y = 2.0
        let player = world.spawn((
            Position {
                x: 0.0,
                y: 2.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RigidBody {
                body_type: RigidBodyType::Kinematic,
                mass: 1.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.5,
                    radius: 0.4,
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
            CharacterController::default(),
        ));

        let mut event_bus = ae_core::events::DynamicEventBus::new();
        physics.step(&mut world, |_| None, 0.016, &mut event_bus);

        // Move character downward
        let is_grounded =
            physics.move_character(&mut world, player, Vec3::new(0.0, -2.0, 0.0), 0.016);
        assert!(
            is_grounded,
            "Character should be grounded after moving down onto ground plane"
        );
    }

    #[test]
    fn test_character_controller_jump() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        let _ground = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 20.0,
                y: 1.0,
                z: 20.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 1.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [10.0, 0.5, 10.0],
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        let player = world.spawn((
            Position {
                x: 0.0,
                y: 2.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RigidBody {
                body_type: RigidBodyType::Kinematic,
                mass: 1.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.5,
                    radius: 0.4,
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
            CharacterController::default(),
        ));

        let mut event_bus = ae_core::events::DynamicEventBus::new();
        physics.step(&mut world, |_| None, 0.016, &mut event_bus);

        // Ground the character first
        let is_grounded =
            physics.move_character(&mut world, player, Vec3::new(0.0, -2.0, 0.0), 0.016);
        assert!(
            is_grounded,
            "Character should be grounded on top of ground box"
        );

        let initial_y = world.get::<&Position>(player).unwrap().y;

        // Perform multi-frame jump simulation
        let dt = 0.016f32;
        let mut vert_vel = 9.0f32;

        if let Ok(mut vel) = world.get::<&mut Velocity>(player) {
            vel.y = vert_vel;
        }

        for _step in 0..10 {
            if let Ok(vel) = world.get::<&Velocity>(player) {
                vert_vel = vel.y;
            }
            vert_vel -= 20.0 * dt;
            if let Ok(mut vel) = world.get::<&mut Velocity>(player) {
                vel.y = vert_vel;
            }

            let is_grounded_during_jump =
                physics.move_character(&mut world, player, Vec3::new(0.0, vert_vel * dt, 0.0), dt);
            assert!(
                !is_grounded_during_jump,
                "Character must not be grounded while ascending"
            );

            physics.step(&mut world, |_| None, dt, &mut event_bus);

            // Verify Velocity was not wiped to 0 by sync_physics_to_ecs
            let vel_after_step = world.get::<&Velocity>(player).unwrap().y;
            assert!(
                (vel_after_step - vert_vel).abs() < 1e-3,
                "ECS Velocity must be preserved across physics steps for Kinematic bodies"
            );
        }

        let final_y = world.get::<&Position>(player).unwrap().y;
        assert!(
            final_y > initial_y + 1.0,
            "Character should jump over 1.0m high in 10 frames, actual height gained: {}",
            final_y - initial_y
        );
    }

    #[test]
    fn test_character_controller_never_falls_through_ground() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        // Static ground box: center at y=0, half_extents=[10, 0.5, 10], top face at y=0.5
        let _ground = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 1.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [10.0, 0.5, 10.0],
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // KCC Player entity: capsule height=1.8 (half_height=0.5, radius=0.4), spawned at y=2.0 in air
        let player = world.spawn((
            Position {
                x: 0.0,
                y: 2.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            CharacterController::default(),
        ));

        let mut event_bus = ae_core::events::DynamicEventBus::new();
        let dt = 0.016f32;

        for step_idx in 0..60 {
            let is_grounded = world
                .get::<&CharacterController>(player)
                .map(|c| c.is_grounded)
                .unwrap_or(false);
            let mut vert_vel = world.get::<&Velocity>(player).map(|v| v.y).unwrap_or(0.0);

            if is_grounded {
                vert_vel = 0.0;
            } else {
                if vert_vel.abs() < 1e-3 {
                    vert_vel = -3.0;
                }
                vert_vel -= 20.0 * dt;
            }

            if let Ok(mut vel) = world.get::<&mut Velocity>(player) {
                vel.y = vert_vel;
            }

            let _g =
                physics.move_character(&mut world, player, Vec3::new(0.0, vert_vel * dt, 0.0), dt);
            physics.step(&mut world, |_| None, dt, &mut event_bus);

            let pos_y = world.get::<&Position>(player).unwrap().y;
            assert!(
                pos_y >= 1.38,
                "Frame {}: Character position y={} fell below ground surface (min expected 1.38)",
                step_idx,
                pos_y
            );
        }
    }

    #[test]
    fn test_character_controller_wall_collision_and_step_climb() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();
        let mut event_bus = ae_core::events::DynamicEventBus::new();
        let dt = 0.016f32;

        // Ground plane at y = 0
        let _ground = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 1.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [20.0, 0.5, 20.0],
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // Tall wall at z = 5.0 (height 2.0m, half_extents [5.0, 1.0, 0.5], extends from z = 4.5 to z = 5.5)
        let _wall = world.spawn((
            Position {
                x: 0.0,
                y: 1.5,
                z: 5.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 1.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [5.0, 1.0, 0.5],
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // Player starting at z = 0.0, resting on ground (y = 1.4)
        let player = world.spawn((
            Position {
                x: 0.0,
                y: 1.4,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            CharacterController::default(),
        ));

        physics.step(&mut world, |_| None, dt, &mut event_bus);
        let _ = physics.move_character(&mut world, player, Vec3::new(0.0, -0.5, 0.0), dt);

        // Walk character towards tall wall at z = 5.0 over 100 frames
        for _ in 0..100 {
            physics.move_character(&mut world, player, Vec3::new(0.0, 0.0, 0.1), dt);
            physics.step(&mut world, |_| None, dt, &mut event_bus);
        }

        let pos_z = world.get::<&Position>(player).unwrap().z;
        // Player radius is 0.4, wall front face is at z = 4.5. Player z must stop at ~4.1 (z <= 4.15) and NEVER enter wall (z > 4.5)
        assert!(
            pos_z <= 4.15,
            "Player z position {} penetrated inside tall wall at z=4.5!",
            pos_z
        );
    }

    #[test]
    fn test_dynamic_object_lands_on_fallback_and_scaled_ground() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        // 1. Spawn a ground plane with Shape::Cube and RigidBody static component
        let ground = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 50.0,
                y: 1.0,
                z: 50.0,
            },
            ae_core::ecs::Shape::Cube,
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 1.0,
                gravity_scale: 1.0,
            },
        ));

        // 2. Spawn a dynamic cube far from center at x = 15.0, y = 5.0
        let cube1 = world.spawn((
            Position {
                x: 15.0,
                y: 5.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 1.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        let mut event_bus = ae_core::events::DynamicEventBus::new();
        for _ in 0..120 {
            physics.step(&mut world, |_| None, 0.016, &mut event_bus);
        }

        let c1_y = world.get::<&Position>(cube1).unwrap().y;
        assert!(
            c1_y >= 0.8,
            "Dynamic cube at x=15 should land on 50m ground plane, but got y = {}",
            c1_y
        );

        // 3. Now scale ground further to 100m in editor
        if let Ok(mut scale) = world.get::<&mut Scale>(ground) {
            scale.x = 100.0;
            scale.z = 100.0;
        }
        let _ = world.insert_one(ground, ae_core::ecs::TransformDirty);

        // 4. Spawn another dynamic cube at x = 35.0, y = 5.0
        let cube2 = world.spawn((
            Position {
                x: 35.0,
                y: 5.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 1.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        for _ in 0..120 {
            physics.step(&mut world, |_| None, 0.016, &mut event_bus);
        }

        let c2_y = world.get::<&Position>(cube2).unwrap().y;
        assert!(
            c2_y >= 0.8,
            "Dynamic cube at x=35 should land on updated 100m ground plane, but got y = {}",
            c2_y
        );
    }

    #[test]
    fn test_inspector_created_static_ground_and_dynamic_cube() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();
        let mut event_bus = ae_core::events::DynamicEventBus::new();

        // 1. Spawn ground object as created in Inspector UI
        let _ground = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 20.0,
                y: 1.0,
                z: 20.0,
            },
            ae_core::ecs::Shape::Cube,
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 1.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // 2. Sync in Edit mode first
        physics.sync_ecs_to_physics(&mut world, |_| None);

        // 3. Spawn dynamic cube as created in Inspector UI
        let cube = world.spawn((
            Position {
                x: 0.0,
                y: 5.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            ae_core::ecs::Shape::Cube,
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 1.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // 4. Sync in Edit mode
        physics.sync_ecs_to_physics(&mut world, |_| None);

        // 5. Toggle Play mode -> reset_simulation_poses
        physics.reset_simulation_poses(&mut world);

        // 6. Step physics simulation for 120 frames (2 seconds)
        for _ in 0..120 {
            physics.step(&mut world, |_| None, 0.016, &mut event_bus);
        }

        let cube_y = world.get::<&Position>(cube).unwrap().y;
        assert!(
            cube_y >= 0.8,
            "Dynamic cube should land on static ground plane at y >= 0.8, but got y = {}",
            cube_y
        );
    }

    #[test]
    fn test_character_controller_presence_does_not_affect_dynamic_cubes() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();
        let mut event_bus = ae_core::events::DynamicEventBus::new();

        // Ground
        let _ground = world.spawn((
            Position {
                x: 0.0,
                y: -0.5,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 50.0,
                y: 1.0,
                z: 50.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 0.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // Dynamic Cube
        let cube = world.spawn((
            Position {
                x: 0.0,
                y: 5.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 1.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // Character Controller Player
        let player = world.spawn((
            Position {
                x: 5.0,
                y: 1.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            CharacterController {
                height: 1.8,
                radius: 0.4,
                max_slope_climb_angle: 45.0,
                step_height: 0.3,
                is_grounded: false,
            },
        ));

        physics.sync_ecs_to_physics(&mut world, |_| None);
        physics.reset_simulation_poses(&mut world);

        for _ in 0..120 {
            physics.move_character(&mut world, player, Vec3::new(0.1, -0.5, 0.0), 0.016);
            physics.step(&mut world, |_| None, 0.016, &mut event_bus);
        }

        let cube_y = world.get::<&Position>(cube).unwrap().y;
        assert!(
            cube_y >= 0.0,
            "Dynamic cube must remain resting on ground even when CharacterController is moving, got y = {}",
            cube_y
        );
    }

    #[test]
    fn test_character_controller_falls_through_trigger_sensor_ground() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        // Sensor Ground (Is Trigger = true)
        let _sensor_ground = world.spawn((
            Position {
                x: 0.0,
                y: -0.5,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 50.0,
                y: 1.0,
                z: 50.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 0.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: true,
            },
        ));

        // Character Controller Player
        let player = world.spawn((
            Position {
                x: 0.0,
                y: 5.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            CharacterController {
                height: 1.8,
                radius: 0.4,
                max_slope_climb_angle: 45.0,
                step_height: 0.3,
                is_grounded: false,
            },
        ));

        physics.sync_ecs_to_physics(&mut world, |_| None);
        physics.reset_simulation_poses(&mut world);

        for _ in 0..60 {
            physics.move_character(&mut world, player, Vec3::new(0.0, -0.5, 0.0), 0.016);
        }

        let player_y = world.get::<&Position>(player).unwrap().y;
        assert!(
            player_y < 0.0,
            "CharacterController must fall through trigger sensor ground (is_sensor=true), got y = {}",
            player_y
        );
    }

    /// Tests that setting is_sensor=true on a CharacterController's OWN collider puts the character in ghost mode, passing through solid walls.
    #[test]
    fn test_character_controller_own_trigger_sensor_pass_through() {
        let mut physics = PhysicsWorld::new();
        let mut world = hecs::World::new();

        // Solid static wall at x = 2.0
        world.spawn((
            Position {
                x: 2.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 5.0,
                z: 5.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 0.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 2.5, 2.5],
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // Player with CharacterController AND Collider with is_sensor=true (ghost character)
        let player = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            CharacterController::default(),
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.5,
                    radius: 0.3,
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: true, // Own collider is a trigger / ghost sensor!
            },
        ));

        physics.sync_ecs_to_physics(&mut world, |_| None);
        physics.reset_simulation_poses(&mut world);

        // Verify that Rapier collider was built with is_sensor = true
        let body_h = physics.entity_to_body.get(&player).unwrap();
        let body = physics.rigid_body_set.get(*body_h).unwrap();
        let col_h = body.colliders().first().unwrap();
        let col = physics.collider_set.get(*col_h).unwrap();
        assert!(
            col.is_sensor(),
            "Character's own Rapier collider must have is_sensor() == true"
        );

        // Move character forward through the solid wall at x = 2.0
        for _ in 0..60 {
            physics.move_character(&mut world, player, Vec3::new(0.1, 0.0, 0.0), 0.016);
        }

        let player_x = world.get::<&Position>(player).unwrap().x;
        assert!(
            player_x > 2.0,
            "Ghost CharacterController (is_sensor=true) must pass through solid wall, got x = {}",
            player_x
        );
    }

    #[test]
    fn test_character_controller_depenetration_direction_verification() {
        let mut world = World::new();
        let mut physics = PhysicsWorld::new();

        // Spawn a static wall at x = 2.0 (width 1.0m, extends x = 1.5..2.5)
        let _wall = world.spawn((
            Position {
                x: 2.0,
                y: 1.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 1.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 2.0, 5.0],
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        // Spawn character deliberately penetrating the wall (center at x = 1.4, radius 0.4 -> right edge at x = 1.8, inside wall x=1.5..2.5)
        let player = world.spawn((
            Position {
                x: 1.4,
                y: 1.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RigidBody {
                body_type: RigidBodyType::Kinematic,
                mass: 1.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.5,
                    radius: 0.4,
                },
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },
            CharacterController::default(),
        ));

        let mut event_bus = ae_core::events::DynamicEventBus::new();
        physics.step(&mut world, |_| None, 0.016, &mut event_bus);

        // Perform zero-displacement move_character to trigger contact depenetration
        physics.move_character(&mut world, player, Vec3::ZERO, 0.016);

        let ejected_x = world.get::<&Position>(player).unwrap().x;
        // Wall front face is at x = 1.5. Player radius is 0.4.
        // Depenetration must eject character to the LEFT (x <= 1.10), NOT deeper right (x > 1.4).
        assert!(
            ejected_x <= 1.10,
            "Depenetration direction failed! Expected character to be ejected left (x <= 1.10), got x = {}",
            ejected_x
        );
    }
}