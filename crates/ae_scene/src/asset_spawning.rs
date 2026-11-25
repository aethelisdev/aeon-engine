// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU asset upload and scene-entity spawning helpers bridging [`ae_mesh`], [`ae_renderer`],
//! and [`crate::spawning`] without editor UI dependencies.

use crate::spawning::{
    ModelSpawnDesc, SpriteSpawnDesc, resolve_asset_display_name, spawn_model_entity,
    spawn_sprite_entity,
};
use ae_ecs::{Entity, World};
use ae_mesh::{ParsedModelData, parse_gltf_file};
use ae_renderer::asset::AssetManager;
use ae_renderer::render::RenderState;
use std::path::Path;

/// Uploads parsed CPU model geometry onto the GPU and spawns a configured ECS model entity at `pos`.
pub fn spawn_parsed_model_at(
    world: &mut World,
    render_state: &RenderState,
    asset_manager: &mut AssetManager,
    parsed_data: ParsedModelData,
    pos: [f32; 3],
    apply_fbx_axis_fix: bool,
) -> Entity {
    let source_override = parsed_data.original_path.clone();
    let final_name = parsed_data.final_name.clone();
    let (model_id, _min, _max) = render_state.upload_model_data(asset_manager, parsed_data);
    spawn_model_entity(
        world,
        asset_manager,
        ModelSpawnDesc {
            model_id,
            name_override: Some(final_name),
            position: pos,
            source_path_override: Some(source_override.as_str()),
            apply_fbx_axis_fix,
        },
    )
}

/// Loads (or reuses cached) GLTF/GLB asset at `path` onto GPU storage and spawns it at `pos`.
///
/// # Errors
/// Returns an error string when GLTF/GLB parsing fails.
pub fn spawn_model_from_path(
    world: &mut World,
    render_state: &RenderState,
    asset_manager: &mut AssetManager,
    path: &Path,
    pos: [f32; 3],
) -> Result<Entity, String> {
    if let Ok(canonical) = std::fs::canonicalize(path)
        && let Some(&handle) = asset_manager.model_path_map.get(&canonical)
    {
        return Ok(spawn_model_entity(
            world,
            asset_manager,
            ModelSpawnDesc {
                model_id: handle,
                name_override: None,
                position: pos,
                source_path_override: None,
                apply_fbx_axis_fix: false,
            },
        ));
    }

    let path_str = path.to_string_lossy();
    let final_name = resolve_asset_display_name(path, "Model");
    match parse_gltf_file(path_str.as_ref(), final_name) {
        Ok(parsed_data) => Ok(spawn_parsed_model_at(
            world,
            render_state,
            asset_manager,
            parsed_data,
            pos,
            false,
        )),
        Err(e) => {
            log::error!("Failed to load 3D model from '{}': {}", path_str, e);
            Err(e)
        }
    }
}

/// Loads a texture image at `path` into GPU storage and spawns a sprite entity at `pos`.
pub fn spawn_sprite_from_path(
    world: &mut World,
    render_state: &RenderState,
    asset_manager: &mut AssetManager,
    path: &Path,
    pos: [f32; 3],
) -> Entity {
    let path_str = path.to_string_lossy();
    let texture_handle = render_state.load_texture(asset_manager, path_str.as_ref());
    spawn_sprite_entity(
        world,
        asset_manager,
        SpriteSpawnDesc {
            texture_id: texture_handle,
            name_override: None,
            position: pos,
        },
    )
}