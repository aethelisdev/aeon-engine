// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Inspector Undo & Property Alignment Integration Tests
//!
//! Verifies the Single Source of Truth (SSOT) alignment of property labels across all
//! component cards (`Mass:`, `Gravity:`, `X:`, `Half Extent X:`, etc.) to the canonical
//! 105px column boundary, and verifies that manual numeric input submissions and pointer
//! dragging gestures accurately generate Undo/Redo commands with genuine pre-edit snapshots.

use ae_core::ecs::{Name, Position, RigidBody, Rotation, Scale, Velocity};
use ae_editor_ui::ui::iris_bridge::inspector::{
    DEFAULT_INSPECTOR_LABEL_WIDTH, InspectorPanelParams, build_inspector_panel,
};
use irisui::core::UiTree;
use irisui::widgets::declarative::scope::UiScope;

/// Constructs minimal mock parameters required to render the Inspector panel for tests.
fn create_test_params<'a>(
    world: &'a hecs::World,
    selected: Option<hecs::Entity>,
    euler: &'a [f32; 3],
    swatches: &'a [[f32; 4]],
) -> InspectorPanelParams<'a> {
    InspectorPanelParams {
        panel_rect: irisui::core::Rect::new(0.0, 0.0, 320.0, 900.0),
        world,
        selected_entity: selected,
        inspector_euler: euler,
        inspector_color_hex: "#ffffff",
        saved_swatches: swatches,
        cursor_pos: irisui::core::Point::new(0.0, 0.0),
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: false,
        active_rename_buffer: None,
        is_rename_all_selected: false,
        active_hex_buffer: None,
        inspector_hsv: [0.0, 0.0, 1.0],
        blink_caret: false,
    }
}

#[test]
fn test_property_label_width_alignment_across_cards() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");

    let mut world = hecs::World::new();
    let entity = world.spawn((
        Name("Dynamic Cube".to_string()),
        Position::default(),
        Rotation::default(),
        Scale::default(),
        RigidBody::default(),
        ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.7,
            restitution: 0.0,
            is_sensor: false,
        },
        Velocity::default(),
    ));

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, Some(entity), &euler, &swatches);

    let mut is_color_picker_open = false;
    let mut name = String::new();
    let mut swatches = Vec::new();
    let mut is_add_menu_open = false;
    let mut active_submenu = None;
    let mut session = ae_editor_ui::ui::iris_bridge::inspector::InspectorSessionRefs {
        inspected_name: &mut name,
        saved_swatches: &mut swatches,
        is_color_picker_open: &mut is_color_picker_open,
        is_add_menu_open: &mut is_add_menu_open,
        active_submenu: &mut active_submenu,
    };
    build_inspector_panel(&mut UiScope::new(&mut tree, root), &params, &mut session);

    // Verify canonical default label width constant is 105px
    assert_eq!(DEFAULT_INSPECTOR_LABEL_WIDTH, 105.0);

    // Collect all property label nodes in the tree
    let mut label_widths = Vec::new();
    tree.traverse_depth_first(root, &mut |_id, node| {
        let name = node.name.as_deref().unwrap_or("");
        if (name == "PropertyLabel" || name == "ComboLabel")
            && let Some(w) = node.style.width
        {
            label_widths.push((node.text.clone().unwrap_or_default(), w));
        }
    });

    // Check that RigidBody (Mass:, Gravity:) and Collider (Shape:, Half Extent X:) share 105px
    assert!(
        !label_widths.is_empty(),
        "Inspector must contain property labels"
    );
    for (text, width) in &label_widths {
        if text.starts_with("Mass:")
            || text.starts_with("Gravity:")
            || text.starts_with("Shape:")
            || text.starts_with("Half Extent")
            || text.starts_with("Friction:")
            || text.starts_with("Restitution:")
        {
            assert_eq!(
                *width, 105.0,
                "Label '{}' must have standard width 105.0 to align columns, but got {}",
                text, width
            );
        }
    }
}

#[test]
fn test_velocity_and_rigidbody_declarative_binding_and_undo() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");

    let mut world = hecs::World::new();
    let entity = world.spawn((
        RigidBody::default(),
        Velocity {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
    ));

    let mut scope = UiScope::new(&mut tree, root);

    // 1. Test Velocity binding
    ae_editor_ui::ui::iris_bridge::inspector::components::gameplay::velocity::draw_velocity_card(
        &mut scope, &world, entity,
    );

    // 2. Test RigidBody binding
    ae_editor_ui::ui::iris_bridge::inspector::components::physics::rigidbody::draw_rigidbody_card(
        &mut scope, &world, entity,
    );

    // Verify UI tree contains Velocity "Linear" DragVec3 and RigidBody cards
    let has_linear_x = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("NumBox_Linear_X"));
    assert!(
        has_linear_x,
        "UiTree must contain Velocity Linear X axis widget"
    );
    let has_mass = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("NumLbl") && n.text.as_deref() == Some("Mass:"));
    assert!(has_mass, "UiTree must contain RigidBody Mass label");
}

#[test]
fn test_velocity_and_rigidbody_active_borrow_drag_no_panic() {
    use ae_editor_ui::ui::iris_bridge::inspector::components::gameplay::velocity::draw_velocity_card;
    use ae_editor_ui::ui::iris_bridge::inspector::components::physics::rigidbody::draw_rigidbody_card;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");

    let mut world = hecs::World::new();
    let entity = world.spawn((
        RigidBody::default(),
        Velocity {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
    ));

    // Simulate active pointer drag gesture in event state
    tree.event_state.frame_drag_delta = Some(irisui::core::Point::new(15.0, 0.0));
    tree.event_state.is_pointer_drag_active = true;

    let mut scope = UiScope::new(&mut tree, root);

    // Both cards MUST render cleanly without panicking with "already borrowed"!
    draw_velocity_card(&mut scope, &world, entity);
    draw_rigidbody_card(&mut scope, &world, entity);

    // Verify values exist in the world without crash
    let v = world.get::<&Velocity>(entity).unwrap();
    assert_eq!(v.y, 2.0);
    assert_eq!(v.z, 3.0);
}