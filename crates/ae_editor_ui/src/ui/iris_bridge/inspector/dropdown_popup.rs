// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Floating ComboBox Dropdown Popup Builder
//!
//! Renders hardware-accelerated declarative [`UiScope`] floating popup lists for active
//! ComboBox selections in the Inspector panel with neon cyan border styling and single-pass flexbox.

use super::tags::{TAG_INSPECTOR_DROPDOWN_ITEM_BASE, encode_inspector_dropdown_tag};
use super::types::{InspectorDropdownId, InspectorPanelParams};
use irisui::prelude::*;

/// Builds the floating dropdown menu popup for the currently active Inspector ComboBox using declarative [`UiScope`].
///
/// Returns the screen-space bounding rectangle of the constructed dropdown popup, if active.
pub fn build_inspector_dropdown_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
) -> Option<Rect> {
    params.active_dropdown.map(|active_id| {
        let dropdown_tag = encode_inspector_dropdown_tag(active_id);
        let anchor_rect = tree
            .node_rect_tagged(dropdown_tag, Point::ZERO)
            .or_else(|| tree.tag_rect(dropdown_tag))
            .unwrap_or(Rect::new(
                params.panel_rect.x + 8.0,
                params.panel_rect.y + 40.0,
                110.0,
                22.0,
            ));
        let menu_width = anchor_rect.width.max(110.0);
        UiScope::new(tree, parent_id)
            .popup_anchored_named("InspectorDropdownPopup", anchor_rect, menu_width, |menu| {
                for (idx, &item_label) in get_dropdown_options(active_id).iter().enumerate() {
                    menu.dropdown_item(
                        TAG_INSPECTOR_DROPDOWN_ITEM_BASE + idx as u64,
                        "",
                        item_label,
                        None,
                        true,
                    );
                }
            })
            .response
            .rect
    })
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