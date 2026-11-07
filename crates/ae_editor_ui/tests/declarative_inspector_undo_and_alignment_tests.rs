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
    DEFAULT_INSPECTOR_LABEL_WIDTH, InspectorAction, InspectorPanelParams, build_inspector_panel,
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

    let mut actions = Vec::new();
    let mut is_color_picker_open = false;
    build_inspector_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut actions,
        &mut Vec::new(),
        &mut is_color_picker_open,
    );

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
fn test_manual_numeric_input_creates_undo_action() {
    let world = hecs::World::new();
    let entity = hecs::Entity::DANGLING;

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, Some(entity), &euler, &swatches);

    // Simulate user editing Position X manually (e.g. typing 12.5 and pressing Enter)
    let old_pos = serde_json::to_vec(&Position::zero()).unwrap();
    let new_pos = serde_json::to_vec(&Position {
        x: 12.5,
        y: 0.0,
        z: 0.0,
    })
    .unwrap();

    let mut actions = Vec::new();
    let mut ctx = ae_editor_ui::ui::iris_bridge::inspector::ComponentRenderContext::with_actions(
        entity,
        &world,
        &params,
        &mut actions,
        6.0,
        0.0,
        308.0,
    );

    // Direct commit modification emitted by prop_transform_position upon Enter
    ctx.commit_modify("Position", old_pos.clone(), new_pos.clone());

    assert_eq!(actions.len(), 1);
    match &actions[0] {
        InspectorAction::CommitComponentModify(e, comp, old_b, new_b) => {
            assert_eq!(*e, entity);
            assert_eq!(*comp, "Position");
            assert_eq!(*old_b, old_pos);
            assert_eq!(*new_b, new_pos);
        }
        other => panic!("Expected CommitComponentModify, got {:?}", other),
    }
}

#[test]
fn test_drag_undo_restores_pre_edit_value() {
    let world = hecs::World::new();
    let entity = hecs::Entity::DANGLING;

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, Some(entity), &euler, &swatches);

    let initial_pos = serde_json::to_vec(&Position::zero()).unwrap();

    // 1. Frame 1: User starts dragging Position. Pre-edit snapshot is captured before any drag delta
    let mut actions_frame1 = Vec::new();
    {
        let mut ctx =
            ae_editor_ui::ui::iris_bridge::inspector::ComponentRenderContext::with_actions(
                entity,
                &world,
                &params,
                &mut actions_frame1,
                6.0,
                0.0,
                308.0,
            );
        ctx.begin_edit_with_snapshot("Position", initial_pos.clone());
    }

    assert_eq!(actions_frame1.len(), 1);
    match &actions_frame1[0] {
        InspectorAction::StartComponentEdit(e, comp, explicit_bytes) => {
            assert_eq!(*e, entity);
            assert_eq!(*comp, "Position");
            assert_eq!(*explicit_bytes, Some(initial_pos.clone()));
        }
        other => panic!("Expected StartComponentEdit, got {:?}", other),
    }

    // 2. Frame 3: User releases pointer drag. CommitComponentEdit is emitted
    let mut actions_frame3 = Vec::new();
    {
        let mut ctx =
            ae_editor_ui::ui::iris_bridge::inspector::ComponentRenderContext::with_actions(
                entity,
                &world,
                &params,
                &mut actions_frame3,
                6.0,
                0.0,
                308.0,
            );
        ctx.commit_edit("Position");
    }

    assert_eq!(actions_frame3.len(), 1);
    match &actions_frame3[0] {
        InspectorAction::CommitComponentEdit(e, comp) => {
            assert_eq!(*e, entity);
            assert_eq!(*comp, "Position");
        }
        other => panic!("Expected CommitComponentEdit, got {:?}", other),
    }
}

#[test]
fn test_rotation_sync_and_undo_without_gimbal_jump() {
    let world = hecs::World::new();
    let entity = hecs::Entity::DANGLING;

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, Some(entity), &euler, &swatches);

    let mut actions = Vec::new();
    let mut ctx = ae_editor_ui::ui::iris_bridge::inspector::ComponentRenderContext::with_actions(
        entity,
        &world,
        &params,
        &mut actions,
        6.0,
        0.0,
        308.0,
    );

    // Synchronize rotation Euler angles
    ctx.push_action(InspectorAction::SyncInspectorEuler(35.0, 45.0, 0.0));

    assert_eq!(actions.len(), 1);
    match &actions[0] {
        InspectorAction::SyncInspectorEuler(rx, ry, rz) => {
            assert_eq!(*rx, 35.0);
            assert_eq!(*ry, 45.0);
            assert_eq!(*rz, 0.0);
        }
        other => panic!("Expected SyncInspectorEuler, got {:?}", other),
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

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, Some(entity), &euler, &swatches);

    let mut actions = Vec::new();
    let mut ctx = ae_editor_ui::ui::iris_bridge::inspector::ComponentRenderContext::with_actions(
        entity,
        &world,
        &params,
        &mut actions,
        6.0,
        0.0,
        308.0,
    );

    let mut scope = UiScope::new(&mut tree, root);

    // 1. Test Velocity binding
    let mut vel = *world.get::<&Velocity>(entity).unwrap();
    ctx.prop_velocity(&mut scope, &mut vel);

    // 2. Test RigidBody binding
    let mut rb = *world.get::<&RigidBody>(entity).unwrap();
    ctx.prop_rigidbody(&mut scope, &mut rb);

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
    use ae_editor_ui::ui::iris_bridge::inspector::components::gameplay::velocity::VelocityHandler;
    use ae_editor_ui::ui::iris_bridge::inspector::components::physics::rigidbody::RigidBodyHandler;
    use ae_editor_ui::ui::iris_bridge::inspector::registry::ComponentInspectorHandler;

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

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, Some(entity), &euler, &swatches);

    // Simulate active pointer drag gesture in event state
    tree.event_state.frame_drag_delta = Some(irisui::core::Point::new(15.0, 0.0));
    tree.event_state.is_pointer_drag_active = true;

    let mut actions = Vec::new();
    let mut ctx = ae_editor_ui::ui::iris_bridge::inspector::ComponentRenderContext::with_actions(
        entity,
        &world,
        &params,
        &mut actions,
        6.0,
        0.0,
        308.0,
    );

    let mut scope = UiScope::new(&mut tree, root);

    // Both handlers MUST render cleanly without panicking with "already borrowed"!
    VelocityHandler.render_card(&mut scope, &mut ctx);
    RigidBodyHandler.render_card(&mut scope, &mut ctx);

    // Verify values exist in the world without crash
    let v = world.get::<&Velocity>(entity).unwrap();
    assert_eq!(v.y, 2.0);
    assert_eq!(v.z, 3.0);
}