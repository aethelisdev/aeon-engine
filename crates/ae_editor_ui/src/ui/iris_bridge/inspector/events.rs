// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Event Handling and Semantic Tag Hit-Testing
//!
//! Resolves mouse clicks, number input edits, color palette picks,
//! and component life-cycle commands via direct zero-allocation [`UiTree`] hit-testing.

use super::appearance::{
    resolve_appearance_add_palette_tag, resolve_appearance_clear_palette_tag,
    resolve_appearance_color_swatch_tag, resolve_appearance_hex_input_tag,
    resolve_appearance_palette_swatch_tag,
};
use super::header::resolve_entity_name_input_tag;
use super::tags::{
    resolve_add_component_btn_tag, resolve_audio_pick_tag, resolve_audio_play_tag,
    resolve_component_checkbox_tag, resolve_component_delete_tag, resolve_inspector_dropdown_tag,
    resolve_preset_reset_tag, resolve_save_prefab_btn_tag, resolve_unparent_tag,
};
use super::transform::resolve_transform_reset_tag;
use super::types::InspectorAction;
use irisui::prelude::*;

/// Resolves semantic tag click input over the Inspector panel.
///
/// Converts a persistent 64-bit widget hit tag into discrete [`InspectorAction`]s
/// without spatial search loops or rectangular allocations.
pub fn resolve_inspector_tag_click(
    hit_tag: u64,
    entity_opt: Option<hecs::Entity>,
    saved_swatches: &[[f32; 4]],
    out_actions: &mut Vec<InspectorAction>,
) -> bool {
    if resolve_add_component_btn_tag(hit_tag) {
        out_actions.push(InspectorAction::OpenAddComponentMenu(Point::ZERO));
        return true;
    }

    if resolve_save_prefab_btn_tag(hit_tag) {
        if let Some(entity) = entity_opt {
            out_actions.push(InspectorAction::SaveAsPrefab(entity));
        }
        return true;
    }

    if let Some(comp_name) = resolve_component_delete_tag(hit_tag) {
        if let Some(entity) = entity_opt {
            out_actions.push(InspectorAction::RemoveComponent(entity, comp_name));
        }
        return true;
    }

    if let Some(dd_id) = resolve_inspector_dropdown_tag(hit_tag) {
        if let Some(entity) = entity_opt {
            out_actions.push(InspectorAction::SelectDropdown(entity, dd_id, 0));
        }
        return true;
    }

    if let Some(cb_id) = resolve_component_checkbox_tag(hit_tag) {
        if let Some(entity) = entity_opt {
            out_actions.push(InspectorAction::ToggleCheckbox(entity, cb_id));
        }
        return true;
    }

    if resolve_appearance_color_swatch_tag(hit_tag) {
        out_actions.push(InspectorAction::ToggleColorPicker);
        return true;
    }

    if resolve_appearance_hex_input_tag(hit_tag) {
        out_actions.push(InspectorAction::FocusHexInput);
        return true;
    }

    if resolve_appearance_add_palette_tag(hit_tag) {
        out_actions.push(InspectorAction::AddColorToPalette(Color::TRANSPARENT));
        return true;
    }

    if resolve_appearance_clear_palette_tag(hit_tag) {
        out_actions.push(InspectorAction::ClearCustomPalette);
        return true;
    }

    if let Some(swatch_idx) = resolve_appearance_palette_swatch_tag(hit_tag) {
        if let Some(entity) = entity_opt
            && let Some(col) = saved_swatches.get(swatch_idx)
        {
            out_actions.push(InspectorAction::SetObjectColor(
                entity,
                Color::rgba(col[0], col[1], col[2], col[3]),
            ));
        }
        return true;
    }

    if resolve_preset_reset_tag(hit_tag) {
        if let Some(entity) = entity_opt {
            out_actions.push(InspectorAction::ResetPhysMatPreset(entity));
        }
        return true;
    }

    if resolve_audio_pick_tag(hit_tag) {
        if let Some(entity) = entity_opt {
            out_actions.push(InspectorAction::PickAudioFile(entity));
        }
        return true;
    }

    if resolve_audio_play_tag(hit_tag) {
        if let Some(entity) = entity_opt {
            out_actions.push(InspectorAction::ToggleAudioPlayback(entity));
        }
        return true;
    }

    if resolve_unparent_tag(hit_tag) {
        if let Some(entity) = entity_opt {
            out_actions.push(InspectorAction::Unparent(entity));
        }
        return true;
    }

    if let Some(axis_type) = resolve_transform_reset_tag(hit_tag) {
        if let Some(entity) = entity_opt {
            out_actions.push(InspectorAction::ResetTransform(entity, axis_type));
        }
        return true;
    }

    if resolve_entity_name_input_tag(hit_tag) {
        out_actions.push(InspectorAction::FocusRename);
        return true;
    }

    false
}