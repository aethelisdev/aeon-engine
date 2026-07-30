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