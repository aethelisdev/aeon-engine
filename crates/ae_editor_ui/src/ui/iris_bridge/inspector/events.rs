// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Event Handling and Semantic Tag Hit-Testing
//!
//! Resolves mouse clicks, number input edits, color palette picks,
//! and component life-cycle commands via direct zero-allocation [`UiTree`] hit-testing.

use super::appearance::{
    resolve_appearance_color_swatch_tag, resolve_appearance_hex_input_tag,
    resolve_appearance_palette_swatch_tag,
};
use super::header::resolve_entity_name_input_tag;
use super::tags::{resolve_add_component_btn_tag, resolve_component_delete_tag};
use super::types::InspectorAction;

/// Resolves semantic tag click input over the Inspector panel.
///
/// Converts a persistent 64-bit widget hit tag into discrete [`InspectorAction`]s
/// without spatial search loops or rectangular allocations.
pub fn resolve_inspector_tag_click(
    hit_tag: u64,
    entity_opt: Option<hecs::Entity>,
    _saved_swatches: &[[f32; 4]],
    out_actions: &mut Vec<InspectorAction>,
) -> bool {
    if resolve_add_component_btn_tag(hit_tag) {
        return true;
    }

    if let Some(comp_name) = resolve_component_delete_tag(hit_tag) {
        if let Some(entity) = entity_opt {
            out_actions.push(InspectorAction::RemoveComponent(entity, comp_name));
        }
        return true;
    }

    if resolve_appearance_color_swatch_tag(hit_tag)
        || resolve_appearance_hex_input_tag(hit_tag)
        || resolve_appearance_palette_swatch_tag(hit_tag).is_some()
        || resolve_entity_name_input_tag(hit_tag)
    {
        return true;
    }

    false
}