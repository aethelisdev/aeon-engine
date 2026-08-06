// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

/// AE Physics — Collider, Trimesh, Sensor, and Raycast unit tests.

#[cfg(test)]
mod tests {
    use super::super::super::*;
    use ae_core::ecs::{
        AssetHandle, Collider, ColliderShape, ModelId, Position, RigidBody, RigidBodyType,
        Rotation, Scale, Velocity,
    };
    use glam::Vec3;
    use hecs::World;

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
}