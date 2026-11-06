// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector and Component Property Editor
//!
//! Orchestrates the inspection, modification, reflection, and prefab export
//! of selected ECS entities via 100% hardware-accelerated Iris UI GPU SDF.

pub mod actions;
pub mod add_menu;
pub mod appearance;
pub mod builder;
pub mod color_picker_popup;
pub mod components;
pub mod context;
pub mod dynamic_reflection;
pub mod events;
pub mod footer;
pub mod handler;
pub mod header;
pub mod math;
pub mod math_eval;
pub mod panel;
pub mod registry;
pub mod state;
pub mod tags;
pub mod transform;
pub mod types;
pub mod ui_transform;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_popups;

pub use appearance::{
    DEFAULT_PALETTE, appearance_add_palette_tag, appearance_clear_palette_tag,
    appearance_color_swatch_tag, appearance_hex_input_tag, appearance_palette_swatch_tag,
    resolve_appearance_add_palette_tag, resolve_appearance_clear_palette_tag,
    resolve_appearance_color_swatch_tag, resolve_appearance_hex_input_tag,
    resolve_appearance_palette_swatch_tag,
};
pub use context::DEFAULT_INSPECTOR_LABEL_WIDTH;
pub use events::resolve_inspector_tag_click;
pub use header::{entity_name_input_tag, resolve_entity_name_input_tag};
pub use math::{euler_deg_to_quaternion, quaternion_to_euler_deg};
pub use math_eval::{MathEvalError, evaluate_inspector_math};
pub use panel::{build_inspector_overlays, build_inspector_panel};
pub use registry::{ComponentInspectorHandler, ComponentRenderContext, InspectorRegistry};
pub use state::{
    InspectorPanelState, InspectorState, InspectorTransformSnapshot, is_widget_descendant_of,
};
pub use tags::{
    TAG_COMPONENT_DELETE_BASE, TAG_INSPECTOR_ACTION_BASE, TAG_INSPECTOR_ADD_COMPONENT,
    TAG_INSPECTOR_AUDIO_PICK, TAG_INSPECTOR_AUDIO_PLAY, TAG_INSPECTOR_PANEL_ROOT,
    TAG_INSPECTOR_PRESET_RESET, TAG_INSPECTOR_SAVE_PREFAB, TAG_INSPECTOR_UNPARENT,
    encode_component_delete_tag, is_inspector_tag, resolve_add_component_btn_tag,
    resolve_audio_pick_tag, resolve_audio_play_tag, resolve_component_delete_tag,
    resolve_preset_reset_tag, resolve_save_prefab_btn_tag, resolve_unparent_tag,
};
pub use transform::build_transform_card;
pub use types::{
    ComponentCategory, InspectorAction, InspectorColorDragMode, InspectorPanelParams,
    TransformAxisType,
};