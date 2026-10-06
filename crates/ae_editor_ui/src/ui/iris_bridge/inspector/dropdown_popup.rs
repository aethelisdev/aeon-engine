// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Floating ComboBox Dropdown Popup Builder
//!
//! Renders hardware-accelerated declarative [`UiScope`] floating popup lists for active
//! ComboBox selections in the Inspector panel with neon cyan border styling and single-pass flexbox.
//!

use super::tags::{TAG_INSPECTOR_DROPDOWN_ITEM_BASE, encode_inspector_dropdown_tag};
use super::types::{InspectorDropdownId, InspectorPanelParams};
use irisui::prelude::*;

/// Builds the floating dropdown menu popup for the currently active Inspector ComboBox using declarative [`UiScope`].
pub fn build_inspector_dropdown_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
) {
    let Some(active_id) = params.active_dropdown else {
        return;
    };

    let active_tag = encode_inspector_dropdown_tag(active_id);
    let anchor_rect = tree
        .iter()
        .find(|(_, node)| node.tag == active_tag)
        .map(|(_, node)| node.computed_rect)
        .unwrap_or(Rect::new(
            params.panel_rect.x + 8.0,
            params.panel_rect.y + 40.0,
            110.0,
            22.0,
        ));

    let options = get_dropdown_options(active_id);
    let popup_w = anchor_rect.width.max(110.0);
    let item_h = 24.0;
    let popup_h = options.len() as f32 * item_h + 8.0;

    let popup_x = anchor_rect.x;
    let popup_y = if anchor_rect.bottom() + popup_h > params.panel_rect.bottom() - 10.0 {
        (anchor_rect.y - popup_h - 2.0).max(params.panel_rect.y + 10.0)
    } else {
        anchor_rect.bottom() + 2.0
    };

    let mut scope = UiScope::with_tagged_interactions(tree, parent_id, &[], params.hovered_tag);
    scope.dropdown_menu_card_named(
        "InspectorDropdownPopup",
        popup_x,
        popup_y,
        popup_w,
        |menu| {
            for (idx, &item_label) in options.iter().enumerate() {
                let item_tag = TAG_INSPECTOR_DROPDOWN_ITEM_BASE + idx as u64;
                menu.dropdown_item(item_tag, "", item_label, None, true);
            }
        },
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declarative_dropdown_popup_builds_in_tree() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let world = hecs::World::new();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];

        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: None,
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            scroll_y: 0.0,
            active_dropdown: Some(InspectorDropdownId::RigidBodyType),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_number_input: None,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
            hovered_tag: Some(1),
        };

        build_inspector_dropdown_popup(&mut tree, root, &params);

        let popup_node = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("InspectorDropdownPopup"));
        assert!(popup_node.is_some(), "InspectorDropdownPopup must be built");
        let (_, node) = popup_node.unwrap();
        assert_eq!(node.layer, UiLayer::Popup);
        assert_eq!(node.role, WidgetRole::DropdownPopup);

        let items: Vec<_> = tree
            .iter()
            .filter(|(_, n)| n.role == WidgetRole::DropdownItem)
            .collect();
        assert_eq!(items.len(), 3, "RigidBodyType has 3 options");
        assert_eq!(items[0].1.tag, TAG_INSPECTOR_DROPDOWN_ITEM_BASE);
        assert_eq!(items[1].1.tag, TAG_INSPECTOR_DROPDOWN_ITEM_BASE + 1);
        assert_eq!(items[2].1.tag, TAG_INSPECTOR_DROPDOWN_ITEM_BASE + 2);
    }
}