// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector and Component Property Editor
//!
//! Orchestrates the inspection, modification, reflection, and prefab export
//! of selected ECS entities via 100% hardware-accelerated Iris UI GPU SDF.
//!

pub mod add_menu;
pub mod appearance;
pub mod color_picker_popup;
pub mod components;
pub mod dropdown_popup;
pub mod dynamic_card;
pub mod dynamic_reflection;
pub mod events;
pub mod footer;
pub mod header;
pub mod math;
pub mod panel;
pub mod registry;
pub mod transform;
pub mod types;
pub mod ui_transform;

#[cfg(test)]
mod tests;

pub use events::handle_inspector_click;
pub use math::{euler_deg_to_quaternion, quaternion_to_euler_deg};
pub use panel::build_inspector_panel;
pub use registry::{ComponentInspectorHandler, ComponentRenderContext, InspectorRegistry};
pub use types::{
    ComponentCategory, ComponentCheckboxId, InspectorAction, InspectorDropdownId,
    InspectorNumberInputId, InspectorPanelParams, InspectorPanelTargets, InspectorTextInputId,
    TransformAxisType,
};