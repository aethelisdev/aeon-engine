// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! ECS component, spatial transform, material submesh, and LOD action mutation handlers.

use crate::engine::AeEngine;
use ae_ecs::{Component, Entity, World};
use ae_plugin_api::{
    CharacterController, Collider, ColliderShape, Color, Hidden, Light, LodGroup, Name, Position,
    Rotation, Scale, SpriteId, TransformDirty, Velocity,
};
use serde::de::DeserializeOwned;

fn reset_entity_velocity(world: &mut World, entity: Entity) {
    if let Ok(vel) = world.get_mut::<Velocity>(entity) {
        vel.x = 0.0;
        vel.y = 0.0;
        vel.z = 0.0;
    }
}

fn apply_postcard_component<T: Component + DeserializeOwned>(
    world: &mut World,
    entity: Entity,
    data: &[u8],
) {
    if let Ok(comp) = postcard::from_bytes::<T>(data) {
        let _ = world.insert_one(entity, comp);
        let _ = world.insert_one(entity, TransformDirty);
    }
}

fn sync_collider_capsule_to_controller(world: &mut World, entity: Entity) {
    if let Ok(Collider {
        shape:
            ColliderShape::Capsule {
                half_height,
                radius,
                center_y,
            },
        ..
    }) = world.get_copied::<Collider>(entity)
        && let Ok(ctrl) = world.get_mut::<CharacterController>(entity)
    {
        ctrl.height = (half_height + radius) * 2.0;
        ctrl.radius = radius;
        ctrl.center_y = center_y;
    }
}

/// Attaches a default built-in or registry-backed ECS component by type name.
pub fn handle_add_component(world: &mut World, entity: Entity, type_name: &str) {
    if let Some(entry) =
        ae_editor_ui::ui::iris_bridge::inspector::add_menu::find_builtin_component_entry(type_name)
    {
        (entry.spawn_default)(world, entity);
        log::info!(
            "➕ Added built-in component '{}' to entity {:?}",
            type_name,
            entity
        );
        return;
    }

    if let Some(handler) = ae_plugin_api::engine_component_registry().get_by_name(type_name) {
        if let Err(e) = handler.add_default(world, entity) {
            log::error!(
                "Failed to add dynamic component '{}' to entity {:?}: {}",
                type_name,
                entity,
                e
            );
        } else {
            let _ = world.insert_one(entity, TransformDirty);
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

/// Removes a built-in or registry-backed ECS component from `entity` by type name.
pub fn handle_remove_component(world: &mut World, entity: Entity, type_name: &str) {
    if let Some(entry) =
        ae_editor_ui::ui::iris_bridge::inspector::add_menu::find_builtin_component_entry(type_name)
    {
        if (entry.remove_component)(world, entity) {
            log::info!(
                "🗑 Removed built-in component '{}' from entity {:?}",
                type_name,
                entity
            );
        }
        return;
    }

    if let Some(handler) = ae_plugin_api::engine_component_registry().get_by_name(type_name) {
        if handler.remove_component(world, entity) {
            let _ = world.insert_one(entity, TransformDirty);
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

/// Applies serialized `postcard` component data onto `entity` and marks transform dirty.
pub fn handle_modify_component(world: &mut World, entity: Entity, type_name: &str, data: &[u8]) {
    match type_name {
        "AudioSource" => {
            apply_postcard_component::<ae_audio::AudioSource>(world, entity, data);
            return;
        }
        "AudioListener" => {
            apply_postcard_component::<ae_audio::AudioListener>(world, entity, data);
            return;
        }
        "AnimationPlayer" => {
            apply_postcard_component::<ae_animation::AnimationPlayer>(world, entity, data);
            return;
        }
        _ => {}
    }

    let registry = ae_plugin_api::engine_component_registry();
    if let Some(handler) = registry.get_by_name(type_name) {
        if let Err(e) = handler.apply(world, entity, data) {
            log::error!(
                "Failed to apply component '{}' modification on entity {:?}: {}",
                type_name,
                entity,
                e
            );
        } else {
            if type_name == "Collider" {
                sync_collider_capsule_to_controller(world, entity);
            }
            let _ = world.insert_one(entity, TransformDirty);
        }
    } else {
        log::warn!(
            "Attempted to modify unregistered component '{}' on entity {:?}",
            type_name,
            entity
        );
    }
}

/// Modifies entity [`Position`], resets zero-step velocity, and flags [`TransformDirty`].
pub fn handle_modify_position(world: &mut World, entity: Entity, pos: Position) {
    if let Ok(existing) = world.get_mut::<Position>(entity) {
        *existing = pos;
    }
    let _ = world.insert_one(entity, TransformDirty);
    reset_entity_velocity(world, entity);
}

/// Modifies entity [`Rotation`], resets velocity, and flags [`TransformDirty`].
pub fn handle_modify_rotation(world: &mut World, entity: Entity, rot: Rotation) {
    if let Ok(existing) = world.get_mut::<Rotation>(entity) {
        *existing = rot;
    }
    let _ = world.insert_one(entity, TransformDirty);
    reset_entity_velocity(world, entity);
}

/// Modifies entity [`Scale`] with non-degenerate epsilon clamping and flags [`TransformDirty`].
pub fn handle_modify_scale(world: &mut World, entity: Entity, mut scale: Scale) {
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

    if let Ok(existing) = world.get_mut::<Scale>(entity) {
        *existing = scale;
    }
    let _ = world.insert_one(entity, TransformDirty);
    reset_entity_velocity(world, entity);
}

/// Updates an entity's [`Name`] component.
pub fn handle_modify_name(world: &mut World, entity: Entity, new_name: String) {
    if let Ok(existing) = world.get_mut::<Name>(entity) {
        existing.0 = new_name;
    }
}

/// Updates or inserts an entity's [`Color`] component.
pub fn handle_modify_color(world: &mut World, entity: Entity, color: Color) {
    if let Ok(existing) = world.get_mut::<Color>(entity) {
        *existing = color;
    } else {
        let _ = world.insert_one(entity, color);
    }
}

/// Updates or inserts an entity's [`Light`] RGB color and marks [`TransformDirty`].
pub fn handle_modify_light_color(world: &mut World, entity: Entity, color: [f32; 3]) {
    if let Ok(existing) = world.get_mut::<Light>(entity) {
        existing.color = color;
    } else {
        let _ = world.insert_one(
            entity,
            Light {
                position: [0.0, 0.0, 0.0],
                color,
            },
        );
    }
    let _ = world.insert_one(entity, TransformDirty);
}

/// Loads a texture asset from disk and attaches its [`SpriteId`] onto `entity`.
pub fn handle_assign_texture(engine: &mut AeEngine, entity: Entity, path: String) {
    let handle = engine
        .render_state
        .load_texture(&mut engine.asset_manager, &path);
    let _ = engine.world.insert_one(entity, SpriteId(handle));
    log::info!("🖼️ Assigned texture '{}' to entity {:?}", path, entity);
}

/// Removes [`SpriteId`] from `entity`.
pub fn handle_remove_texture(world: &mut World, entity: Entity) {
    let _ = world.remove_one::<SpriteId>(entity);
    log::info!("🖼️ Removed SpriteId texture from entity {:?}", entity);
}

/// Sets the alpha blending mode on a specific model submesh slot.
pub fn handle_set_model_submesh_alpha_mode(
    engine: &mut AeEngine,
    model_handle: ae_renderer::asset::AssetHandle,
    submesh_index: usize,
    mode: ae_mesh::SubmeshAlphaMode,
) {
    if let Some(model) = engine.asset_manager.models.get_mut(model_handle)
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

/// Loads and binds a custom texture override onto a specific model submesh slot.
pub fn handle_set_model_submesh_texture(
    engine: &mut AeEngine,
    model_handle: ae_renderer::asset::AssetHandle,
    submesh_index: usize,
    path: String,
) {
    let texture_handle = engine
        .render_state
        .load_texture(&mut engine.asset_manager, &path);
    if let Some(model) = engine.asset_manager.models.get_mut(model_handle)
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

/// Toggles [`Hidden`] visibility marker on `entity`.
pub fn handle_toggle_visibility(world: &mut World, entity: Entity) {
    if world.has::<Hidden>(entity) {
        let _ = world.remove_one::<Hidden>(entity);
        log::info!("👁️ Entity {:?} made VISIBLE", entity);
    } else {
        let _ = world.insert_one(entity, Hidden);
        log::info!("🚫 Entity {:?} HIDDEN", entity);
    }
    let _ = world.insert_one(entity, TransformDirty);
}

/// Updates distance transition thresholds on an entity's [`LodGroup`].
pub fn handle_modify_lod_thresholds(world: &mut World, entity: Entity, t1: f32, t2: f32) {
    if let Ok(lod) = world.get_mut::<LodGroup>(entity) {
        lod.threshold_1 = t1;
        lod.threshold_2 = t2;
    }
}

/// Updates an assigned LOD mesh handle slot (`0..=2`) on [`LodGroup`].
pub fn handle_modify_lod_model(
    world: &mut World,
    entity: Entity,
    slot: u8,
    handle_opt: Option<ae_renderer::asset::AssetHandle>,
) {
    if let Ok(lod) = world.get_mut::<LodGroup>(entity) {
        match slot {
            0 => {
                if let Some(h) = handle_opt {
                    lod.lod_0 = h;
                }
            }
            1 => lod.lod_1 = handle_opt,
            2 => lod.lod_2 = handle_opt,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modify_component_and_undo_redo_and_scale_clamp() {
        let mut world = World::new();
        let entity = world.spawn((
            Position::new(0.0, 0.0, 0.0),
            Scale::one(),
            Velocity {
                x: 5.0,
                y: 2.0,
                z: 1.0,
            },
        ));

        let mut editor = ae_editor::editor_state::EditorState::default();
        let old_vel = Velocity {
            x: 5.0,
            y: 2.0,
            z: 1.0,
        };
        let new_vel = Velocity {
            x: 10.0,
            y: 20.0,
            z: 30.0,
        };
        let old_data = postcard::to_stdvec(&old_vel).expect("encode old");
        let new_data = postcard::to_stdvec(&new_vel).expect("encode new");

        handle_modify_component(&mut world, entity, "Velocity", &new_data);
        ae_editor::history::push_undo(
            &mut editor,
            ae_editor::undo_redo::Command::Modify(
                entity,
                ae_editor::undo_redo::Property::Component {
                    type_name: "Velocity".to_string(),
                    old_data,
                    new_data,
                },
            ),
        );
        assert_eq!(world.get_copied::<Velocity>(entity).unwrap().x, 10.0);

        ae_editor::history::undo(&mut editor, &mut world);
        assert_eq!(world.get_copied::<Velocity>(entity).unwrap().x, 5.0);

        ae_editor::history::redo(&mut editor, &mut world);
        assert_eq!(world.get_copied::<Velocity>(entity).unwrap().x, 10.0);

        // Negative / near-zero scale boundary clamping also clears velocity
        handle_modify_scale(&mut world, entity, Scale::new(0.0, -0.0, 2.0));
        let s = world.get_copied::<Scale>(entity).unwrap();
        assert!(s.x.abs() >= 1e-4 && s.y.abs() >= 1e-4);
        assert_eq!(world.get_copied::<Velocity>(entity).unwrap().x, 0.0);

        // Unknown component is ignored safely without panicking
        handle_modify_component(&mut world, entity, "NonExistentComponent", &[]);
    }
}