// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

/// AE Physics — RigidBody and Gravity simulation unit tests.

#[cfg(test)]
mod tests {
    use super::super::super::*;
    use ae_core::ecs::{
        Collider, ColliderShape, Position, RigidBody, RigidBodyType, Rotation, Scale, Velocity,
    };
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
    fn test_entities_without_physics_are_pass_through() {
        let mut physics = PhysicsWorld::new();
        let mut world = World::new();

        // Spawn a purely visual Shape::Cube mesh entity without RigidBody, Collider, or CharacterController
        let visual_cube = world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 5.0,
                y: 5.0,
                z: 5.0,
            },
            ae_core::ecs::Shape::Cube,
        ));

        let mut event_bus = ae_core::events::DynamicEventBus::new();
        physics.step(&mut world, |_| None, 0.016, &mut event_bus);

        // Verify that visual_cube is NOT added to Rapier simulation physics bodies
        assert!(
            !physics.entity_to_body.contains_key(&visual_cube),
            "Purely visual entity without RigidBody/Collider must NOT create a Rapier physics body"
        );
    }
}