// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Generic ECS Component and Entity Processor Handlers.
//!
//! Provides dynamic component addition, removal, and mutation through `ComponentRegistry`.
//!

use super::UiContext;

/// Handles dynamically adding a component to an entity using `ComponentRegistry` default constructor.
pub fn handle_add_component(ctx: &mut UiContext, entity: hecs::Entity, type_name: &str) {
    match type_name {
        "AudioSource" => {
            let _ = ctx
                .world
                .insert_one(entity, ae_audio::AudioSource::default());
            let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            log::info!("➕ Added component 'AudioSource' to entity {:?}", entity);
            return;
        }
        "AudioListener" => {
            let _ = ctx.world.insert_one(entity, ae_audio::AudioListener);
            let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            log::info!("➕ Added component 'AudioListener' to entity {:?}", entity);
            return;
        }
        "AnimationPlayer" => {
            let _ = ctx
                .world
                .insert_one(entity, ae_animation::AnimationPlayer::default());
            let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            log::info!(
                "➕ Added component 'AnimationPlayer' to entity {:?}",
                entity
            );
            return;
        }
        "Collider" => {
            let _ = ctx
                .world
                .insert_one(entity, ae_core::ecs::Collider::default());
            if ctx.world.get::<&ae_core::ecs::RigidBody>(entity).is_err() {
                let _ = ctx
                    .world
                    .insert_one(entity, ae_core::ecs::RigidBody::default());
            }
            let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            log::info!(
                "➕ Added component 'Collider' & 'RigidBody' to entity {:?}",
                entity
            );
            return;
        }
        "RigidBody" => {
            let _ = ctx
                .world
                .insert_one(entity, ae_core::ecs::RigidBody::default());
            if ctx.world.get::<&ae_core::ecs::Collider>(entity).is_err() {
                let _ = ctx
                    .world
                    .insert_one(entity, ae_core::ecs::Collider::default());
            }
            let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            log::info!(
                "➕ Added component 'RigidBody' & 'Collider' to entity {:?}",
                entity
            );
            return;
        }
        _ => {}
    }

    let registry = ae_core::registry::ComponentRegistry::global();
    if let Some(handler) = registry.get_by_name(type_name) {
        if let Err(e) = handler.add_default(ctx.world, entity) {
            log::error!(
                "Failed to add component '{}' to entity {:?}: {}",
                type_name,
                entity,
                e
            );
        } else {
            let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            log::info!("➕ Added component '{}' to entity {:?}", type_name, entity);
        }
    } else {
        log::warn!(
            "Attempted to add unregistered component '{}' to entity {:?}",
            type_name,
            entity
        );
    }
}

/// Handles dynamically removing a component from an entity using `ComponentRegistry`.
pub fn handle_remove_component(ctx: &mut UiContext, entity: hecs::Entity, type_name: &str) {
    match type_name {
        "AudioSource" => {
            let _ = ctx.world.remove_one::<ae_audio::AudioSource>(entity);
            let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            log::info!("🗑 Removed component 'AudioSource' from entity {:?}", entity);
            return;
        }
        "AudioListener" => {
            let _ = ctx.world.remove_one::<ae_audio::AudioListener>(entity);
            let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            log::info!(
                "🗑 Removed component 'AudioListener' from entity {:?}",
                entity
            );
            return;
        }
        "AnimationPlayer" => {
            let _ = ctx
                .world
                .remove_one::<ae_animation::AnimationPlayer>(entity);
            let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            log::info!(
                "🗑 Removed component 'AnimationPlayer' from entity {:?}",
                entity
            );
            return;
        }
        _ => {}
    }

    let registry = ae_core::registry::ComponentRegistry::global();
    if let Some(handler) = registry.get_by_name(type_name) {
        if handler.remove_component(ctx.world, entity) {
            let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            log::info!(
                "🗑 Removed component '{}' from entity {:?}",
                type_name,
                entity
            );
        }
    } else {
        log::warn!(
            "Attempted to remove unregistered component '{}' from entity {:?}",
            type_name,
            entity
        );
    }
}

/// Handles dynamically modifying a component on an entity using `ComponentRegistry`.
pub fn handle_modify_component(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    type_name: &str,
    data: &[u8],
) {
    match type_name {
        "AudioSource" => {
            if let Ok(comp) = serde_json::from_slice::<ae_audio::AudioSource>(data) {
                let _ = ctx.world.insert_one(entity, comp);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            return;
        }
        "AudioListener" => {
            if let Ok(comp) = serde_json::from_slice::<ae_audio::AudioListener>(data) {
                let _ = ctx.world.insert_one(entity, comp);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            return;
        }
        "AnimationPlayer" => {
            if let Ok(comp) = serde_json::from_slice::<ae_animation::AnimationPlayer>(data) {
                let _ = ctx.world.insert_one(entity, comp);
                let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
            }
            return;
        }
        _ => {}
    }

    let registry = ae_core::registry::ComponentRegistry::global();
    if let Some(handler) = registry.get_by_name(type_name) {
        if let Err(e) = handler.apply(ctx.world, entity, data) {
            log::error!(
                "Failed to apply component '{}' modification on entity {:?}: {}",
                type_name,
                entity,
                e
            );
        } else {
            // Synchronize CharacterController capsule geometry when Collider shape is modified
            if type_name == "Collider"
                && let Ok(collider) = ctx.world.get::<&ae_core::ecs::Collider>(entity)
                && let ae_core::ecs::ColliderShape::Capsule {
                    half_height,
                    radius,
                    center_y,
                } = collider.shape
                && let Ok(mut ctrl) = ctx
                    .world
                    .get::<&mut ae_core::ecs::CharacterController>(entity)
            {
                ctrl.height = (half_height + radius) * 2.0;
                ctrl.radius = radius;
                ctrl.center_y = center_y;
            }
            let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
        }
    } else {
        log::warn!(
            "Attempted to modify unregistered component '{}' on entity {:?}",
            type_name,
            entity
        );
    }
}

/// Handles modifying position component of an entity.
pub fn handle_modify_position(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    pos: ae_core::ecs::Position,
) {
    if let Ok(mut existing) = ctx.world.get::<&mut ae_core::ecs::Position>(entity) {
        *existing = pos;
    }
    let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
}

/// Handles modifying rotation component of an entity.
pub fn handle_modify_rotation(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    rot: ae_core::ecs::Rotation,
) {
    if let Ok(mut existing) = ctx.world.get::<&mut ae_core::ecs::Rotation>(entity) {
        *existing = rot;
    }
    let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
}

/// Handles modifying scale component of an entity.
pub fn handle_modify_scale(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    mut scale: ae_core::ecs::Scale,
) {
    let min = 1e-4;
    if scale.x.abs() < min {
        scale.x = f32::copysign(min, scale.x);
    }
    if scale.y.abs() < min {
        scale.y = f32::copysign(min, scale.y);
    }
    if scale.z.abs() < min {
        scale.z = f32::copysign(min, scale.z);
    }

    if let Ok(mut existing) = ctx.world.get::<&mut ae_core::ecs::Scale>(entity) {
        *existing = scale;
    }
    let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
}

/// Handles modifying name component of an entity.
pub fn handle_modify_name(ctx: &mut UiContext, entity: hecs::Entity, new_name: String) {
    if let Ok(mut existing) = ctx.world.get::<&mut ae_core::ecs::Name>(entity) {
        existing.0 = new_name;
    }
}

/// Handles modifying color component of an entity.
pub fn handle_modify_color(ctx: &mut UiContext, entity: hecs::Entity, color: ae_core::ecs::Color) {
    if let Ok(mut existing) = ctx.world.get::<&mut ae_core::ecs::Color>(entity) {
        *existing = color;
    } else {
        let _ = ctx.world.insert_one(entity, color);
    }
}

/// Handles modifying light component of an entity.
pub fn handle_modify_light_color(ctx: &mut UiContext, entity: hecs::Entity, color: [f32; 3]) {
    if let Ok(mut existing) = ctx.world.get::<&mut ae_core::ecs::Light>(entity) {
        existing.color = color;
    } else {
        let _ = ctx.world.insert_one(
            entity,
            ae_core::ecs::Light {
                position: [0.0, 0.0, 0.0],
                color,
            },
        );
    }
    let _ = ctx.world.insert_one(entity, ae_core::ecs::TransformDirty);
}

/// Handles loading a texture file from disk and assigning a `SpriteId` component to an entity.
pub fn handle_assign_texture(ctx: &mut UiContext, entity: hecs::Entity, path: String) {
    let handle = ctx.render_state.load_texture(ctx.asset_manager, &path);
    let _ = ctx.world.insert_one(entity, ae_core::ecs::SpriteId(handle));
    log::info!("🖼️ Assigned texture '{}' to entity {:?}", path, entity);
}

/// Handles removing `SpriteId` texture component from an entity.
pub fn handle_remove_texture(ctx: &mut UiContext, entity: hecs::Entity) {
    let _ = ctx.world.remove_one::<ae_core::ecs::SpriteId>(entity);
    log::info!("🖼️ Removed SpriteId texture from entity {:?}", entity);
}

/// Handles setting the alpha mode of a specific 3D model submesh.
pub fn handle_set_model_submesh_alpha_mode(
    ctx: &mut UiContext,
    model_handle: ae_renderer::asset::AssetHandle,
    submesh_index: usize,
    mode: ae_renderer::render::types::SubmeshAlphaMode,
) {
    if let Some(model) = ctx.asset_manager.models.get_mut(model_handle)
        && let Some(submesh) = model.submeshes.get_mut(submesh_index)
    {
        submesh.alpha_mode = mode;
        log::info!(
            "🎨 Set model {:?} submesh #{} alpha mode to {:?}",
            model_handle,
            submesh_index,
            mode
        );
    }
}

/// Handles assigning a custom texture to a specific 3D model submesh slot.
pub fn handle_set_model_submesh_texture(
    ctx: &mut UiContext,
    model_handle: ae_renderer::asset::AssetHandle,
    submesh_index: usize,
    path: String,
) {
    let texture_handle = ctx.render_state.load_texture(ctx.asset_manager, &path);
    if let Some(model) = ctx.asset_manager.models.get_mut(model_handle)
        && let Some(submesh) = model.submeshes.get_mut(submesh_index)
    {
        let new_tex_idx = model.embedded_textures.len();
        model.embedded_textures.push(texture_handle);
        submesh.texture_index = Some(new_tex_idx);
        log::info!(
            "🖼️ Set model {:?} submesh #{} texture to '{}'",
            model_handle,
            submesh_index,
            path
        );
    }
}

/// Handles toggling the `Hidden` component on an entity (Show / Hide in Viewport).
pub fn handle_toggle_visibility(ctx: &mut UiContext, entity: hecs::Entity) {
    if ctx.world.get::<&ae_core::ecs::Hidden>(entity).is_ok() {
        let _ = ctx.world.remove_one::<ae_core::ecs::Hidden>(entity);
        log::info!("👁️ Entity {:?} made VISIBLE", entity);
    } else {
        let _ = ctx.world.insert_one(entity, ae_core::ecs::Hidden);
        log::info!("🚫 Entity {:?} HIDDEN", entity);
    }
}

/// Handles modifying LOD thresholds of an entity.
pub fn handle_modify_lod_thresholds(ctx: &mut UiContext, entity: hecs::Entity, t1: f32, t2: f32) {
    if let Ok(mut lod) = ctx.world.get::<&mut ae_core::ecs::LodGroup>(entity) {
        lod.threshold_1 = t1;
        lod.threshold_2 = t2;
    }
}

/// Handles modifying LOD model handle of an entity.
pub fn handle_modify_lod_model(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    slot: u8,
    handle_opt: Option<ae_renderer::asset::AssetHandle>,
) {
    if let Ok(mut lod) = ctx.world.get::<&mut ae_core::ecs::LodGroup>(entity) {
        match slot {
            0 => {
                if let Some(h) = handle_opt {
                    lod.lod_0 = h;
                }
            }
            1 => {
                lod.lod_1 = handle_opt;
            }
            2 => {
                lod.lod_2 = handle_opt;
            }
            _ => {}
        }
    }
}