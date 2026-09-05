// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Event Dispatcher
//!
//! Evaluates mouse click coordinates and mouse wheel scroll deltas against the cached
//! MaterialPanelTargets to produce high-level MaterialAction commands.
//!

use super::types::{MaterialAction, MaterialPanelTargets};
use irisui::prelude::*;

/// Hit-tests cursor clicks against interactive targets in the Material Studio.
pub fn handle_material_click(
    click_point: Point,
    selected_entity: Option<hecs::Entity>,
    targets: &MaterialPanelTargets,
) -> Option<MaterialAction> {
    if let Some(ent) = selected_entity {
        // 1. Change 2D Sprite Texture Button
        if let Some(btn_rect) = targets.btn_change_texture
            && btn_rect.contains_point(click_point)
        {
            return Some(MaterialAction::PickAndAssignEntityTexture(ent));
        }

        // 2. Remove 2D Sprite Texture Button
        if let Some(btn_rect) = targets.btn_remove_texture
            && btn_rect.contains_point(click_point)
        {
            return Some(MaterialAction::RemoveTextureFromEntity(ent));
        }

        // 3. Add Texture / Sprite when no geometry is present
        if let Some(btn_rect) = targets.btn_add_texture
            && btn_rect.contains_point(click_point)
        {
            return Some(MaterialAction::PickAndAssignEntityTexture(ent));
        }

        // 4. Add Color Tint Component Button
        if let Some(btn_rect) = targets.btn_add_color
            && btn_rect.contains_point(click_point)
        {
            return Some(MaterialAction::AddColorComponent(ent));
        }
    }

    // 5. Submesh Alpha Mode Pill Buttons
    for &(model_handle, submesh_idx, mode, btn_rect) in &targets.submesh_alpha_buttons {
        if btn_rect.contains_point(click_point) {
            return Some(MaterialAction::SetModelSubmeshAlphaMode(
                model_handle,
                submesh_idx,
                mode,
            ));
        }
    }

    // 6. Submesh Change Texture Buttons
    for &(model_handle, submesh_idx, btn_rect) in &targets.submesh_texture_buttons {
        if btn_rect.contains_point(click_point) {
            return Some(MaterialAction::PickAndSetSubmeshTexture(
                model_handle,
                submesh_idx,
            ));
        }
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
    let max_scroll = (targets.content_height - (targets.panel_rect.height - 36.0)).max(0.0);
    (cur_scroll_y - delta_y * scroll_step).clamp(0.0, max_scroll)
}