// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Unit Tests
//!
//! Verifies scene inspector hierarchy, dynamic component card switching,
//! and industry-standard 2D Screen Transform vs 3D Transform isolation.

use super::*;
use ae_core::ecs::{
    Name, Position, Rotation, Scale, UiAnchor, UiButton, UiElement, UiPanel, UiText,
    UiTextAlignment,
};
use irisui::prelude::*;

fn create_default_test_params<'a>(
    world: &'a hecs::World,
    selected_entity: Option<hecs::Entity>,
    euler: &'a [f32; 3],
    swatches: &'a [[f32; 4]],
) -> InspectorPanelParams<'a> {
    InspectorPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 320.0, 900.0),
        world,
        selected_entity,
        inspector_euler: euler,
        inspector_color_hex: "#ffffff",
        saved_swatches: swatches,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        active_dropdown: None,
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: false,
        active_number_input: None,
        active_text_input: None,
        active_rename_buffer: None,
        active_hex_buffer: None,
        inspector_hsv: [0.0, 0.0, 1.0],
        blink_caret: false,
    }
}

#[test]
fn test_inspector_empty_selection_renders_placeholder() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let world = hecs::World::new();
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];

    let params = create_default_test_params(&world, None, &euler, &swatches);

    let mut targets = InspectorPanelTargets::default();
    build_inspector_panel(&mut tree, root, &params, &mut targets);

    assert!(targets.number_inputs.is_empty());
    assert!(targets.dropdowns.is_empty());
    assert!(targets.checkboxes.is_empty());
}

#[test]
fn test_inspector_3d_entity_renders_transform_and_appearance() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut world = hecs::World::new();

    let entity = world.spawn((
        Name("3D Cube".to_string()),
        Position::new(0.0, 1.0, 0.0),
        Rotation::identity(),
        Scale::one(),
    ));

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_default_test_params(&world, Some(entity), &euler, &swatches);

    let mut targets = InspectorPanelTargets::default();
    build_inspector_panel(&mut tree, root, &params, &mut targets);

    // Verify 3D Transform inputs are present
    let has_pos_x = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::PosX));
    let has_rot_x = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::RotX));
    let has_scale_x = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::ScaleX));
    assert!(has_pos_x, "3D Transform PosX must be present");
    assert!(has_rot_x, "3D Transform RotX must be present");
    assert!(has_scale_x, "3D Transform ScaleX must be present");

    // Verify 2D Screen Transform inputs are NOT present
    let has_ui_offset = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::UiOffsetX));
    assert!(!has_ui_offset, "3D entity must not have 2D UiOffsetX input");
}

#[test]
fn test_inspector_2d_ui_entity_replaces_3d_transform_with_screen_transform() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut world = hecs::World::new();

    let entity = world.spawn((
        Name("Title Text".to_string()),
        UiElement {
            anchor: UiAnchor::Center,
            offset: [120.0, -40.0],
            size: [240.0, 50.0],
            pivot: [0.5, 0.5],
            visible: true,
            z_index: 2,
            alpha: 0.95,
        },
        UiText {
            text: "Welcome to Aeon".to_string(),
            font_size: 24.0,
            alignment: UiTextAlignment::Center,
            color: [1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        },
        UiPanel {
            background_color: [0.1, 0.1, 0.12, 0.8],
            corner_radius: 8.0,
            border_width: 1.5,
            border_color: [0.3, 0.6, 0.9, 1.0],
        },
        UiButton {
            is_enabled: true,
            ..Default::default()
        },
    ));

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_default_test_params(&world, Some(entity), &euler, &swatches);

    let mut targets = InspectorPanelTargets::default();
    build_inspector_panel(&mut tree, root, &params, &mut targets);

    // Verify 3D Transform inputs are NOT present
    let has_3d_pos = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::PosX));
    assert!(
        !has_3d_pos,
        "2D UI element must not show 3D Position in Inspector"
    );

    // Verify 2D Screen Transform inputs ARE present
    let has_offset_x = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::UiOffsetX));
    let has_offset_y = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::UiOffsetY));
    let has_size_w = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::UiSizeW));
    let has_size_h = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::UiSizeH));
    let has_pivot_x = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::UiPivotX));
    let has_pivot_y = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::UiPivotY));
    let has_z_index = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::UiZIndex));
    let has_alpha = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::UiAlpha));

    assert!(has_offset_x, "UiOffsetX must be present");
    assert!(has_offset_y, "UiOffsetY must be present");
    assert!(has_size_w, "UiSizeW must be present");
    assert!(has_size_h, "UiSizeH must be present");
    assert!(has_pivot_x, "UiPivotX must be present");
    assert!(has_pivot_y, "UiPivotY must be present");
    assert!(has_z_index, "UiZIndex must be present");
    assert!(has_alpha, "UiAlpha must be present");

    // Verify Anchor dropdown is present
    let has_anchor_dropdown = targets
        .dropdowns
        .iter()
        .any(|(id, ..)| matches!(id, InspectorDropdownId::UiAnchor));
    assert!(has_anchor_dropdown, "UiAnchor dropdown must be present");

    // Verify UiVisible checkbox is present
    let has_visible_cb = targets
        .checkboxes
        .iter()
        .any(|(id, ..)| matches!(id, ComponentCheckboxId::UiVisible));
    assert!(has_visible_cb, "UiVisible checkbox must be present");

    // Verify UiPanel properties
    let has_border_w = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::UiBorderWidth));
    let has_corner_r = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::UiCornerRadius));
    assert!(has_border_w, "UiBorderWidth must be present");
    assert!(has_corner_r, "UiCornerRadius must be present");

    // Verify UiText properties
    let has_font_size = targets
        .number_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorNumberInputId::UiFontSize));
    let has_align_dropdown = targets
        .dropdowns
        .iter()
        .any(|(id, ..)| matches!(id, InspectorDropdownId::UiTextAlignment));
    assert!(has_font_size, "UiFontSize must be present");
    assert!(
        has_align_dropdown,
        "UiTextAlignment dropdown must be present"
    );

    let has_text_input = targets
        .text_inputs
        .iter()
        .any(|(id, ..)| matches!(id, InspectorTextInputId::UiTextContent));
    assert!(has_text_input, "UiTextContent text input must be present");

    // Verify UiButton properties
    let has_button_interactable = targets
        .checkboxes
        .iter()
        .any(|(id, ..)| matches!(id, ComponentCheckboxId::UiInteractable));
    assert!(
        has_button_interactable,
        "UiInteractable checkbox must be present"
    );
}

#[test]
fn test_inspector_entity_isolation_invariant() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut world = hecs::World::new();
    let ent_a = world.spawn((
        Name("Dynamic Cube".to_string()),
        Position {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        Rotation::identity(),
        Scale {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    ));

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_default_test_params(&world, Some(ent_a), &euler, &swatches);

    let mut targets = InspectorPanelTargets::default();
    build_inspector_panel(&mut tree, root, &params, &mut targets);

    assert_eq!(
        targets.inspected_entity,
        Some(ent_a),
        "InspectorPanelTargets must explicitly carry the inspected entity"
    );

    // Verify reset transform button produces action targeting ent_a
    if let Some(&(axis, rect)) = targets.transform_reset_btns.first() {
        let mut actions = Vec::new();
        let clicked = super::events::handle_inspector_click(
            Point::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5),
            MouseButton::Left,
            &targets,
            &mut actions,
        );
        assert!(clicked);
        assert_eq!(actions.len(), 1);
        match actions[0] {
            InspectorAction::ResetTransform(target_ent, target_axis) => {
                assert_eq!(target_ent, ent_a);
                assert_eq!(target_axis, axis);
            }
            _ => panic!("Expected ResetTransform with target entity"),
        }
    }
}

#[test]
fn test_inspector_select_all_number_input_styling() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut world = hecs::World::new();
    let ent = world.spawn((
        Position {
            x: 5.0,
            y: 10.0,
            z: 15.0,
        },
        Rotation::identity(),
        Scale {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    ));

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let mut params = create_default_test_params(&world, Some(ent), &euler, &swatches);
    params.active_number_input = Some(ActiveNumberInputState {
        id: InspectorNumberInputId::PosX,
        buffer: "5.0",
        cursor_idx: 3,
        is_all_selected: true,
    });

    let mut targets = InspectorPanelTargets::default();
    build_inspector_panel(&mut tree, root, &params, &mut targets);

    // Verify node tree contains NumBox_PosX with dark background and glowing cyan active border
    let mut pos_x_style = None;
    let mut sel_pill_color = None;
    tree.traverse_depth_first(root, &mut |_id, node| {
        if node.name.as_deref() == Some("NumBox_PosX") {
            pos_x_style = Some((node.style.background_color, node.style.border.color));
        } else if node.name.as_deref() == Some("NumSel_PosX") {
            sel_pill_color = Some(node.style.background_color);
        }
    });
    assert_eq!(
        pos_x_style,
        Some((
            Color::rgba(0.118, 0.125, 0.145, 1.0),
            Color::rgba(0.0, 0.80, 1.00, 0.95),
        )),
        "Active number input must have dark background and glowing cyan active border"
    );
    assert_eq!(
        sel_pill_color,
        Some(Color::rgba(0.14, 0.46, 0.88, 0.95)),
        "Active number input must render vivid blue selection pill when is_all_selected is true"
    );
}

#[test]
fn test_active_number_input_caret_position() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut world = hecs::World::new();
    let ent = world.spawn((
        Name("TestObject".to_string()),
        Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Rotation::default(),
        Scale::default(),
    ));

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let mut params = create_default_test_params(&world, Some(ent), &euler, &swatches);
    params.blink_caret = true;
    params.active_number_input = Some(ActiveNumberInputState {
        id: InspectorNumberInputId::PosX,
        buffer: "42.5",
        cursor_idx: 2, // between "42" and ".5"
        is_all_selected: false,
    });

    let mut targets = InspectorPanelTargets::default();
    build_inspector_panel(&mut tree, root, &params, &mut targets);

    let mut pos_x_text = None;
    let mut has_sel_pill = false;
    tree.traverse_depth_first(root, &mut |_id, node| {
        if node.name.as_deref() == Some("NumText_PosX") {
            pos_x_text = node.text.clone();
        } else if node.name.as_deref() == Some("NumSel_PosX") {
            has_sel_pill = true;
        }
    });
    assert_eq!(
        pos_x_text,
        Some("X: 42|.5".to_string()),
        "Active number input must place caret at the exact cursor index"
    );
    assert!(
        !has_sel_pill,
        "Selection pill must NOT be rendered when is_all_selected is false"
    );
}

#[test]
fn test_inspector_rotation_reflects_entity_component_and_undo() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut world = hecs::World::new();
    let ent = world.spawn((
        Name("Dynamic Cube".to_string()),
        Position::default(),
        Rotation::default(),
        Scale::default(),
    ));

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_default_test_params(&world, Some(ent), &euler, &swatches);
    let mut targets = InspectorPanelTargets::default();

    // 1. Initial State: Rotation Y must be 0.0
    build_inspector_panel(&mut tree, root, &params, &mut targets);
    let mut rot_y_text = None;
    tree.traverse_depth_first(root, &mut |_id, node| {
        if node.name.as_deref() == Some("NumText_RotY") {
            rot_y_text = node.text.clone();
        }
    });
    assert_eq!(rot_y_text, Some("Y: 0.0".to_string()));

    // 2. Modified State: Change Rotation component to 55.0 degrees Y
    if let Ok(mut r) = world.get::<&mut Rotation>(ent) {
        *r = super::euler_deg_to_quaternion(0.0, 55.0, 0.0);
    }
    let mut tree2 = UiTree::new();
    let root2 = tree2.create_node();
    let mut targets2 = InspectorPanelTargets::default();
    build_inspector_panel(&mut tree2, root2, &params, &mut targets2);
    let mut rot_y_text2 = None;
    tree2.traverse_depth_first(root2, &mut |_id, node| {
        if node.name.as_deref() == Some("NumText_RotY") {
            rot_y_text2 = node.text.clone();
        }
    });
    assert_eq!(rot_y_text2, Some("Y: 55.0".to_string()));

    // 3. Undo State: Revert Rotation component back to identity
    if let Ok(mut r) = world.get::<&mut Rotation>(ent) {
        *r = Rotation::default();
    }
    let mut tree3 = UiTree::new();
    let root3 = tree3.create_node();
    let mut targets3 = InspectorPanelTargets::default();
    build_inspector_panel(&mut tree3, root3, &params, &mut targets3);
    let mut rot_y_text3 = None;
    tree3.traverse_depth_first(root3, &mut |_id, node| {
        if node.name.as_deref() == Some("NumText_RotY") {
            rot_y_text3 = node.text.clone();
        }
    });
    assert_eq!(
        rot_y_text3,
        Some("Y: 0.0".to_string()),
        "Inspector Rotation row must immediately reflect undone component value"
    );
}

#[test]
fn test_inspector_numeric_input_clamping_and_non_negative_invariants() {
    use super::types::InspectorNumberInputId;

    // 1. Collider Box Extents cannot be negative or zero
    assert_eq!(
        InspectorNumberInputId::ColliderBoxX.clamp_value(-100.0),
        0.001
    );
    assert_eq!(
        InspectorNumberInputId::ColliderBoxY.clamp_value(-0.5),
        0.001
    );
    assert_eq!(
        InspectorNumberInputId::ColliderBoxZ.clamp_value(-1000.0),
        0.001
    );
    assert_eq!(InspectorNumberInputId::ColliderBoxX.clamp_value(5.0), 5.0);

    // 2. Collider Radius and Height cannot be negative
    assert_eq!(
        InspectorNumberInputId::ColliderRadius.clamp_value(-50.0),
        0.001
    );
    assert_eq!(
        InspectorNumberInputId::ColliderHalfHeight.clamp_value(-10.0),
        0.001
    );

    // 3. RigidBody mass cannot be negative or zero
    assert_eq!(
        InspectorNumberInputId::RigidBodyMass.clamp_value(-100.0),
        0.001
    );
    assert_eq!(
        InspectorNumberInputId::RigidBodyMass.clamp_value(0.0),
        0.001
    );
    assert_eq!(
        InspectorNumberInputId::RigidBodyMass.clamp_value(25.0),
        25.0
    );

    // 4. UI dimensions and opacity invariants
    assert_eq!(InspectorNumberInputId::UiSizeW.clamp_value(-100.0), 1.0);
    assert_eq!(InspectorNumberInputId::UiSizeH.clamp_value(0.0), 1.0);
    assert_eq!(InspectorNumberInputId::UiAlpha.clamp_value(-0.5), 0.0);
    assert_eq!(InspectorNumberInputId::UiAlpha.clamp_value(1.5), 1.0);

    // 5. Physics friction and restitution invariants
    assert_eq!(
        InspectorNumberInputId::ColliderFriction.clamp_value(-2.0),
        0.0
    );
    assert_eq!(
        InspectorNumberInputId::ColliderRestitution.clamp_value(-1.0),
        0.0
    );
    assert_eq!(
        InspectorNumberInputId::ColliderRestitution.clamp_value(2.0),
        1.0
    );

    // 6. Verify handle_set_number_value enforces clamping on Collider entity in hecs::World
    let mut world = hecs::World::new();
    let ent = world.spawn((ae_core::ecs::Collider {
        shape: ae_core::ecs::ColliderShape::Box {
            half_extents: [0.5, 0.5, 0.5],
        },
        friction: 0.7,
        restitution: 0.0,
        is_sensor: false,
    },));

    let mut euler = [0.0, 0.0, 0.0];
    crate::ui::workbench::render::inspector_actions::handle_set_number_value(
        &world,
        ent,
        InspectorNumberInputId::ColliderBoxX,
        -100.0,
        &mut euler,
    );

    let col = world.get::<&ae_core::ecs::Collider>(ent).unwrap();
    if let ae_core::ecs::ColliderShape::Box { half_extents } = col.shape {
        assert!(
            half_extents[0] > 0.0,
            "Half extent X must be clamped to positive value, got {}",
            half_extents[0]
        );
        assert_eq!(half_extents[0], 0.001);
    } else {
        panic!("Expected Box shape");
    }
}