// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Event Handling and Semantic Tag Hit-Testing
//!
//! Classifies clicked semantic tags that belong to the Inspector panel so the event loop can
//! consume the click and schedule an Inspector redraw. The redraw rebuilds the declarative
//! panel, whose widgets map the persisted frame click directly into `EngineUiAction`s.

use super::appearance::{
    resolve_appearance_color_swatch_tag, resolve_appearance_hex_input_tag,
    resolve_appearance_palette_swatch_tag,
};
use super::header::resolve_entity_name_input_tag;
use super::tags::{resolve_add_component_btn_tag, resolve_component_delete_tag};

/// Returns `true` when `hit_tag` belongs to an interactive Inspector widget.
///
/// The caller consumes the click and tags the Inspector panel for redraw; no action is queued
/// here. Actions are produced exactly once by the declarative panel rebuild (for example a card
/// delete button maps to `EngineUiAction::RemoveComponent` in `panel.rs`), which prevents the
/// duplicate dispatch that a second, event-time action queue would cause.
#[must_use]
pub fn resolve_inspector_tag_click(hit_tag: u64) -> bool {
    resolve_add_component_btn_tag(hit_tag)
        || resolve_component_delete_tag(hit_tag).is_some()
        || resolve_appearance_color_swatch_tag(hit_tag)
        || resolve_appearance_hex_input_tag(hit_tag)
        || resolve_appearance_palette_swatch_tag(hit_tag).is_some()
        || resolve_entity_name_input_tag(hit_tag)
}