// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Floating ComboBox Dropdown Popup Builder
//!
//! Renders hardware-accelerated GPU SDF floating popup lists for active
//! ComboBox selections in the Inspector panel with neon cyan border styling.

use super::types::{InspectorDropdownId, InspectorPanelParams, InspectorPanelTargets};
use irisui::prelude::*;

/// Builds the floating dropdown menu popup for the currently active Inspector ComboBox.
pub fn build_inspector_dropdown_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    targets: &InspectorPanelTargets,
) {
    let Some(active_id) = params.active_dropdown else {
        return;
    };

    // Find the anchor rect of the triggering ComboBox
    let Some(&(_, anchor_rect, _)) = targets.dropdowns.iter().find(|(id, _, _)| *id == active_id)
    else {
        return;
    };

    let options = get_dropdown_options(active_id);
    let min_width = anchor_rect.width.max(110.0);

    ComboboxPopupBuilder::new(anchor_rect)
        .name("InspectorDropdownPopup")
        .min_width(min_width)
        .items(options)
        .cursor_pos(params.cursor_pos)
        .build(tree, parent_id);
}

/// Returns the slice of option label strings for a given dropdown identifier.
pub fn get_dropdown_options(id: InspectorDropdownId) -> &'static [&'static str] {
    match id {
        InspectorDropdownId::RigidBodyType => &["Dynamic", "Kinematic", "Static"],
        InspectorDropdownId::ColliderShape => {
            &["Capsule", "Box", "Sphere", "Trimesh", "Convex Hull"]
        }
        InspectorDropdownId::SurfaceType => &[
            "Default", "Metal", "Wood", "Stone", "Flesh", "Dirt", "Glass", "Rubber",
        ],
        InspectorDropdownId::ShapeType => {
            &["Cube", "Sphere", "Cylinder", "Capsule", "Torus", "Triangle"]
        }
        InspectorDropdownId::LightType => &["Point", "Directional", "Spot"],
        InspectorDropdownId::CameraProjection => &["Perspective", "Orthographic"],
        InspectorDropdownId::UiAnchor => &[
            "Top-Left",
            "Top-Center",
            "Top-Right",
            "Center-Left",
            "Center",
            "Center-Right",
            "Bottom-Left",
            "Bottom-Center",
            "Bottom-Right",
        ],
        InspectorDropdownId::UiTextAlignment => &["Left", "Center", "Right"],
    }
}