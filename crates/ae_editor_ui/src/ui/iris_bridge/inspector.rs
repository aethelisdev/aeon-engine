// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector and Component Property Editor
//!
//! Orchestrates the inspection, modification, reflection, and prefab export
//! of selected ECS entities via 100% hardware-accelerated Iris UI GPU SDF.

pub mod add_menu;
pub mod appearance;
pub mod builder;
pub mod color_picker_popup;
pub mod components;
pub mod context;
pub mod dynamic_reflection;
pub mod footer;
pub mod handler;
pub mod header;
pub mod math;
pub mod math_eval;
pub mod panel;
pub mod registry;
pub mod state;
pub mod transform;
pub mod types;
pub mod ui_transform;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_popups;
#[cfg(test)]
mod tests_remove_component;

pub use appearance::{
    DEFAULT_PALETTE, appearance_color_swatch_tag, appearance_hex_input_tag,
    appearance_palette_swatch_tag,
};
pub use color_picker_popup::{
    TAG_INSPECTOR_COLOR_PICKER_CARD, TAG_INSPECTOR_COLOR_PICKER_CLOSE,
    TAG_INSPECTOR_COLOR_PICKER_HUE_BAR, TAG_INSPECTOR_COLOR_PICKER_SV_BOX,
};
pub use context::DEFAULT_INSPECTOR_LABEL_WIDTH;
pub use footer::TAG_INSPECTOR_ADD_COMPONENT;
pub use header::entity_name_input_tag;
pub use math::{euler_deg_to_quaternion, quaternion_to_euler_deg, resolve_euler_deg};
pub use math_eval::{MathEvalError, evaluate_inspector_math};
pub use panel::{
    TAG_INSPECTOR_CARDS_CONTAINER, TAG_INSPECTOR_PANEL_ROOT, build_inspector_overlays,
    build_inspector_overlays_with_action, build_inspector_panel,
};
pub use registry::{ComponentInspectorHandler, ComponentRenderContext, InspectorRegistry};
pub use state::{
    InspectorPanelState, InspectorState, InspectorTransformSnapshot, is_inspector_tag,
    is_widget_descendant_of,
};
pub use transform::{build_transform_card, draw_transform_card};
pub use types::{
    ComponentCategory, InspectorColorDragMode, InspectorPanelParams, InspectorSessionRefs,
};