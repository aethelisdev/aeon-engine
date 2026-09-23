// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Event Dispatcher
//!
//! Evaluates mouse click coordinates and mouse wheel scroll deltas against the cached
//! MaterialPanelTargets to produce high-level MaterialAction commands.
//!

use super::header::MATERIAL_HEADER_HEIGHT;
use super::types::{
    MATERIAL_TAG_ADD_COLOR, MATERIAL_TAG_ADD_TEXTURE, MATERIAL_TAG_SPRITE_CHANGE,
    MATERIAL_TAG_SPRITE_REMOVE, MaterialAction, MaterialPanelTargets, decode_submesh_alpha_tag,
    decode_submesh_texture_tag,
};

/// Hit-tests semantic tags against interactive buttons in the Material Studio.
pub fn handle_material_click(
    hit_tag: u64,
    selected_entity: Option<hecs::Entity>,
    active_model: Option<ae_renderer::asset::AssetHandle>,
) -> Option<MaterialAction> {
    let ent = selected_entity?;

    match hit_tag {
        MATERIAL_TAG_SPRITE_CHANGE => {
            return Some(MaterialAction::PickAndAssignEntityTexture(ent));
        }
        MATERIAL_TAG_SPRITE_REMOVE => {
            return Some(MaterialAction::RemoveTextureFromEntity(ent));
        }
        MATERIAL_TAG_ADD_TEXTURE => {
            return Some(MaterialAction::PickAndAssignEntityTexture(ent));
        }
        MATERIAL_TAG_ADD_COLOR => {
            return Some(MaterialAction::AddColorComponent(ent));
        }
        _ => {}
    }

    // Submesh alpha mode pill buttons
    if let Some((submesh_idx, mode)) = decode_submesh_alpha_tag(hit_tag) {
        let model_handle = active_model?;
        return Some(MaterialAction::SetModelSubmeshAlphaMode(
            model_handle,
            submesh_idx,
            mode,
        ));
    }

    // Submesh change texture buttons
    if let Some(submesh_idx) = decode_submesh_texture_tag(hit_tag) {
        let model_handle = active_model?;
        return Some(MaterialAction::PickAndSetSubmeshTexture(
            model_handle,
            submesh_idx,
        ));
    }

    None
}

/// Calculates updated vertical scroll offset given a mouse wheel delta and viewport height.
pub fn handle_material_scroll(
    delta_y: f32,
    cur_scroll_y: f32,
    targets: &MaterialPanelTargets,
) -> f32 {
    let scroll_step = 24.0;
    let max_scroll =
        (targets.content_height - (targets.panel_rect.height - MATERIAL_HEADER_HEIGHT)).max(0.0);
    (cur_scroll_y - delta_y * scroll_step).clamp(0.0, max_scroll)
}