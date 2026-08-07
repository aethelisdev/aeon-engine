// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::UiContext;

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
pub fn handle_modify_scale(ctx: &mut UiContext, entity: hecs::Entity, scale: ae_core::ecs::Scale) {
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
    }
}

/// Handles modifying rigid body component of an entity.
pub fn handle_modify_rigid_body(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    rb: ae_core::ecs::RigidBody,
) {
    if let Ok(mut existing) = ctx.world.get::<&mut ae_core::ecs::RigidBody>(entity) {
        *existing = rb;
    } else {
        let _ = ctx.world.insert_one(entity, rb);
    }
}

/// Handles modifying collider component of an entity.
pub fn handle_modify_collider(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    collider: ae_core::ecs::Collider,
) {
    if let Ok(mut existing) = ctx.world.get::<&mut ae_core::ecs::Collider>(entity) {
        *existing = collider;
    } else {
        let _ = ctx.world.insert_one(entity, collider);
    }
}

/// Handles adding a rigid body component to an entity.
pub fn handle_add_rigid_body(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    rb: ae_core::ecs::RigidBody,
) {
    let _ = ctx.world.insert_one(entity, rb);
}

/// Handles removing rigid body component from an entity.
pub fn handle_remove_rigid_body(ctx: &mut UiContext, entity: hecs::Entity) {
    let _ = ctx.world.remove_one::<ae_core::ecs::RigidBody>(entity);
}

/// Handles adding a collider to an entity.
pub fn handle_add_collider(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    collider: ae_core::ecs::Collider,
) {
    let _ = ctx.world.insert_one(entity, collider);
}

/// Handles removing collider component from an entity.
pub fn handle_remove_collider(ctx: &mut UiContext, entity: hecs::Entity) {
    let _ = ctx.world.remove_one::<ae_core::ecs::Collider>(entity);
}

/// Handles adding character controller component to an entity.
pub fn handle_add_character_controller(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    cc: ae_core::ecs::CharacterController,
) {
    let _ = ctx.world.insert_one(entity, cc);
}

/// Handles removing character controller component from an entity.
pub fn handle_remove_character_controller(ctx: &mut UiContext, entity: hecs::Entity) {
    let _ = ctx
        .world
        .remove_one::<ae_core::ecs::CharacterController>(entity);
}

/// Handles modifying character controller component of an entity.
pub fn handle_modify_character_controller(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    cc: ae_core::ecs::CharacterController,
) {
    if let Ok(mut existing) = ctx
        .world
        .get::<&mut ae_core::ecs::CharacterController>(entity)
    {
        *existing = cc;
    } else {
        let _ = ctx.world.insert_one(entity, cc);
    }
}

/// Handles adding LOD group component to an entity.
pub fn handle_add_lod_group(ctx: &mut UiContext, entity: hecs::Entity) {
    let dummy_handle = ae_renderer::asset::AssetHandle::default();
    let lod = ae_core::ecs::LodGroup {
        lod_0: dummy_handle,
        lod_1: None,
        lod_2: None,
        threshold_1: 20.0,
        threshold_2: 50.0,
    };
    let _ = ctx.world.insert_one(entity, lod);
}

/// Handles removing LOD group component from an entity.
pub fn handle_remove_lod_group(ctx: &mut UiContext, entity: hecs::Entity) {
    let _ = ctx.world.remove_one::<ae_core::ecs::LodGroup>(entity);
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

/// Handles adding AudioSource component to an entity.
pub fn handle_add_audio_source(ctx: &mut UiContext, entity: hecs::Entity) {
    let source = ae_audio::AudioSource::default();
    let _ = ctx.world.insert_one(entity, source);
    log::info!("🔊 Added AudioSource to entity {:?}", entity);
}

/// Handles removing AudioSource component from an entity.
pub fn handle_remove_audio_source(ctx: &mut UiContext, entity: hecs::Entity) {
    let _ = ctx.world.remove_one::<ae_audio::AudioSource>(entity);
    log::info!("🔊 Removed AudioSource from entity {:?}", entity);
}

/// Handles modifying AudioSource parameters of an entity.
pub fn handle_modify_audio_source(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    source: ae_audio::AudioSource,
) {
    if let Ok(mut existing) = ctx.world.get::<&mut ae_audio::AudioSource>(entity) {
        *existing = source;
    }
}

/// Handles adding AudioListener component to an entity.
pub fn handle_add_audio_listener(ctx: &mut UiContext, entity: hecs::Entity) {
    let listener = ae_audio::AudioListener;
    let _ = ctx.world.insert_one(entity, listener);
    log::info!("👂 Added AudioListener to entity {:?}", entity);
}

/// Handles removing AudioListener component from an entity.
pub fn handle_remove_audio_listener(ctx: &mut UiContext, entity: hecs::Entity) {
    let _ = ctx.world.remove_one::<ae_audio::AudioListener>(entity);
    log::info!("👂 Removed AudioListener from entity {:?}", entity);
}

/// Handles adding PlayerTag component to an entity.
pub fn handle_add_player_tag(ctx: &mut UiContext, entity: hecs::Entity) {
    let _ = ctx.world.insert_one(entity, ae_core::ecs::PlayerTag);
    log::info!("🎮 Added PlayerTag to entity {:?}", entity);
}

/// Handles removing PlayerTag component from an entity.
pub fn handle_remove_player_tag(ctx: &mut UiContext, entity: hecs::Entity) {
    let _ = ctx.world.remove_one::<ae_core::ecs::PlayerTag>(entity);
    log::info!("🎮 Removed PlayerTag from entity {:?}", entity);
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