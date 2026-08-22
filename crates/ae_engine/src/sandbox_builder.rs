// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interactive Phase 1 Gameplay Test Sandbox generator.
//!
//! Spawns a comprehensive, fully interactive playground combining:
//! - Player character with CharacterController & raycast shooting
//! - Destructible targets with health, hit flash, and damage telemetry
//! - Proximity trigger zones & automatic sliding doors
//! - Rotating collectible orbs
//! - Moving waypoint platforms
//! - Dynamic physics bouncing hazard cubes
//!

use ae_core::ecs::{
    CharacterAction, CharacterController, Collider, ColliderShape, Color, DestructibleTarget,
    MovingPlatform, Name, PlayerTag, Position, RigidBody, RigidBodyType, Rotation, Rotator, Scale,
    Shape, TriggerZone, Velocity,
};
use hecs::World;

/// Builds the Phase 1 gameplay test sandbox into the target ECS world.
pub fn spawn_phase_1_test_sandbox(world: &mut World) {
    world.clear();

    // 1. Static Ground Platform (50m x 1m x 50m)
    world.spawn((
        Name("Static Ground Plane".to_string()),
        Position::new(0.0, 0.0, 0.0),
        Rotation::identity(),
        Scale::new(50.0, 1.0, 50.0),
        Shape::Cube,
        Color::dark_gray(),
        RigidBody {
            body_type: RigidBodyType::Static,
            mass: 0.0,
            gravity_scale: 0.0,
        },
        Collider {
            shape: ColliderShape::Box {
                half_extents: [25.0, 0.5, 25.0],
            },
            friction: 0.7,
            restitution: 0.0,
            is_sensor: false,
        },
    ));

    // 2. Player Character with KCC & CharacterAction (Raycast Weapon Shooting)
    world.spawn((
        Name("Player Character".to_string()),
        Position::new(0.0, 2.0, 8.0),
        Rotation::identity(),
        Scale::one(),
        Shape::Capsule,
        Color::soft_blue(),
        CharacterController::default(),
        PlayerTag,
        Velocity::zero(),
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.5,
                radius: 0.4,
                center_y: 0.0,
            },
            friction: 0.7,
            restitution: 0.0,
            is_sensor: false,
        },
        CharacterAction::new(),
    ));

    // 3. Destructible Target Dummies (Shooting Range)
    let target_positions = [
        ("Target Alpha", [-4.0, 1.5, -12.0]),
        ("Target Beta", [0.0, 1.5, -15.0]),
        ("Target Gamma", [4.0, 1.5, -12.0]),
    ];

    for (name, pos) in target_positions {
        world.spawn((
            Name(name.to_string()),
            Position::new(pos[0], pos[1], pos[2]),
            Rotation::identity(),
            Scale::new(1.2, 1.2, 1.2),
            Shape::Sphere,
            Color::red(),
            RigidBody {
                body_type: RigidBodyType::Static,
                mass: 0.0,
                gravity_scale: 0.0,
            },
            Collider {
                shape: ColliderShape::Sphere { radius: 0.6 },
                friction: 0.5,
                restitution: 0.2,
                is_sensor: false,
            },
            DestructibleTarget::new(100.0),
        ));
    }

    // 4. Proximity Sensor & Sliding Door
    // Sensor Zone (Green Pad with accurate trigger volume)
    world.spawn((
        Name("Proximity Sensor Zone".to_string()),
        Position::new(10.0, 0.55, -4.0),
        Rotation::identity(),
        Scale::new(4.0, 0.1, 4.0),
        Shape::Cube,
        Color::green(),
        Collider {
            shape: ColliderShape::Box {
                half_extents: [0.5, 10.0, 0.5],
            },
            friction: 0.0,
            restitution: 0.0,
            is_sensor: true,
        },
        TriggerZone::new(),
    ));

    // Sliding Door (Elevates when player steps into proximity sensor)
    world.spawn((
        Name("Sliding Door".to_string()),
        Position::new(10.0, 2.0, -7.0),
        Rotation::identity(),
        Scale::new(3.5, 3.0, 0.4),
        Shape::Cube,
        Color::soft_blue(),
        RigidBody {
            body_type: RigidBodyType::Kinematic,
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
        TriggerZone {
            is_triggered: false,
            speed: 5.0,
            axis: [0.0, 1.0, 0.0],
            original_position: [10.0, 2.0, -7.0],
            target_position: [10.0, 6.0, -7.0],
            ping_pong_forward: true,
        },
    ));

    // 5. Rotating Collectible Crystals (Distinct Asymmetric Shapes & Vibrant Colors)
    let crystal_configs = [
        (
            "Rotating Crystal Alpha (X-Pitch)",
            [-8.0, 1.5, 2.0],
            [1.0, 0.0, 0.0],
            3.5,
            Color::new(1.0, 0.85, 0.1, 1.0), // Gold
        ),
        (
            "Rotating Crystal Beta (Y-Yaw)",
            [-8.0, 1.5, -3.0],
            [0.0, 1.0, 0.0],
            4.0,
            Color::new(0.1, 0.85, 1.0, 1.0), // Neon Cyan
        ),
        (
            "Rotating Crystal Gamma (Z-Roll)",
            [-8.0, 1.5, -8.0],
            [0.0, 0.0, 1.0],
            3.0,
            Color::new(1.0, 0.2, 0.85, 1.0), // Hot Magenta
        ),
    ];

    for (name, pos, axis, speed, color) in crystal_configs {
        world.spawn((
            Name(name.to_string()),
            Position::new(pos[0], pos[1], pos[2]),
            Rotation::identity(),
            Scale::new(1.4, 0.35, 0.7),
            Shape::Cube,
            color,
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.0,
                restitution: 0.0,
                is_sensor: true,
            },
            Rotator::new(speed, axis),
        ));
    }

    // 6. Moving Waypoint Elevator (Vertical Passenger Elevator)
    world.spawn((
        Name("Moving Waypoint Elevator".to_string()),
        Position::new(-15.0, 0.8, 4.0),
        Rotation::identity(),
        Scale::new(4.0, 0.4, 4.0),
        Shape::Cube,
        Color::new(0.8, 0.4, 0.9, 1.0),
        RigidBody {
            body_type: RigidBodyType::Kinematic,
            mass: 0.0,
            gravity_scale: 0.0,
        },
        Collider {
            shape: ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.9,
            restitution: 0.0,
            is_sensor: false,
        },
        MovingPlatform::new(2.5, [-15.0, 0.8, 4.0], [-15.0, 6.5, 4.0]),
    ));

    // 7. Dynamic Bouncing Hazard Cubes
    let hazard_positions = [
        ("Bouncing Cube Alpha", [5.0, 5.0, 3.0]),
        ("Bouncing Cube Beta", [7.0, 8.0, 4.0]),
    ];

    for (name, pos) in hazard_positions {
        world.spawn((
            Name(name.to_string()),
            Position::new(pos[0], pos[1], pos[2]),
            Rotation::identity(),
            Scale::new(1.0, 1.0, 1.0),
            Shape::Cube,
            Color::new(1.0, 0.5, 0.1, 1.0),
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 2.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.4,
                restitution: 0.85,
                is_sensor: false,
            },
        ));
    }

    log::info!("🎮 [SandboxBuilder] Phase 1 Interactive Test Sandbox successfully spawned!");
}