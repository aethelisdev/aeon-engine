// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Generic ECS Component and Entity Processor Handlers.
//!
//! Provides dynamic component addition, removal, and mutation through `ComponentRegistry`.

use super::UiContext;

/// Handles dynamically adding a component to an entity using the built-in catalog or `ComponentRegistry`.
pub fn handle_add_component(ctx: &mut UiContext, entity: ae_ecs::Entity, type_name: &str) {
    if let Some(entry) =
        crate::ui::iris_bridge::inspector::add_menu::find_builtin_component_entry(type_name)
    {
        (entry.spawn_default)(ctx.world, entity);
        log::info!(
            "➕ Added built-in component '{}' to entity {:?}",
            type_name,
            entity
        );
        return;
    }

    if let Some(handler) = ae_plugin_api::engine_component_registry().get_by_name(type_name) {
        if let Err(e) = handler.add_default(ctx.world, entity) {
            log::error!(
                "Failed to add dynamic component '{}' to entity {:?}: {}",
                type_name,
                entity,
                e
            );
        } else {
            let _ = ctx.world.insert_one(entity, ae_plugin_api::TransformDirty);
            log::info!(
                "➕ Added dynamic component '{}' to entity {:?}",
                type_name,
                entity
            );
        }
    } else {
        log::warn!(
            "Attempted to add unregistered component '{}' to entity {:?}",
            type_name,
            entity
        );
    }
}

/// Handles dynamically removing a component from an entity using the built-in catalog or `ComponentRegistry`.
pub fn handle_remove_component(ctx: &mut UiContext, entity: ae_ecs::Entity, type_name: &str) {
    if let Some(entry) =
        crate::ui::iris_bridge::inspector::add_menu::find_builtin_component_entry(type_name)
    {
        if (entry.remove_component)(ctx.world, entity) {
            log::info!(
                "🗑 Removed built-in component '{}' from entity {:?}",
                type_name,
                entity
            );
        }
        return;
    }

    if let Some(handler) = ae_plugin_api::engine_component_registry().get_by_name(type_name) {
        if handler.remove_component(ctx.world, entity) {
            let _ = ctx.world.insert_one(entity, ae_plugin_api::TransformDirty);
            log::info!(
                "🗑 Removed dynamic component '{}' from entity {:?}",
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
    entity: ae_ecs::Entity,
    type_name: &str,
    data: &[u8],
) {
    match type_name {
        "AudioSource" => {
            if let Ok(comp) = postcard::from_bytes::<ae_audio::AudioSource>(data) {
                let _ = ctx.world.insert_one(entity, comp);
                let _ = ctx.world.insert_one(entity, ae_plugin_api::TransformDirty);
            }
            return;
        }
        "AudioListener" => {
            if let Ok(comp) = postcard::from_bytes::<ae_audio::AudioListener>(data) {
                let _ = ctx.world.insert_one(entity, comp);
                let _ = ctx.world.insert_one(entity, ae_plugin_api::TransformDirty);
            }
            return;
        }
        "AnimationPlayer" => {
            if let Ok(comp) = postcard::from_bytes::<ae_animation::AnimationPlayer>(data) {
                let _ = ctx.world.insert_one(entity, comp);
                let _ = ctx.world.insert_one(entity, ae_plugin_api::TransformDirty);
            }
            return;
        }
        _ => {}
    }

    let registry = ae_plugin_api::engine_component_registry();
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
                && let Ok(ae_plugin_api::Collider {
                    shape:
                        ae_plugin_api::ColliderShape::Capsule {
                            half_height,
                            radius,
                            center_y,
                        },
                    ..
                }) = ctx.world.get_copied::<ae_plugin_api::Collider>(entity)
                && let Ok(ctrl) = ctx
                    .world
                    .get_mut::<ae_plugin_api::CharacterController>(entity)
            {
                ctrl.height = (half_height + radius) * 2.0;
                ctrl.radius = radius;
                ctrl.center_y = center_y;
            }
            let _ = ctx.world.insert_one(entity, ae_plugin_api::TransformDirty);
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
    entity: ae_ecs::Entity,
    pos: ae_plugin_api::Position,
) {
    if let Ok(existing) = ctx.world.get_mut::<ae_plugin_api::Position>(entity) {
        *existing = pos;
    }
    let _ = ctx.world.insert_one(entity, ae_plugin_api::TransformDirty);
    if let Ok(vel) = ctx.world.get_mut::<ae_plugin_api::Velocity>(entity) {
        vel.x = 0.0;
        vel.y = 0.0;
        vel.z = 0.0;
    }
}

/// Handles modifying rotation component of an entity.
pub fn handle_modify_rotation(
    ctx: &mut UiContext,
    entity: ae_ecs::Entity,
    rot: ae_plugin_api::Rotation,
) {
    if let Ok(existing) = ctx.world.get_mut::<ae_plugin_api::Rotation>(entity) {
        *existing = rot;
    }
    let _ = ctx.world.insert_one(entity, ae_plugin_api::TransformDirty);
    if let Ok(vel) = ctx.world.get_mut::<ae_plugin_api::Velocity>(entity) {
        vel.x = 0.0;
        vel.y = 0.0;
        vel.z = 0.0;
    }
}

/// Handles modifying scale component of an entity.
pub fn handle_modify_scale(
    ctx: &mut UiContext,
    entity: ae_ecs::Entity,
    mut scale: ae_plugin_api::Scale,
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

    if let Ok(existing) = ctx.world.get_mut::<ae_plugin_api::Scale>(entity) {
        *existing = scale;
    }
    let _ = ctx.world.insert_one(entity, ae_plugin_api::TransformDirty);
    if let Ok(vel) = ctx.world.get_mut::<ae_plugin_api::Velocity>(entity) {
        vel.x = 0.0;
        vel.y = 0.0;
        vel.z = 0.0;
    }
}

/// Handles modifying name component of an entity.
pub fn handle_modify_name(ctx: &mut UiContext, entity: ae_ecs::Entity, new_name: String) {
    if let Ok(existing) = ctx.world.get_mut::<ae_plugin_api::Name>(entity) {
        existing.0 = new_name;
    }
}

/// Handles modifying color component of an entity.
pub fn handle_modify_color(
    ctx: &mut UiContext,
    entity: ae_ecs::Entity,
    color: ae_plugin_api::Color,
) {
    if let Ok(existing) = ctx.world.get_mut::<ae_plugin_api::Color>(entity) {
        *existing = color;
    } else {
        let _ = ctx.world.insert_one(entity, color);
    }
}

/// Handles modifying light component of an entity.
pub fn handle_modify_light_color(ctx: &mut UiContext, entity: ae_ecs::Entity, color: [f32; 3]) {
    if let Ok(existing) = ctx.world.get_mut::<ae_plugin_api::Light>(entity) {
        existing.color = color;
    } else {
        let _ = ctx.world.insert_one(
            entity,
            ae_plugin_api::Light {
                position: [0.0, 0.0, 0.0],
                color,
            },
        );
    }
    let _ = ctx.world.insert_one(entity, ae_plugin_api::TransformDirty);
}

/// Handles loading a texture file from disk and assigning a `SpriteId` component to an entity.
pub fn handle_assign_texture(ctx: &mut UiContext, entity: ae_ecs::Entity, path: String) {
    let handle = ctx.render_state.load_texture(ctx.asset_manager, &path);
    let _ = ctx
        .world
        .insert_one(entity, ae_plugin_api::SpriteId(handle));
    log::info!("🖼️ Assigned texture '{}' to entity {:?}", path, entity);
}

/// Handles removing `SpriteId` texture component from an entity.
pub fn handle_remove_texture(ctx: &mut UiContext, entity: ae_ecs::Entity) {
    let _ = ctx.world.remove_one::<ae_plugin_api::SpriteId>(entity);
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
pub fn handle_toggle_visibility(ctx: &mut UiContext, entity: ae_ecs::Entity) {
    if ctx.world.has::<ae_plugin_api::Hidden>(entity) {
        let _ = ctx.world.remove_one::<ae_plugin_api::Hidden>(entity);
        log::info!("👁️ Entity {:?} made VISIBLE", entity);
    } else {
        let _ = ctx.world.insert_one(entity, ae_plugin_api::Hidden);
        log::info!("🚫 Entity {:?} HIDDEN", entity);
    }
    let _ = ctx.world.insert_one(entity, ae_plugin_api::TransformDirty);
}

/// Handles modifying LOD thresholds of an entity.
pub fn handle_modify_lod_thresholds(ctx: &mut UiContext, entity: ae_ecs::Entity, t1: f32, t2: f32) {
    if let Ok(lod) = ctx.world.get_mut::<ae_plugin_api::LodGroup>(entity) {
        lod.threshold_1 = t1;
        lod.threshold_2 = t2;
    }
}

/// Handles modifying LOD model handle of an entity.
pub fn handle_modify_lod_model(
    ctx: &mut UiContext,
    entity: ae_ecs::Entity,
    slot: u8,
    handle_opt: Option<ae_renderer::asset::AssetHandle>,
) {
    if let Ok(lod) = ctx.world.get_mut::<ae_plugin_api::LodGroup>(entity) {
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