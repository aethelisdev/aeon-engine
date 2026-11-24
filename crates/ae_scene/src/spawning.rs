// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Pure ECS entity spawning helpers for models, sprites, primitives, and 2D/3D sandboxes.
//!
//! Operates strictly on [`ae_ecs::World`] and [`ae_renderer::asset::AssetManager`] without
//! UI context, file dialogs, or editor undo/selection side effects.

use ae_animation::AnimationPlayer;
use ae_ecs::{Entity, World};
use ae_mesh::{compute_model_auto_scale, is_fbx_model_path};
use ae_plugin_api::{
    BoundingBox, CharacterAction, CharacterController, Collider, ColliderShape, Color,
    DestructibleTarget, LodGroup, ModelId, MovingPlatform, Name, PhysicsMaterial, PlayerTag,
    Position, RigidBody, RigidBodyType, Rotation, Rotator, Scale, Shape, SpriteId, SurfaceType,
    TriggerZone, Velocity,
};
use ae_renderer::asset::{AssetHandle, AssetManager};
use std::path::Path;

/// Descriptor configuring how a loaded 3D model entity is spawned into [`World`].
pub struct ModelSpawnDesc<'a> {
    /// Cached GPU model handle registered inside [`AssetManager`].
    pub model_id: AssetHandle,
    /// Optional explicit entity display name (defaults to derived stem when `None`).
    pub name_override: Option<String>,
    /// Target world-space spawn coordinates.
    pub position: [f32; 3],
    /// Optional override asset path checked for axis conventions such as `.fbx`.
    pub source_path_override: Option<&'a str>,
    /// When `true`, applies the standard -90 deg X axis rotation for `.fbx` sources.
    pub apply_fbx_axis_fix: bool,
}

/// Descriptor configuring how a 2D sprite entity is spawned into [`World`].
pub struct SpriteSpawnDesc {
    /// Cached texture handle registered inside [`AssetManager`].
    pub texture_id: AssetHandle,
    /// Optional explicit entity display name (defaults to texture file stem when `None`).
    pub name_override: Option<String>,
    /// Target world-space spawn coordinates.
    pub position: [f32; 3],
}

/// Derives a clean human-readable asset display name from `path`, substituting the parent
/// directory name when a model file uses a generic `"scene"` filename stem.
#[must_use]
pub fn resolve_asset_display_name(path: &Path, fallback: &str) -> String {
    let raw_stem = path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .filter(|s| !s.is_empty());
    let Some(stem) = raw_stem else {
        return fallback.to_string();
    };
    if stem.eq_ignore_ascii_case("scene") {
        path.parent()
            .and_then(|p| p.file_name())
            .map(|f| f.to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| stem.into_owned())
    } else {
        stem.into_owned()
    }
}

/// Spawns a 3D model entity into `world` with AABB bounds, auto-computed uniform scale,
/// optional default texture, and skeletal animation playback when present.
pub fn spawn_model_entity(
    world: &mut World,
    asset_manager: &AssetManager,
    desc: ModelSpawnDesc<'_>,
) -> Entity {
    let (resolved_name, bbox, default_tex, axis_path) =
        if let Some(m) = asset_manager.models.get(desc.model_id) {
            let derived = resolve_asset_display_name(Path::new(&m.source_path), "Model");
            (
                derived,
                BoundingBox {
                    min: m.min,
                    max: m.max,
                },
                m.default_texture,
                m.source_path.as_str(),
            )
        } else {
            (
                "Model".to_string(),
                BoundingBox {
                    min: [-0.5; 3],
                    max: [0.5; 3],
                },
                None,
                "",
            )
        };

    let final_name = desc.name_override.unwrap_or(resolved_name);
    let effective_path = desc.source_path_override.unwrap_or(axis_path);
    let auto_scale = compute_model_auto_scale(bbox.min, bbox.max);

    let rotation = if desc.apply_fbx_axis_fix && is_fbx_model_path(effective_path) {
        Rotation {
            x: -std::f32::consts::FRAC_1_SQRT_2,
            y: 0.0,
            z: 0.0,
            w: std::f32::consts::FRAC_1_SQRT_2,
        }
    } else {
        Rotation::identity()
    };

    let new_entity = world.spawn((
        Name(final_name),
        ae_plugin_api::ModelId(desc.model_id),
        Position {
            x: desc.position[0],
            y: desc.position[1],
            z: desc.position[2],
        },
        rotation,
        Scale {
            x: auto_scale,
            y: auto_scale,
            z: auto_scale,
        },
        bbox,
        Velocity {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    ));

    if let Some(tex) = default_tex {
        let _ = world.insert_one(new_entity, SpriteId(tex));
    }

    if let Some(model_asset) = asset_manager.models.get(desc.model_id)
        && let Some(ref skel) = model_asset.skeleton
    {
        let _ = world.insert_one(new_entity, skel.clone());
        if !model_asset.animations.is_empty() {
            let mut player = AnimationPlayer::default();
            player.play(model_asset.animations[0].clone());
            let _ = world.insert_one(new_entity, player);
        }
    }

    new_entity
}

/// Spawns a textured sprite entity into `world` at the coordinates specified in `desc`.
pub fn spawn_sprite_entity(
    world: &mut World,
    asset_manager: &AssetManager,
    desc: SpriteSpawnDesc,
) -> Entity {
    let resolved_name = asset_manager.textures.get(desc.texture_id).map_or_else(
        || "Sprite".to_string(),
        |t| resolve_asset_display_name(Path::new(&t.source_path), "Sprite"),
    );
    let final_name = desc.name_override.unwrap_or(resolved_name);

    world.spawn((
        Name(final_name),
        SpriteId(desc.texture_id),
        Position {
            x: desc.position[0],
            y: desc.position[1],
            z: desc.position[2],
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
    ))
}

/// Spawns a dynamic 3D primitive [`Shape`] entity with default physics body and box collider.
pub fn spawn_shape_entity(world: &mut World, shape: Shape) -> Entity {
    let final_name = match shape {
        Shape::Cube => "New Cube",
        Shape::Triangle => "New Triangle",
        Shape::Sphere => "New Sphere",
        Shape::Cylinder => "New Cylinder",
        Shape::Capsule => "New Capsule",
        Shape::Torus => "New Torus",
    }
    .to_string();

    world.spawn((
        Name(final_name),
        shape,
        Position {
            x: 0.0,
            y: 3.0,
            z: 0.0,
        },
        Rotation::identity(),
        Scale {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
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
        ae_plugin_api::TransformDirty,
    ))
}

/// Spawns a default untextured 2D Sprite entity at the world origin.
pub fn spawn_default_sprite_entity(world: &mut World) -> Entity {
    world.spawn((
        Name("New Sprite".to_string()),
        Position::new(0.0, 0.0, 0.0),
        Rotation::identity(),
        Scale::new(2.0, 2.0, 1.0),
        ae_2d::components::SpriteRenderer {
            tint: [1.0, 1.0, 1.0, 1.0],
            sorting_layer: 0,
            order_in_layer: 0,
            ..Default::default()
        },
    ))
}

/// Spawns a default 2D Player Sprite entity tagged with [`PlayerTag`] at the world origin.
pub fn spawn_player_sprite_entity(world: &mut World) -> Entity {
    world.spawn((
        Name("Player Sprite".to_string()),
        PlayerTag,
        Position::new(0.0, 0.0, 0.0),
        Rotation::identity(),
        Scale::new(2.0, 2.0, 1.0),
        ae_2d::components::SpriteRenderer {
            tint: [0.18, 0.80, 0.44, 1.0],
            sorting_layer: 0,
            order_in_layer: 0,
            ..Default::default()
        },
    ))
}

/// Spawns an empty spatial 2D entity (`Name`, `Position`, `Rotation`, `Scale`) at the origin.
pub fn spawn_empty_2d_entity(world: &mut World) -> Entity {
    world.spawn((
        Name("Empty 2D Object".to_string()),
        Position::new(0.0, 0.0, 0.0),
        Rotation::identity(),
        Scale::one(),
    ))
}

/// Batch-spawns `count` pseudo-random colored cube entities into `world` for stress testing.
///
/// Uses a fixed deterministic xorshift32 seed and returns the exact count inserted.
pub fn spawn_stress_test_cubes(world: &mut World, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    let mut seed = 12345u32;
    let mut next_rand = |min: f32, max: f32| {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        let fraction = (seed & 0x00FF_FFFF) as f32 / 16_777_216.0;
        min + fraction * (max - min)
    };

    let bounds = if count >= 1_000_000 {
        10_000.0
    } else if count >= 100_000 {
        80.0
    } else {
        40.0
    };

    let mut batch = Vec::with_capacity(count);
    for i in 0..count {
        batch.push((
            Name(format!("Cube_{i}")),
            Position {
                x: next_rand(-bounds, bounds),
                y: next_rand(0.5, bounds / 2.0),
                z: next_rand(-bounds, bounds),
            },
            Rotation::identity(),
            Scale::one(),
            Shape::Cube,
            Color {
                r: next_rand(0.2, 1.0),
                g: next_rand(0.2, 1.0),
                b: next_rand(0.2, 1.0),
                a: 1.0,
            },
        ));
    }

    world.spawn_batch(batch).count()
}

/// Clears `world` and populates the interactive Phase 1 gameplay test sandbox, returning the
/// spawned player character entity.
pub fn spawn_phase1_sandbox(world: &mut World) -> Entity {
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
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.7,
            restitution: 0.0,
            is_sensor: false,
        },
        PhysicsMaterial::new(SurfaceType::Stone, 0.7, 0.0),
    ));

    // 2. Player Character with KCC & CharacterAction (Raycast Weapon Shooting)
    let player_ent = world.spawn((
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
        PhysicsMaterial::new(SurfaceType::Flesh, 0.7, 0.0),
        CharacterAction::new(),
    ));

    // 3. Destructible Target Dummies
    for (name, pos) in [
        ("Target Alpha", [-4.0, 1.5, -12.0]),
        ("Target Beta", [0.0, 1.5, -15.0]),
        ("Target Gamma", [4.0, 1.5, -12.0]),
    ] {
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
            PhysicsMaterial::new(SurfaceType::Flesh, 0.5, 0.2),
            DestructibleTarget::new(100.0),
        ));
    }

    // 4. Proximity Sensor & Sliding Door
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
        PhysicsMaterial::new(SurfaceType::Metal, 0.7, 0.0),
        TriggerZone {
            is_triggered: false,
            speed: 5.0,
            axis: [0.0, 1.0, 0.0],
            original_position: [10.0, 2.0, -7.0],
            target_position: [10.0, 6.0, -7.0],
            ping_pong_forward: true,
        },
    ));

    // 5. Rotating Collectible Crystals
    for (name, pos, axis, speed, color) in [
        (
            "Rotating Crystal Alpha (X-Pitch)",
            [-8.0, 1.5, 2.0],
            [1.0, 0.0, 0.0],
            3.5,
            Color::new(1.0, 0.85, 0.1, 1.0),
        ),
        (
            "Rotating Crystal Beta (Y-Yaw)",
            [-8.0, 1.5, -3.0],
            [0.0, 1.0, 0.0],
            4.0,
            Color::new(0.1, 0.85, 1.0, 1.0),
        ),
        (
            "Rotating Crystal Gamma (Z-Roll)",
            [-8.0, 1.5, -8.0],
            [0.0, 0.0, 1.0],
            3.0,
            Color::new(1.0, 0.2, 0.85, 1.0),
        ),
    ] {
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

    // 6. Moving Waypoint Elevator
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
        PhysicsMaterial::new(SurfaceType::Metal, 0.9, 0.0),
        MovingPlatform::new(2.5, [-15.0, 0.8, 4.0], [-15.0, 6.5, 4.0]),
    ));

    // 7. Dynamic Bouncing Hazard Cubes
    for (name, pos, surface) in [
        ("Bouncing Cube Alpha", [5.0, 5.0, 3.0], SurfaceType::Rubber),
        ("Bouncing Cube Beta", [7.0, 8.0, 4.0], SurfaceType::Wood),
    ] {
        let mat = PhysicsMaterial::from_preset(surface);
        world.spawn((
            Name(name.to_string()),
            Position::new(pos[0], pos[1], pos[2]),
            Rotation::identity(),
            Scale::one(),
            Shape::Cube,
            Color::new(1.0, 0.5, 0.1, 1.0),
            Velocity::zero(),
            RigidBody {
                body_type: RigidBodyType::Dynamic,
                mass: 2.0,
                gravity_scale: 1.0,
            },
            Collider {
                shape: ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: mat.friction,
                restitution: mat.restitution,
                is_sensor: false,
            },
            mat,
        ));
    }

    player_ent
}

/// Collects all referenced model and texture [`AssetHandle`] sets across live [`ModelId`],
/// [`SpriteId`], and [`LodGroup`] components in `world`.
#[must_use]
pub fn collect_used_asset_handles(
    world: &World,
) -> (
    std::collections::HashSet<AssetHandle>,
    std::collections::HashSet<AssetHandle>,
) {
    let mut used_models = std::collections::HashSet::new();
    let mut used_textures = std::collections::HashSet::new();

    for (_ent, model_id) in world.query::<(Entity, &ModelId)>().iter() {
        used_models.insert(model_id.0);
    }
    for (_ent, sprite_id) in world.query::<(Entity, &SpriteId)>().iter() {
        used_textures.insert(sprite_id.0);
    }
    for (_ent, lod) in world.query::<(Entity, &LodGroup)>().iter() {
        used_models.insert(lod.lod_0);
        if let Some(h) = lod.lod_1 {
            used_models.insert(h);
        }
        if let Some(h) = lod.lod_2 {
            used_models.insert(h);
        }
    }

    (used_models, used_textures)
}

/// Sweeps [`AssetManager`] slots and removes models and textures that are no longer referenced
/// by any live [`ModelId`], [`SpriteId`], or [`LodGroup`] component in `world`.
///
/// Returns `(removed_models, removed_textures)`.
pub fn garbage_collect_unused_assets(
    world: &World,
    asset_manager: &mut AssetManager,
) -> (usize, usize) {
    let (used_models, used_textures) = collect_used_asset_handles(world);

    let unused_models: Vec<_> = asset_manager
        .models
        .iter()
        .map(|(handle, _)| handle)
        .filter(|h| !used_models.contains(h))
        .collect();
    let removed_models = unused_models.len();
    for handle in unused_models {
        asset_manager.models.remove(handle);
    }

    let unused_textures: Vec<_> = asset_manager
        .textures
        .iter()
        .map(|(handle, _)| handle)
        .filter(|h| !used_textures.contains(h))
        .collect();
    let removed_textures = unused_textures.len();
    for handle in unused_textures {
        asset_manager.textures.remove(handle);
    }

    (removed_models, removed_textures)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_renderer::asset::AssetStorage;

    #[test]
    fn test_collect_used_asset_handles_covers_models_sprites_and_lods() {
        let mut handle_pool = AssetStorage::<()>::new();
        let m_primary = handle_pool.insert(());
        let m_lod0 = handle_pool.insert(());
        let m_lod1 = handle_pool.insert(());
        let m_unused = handle_pool.insert(());
        let tex_live = handle_pool.insert(());

        let mut world = World::new();
        assert_eq!(
            garbage_collect_unused_assets(&world, &mut AssetManager::new()),
            (0, 0)
        );

        world.spawn((ModelId(m_primary), SpriteId(tex_live)));
        world.spawn((LodGroup {
            lod_0: m_lod0,
            lod_1: Some(m_lod1),
            lod_2: None,
            threshold_1: 15.0,
            threshold_2: 45.0,
        },));

        let (used_models, used_textures) = collect_used_asset_handles(&world);
        assert!(used_models.contains(&m_primary));
        assert!(used_models.contains(&m_lod0));
        assert!(used_models.contains(&m_lod1));
        assert!(!used_models.contains(&m_unused));
        assert!(used_textures.contains(&tex_live));
        assert_eq!(used_models.len(), 3);
        assert_eq!(used_textures.len(), 1);
    }

    #[test]
    fn test_resolve_asset_display_name_handles_scene_folder_and_empty_path() {
        assert_eq!(
            resolve_asset_display_name(Path::new("assets/Mech/scene.gltf"), "Model"),
            "Mech"
        );
        assert_eq!(
            resolve_asset_display_name(Path::new("assets/Crate.glb"), "Model"),
            "Crate"
        );
        assert_eq!(
            resolve_asset_display_name(Path::new(""), "Fallback"),
            "Fallback"
        );
    }

    #[test]
    fn test_spawn_sandbox_and_stress_test_zero_boundary() {
        let mut world = World::new();
        assert_eq!(spawn_stress_test_cubes(&mut world, 0), 0);
        assert_eq!(world.len(), 0);

        let player = spawn_phase1_sandbox(&mut world);
        assert!(world.contains(player));
        assert!(world.has::<PlayerTag>(player));
        assert!(world.len() >= 10);
    }
}