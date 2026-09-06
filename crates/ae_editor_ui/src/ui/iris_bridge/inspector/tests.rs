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