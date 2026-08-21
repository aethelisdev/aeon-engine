// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::UiContext;

/// Handles spawning a 3D model asset entity into the ECS world.
pub fn handle_spawn_model(ctx: &mut UiContext, mid: ae_renderer::asset::AssetHandle) {
    let mut base_name = "Model".to_string();
    let mut bbox = ae_core::ecs::BoundingBox {
        min: [-0.5; 3],
        max: [0.5; 3],
    };

    if let Some(m) = ctx.asset_manager.models.get(mid) {
        let path = std::path::Path::new(&m.source_path);
        base_name = path
            .file_stem()
            .unwrap_or(std::ffi::OsStr::new("Model"))
            .to_string_lossy()
            .into_owned();
        bbox = ae_core::ecs::BoundingBox {
            min: m.min,
            max: m.max,
        };
    }

    let new_entity = ctx.world.spawn((
        ae_core::ecs::Name(base_name),
        ae_core::ecs::ModelId(mid),
        ae_core::ecs::Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        ae_core::ecs::Rotation::identity(),
        ae_core::ecs::Scale {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        bbox,
        ae_core::ecs::Velocity {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    ));
    let snap = ae_editor::undo_redo::EntitySnapshot::capture(ctx.world, new_entity);
    ae_editor::history::push_undo(
        ctx.editor,
        ae_editor::undo_redo::Command::Spawn(new_entity, snap),
    );
    ctx.editor.redo_stack.clear();
    ctx.editor.selected_entities.clear();
    ctx.editor.selected_entities_set.clear();
    ctx.editor.selected_entities.push(new_entity);
    ctx.editor.selected_entities_set.insert(new_entity);
    ctx.ui.selected_entity = Some(new_entity);
}

/// Handles spawning a 2D sprite texture asset entity into the ECS world.
pub fn handle_spawn_sprite(ctx: &mut UiContext, tid: ae_renderer::asset::AssetHandle) {
    let mut base_name = "Sprite".to_string();
    if let Some(t) = ctx.asset_manager.textures.get(tid) {
        let path = std::path::Path::new(&t.source_path);
        base_name = path
            .file_stem()
            .unwrap_or(std::ffi::OsStr::new("Sprite"))
            .to_string_lossy()
            .into_owned();
    }

    let new_entity = ctx.world.spawn((
        ae_core::ecs::Name(base_name),
        ae_core::ecs::SpriteId(tid),
        ae_core::ecs::Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        ae_core::ecs::Rotation::identity(),
        ae_core::ecs::Scale {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        ae_core::ecs::Velocity {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    ));
    let snap = ae_editor::undo_redo::EntitySnapshot::capture(ctx.world, new_entity);
    ae_editor::history::push_undo(
        ctx.editor,
        ae_editor::undo_redo::Command::Spawn(new_entity, snap),
    );
    ctx.editor.redo_stack.clear();
    ctx.editor.selected_entities.clear();
    ctx.editor.selected_entities_set.clear();
    ctx.editor.selected_entities.push(new_entity);
    ctx.editor.selected_entities_set.insert(new_entity);
    ctx.ui.selected_entity = Some(new_entity);
}

/// Handles spawning a parametric 3D primitive shape entity.
pub fn handle_spawn_shape(ctx: &mut UiContext, shape: ae_core::ecs::Shape) {
    let final_name = match shape {
        ae_core::ecs::Shape::Cube => "New Cube",
        ae_core::ecs::Shape::Triangle => "New Triangle",
        ae_core::ecs::Shape::Sphere => "New Sphere",
        ae_core::ecs::Shape::Cylinder => "New Cylinder",
        ae_core::ecs::Shape::Capsule => "New Capsule",
        ae_core::ecs::Shape::Torus => "New Torus",
    }
    .to_string();

    let new_entity = ctx.world.spawn((
        ae_core::ecs::Name(final_name),
        shape,
        ae_core::ecs::Position {
            x: 0.0,
            y: 3.0,
            z: 0.0,
        },
        ae_core::ecs::Rotation::identity(),
        ae_core::ecs::Scale {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        ae_core::ecs::Color::soft_blue(),
        ae_core::ecs::Velocity {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        ae_core::ecs::RigidBody {
            body_type: ae_core::ecs::RigidBodyType::Dynamic,
            mass: 1.0,
            gravity_scale: 1.0,
        },
        ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.7,
            restitution: 0.0,
            is_sensor: false,
        },
        ae_core::ecs::TransformDirty,
    ));
    let snap = ae_editor::undo_redo::EntitySnapshot::capture(ctx.world, new_entity);
    ae_editor::history::push_undo(
        ctx.editor,
        ae_editor::undo_redo::Command::Spawn(new_entity, snap),
    );
    ctx.editor.redo_stack.clear();
    ctx.editor.selected_entities.clear();
    ctx.editor.selected_entities_set.clear();
    ctx.editor.selected_entities.push(new_entity);
    ctx.editor.selected_entities_set.insert(new_entity);
    ctx.ui.selected_entity = Some(new_entity);
}

/// Handles selecting an entity in the editor hierarchy or viewport.
pub fn handle_select_entity(ctx: &mut UiContext, entity_opt: Option<hecs::Entity>) {
    ctx.editor.selected_entities.clear();
    ctx.editor.selected_entities_set.clear();
    if let Some(e) = entity_opt {
        ctx.editor.selected_entities.push(e);
        ctx.editor.selected_entities_set.insert(e);
    }
    ctx.ui.selected_entity = entity_opt;
}

/// Handles deleting selected entities from the ECS world and editor state.
pub fn handle_delete_selected(ctx: &mut UiContext) {
    ae_editor::actions::delete_selected(ctx.world, ctx.editor, &mut ctx.ui.selected_entity);
}

/// Handles running the multi-object stress test spawning benchmark.
pub fn handle_stress_test(ctx: &mut UiContext, count: usize) {
    let mut batch = Vec::new();
    let mut seed = 12345u32;
    let mut next_rand = |min: f32, max: f32| {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        let fraction = (seed & 0xFFFFFF) as f32 / 16777216.0;
        min + fraction * (max - min)
    };

    let bounds = if count >= 1000000 {
        10000.0
    } else if count >= 100000 {
        80.0
    } else {
        40.0
    };

    for i in 0..count {
        let name = ae_core::ecs::Name(format!("Cube_{}", i));
        let pos = ae_core::ecs::Position {
            x: next_rand(-bounds, bounds),
            y: next_rand(0.5, bounds / 2.0),
            z: next_rand(-bounds, bounds),
        };
        let rot = ae_core::ecs::Rotation::identity();
        let scale = ae_core::ecs::Scale {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        let shape = ae_core::ecs::Shape::Cube;
        let color = ae_core::ecs::Color {
            r: next_rand(0.2, 1.0),
            g: next_rand(0.2, 1.0),
            b: next_rand(0.2, 1.0),
            a: 1.0,
        };
        batch.push((name, pos, rot, scale, shape, color));
    }

    ctx.world.spawn_batch(batch);
    log::info!("🚀 Spawned stress test batch of {} entities!", count);
}

/// Handles spawning the Phase 1 interactive test sandbox.
pub fn handle_spawn_phase1_test_sandbox(ctx: &mut UiContext) {
    ctx.world.clear();
    ctx.editor.selected_entities.clear();
    ctx.editor.selected_entities_set.clear();
    ctx.ui.selected_entity = None;

    // 1. Static Ground Platform (50m x 1m x 50m)
    ctx.world.spawn((
        ae_core::ecs::Name("Static Ground Plane".to_string()),
        ae_core::ecs::Position::new(0.0, 0.0, 0.0),
        ae_core::ecs::Rotation::identity(),
        ae_core::ecs::Scale::new(50.0, 1.0, 50.0),
        ae_core::ecs::Shape::Cube,
        ae_core::ecs::Color::dark_gray(),
        ae_core::ecs::RigidBody {
            body_type: ae_core::ecs::RigidBodyType::Static,
            mass: 0.0,
            gravity_scale: 0.0,
        },
        ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [25.0, 0.5, 25.0],
            },
            friction: 0.7,
            restitution: 0.0,
            is_sensor: false,
        },
    ));

    // 2. Player Character with KCC & CharacterAction (Raycast Weapon Shooting)
    let player_ent = ctx.world.spawn((
        ae_core::ecs::Name("Player Character".to_string()),
        ae_core::ecs::Position::new(0.0, 2.0, 8.0),
        ae_core::ecs::Rotation::identity(),
        ae_core::ecs::Scale::one(),
        ae_core::ecs::Shape::Capsule,
        ae_core::ecs::Color::soft_blue(),
        ae_core::ecs::CharacterController::default(),
        ae_core::ecs::PlayerTag,
        ae_core::ecs::Velocity::zero(),
        ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Capsule {
                half_height: 0.5,
                radius: 0.4,
                center_y: 0.0,
            },
            friction: 0.7,
            restitution: 0.0,
            is_sensor: false,
        },
        ae_core::ecs::BehaviorComponent::character_action(),
    ));

    // 3. Destructible Target Dummies (Shooting Range)
    let target_positions = [
        ("Target Alpha", [-4.0, 1.5, -12.0]),
        ("Target Beta", [0.0, 1.5, -15.0]),
        ("Target Gamma", [4.0, 1.5, -12.0]),
    ];

    for (name, pos) in target_positions {
        ctx.world.spawn((
            ae_core::ecs::Name(name.to_string()),
            ae_core::ecs::Position::new(pos[0], pos[1], pos[2]),
            ae_core::ecs::Rotation::identity(),
            ae_core::ecs::Scale::new(1.2, 1.2, 1.2),
            ae_core::ecs::Shape::Sphere,
            ae_core::ecs::Color::red(),
            ae_core::ecs::RigidBody {
                body_type: ae_core::ecs::RigidBodyType::Static,
                mass: 0.0,
                gravity_scale: 0.0,
            },
            ae_core::ecs::Collider {
                shape: ae_core::ecs::ColliderShape::Sphere { radius: 0.6 },
                friction: 0.5,
                restitution: 0.2,
                is_sensor: false,
            },
            ae_core::ecs::BehaviorComponent::destructible_target(100.0),
        ));
    }

    // 4. Proximity Sensor & Sliding Door
    ctx.world.spawn((
        ae_core::ecs::Name("Proximity Sensor Zone".to_string()),
        ae_core::ecs::Position::new(10.0, 0.55, -4.0),
        ae_core::ecs::Rotation::identity(),
        ae_core::ecs::Scale::new(4.0, 0.1, 4.0),
        ae_core::ecs::Shape::Cube,
        ae_core::ecs::Color::green(),
        ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 10.0, 0.5],
            },
            friction: 0.0,
            restitution: 0.0,
            is_sensor: true,
        },
    ));

    ctx.world.spawn((
        ae_core::ecs::Name("Sliding Door".to_string()),
        ae_core::ecs::Position::new(10.0, 2.0, -7.0),
        ae_core::ecs::Rotation::identity(),
        ae_core::ecs::Scale::new(3.5, 3.0, 0.4),
        ae_core::ecs::Shape::Cube,
        ae_core::ecs::Color::soft_blue(),
        ae_core::ecs::RigidBody {
            body_type: ae_core::ecs::RigidBodyType::Kinematic,
            mass: 0.0,
            gravity_scale: 0.0,
        },
        ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.7,
            restitution: 0.0,
            is_sensor: false,
        },
        ae_core::ecs::BehaviorComponent {
            behavior_type: ae_core::ecs::BehaviorType::TriggerZone,
            speed: 5.0,
            axis: [0.0, 1.0, 0.0],
            health: 100.0,
            max_health: 100.0,
            is_triggered: false,
            original_position: [10.0, 2.0, -7.0],
            target_position: [10.0, 6.0, -7.0],
            ping_pong_forward: true,
            timer: 0.0,
            hit_flash_timer: 0.0,
        },
    ));

    // 5. Rotating Collectible Crystals (Distinct Asymmetric Shapes & Vibrant Colors)
    let crystal_configs = [
        (
            "Rotating Crystal Alpha (X-Pitch)",
            [-8.0, 1.5, 2.0],
            [1.0, 0.0, 0.0],
            3.5,
            ae_core::ecs::Color::new(1.0, 0.85, 0.1, 1.0), // Gold
        ),
        (
            "Rotating Crystal Beta (Y-Yaw)",
            [-8.0, 1.5, -3.0],
            [0.0, 1.0, 0.0],
            4.0,
            ae_core::ecs::Color::new(0.1, 0.85, 1.0, 1.0), // Neon Cyan
        ),
        (
            "Rotating Crystal Gamma (Z-Roll)",
            [-8.0, 1.5, -8.0],
            [0.0, 0.0, 1.0],
            3.0,
            ae_core::ecs::Color::new(1.0, 0.2, 0.85, 1.0), // Hot Magenta
        ),
    ];

    for (name, pos, axis, speed, color) in crystal_configs {
        ctx.world.spawn((
            ae_core::ecs::Name(name.to_string()),
            ae_core::ecs::Position::new(pos[0], pos[1], pos[2]),
            ae_core::ecs::Rotation::identity(),
            ae_core::ecs::Scale::new(1.4, 0.35, 0.7),
            ae_core::ecs::Shape::Cube,
            color,
            ae_core::ecs::Collider {
                shape: ae_core::ecs::ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.0,
                restitution: 0.0,
                is_sensor: true,
            },
            ae_core::ecs::BehaviorComponent::rotator(speed, axis),
        ));
    }

    // 6. Moving Waypoint Elevator (Vertical Passenger Elevator)
    ctx.world.spawn((
        ae_core::ecs::Name("Moving Waypoint Elevator".to_string()),
        ae_core::ecs::Position::new(-15.0, 0.8, 4.0),
        ae_core::ecs::Rotation::identity(),
        ae_core::ecs::Scale::new(4.0, 0.4, 4.0),
        ae_core::ecs::Shape::Cube,
        ae_core::ecs::Color::new(0.8, 0.4, 0.9, 1.0),
        ae_core::ecs::RigidBody {
            body_type: ae_core::ecs::RigidBodyType::Kinematic,
            mass: 0.0,
            gravity_scale: 0.0,
        },
        ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.9,
            restitution: 0.0,
            is_sensor: false,
        },
        ae_core::ecs::BehaviorComponent::moving_platform(2.5, [-15.0, 0.8, 4.0], [-15.0, 6.5, 4.0]),
    ));

    // 7. Dynamic Bouncing Hazard Cubes
    let hazard_positions = [
        ("Bouncing Cube Alpha", [5.0, 5.0, 3.0]),
        ("Bouncing Cube Beta", [7.0, 8.0, 4.0]),
    ];

    for (name, pos) in hazard_positions {
        ctx.world.spawn((
            ae_core::ecs::Name(name.to_string()),
            ae_core::ecs::Position::new(pos[0], pos[1], pos[2]),
            ae_core::ecs::Rotation::identity(),
            ae_core::ecs::Scale::new(1.0, 1.0, 1.0),
            ae_core::ecs::Shape::Cube,
            ae_core::ecs::Color::new(1.0, 0.5, 0.1, 1.0),
            ae_core::ecs::Velocity::zero(),
            ae_core::ecs::RigidBody {
                body_type: ae_core::ecs::RigidBodyType::Dynamic,
                mass: 2.0,
                gravity_scale: 1.0,
            },
            ae_core::ecs::Collider {
                shape: ae_core::ecs::ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.4,
                restitution: 0.85,
                is_sensor: false,
            },
        ));
    }

    ctx.editor.selected_entities.push(player_ent);
    ctx.editor.selected_entities_set.insert(player_ent);
    ctx.ui.selected_entity = Some(player_ent);

    log::info!("🎮 Spawned Phase 1 Interactive Test Sandbox into editor!");
}