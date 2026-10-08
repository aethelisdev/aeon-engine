// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Event Dispatcher
//!
//! Evaluates semantic widget tags and mouse wheel scroll deltas to produce
//! high-level MaterialAction commands.
//!

use super::types::{MaterialAction, MaterialTagTarget, resolve_material_tag};

/// Hit-tests semantic tags against interactive buttons in the Material Studio.
pub fn handle_material_click(
    hit_tag: u64,
    selected_entity: Option<hecs::Entity>,
    active_model: Option<ae_renderer::asset::AssetHandle>,
) -> Option<MaterialAction> {
    let ent = selected_entity?;
    let target = resolve_material_tag(hit_tag)?;

    match target {
        MaterialTagTarget::SpriteChange => Some(MaterialAction::PickAndAssignEntityTexture(ent)),
        MaterialTagTarget::SpriteRemove => Some(MaterialAction::RemoveTextureFromEntity(ent)),
        MaterialTagTarget::AddTexture => Some(MaterialAction::PickAndAssignEntityTexture(ent)),
        MaterialTagTarget::AddColor => Some(MaterialAction::AddColorComponent(ent)),
        MaterialTagTarget::SubmeshAlpha(idx, mode) => {
            let model_handle = active_model?;
            Some(MaterialAction::SetModelSubmeshAlphaMode(
                model_handle,
                idx,
                mode,
            ))
        }
        MaterialTagTarget::SubmeshTexture(idx) => {
            let model_handle = active_model?;
            Some(MaterialAction::PickAndSetSubmeshTexture(model_handle, idx))
        }
    }
}

/// Calculates updated vertical scroll offset given a mouse wheel delta and maximum scroll limit.
///
/// # Arguments
/// * `delta_y` - Vertical mouse wheel scroll delta.
/// * `cur_scroll_y` - Current vertical scroll offset in physical pixels.
/// * `max_scroll` - Maximum permitted vertical scroll offset in physical pixels.
pub fn handle_material_scroll(delta_y: f32, cur_scroll_y: f32, max_scroll: f32) -> f32 {
    let scroll_step = 24.0;
    (cur_scroll_y - delta_y * scroll_step).clamp(0.0, max_scroll)
}