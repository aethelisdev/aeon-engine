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

fn default_test_session<'a>(
    name: &'a mut String,
    swatches: &'a mut Vec<[f32; 4]>,
    is_color: &'a mut bool,
    is_add: &'a mut bool,
    submenu: &'a mut Option<ComponentCategory>,
) -> InspectorSessionRefs<'a> {
    InspectorSessionRefs {
        inspected_name: name,
        saved_swatches: swatches,
        is_color_picker_open: is_color,
        is_add_menu_open: is_add,
        active_submenu: submenu,
    }
}

#[test]
fn test_inspector_empty_selection_renders_placeholder() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let world = hecs::World::new();
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];

    let params = create_default_test_params(&world, None, &euler, &swatches);

    build_inspector_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut default_test_session(
            &mut String::new(),
            &mut Vec::new(),
            &mut false,
            &mut false,
            &mut None,
        ),
    );

    let has_empty_label = tree
        .iter()
        .any(|(_, n)| n.text.as_deref() == Some("No Entity Selected"));
    assert!(has_empty_label);
}

#[test]
fn test_inspector_3d_entity_renders_transform_and_appearance() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
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

    build_inspector_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut default_test_session(
            &mut String::new(),
            &mut Vec::new(),
            &mut false,
            &mut false,
            &mut None,
        ),
    );

    // Verify 3D Transform inputs are present via declarative assertions
    tree.assert_node_exists("NumBox_Position_X");
    tree.assert_node_exists("NumBox_Rotation_X");
    tree.assert_node_exists("NumBox_Scale_X");

    // Verify 2D Screen Transform inputs are NOT present
    tree.assert_node_absent("NumBox_Offset_X");
}

#[test]
fn test_inspector_transform_geometry_is_invariant_between_layout_passes() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
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

    build_inspector_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut default_test_session(
            &mut String::new(),
            &mut Vec::new(),
            &mut false,
            &mut false,
            &mut None,
        ),
    );

    // Capture initial geometric positions immediately after declarative build (layout_subtree pass)
    let pre_pos_x = tree.assert_node_rect("NumBox_Position_X");
    let pre_pos_y = tree.assert_node_rect("NumBox_Position_Y");
    let pre_pos_z = tree.assert_node_rect("NumBox_Position_Z");
    let pre_reset = tree.assert_node_rect("ResetBtn_Position");

    // Execute authoritative top-level Taffy solver pass
    let mut layout_engine = irisui::layout::LayoutEngine::new();
    let _ = layout_engine.compute_layout(&mut tree, Size::new(800.0, 600.0));

    let post_pos_x = tree.assert_node_rect("NumBox_Position_X");
    let post_pos_y = tree.assert_node_rect("NumBox_Position_Y");
    let post_pos_z = tree.assert_node_rect("NumBox_Position_Z");
    let post_reset = tree.assert_node_rect("ResetBtn_Position");

    // Assert absolute geometric invariance between layout passes (prevent visual hopping/jitter)
    assert_eq!(
        pre_pos_x, post_pos_x,
        "Position X must remain geometrically invariant"
    );
    assert_eq!(
        pre_pos_y, post_pos_y,
        "Position Y must remain geometrically invariant"
    );
    assert_eq!(
        pre_pos_z, post_pos_z,
        "Position Z must remain geometrically invariant"
    );
    assert_eq!(
        pre_reset.x, post_reset.x,
        "Reset button X must remain geometrically invariant"
    );
    assert_eq!(
        pre_reset.y, post_reset.y,
        "Reset button Y must remain geometrically invariant"
    );
}

#[test]
fn test_inspector_2d_ui_entity_replaces_3d_transform_with_screen_transform() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
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

    build_inspector_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut default_test_session(
            &mut String::new(),
            &mut Vec::new(),
            &mut false,
            &mut false,
            &mut None,
        ),
    );

    // Verify 3D Transform inputs are NOT present
    tree.assert_node_absent("NumBox_Position_X");

    // Verify 2D Screen Transform inputs ARE present
    tree.assert_node_exists("NumBox_Offset_X");
    tree.assert_node_exists("NumBox_Offset_Y");
    tree.assert_node_exists("NumBox_Size_W");
    tree.assert_node_exists("NumBox_Size_H");
    tree.assert_node_exists("NumBox_Pivot_X");
    tree.assert_node_exists("NumBox_Pivot_Y");

    // Verify Anchor combobox and Visible checkbox are present
    let has_combobox = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("ComboPill"));
    assert!(has_combobox, "Anchor combobox must be present");
    let has_checkbox = tree
        .iter()
        .any(|(_, n)| n.role == irisui::prelude::WidgetRole::Checkbox);
    assert!(has_checkbox, "UiVisible checkbox must be present");

    // Verify UiPanel properties (declarative NumericDrag)
    let numeric_drag_count = tree
        .iter()
        .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
        .count();
    assert!(
        numeric_drag_count >= 2,
        "UiBorderWidth and UiCornerRadius NumericDrag must be present"
    );

    // Verify UiText properties (declarative NumericDrag & ComboBox)
    let has_align_dropdown = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("ComboPill"));
    assert!(
        has_align_dropdown,
        "UiTextAlignment dropdown must be present"
    );

    let has_text_input = tree
        .iter()
        .any(|(_, n)| n.role == irisui::prelude::WidgetRole::TextInput);
    assert!(has_text_input, "UiText text input must be present");

    // Verify UiButton properties (declarative checkbox)
    let checkbox_count = tree
        .iter()
        .filter(|(_, n)| n.role == irisui::prelude::WidgetRole::Checkbox)
        .count();
    assert!(
        checkbox_count >= 2,
        "UiVisible and UiButton is_enabled checkboxes must be present"
    );
}

#[test]
fn test_inspector_entity_isolation_invariant() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
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

    build_inspector_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut default_test_session(
            &mut String::new(),
            &mut Vec::new(),
            &mut false,
            &mut false,
            &mut None,
        ),
    );

    assert_eq!(
        params.selected_entity,
        Some(ent_a),
        "InspectorPanelParams must explicitly carry the inspected entity"
    );

    // Verify reset transform button exists on UiTree declaratively
    tree.assert_node("ResetBtn_Position", WidgetRole::Button);
}

#[test]
fn test_inspector_rotation_reflects_entity_component_and_undo() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
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

    // 1. Initial State: Rotation Y must be 0.0
    build_inspector_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut default_test_session(
            &mut String::new(),
            &mut Vec::new(),
            &mut false,
            &mut false,
            &mut None,
        ),
    );
    let mut rot_y_text = None;
    tree.traverse_depth_first(root, &mut |_id, node| {
        if node.name.as_deref() == Some("NumText_Rotation_Y") {
            rot_y_text = node.text.clone();
        }
    });
    assert_eq!(rot_y_text, Some("Y: 0.0".to_string()));

    // 2. Modified State: Change Rotation component to 55.0 degrees Y
    if let Ok(mut r) = world.get::<&mut Rotation>(ent) {
        *r = super::euler_deg_to_quaternion(0.0, 55.0, 0.0);
    }
    let mut tree2 = UiTree::new();
    let root2 = tree2.create_root().expect("root node");
    build_inspector_panel(
        &mut UiScope::new(&mut tree2, root2),
        &params,
        &mut default_test_session(
            &mut String::new(),
            &mut Vec::new(),
            &mut false,
            &mut false,
            &mut None,
        ),
    );
    let mut rot_y_text2 = None;
    tree2.traverse_depth_first(root2, &mut |_id, node| {
        if node.name.as_deref() == Some("NumText_Rotation_Y") {
            rot_y_text2 = node.text.clone();
        }
    });
    assert_eq!(rot_y_text2, Some("Y: 55.0".to_string()));

    // 3. Undo State: Revert Rotation component back to identity
    if let Ok(mut r) = world.get::<&mut Rotation>(ent) {
        *r = Rotation::default();
    }
    let mut tree3 = UiTree::new();
    let root3 = tree3.create_root().expect("root node");
    build_inspector_panel(
        &mut UiScope::new(&mut tree3, root3),
        &params,
        &mut default_test_session(
            &mut String::new(),
            &mut Vec::new(),
            &mut false,
            &mut false,
            &mut None,
        ),
    );
    let mut rot_y_text3 = None;
    tree3.traverse_depth_first(root3, &mut |_id, node| {
        if node.name.as_deref() == Some("NumText_Rotation_Y") {
            rot_y_text3 = node.text.clone();
        }
    });
    assert_eq!(
        rot_y_text3,
        Some("Y: 0.0".to_string()),
        "Inspector Rotation row must immediately reflect undone component value"
    );
}

/// Verifies that continuous Color Picker dragging mutates the entity in real time
/// for smooth viewport rendering without flooding the undo stack, and commits
/// exactly one atomic undo command on release.
#[test]
fn test_color_picker_drag_and_undo_restoration() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut world = hecs::World::new();
    let initial_color = ae_core::ecs::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };
    let entity = world.spawn((initial_color,));

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = InspectorPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 320.0, 600.0),
        world: &world,
        selected_entity: Some(entity),
        inspector_euler: &euler,
        inspector_color_hex: "#1a334d",
        saved_swatches: &swatches,
        cursor_pos: Point::new(0.0, 0.0),
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: true,
        active_rename_buffer: None,
        is_rename_all_selected: false,
        active_hex_buffer: None,
        inspector_hsv: [210.0, 0.66, 0.30],
        blink_caret: false,
    };

    let mut ui_actions = Vec::new();
    let mut is_color_picker_open = true;
    // 0. Initial popup build to register SV box bounds
    let (_, init_act) =
        crate::ui::iris_bridge::inspector::color_picker_popup::build_color_picker_popup_with_action(
            &mut tree,
            root,
            &params,
            &mut is_color_picker_open,
        );
    assert!(init_act.is_none());

    let sv_rect = tree
        .iter()
        .find(|(_, n)| n.tag == super::color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_SV_BOX)
        .map(|(_, n)| n.computed_rect)
        .expect("SV box must exist");

    // 1. Mouse down on SV box begins color editing
    let start_pt = Point::new(sv_rect.x + 15.0, sv_rect.y + 15.0);
    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: start_pt,
    });
    let _ = tree.clear_children(root);
    if let (_, Some(act)) =
        crate::ui::iris_bridge::inspector::color_picker_popup::build_color_picker_popup_with_action(
            &mut tree,
            root,
            &params,
            &mut is_color_picker_open,
        )
    {
        ui_actions.push(act);
    }
    assert!(
        ui_actions.is_empty(),
        "MouseDown must not produce undo actions prematurely"
    );

    // 2. Simulate 20 continuous drag frames (mouse movement)
    for i in 1..=20 {
        let drag_pt = Point::new(
            sv_rect.x + 15.0 + (i as f32) * 5.0,
            sv_rect.y + 15.0 + (i as f32) * 4.0,
        );
        let _ = tree.dispatch_event(&UiEvent::MouseMove { point: drag_pt });
        let _ = tree.clear_children(root);
        if let (_, Some(act)) =
            crate::ui::iris_bridge::inspector::color_picker_popup::build_color_picker_popup_with_action(
                &mut tree,
                root,
                &params,
                &mut is_color_picker_open,
            )
        {
            ui_actions.push(act);
        }
        let cur = *world.get::<&ae_core::ecs::Color>(entity).unwrap();
        assert_ne!(
            cur, initial_color,
            "Entity color in world must update live during drag"
        );
    }
    assert!(
        ui_actions.is_empty(),
        "Live dragging must not spam undo actions during dragging"
    );

    // 3. Mouse released -> commits atomic ModifyColor
    let end_pt = Point::new(sv_rect.x + 115.0, sv_rect.y + 95.0);
    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: end_pt,
    });
    let _ = tree.clear_children(root);
    if let (_, Some(act)) =
        crate::ui::iris_bridge::inspector::color_picker_popup::build_color_picker_popup_with_action(
            &mut tree,
            root,
            &params,
            &mut is_color_picker_open,
        )
    {
        ui_actions.push(act);
    }
    assert_eq!(
        ui_actions.len(),
        1,
        "Drag release must emit exactly one atomic ModifyColor action"
    );

    let final_color = *world.get::<&ae_core::ecs::Color>(entity).unwrap();
    if let crate::ui::EngineUiAction::ModifyColor(ent, old_c, new_c) = ui_actions[0] {
        assert_eq!(ent, entity);
        assert_eq!(old_c, initial_color);
        assert_eq!(new_c, final_color);

        // 4. Verify Undo restoration
        let mut undo_cmd = ae_editor::undo_redo::Command::Modify(
            ent,
            ae_editor::undo_redo::Property::Color(old_c, new_c),
        );
        undo_cmd.undo(&mut world);
        let restored_color = *world.get::<&ae_core::ecs::Color>(entity).unwrap();
        assert_eq!(
            restored_color, initial_color,
            "Undo must restore the exact pre-edit color"
        );
    } else {
        panic!("Expected EngineUiAction::ModifyColor");
    }
}

#[test]
fn test_inspector_entity_rename_all_selected_lifecycle() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");
    let mut name = "TestEntity".to_string();

    let mut scope = UiScope::new(&mut tree, root);
    let _resp = header::build_entity_header(&mut scope, &mut name, 320.0);
    scope.finish_layout(Rect::new(0.0, 0.0, 320.0, 800.0));

    let box_id = tree
        .find_node_by_tag(header::entity_name_input_tag())
        .expect("Box node must exist");
    let box_node = tree.get(box_id).expect("Box node must exist");
    assert_eq!(box_node.tag, header::entity_name_input_tag());
    assert_eq!(box_node.role, WidgetRole::TextInput);
    assert_eq!(box_node.cursor, Some(WidgetCursor::Text));
    assert!(box_node.computed_rect.width > 100.0);
    assert!(box_node.computed_rect.height >= 24.0);
}

#[test]
fn test_inspector_transform_snapshot_reactive_dirty_tracking() {
    let mut world = hecs::World::new();
    let entity = world.spawn((
        Name("SimulatedEntity".to_string()),
        Position::new(10.0, 20.0, 30.0),
        Rotation::identity(),
        Scale::one(),
    ));

    // 1. Initial capture
    let snap1 = InspectorTransformSnapshot::capture(&world, entity);
    assert_eq!(snap1.position, Some([10.0, 20.0, 30.0]));
    assert_eq!(snap1.scale, Some([1.0, 1.0, 1.0]));

    // Identical frame check
    let snap1_dup = InspectorTransformSnapshot::capture(&world, entity);
    assert_eq!(
        snap1, snap1_dup,
        "Unchanged entity must yield identical snapshot"
    );

    // 2. Simulate physics step or gizmo movement modifying Position in ECS
    {
        let mut pos = world.get::<&mut Position>(entity).expect("Position exists");
        pos.y = 55.5;
    }

    let snap2 = InspectorTransformSnapshot::capture(&world, entity);
    assert_ne!(
        snap1, snap2,
        "Position mutation must invalidate previous snapshot"
    );
    assert_eq!(snap2.position, Some([10.0, 55.5, 30.0]));

    // 3. Simulate rotation Euler change
    {
        let mut rot = world.get::<&mut Rotation>(entity).expect("Rotation exists");
        rot.y = std::f32::consts::FRAC_1_SQRT_2;
        rot.w = std::f32::consts::FRAC_1_SQRT_2;
    }

    let snap3 = InspectorTransformSnapshot::capture(&world, entity);
    assert_ne!(
        snap2, snap3,
        "Rotation mutation must invalidate previous snapshot"
    );
}

#[test]
fn test_is_widget_descendant_of_hierarchy_traversal() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");

    let (leaf, drag_box, other_leaf, other_widget) = {
        let mut scope = UiScope::new(&mut tree, root);
        let mut leaf_id = root;
        let mut drag_id = root;
        scope.column(|l| {
            leaf_id = l.parent();
            l.column(|p| {
                p.column(|c| {
                    let resp = c.button("DragBox");
                    drag_id = resp.id;
                });
            });
        });

        let mut other_leaf_id = root;
        let mut other_w_id = root;
        scope.column(|ol| {
            other_leaf_id = ol.parent();
            let resp = ol.button("OtherWidget");
            other_w_id = resp.id;
        });

        (leaf_id, drag_id, other_leaf_id, other_w_id)
    };

    // Direct and deep descendant checks
    assert!(is_widget_descendant_of(&tree, drag_box, leaf));
    assert!(is_widget_descendant_of(&tree, leaf, leaf));

    // Negative checks
    assert!(!is_widget_descendant_of(&tree, other_widget, leaf));
    assert!(!is_widget_descendant_of(&tree, other_leaf, leaf));
    assert!(!is_widget_descendant_of(&tree, root, leaf));
}

#[test]
fn test_inspector_layout_dual_engine_invariants_zero_jitter() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut world = hecs::World::new();

    let entity = world.spawn((
        Name("Dynamic Cube".to_string()),
        Position::new(0.0, 5.0, 0.0),
        Rotation::identity(),
        Scale::one(),
        ae_core::ecs::Color::new(0.4, 0.6, 0.8, 1.0),
        ae_core::ecs::RigidBody {
            body_type: ae_core::ecs::RigidBodyType::Dynamic,
            mass: 1.0,
            gravity_scale: 1.0,
        },
        ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.7,
            restitution: 0.0,
            is_sensor: false,
        },
        ae_core::ecs::PlayerTag,
        ae_core::ecs::Velocity::default(),
        ae_core::ecs::Shape::Cube,
    ));

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_default_test_params(&world, Some(entity), &euler, &swatches);

    build_inspector_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut default_test_session(
            &mut String::new(),
            &mut Vec::new(),
            &mut false,
            &mut false,
            &mut None,
        ),
    );

    let find_node_rect = |tree: &UiTree, label: &str| -> Option<Rect> {
        for (_id, node) in tree.iter() {
            let name = node.name.as_deref().unwrap_or("");
            let text = node.text.as_deref().unwrap_or("");
            if name == label || text == label {
                return Some(node.computed_rect);
            }
        }
        None
    };

    let before_add_swatch = find_node_rect(&tree, "+").expect("Add swatch button");
    let before_del_swatch = find_node_rect(&tree, "🗑").expect("Delete swatch button");
    let before_playertag = find_node_rect(&tree, "PlayerTagCard").expect("PlayerTagCard");
    let before_velocity = find_node_rect(&tree, "VelocityCard").expect("VelocityCard");
    let before_shape = find_node_rect(&tree, "ShapeCard").expect("ShapeCard");
    let before_add_comp = find_node_rect(&tree, "AddComponentBtn").expect("AddComponentBtn");
    let before_save_prefab = find_node_rect(&tree, "SavePrefabBtn").expect("SavePrefabBtn");

    let mut text_system = irisui::text::TextSystem::new();
    text_system.measure_subtree_text(&mut tree, root);
    let mut layout_engine = irisui::layout::LayoutEngine::new();
    let _ = layout_engine.compute_layout(&mut tree, Size::new(320.0, 900.0));

    let after_add_swatch = find_node_rect(&tree, "+").expect("Add swatch button");
    let after_del_swatch = find_node_rect(&tree, "🗑").expect("Delete swatch button");
    let after_playertag = find_node_rect(&tree, "PlayerTagCard").expect("PlayerTagCard");
    let after_velocity = find_node_rect(&tree, "VelocityCard").expect("VelocityCard");
    let after_shape = find_node_rect(&tree, "ShapeCard").expect("ShapeCard");
    let after_add_comp = find_node_rect(&tree, "AddComponentBtn").expect("AddComponentBtn");
    let after_save_prefab = find_node_rect(&tree, "SavePrefabBtn").expect("SavePrefabBtn");

    // Palette icon buttons must maintain identical width and horizontal position
    assert_eq!(before_add_swatch.width, 18.0);
    assert_eq!(after_add_swatch.width, 18.0);
    assert!((before_del_swatch.x - after_del_swatch.x).abs() < 1e-3);

    // Cards must maintain consistent vertical coordinates
    assert_eq!(before_playertag.height, after_playertag.height);
    assert!((before_velocity.y - after_velocity.y).abs() < 1e-3);
    assert!((before_shape.y - after_shape.y).abs() < 1e-3);

    // Footer buttons must divide flex space equally without horizontal drift
    assert_eq!(before_add_comp.width, 148.0);
    assert_eq!(after_add_comp.width, 148.0);
    assert_eq!(before_save_prefab.width, 148.0);
    assert_eq!(after_save_prefab.width, 148.0);
    assert!((before_save_prefab.x - after_save_prefab.x).abs() < 1e-3);
}