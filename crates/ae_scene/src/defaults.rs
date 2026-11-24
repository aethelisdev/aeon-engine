// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Built-in starter 2D and 3D editor scenes spawned for new empty worlds.

use crate::spawning::spawn_player_sprite_entity;
use ae_ecs::World;
use ae_plugin_api::{
    Collider, ColliderShape, Color, Name, PlayerTag, Position, RigidBody, RigidBodyType, Rotation,
    Scale, Shape, Velocity,
};

/// Spawns the starter 2D editor scene (`Player Sprite` and `Background Tile`) into `world`.
pub fn spawn_default_2d_scene(world: &mut World) {
    spawn_player_sprite_entity(world);
    world.spawn((
        Name("Background Tile".to_string()),
        Position::new(0.0, 0.0, -1.0),
        Rotation::identity(),
        Scale::new(10.0, 6.0, 1.0),
        ae_2d::components::SpriteRenderer {
            tint: [0.1, 0.12, 0.16, 1.0],
            sorting_layer: -1,
            order_in_layer: 0,
            ..Default::default()
        },
    ));
}

/// Spawns the starter 3D editor scene (`Ground Plane`, `Dynamic Cube`, and `Sun` light).
pub fn spawn_default_3d_scene(world: &mut World) {
    world.spawn((
        Name("Ground Plane".to_string()),
        Shape::Cube,
        Position::new(0.0, -0.5, 0.0),
        Rotation::identity(),
        Scale::new(50.0, 1.0, 50.0),
        Color {
            r: 0.2,
            g: 0.25,
            b: 0.3,
            a: 1.0,
        },
        Velocity {
            x: 0.0,
            y: 0.0,
            z: 0.0,
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

    world.spawn((
        PlayerTag,
        Name("Dynamic Cube".to_string()),
        Shape::Cube,
        Position::new(0.0, 5.0, 0.0),
        Rotation::identity(),
        Scale::one(),
        Color::soft_blue(),
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

    world.spawn((
        Name("Sun".to_string()),
        ae_light::Light {
            position: [5.0, 15.0, 5.0],
            color: [1.0, 1.0, 0.9],
        },
        Position::new(5.0, 15.0, 5.0),
        Rotation::identity(),
        Scale::one(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_default_2d_and_3d_scenes() {
        let mut w2d = World::new();
        spawn_default_2d_scene(&mut w2d);
        assert_eq!(w2d.len(), 2);
        assert_eq!(w2d.query::<&PlayerTag>().iter().count(), 1);

        let mut w3d = World::new();
        spawn_default_3d_scene(&mut w3d);
        assert_eq!(w3d.len(), 3);
        assert_eq!(w3d.query::<&PlayerTag>().iter().count(), 1);
        assert_eq!(w3d.query::<&ae_light::Light>().iter().count(), 1);
    }
}