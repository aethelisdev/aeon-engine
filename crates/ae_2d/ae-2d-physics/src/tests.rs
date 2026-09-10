// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use glam::Vec2;

use crate::{BodyType2D, Collider2D, Physics2DWorld, RigidBody2D};

#[test]
fn test_physics_world_gravity_fall() {
    let mut world = Physics2DWorld::new(Vec2::new(0.0, -9.81));
    let mut body = RigidBody2D::new(BodyType2D::Dynamic);

    let handle = world.spawn_rigid_body(Vec2::new(0.0, 10.0), 0.0, &mut body);
    let mut collider = Collider2D::ball(0.5);
    world.attach_collider(handle, &mut collider);

    // Step world 10 frames
    for _ in 0..10 {
        world.step(0.016);
    }

    let rb = world
        .rigid_body_set
        .get(handle)
        .expect("Rigid body must exist");
    assert!(
        rb.translation().y < 10.0,
        "Dynamic body should have fallen under gravity"
    );
}

#[test]
fn test_physics_world_raycast() {
    let mut world = Physics2DWorld::new(Vec2::ZERO);
    let mut body = RigidBody2D::new(BodyType2D::Fixed);
    let body_handle = world.spawn_rigid_body(Vec2::new(5.0, 0.0), 0.0, &mut body);

    let mut collider = Collider2D::cuboid(1.0, 1.0);
    world.attach_collider(body_handle, &mut collider);

    // Step once to ensure broad-phase BVH is synchronized
    world.step(0.016);

    // Cast ray from origin towards the box
    let hit = world.cast_ray(Vec2::ZERO, Vec2::new(1.0, 0.0), 10.0, true);
    assert!(hit.is_some(), "Ray should intersect the box collider");

    let (_, hit_point, hit_normal, toi) = hit.unwrap();
    assert_eq!(hit_point.x, 4.0); // Box starts at 5.0 - 1.0 = 4.0
    assert_eq!(hit_normal, Vec2::new(-1.0, 0.0));
    assert_eq!(toi, 4.0);
}