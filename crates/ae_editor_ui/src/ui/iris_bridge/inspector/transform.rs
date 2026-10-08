// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Transform Component Inspector Card Builder
//!
//! Renders the 3D Position, Rotation Euler, and Scale axes using high-level
//! declarative [`UiScope`] cards and property row primitives.

use super::registry::ComponentRenderContext;
use super::types::{InspectorNumberInputId, TransformAxisType};
use irisui::prelude::*;

/// Resolves an interactive semantic tag to a Transform numeric input ID, bounds, and sensitivity.
///
/// Enables $O(1)$ dispatch for numeric dragging, inline typing, and text selection
/// without requiring coordinate allocations or imperative target buffers.
#[inline]
pub fn resolve_transform_number_input_tag(
    tag: u64,
) -> Option<(InspectorNumberInputId, f32, f32, f32)> {
    let pos_base = hash_label_with_seed(0, "Position");
    if tag == pos_base.wrapping_add(1) {
        return Some((InspectorNumberInputId::PosX, -10000.0, 10000.0, 0.1));
    }
    if tag == pos_base.wrapping_add(2) {
        return Some((InspectorNumberInputId::PosY, -10000.0, 10000.0, 0.1));
    }
    if tag == pos_base.wrapping_add(3) {
        return Some((InspectorNumberInputId::PosZ, -10000.0, 10000.0, 0.1));
    }

    let rot_base = hash_label_with_seed(0, "Rotation");
    if tag == rot_base.wrapping_add(1) {
        return Some((InspectorNumberInputId::RotX, -360.0, 360.0, 0.5));
    }
    if tag == rot_base.wrapping_add(2) {
        return Some((InspectorNumberInputId::RotY, -360.0, 360.0, 0.5));
    }
    if tag == rot_base.wrapping_add(3) {
        return Some((InspectorNumberInputId::RotZ, -360.0, 360.0, 0.5));
    }

    let scale_base = hash_label_with_seed(0, "Scale");
    if tag == scale_base.wrapping_add(1) {
        return Some((InspectorNumberInputId::ScaleX, 0.001, 1000.0, 0.01));
    }
    if tag == scale_base.wrapping_add(2) {
        return Some((InspectorNumberInputId::ScaleY, 0.001, 1000.0, 0.01));
    }
    if tag == scale_base.wrapping_add(3) {
        return Some((InspectorNumberInputId::ScaleZ, 0.001, 1000.0, 0.01));
    }

    None
}

/// Resolves an interactive semantic tag to a Transform reset axis type.
///
/// Enables $O(1)$ event routing for reset buttons without coordinate traversal.
#[inline]
pub fn resolve_transform_reset_tag(tag: u64) -> Option<TransformAxisType> {
    let pos_base = hash_label_with_seed(0, "Position");
    if tag == pos_base.wrapping_add(4) {
        return Some(TransformAxisType::Position);
    }
    let rot_base = hash_label_with_seed(0, "Rotation");
    if tag == rot_base.wrapping_add(4) {
        return Some(TransformAxisType::Rotation);
    }
    let scale_base = hash_label_with_seed(0, "Scale");
    if tag == scale_base.wrapping_add(4) {
        return Some(TransformAxisType::Scale);
    }
    None
}

/// Builds the `📐 Transform` card declaratively in the `UiTree` and returns the measured height.
///
/// Encapsulates the entire Transform inspector interface within a pure declarative [`UiScope`]
/// container hierarchy, managing flex layout, hover states, caret blinking, and Select-All highlight
/// capsules without manual coordinate arithmetic or imperative target buffers.
pub fn build_transform_card(scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
    let mut pos = ctx
        .world
        .get::<&ae_core::ecs::Position>(ctx.entity)
        .map(|p| [p.x, p.y, p.z])
        .unwrap_or([0.0, 0.0, 0.0]);

    let mut rot = ctx
        .world
        .get::<&ae_core::ecs::Rotation>(ctx.entity)
        .map(|r| crate::ui::iris_bridge::inspector::quaternion_to_euler_deg(&r))
        .unwrap_or([0.0, 0.0, 0.0]);

    let mut scale = ctx
        .world
        .get::<&ae_core::ecs::Scale>(ctx.entity)
        .map(|s| [s.x, s.y, s.z])
        .unwrap_or([1.0, 1.0, 1.0]);

    let get_edit_state = |id: InspectorNumberInputId| {
        ctx.params
            .active_number_input
            .filter(|s| s.id == id)
            .map(|s| s.to_edit_state(ctx.params.blink_caret))
    };

    scope.card_named("TransformCard", "📐 Transform", |card| {
        let pos_opts = PropertyVec3Options::new(0.1)
            .with_decimals(3)
            .with_reset([0.0, 0.0, 0.0])
            .with_edit_states([
                get_edit_state(InspectorNumberInputId::PosX),
                get_edit_state(InspectorNumberInputId::PosY),
                get_edit_state(InspectorNumberInputId::PosZ),
            ]);
        card.property_vec3_with_options("Position", &mut pos, pos_opts);

        let rot_opts = PropertyVec3Options::new(0.5)
            .with_decimals(1)
            .with_reset([0.0, 0.0, 0.0])
            .with_edit_states([
                get_edit_state(InspectorNumberInputId::RotX),
                get_edit_state(InspectorNumberInputId::RotY),
                get_edit_state(InspectorNumberInputId::RotZ),
            ]);
        card.property_vec3_with_options("Rotation", &mut rot, rot_opts);

        let scale_opts = PropertyVec3Options::new(0.01)
            .with_decimals(3)
            .with_reset([1.0, 1.0, 1.0])
            .with_edit_states([
                get_edit_state(InspectorNumberInputId::ScaleX),
                get_edit_state(InspectorNumberInputId::ScaleY),
                get_edit_state(InspectorNumberInputId::ScaleZ),
            ]);
        card.property_vec3_with_options("Scale", &mut scale, scale_opts);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::types::{ActiveNumberInputState, InspectorPanelParams};
    use ae_core::ecs::{Name, Position, Rotation, Scale};

    #[test]
    fn test_declarative_transform_card_structure() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Name("TestTransformEntity".to_string()),
            Position::new(1.0, 2.0, 3.0),
            Rotation::identity(),
            Scale::one(),
        ));

        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            scroll_y: 0.0,
            active_dropdown: None,
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

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 6.0, 10.0, 308.0);

        let mut scope = UiScope::new(&mut tree, root);
        build_transform_card(&mut scope, &mut ctx);
        layout_subtree(scope.tree_mut(), root, Rect::new(6.0, 10.0, 308.0, 300.0));

        // Verify card node exists and has non-zero computed_rect after layout pass
        let card_node = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("TransformCard"))
            .map(|(_, n)| n)
            .expect("TransformCard node must exist");
        assert_eq!(card_node.style.corner_radii.top_left, 6.0);
        assert_eq!(card_node.computed_rect.x, 6.0);
        assert_eq!(card_node.computed_rect.y, 10.0);
        assert_eq!(card_node.computed_rect.width, 308.0);
        assert!(card_node.computed_rect.height > 100.0);

        // Verify all 9 input boxes exist with correct tags and WidgetRole::NumericInput
        let expected_input_ids = [
            InspectorNumberInputId::PosX,
            InspectorNumberInputId::PosY,
            InspectorNumberInputId::PosZ,
            InspectorNumberInputId::RotX,
            InspectorNumberInputId::RotY,
            InspectorNumberInputId::RotZ,
            InspectorNumberInputId::ScaleX,
            InspectorNumberInputId::ScaleY,
            InspectorNumberInputId::ScaleZ,
        ];

        let mut resolved_inputs = Vec::new();
        for (_, node) in tree.iter() {
            if let Some((id, _, _, _)) = resolve_transform_number_input_tag(node.tag) {
                assert_eq!(node.role, WidgetRole::NumericInput);
                assert!(
                    node.computed_rect.width > 0.0,
                    "Input width must be non-zero"
                );
                assert!(
                    node.computed_rect.height > 0.0,
                    "Input height must be non-zero"
                );
                resolved_inputs.push(id);
            }
        }
        assert_eq!(resolved_inputs.len(), 9);
        for expected in expected_input_ids {
            assert!(
                resolved_inputs.contains(&expected),
                "Missing tag for {:?}",
                expected
            );
        }

        // Verify all 3 reset buttons exist with correct tags and WidgetRole::Button
        let expected_axes = [
            TransformAxisType::Position,
            TransformAxisType::Rotation,
            TransformAxisType::Scale,
        ];
        let mut resolved_resets = Vec::new();
        for (_, node) in tree.iter() {
            if let Some(axis) = resolve_transform_reset_tag(node.tag) {
                assert_eq!(node.role, WidgetRole::Button);
                assert!(
                    node.computed_rect.width > 0.0,
                    "Reset button width must be non-zero"
                );
                assert!(
                    node.computed_rect.height > 0.0,
                    "Reset button height must be non-zero"
                );
                resolved_resets.push(axis);
            }
        }
        assert_eq!(resolved_resets.len(), 3);
        for expected in expected_axes {
            assert!(
                resolved_resets.contains(&expected),
                "Missing reset button for {:?}",
                expected
            );
        }
    }

    #[test]
    fn test_declarative_transform_active_edit_and_select_all() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Name("ActiveEditEntity".to_string()),
            Position::new(5.0, 10.0, 15.0),
            Rotation::identity(),
            Scale::one(),
        ));

        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let active_input = ActiveNumberInputState {
            id: InspectorNumberInputId::RotY,
            buffer: "45.0",
            cursor_idx: 4,
            is_all_selected: true,
        };

        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            scroll_y: 0.0,
            active_dropdown: None,
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_number_input: Some(active_input),
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 6.0, 10.0, 308.0);

        let mut scope = UiScope::new(&mut tree, root);
        build_transform_card(&mut scope, &mut ctx);

        let rot_y_box = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("NumBox_Rotation_Y"))
            .map(|(_, n)| n)
            .expect("NumBox_Rotation_Y node must exist");

        // Verify active cyan border
        assert_eq!(
            rot_y_box.style.border.color,
            Color::rgba(0.0, 0.80, 1.00, 0.95)
        );

        // Verify selection highlight pill was created
        let sel_node = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("NumSel_Rotation_Y"))
            .map(|(_, n)| n);
        assert!(
            sel_node.is_some(),
            "NumSel_Rotation_Y must be rendered when is_all_selected is true"
        );
    }

    #[test]
    fn test_declarative_transform_left_aligned_across_resizes() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Name("AlignEntity".to_string()),
            Position::new(1.0, 2.0, 3.0),
            Rotation::identity(),
            Scale::one(),
        ));
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            scroll_y: 0.0,
            active_dropdown: None,
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

        // 1. Render in a standard narrow card (308px)
        let mut ctx_narrow = ComponentRenderContext::new(entity, &world, &params, 6.0, 10.0, 308.0);
        let mut scope_narrow = UiScope::new(&mut tree, root);
        build_transform_card(&mut scope_narrow, &mut ctx_narrow);
        layout_subtree(
            scope_narrow.tree_mut(),
            root,
            Rect::new(6.0, 10.0, 308.0, 300.0),
        );

        let get_rect = |tree: &UiTree, name: &str| {
            tree.iter()
                .find(|(_, n)| n.name.as_deref() == Some(name))
                .map(|(_, n)| n.computed_rect)
                .unwrap_or_else(|| panic!("Node {} must exist in tree", name))
        };

        let pos_x_narrow = get_rect(&tree, "NumBox_Position_X");
        let rot_x_narrow = get_rect(&tree, "NumBox_Rotation_X");
        let scale_x_narrow = get_rect(&tree, "NumBox_Scale_X");

        // Verify all 3 rows have identical X coordinates (column aligned)
        assert_eq!(
            pos_x_narrow.x, rot_x_narrow.x,
            "Position X and Rotation X must share horizontal start coordinate"
        );
        assert_eq!(
            rot_x_narrow.x, scale_x_narrow.x,
            "Rotation X and Scale X must share horizontal start coordinate"
        );

        // Verify inputs are left-aligned (close to card start, base_x = 6.0 + padding + label_width)
        assert!(
            pos_x_narrow.x < 100.0,
            "Inputs must be left-aligned, found X: {}",
            pos_x_narrow.x
        );

        // 2. Render in a wide card (600px)
        let mut tree_wide = UiTree::new();
        let root_wide = tree_wide.create_root().expect("Root must be created");
        let mut ctx_wide = ComponentRenderContext::new(entity, &world, &params, 6.0, 10.0, 600.0);
        let mut scope_wide = UiScope::new(&mut tree_wide, root_wide);
        build_transform_card(&mut scope_wide, &mut ctx_wide);
        layout_subtree(
            scope_wide.tree_mut(),
            root_wide,
            Rect::new(6.0, 10.0, 600.0, 300.0),
        );

        let pos_x_wide = get_rect(&tree_wide, "NumBox_Position_X");
        let rot_x_wide = get_rect(&tree_wide, "NumBox_Rotation_X");
        let scale_x_wide = get_rect(&tree_wide, "NumBox_Scale_X");

        // In the wide card, the inputs must REMAIN at the exact same left-aligned X position
        assert_eq!(
            pos_x_wide.x, pos_x_narrow.x,
            "Inputs must not shift to the right when card is widened!"
        );
        assert_eq!(rot_x_wide.x, rot_x_narrow.x);
        assert_eq!(scale_x_wide.x, scale_x_narrow.x);
    }
}