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
    targets: &mut InspectorPanelTargets,
) {
    targets.active_dropdown_popup_rect = None;
    targets.dropdown_items.clear();

    let Some(active_id) = params.active_dropdown else {
        return;
    };

    // Find the anchor rect of the triggering ComboBox
    let Some(&(_, anchor_rect, _)) = targets.dropdowns.iter().find(|(id, _, _)| *id == active_id)
    else {
        return;
    };

    let options = get_dropdown_options(active_id);
    let item_h = 22.0;
    let popup_w = anchor_rect.width.max(110.0);
    let total_h = (options.len() as f32) * item_h + 8.0;

    let popup_x = anchor_rect.x;
    let popup_y = if anchor_rect.bottom() + total_h > params.panel_rect.bottom() - 30.0 {
        (anchor_rect.y - total_h - 2.0).max(30.0)
    } else {
        anchor_rect.bottom() + 2.0
    };

    let popup_rect = Rect::new(popup_x, popup_y, popup_w, total_h);
    targets.active_dropdown_popup_rect = Some(popup_rect);

    // Root Dropdown Popup Card (Clean neutral dark charcoal matching Image 2 & 4)
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("InspectorDropdownPopup");
        node.computed_rect = popup_rect;
        node.style = Style::new()
            .background(Color::rgba(0.086, 0.090, 0.106, 0.98))
            .border(1.0, Color::rgba(0.173, 0.180, 0.208, 0.90))
            .border_radius(5.0)
            .box_shadow(0.0, 6.0, 18.0, Color::rgba(0.0, 0.0, 0.0, 0.70));
    }
    let _ = tree.add_child(parent_id, card_id);

    let mut cur_y = popup_y + 4.0;

    for (idx, &opt_label) in options.iter().enumerate() {
        let item_rect = Rect::new(popup_x + 4.0, cur_y, popup_w - 8.0, item_h);
        let is_hovered = item_rect.contains_point(params.cursor_pos);

        let (bg, text_col) = if is_hovered {
            (Color::rgba(0.157, 0.165, 0.188, 0.98), Color::WHITE)
        } else {
            (Color::TRANSPARENT, Color::rgba(0.886, 0.894, 0.918, 1.0))
        };

        let row_id = tree.create_node();
        if let Some(node) = tree.get_mut(row_id) {
            node.set_name(format!("DropdownItem_{}", idx));
            node.computed_rect = item_rect;
            node.style = Style::new().background(bg).border_radius(3.0);
            node.set_text(opt_label);
            node.font_size = 11.0;
            node.line_height = item_h;
            node.text_align = TextAlign::Center;
            node.text_color = text_col;
        }
        let _ = tree.add_child(card_id, row_id);

        targets.dropdown_items.push((idx, item_rect));
        cur_y += item_h;
    }
}

/// Returns the slice of option label strings for a given dropdown identifier.
fn get_dropdown_options(id: InspectorDropdownId) -> &'static [&'static str] {
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