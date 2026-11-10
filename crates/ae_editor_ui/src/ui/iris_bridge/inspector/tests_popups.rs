// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Floating Popups Unit Tests
//!
//! Verifies 100% declarative [`UiScope`] floating popup construction, layout isolation,
//! and $O(1)$ semantic tag hit-testing for ComboBox dropdowns, Cascading Add Component menus,
//! and 2D HSV Color Pickers.

use super::*;
use ae_core::ecs::Position;
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
        inspector_hsv: [0.0, 0.0, 1.0],
    }
}

#[test]
fn test_inspector_add_menu_cascading_declarative_scope_and_hit_testing() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut world = hecs::World::new();
    let entity = world.spawn((Position::default(),));

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let mut params = create_default_test_params(&world, Some(entity), &euler, &swatches);
    params.is_add_menu_open = true;
    params.active_submenu = Some(ComponentCategory::Physics);

    // Build Add Component button anchor via declarative UiScope directly in tree
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.container_tagged(
            "AddComponentButton",
            Style::new()
                .position_absolute()
                .left(50.0)
                .top(500.0)
                .width(220.0)
                .height(28.0),
            WidgetRole::Button,
            TAG_INSPECTOR_ADD_COMPONENT,
            |btn| {
                btn.label_styled_passive(
                    "AddButtonLabel",
                    "➕ Add Component",
                    12.0,
                    Color::WHITE,
                    TextAlign::Center,
                    Style::new().height(24.0),
                );
            },
        );
        scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));
    }

    // Build the cascading add menu via declarative UiScope
    let mut is_add_menu_open = true;
    let mut active_submenu = Some(ComponentCategory::Physics);
    add_menu::draw_add_component_menu(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut is_add_menu_open,
        &mut active_submenu,
    );

    // Find Physics category card item
    let mut physics_cat_node = None;
    for (_id, node) in tree.iter() {
        if node.layer == UiLayer::Popup
            && node.role == WidgetRole::DropdownItem
            && node.tag == ComponentCategory::Physics.to_tag()
        {
            physics_cat_node = Some(node.clone());
            break;
        }
    }
    assert!(
        physics_cat_node.is_some(),
        "Physics category item must be constructed in Level 0 menu"
    );

    // Hit-test an item inside the Submenu (Level 1)
    let item_node = tree.iter().find(|(_, n)| {
        n.role == WidgetRole::DropdownItem
            && add_menu::resolve_component_name_from_tag(n.tag).is_some()
    });
    let (_, hit_node) = item_node.expect("Level 1 submenu item must exist");
    let hit_center = Point::new(
        hit_node.computed_rect.x + hit_node.computed_rect.width * 0.5,
        hit_node.computed_rect.y + hit_node.computed_rect.height * 0.5,
    );

    let hit = tree
        .hit_test_target(hit_center)
        .expect("Must hit component item in submenu");
    assert_eq!(hit.layer, UiLayer::Popup);
    assert_eq!(hit.role, WidgetRole::DropdownItem);

    let resolved_name = add_menu::resolve_component_name_from_tag(hit.tag);
    assert!(
        resolved_name.is_some(),
        "Hit tag must resolve to a valid component name"
    );
}

#[test]
fn test_inspector_color_picker_popup_declarative_scope_and_hit_testing() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut world = hecs::World::new();
    let entity = world.spawn((Position::default(),));

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let mut params = create_default_test_params(&world, Some(entity), &euler, &swatches);
    params.is_color_picker_open = true;
    params.inspector_hsv = [180.0, 0.5, 0.8];
    params.inspector_color_hex = "#33cccc";

    // Build Swatch trigger anchor via declarative UiScope directly in tree
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.container_tagged(
            "ColorSwatchAnchor",
            Style::new()
                .position_absolute()
                .left(100.0)
                .top(200.0)
                .width(48.0)
                .height(20.0),
            WidgetRole::ColorSwatch,
            appearance_color_swatch_tag(),
            |btn| {
                btn.label_styled_passive(
                    "SwatchLabel",
                    "■",
                    11.0,
                    Color::WHITE,
                    TextAlign::Center,
                    Style::new().height(18.0),
                );
            },
        );
        scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));
    }

    // Build color picker popup via declarative UiScope
    let mut is_color_picker_open = true;
    let mut scope = UiScope::new(&mut tree, root);
    color_picker_popup::build_color_picker_popup(&mut scope, &params, &mut is_color_picker_open);

    // Verify popup elements exist with UiLayer::Popup
    let mut sv_box_found = false;
    let mut hue_bar_found = false;
    let mut close_btn_found = false;

    for (id, node) in tree.iter() {
        if tree.effective_layer(id) == UiLayer::Popup {
            if node.tag == color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_SV_BOX {
                sv_box_found = true;
                assert!(node.computed_rect.width >= 150.0);
                assert!(node.computed_rect.height >= 120.0);
            } else if node.tag == color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_HUE_BAR {
                hue_bar_found = true;
                assert!(node.computed_rect.width >= 15.0);
                assert!(node.computed_rect.height >= 120.0);
            } else if node.tag == color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_CLOSE {
                close_btn_found = true;
            }
        }
    }

    assert!(sv_box_found, "2D SV Box must exist in Popup layer");
    assert!(hue_bar_found, "Hue Bar must exist in Popup layer");
    assert!(close_btn_found, "Close Button must exist in Popup layer");

    // Verify hardware hit-testing hits the SV box and returns the full container's bounding rectangle
    let (_, sv_node) = tree
        .iter()
        .find(|(_, n)| n.tag == color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_SV_BOX)
        .expect("SV box must exist");
    let test_point = Point::new(
        sv_node.computed_rect.x + 20.0,
        sv_node.computed_rect.y + 20.0,
    );
    let hit = tree
        .hit_test_target(test_point)
        .expect("SV box point must hit a target");
    assert_eq!(
        hit.tag,
        color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_SV_BOX,
        "Clicking inside SV box must hit TAG_INSPECTOR_COLOR_PICKER_SV_BOX"
    );
    assert_eq!(
        hit.rect, sv_node.computed_rect,
        "Hit rect must match the full SV box container, not an internal cell"
    );
}

#[test]
fn test_clicking_appearance_color_swatch_does_not_double_toggle() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut world = hecs::World::new();
    let entity = world.spawn((Position::default(),));
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_default_test_params(&world, Some(entity), &euler, &swatches);

    let mut name = String::new();
    let mut swatches = Vec::new();
    let mut is_color_picker_open = false;
    let mut is_add_menu_open = false;
    let mut active_submenu = None;
    let mut session = super::types::InspectorSessionRefs {
        inspected_name: &mut name,
        saved_swatches: &mut swatches,
        is_color_picker_open: &mut is_color_picker_open,
        is_add_menu_open: &mut is_add_menu_open,
        active_submenu: &mut active_submenu,
    };
    let _ = panel::build_inspector_panel(&mut UiScope::new(&mut tree, root), &params, &mut session);
    let _ = LayoutEngine::new().compute_layout(&mut tree, Size::new(320.0, 900.0));

    let swatch_rect = tree
        .node_rect_tagged(appearance_color_swatch_tag(), Point::ZERO)
        .expect("Color swatch must exist in Inspector panel");
    let click_pt = Point::new(swatch_rect.x + 4.0, swatch_rect.y + 4.0);

    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        point: click_pt,
        button: MouseButton::Left,
    });
    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        point: click_pt,
        button: MouseButton::Left,
    });

    assert_eq!(
        tree.event_state.frame_clicked_tag,
        Some(appearance_color_swatch_tag()),
        "Swatch click must record semantic appearance_color_swatch_tag in event state"
    );

    tree.clear();
    let root2 = tree.create_root().expect("root2");
    let _ =
        panel::build_inspector_panel(&mut UiScope::new(&mut tree, root2), &params, &mut session);

    assert!(
        is_color_picker_open,
        "Clicking Object Color swatch must directly toggle is_color_picker_open to true"
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
        inspector_hsv: [210.0, 0.66, 0.30],
    };

    let mut ui_actions = Vec::new();
    let mut is_color_picker_open = true;
    // 0. Initial popup build to register SV box bounds
    let (_, init_act) =
        crate::ui::iris_bridge::inspector::color_picker_popup::build_color_picker_popup_with_action(
            &mut UiScope::new(&mut tree, root),
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
            &mut UiScope::new(&mut tree, root),
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
                &mut UiScope::new(&mut tree, root),
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
            &mut UiScope::new(&mut tree, root),
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

/// Verifies that clicking the close button on the color picker popup toggles `is_color_picker_open` to false.
#[test]
fn test_color_picker_popup_close_button_interaction() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let world = hecs::World::new();
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let mut params = create_default_test_params(&world, None, &euler, &swatches);
    params.is_color_picker_open = true;

    let mut is_color_picker_open = true;
    {
        let mut scope = UiScope::new(&mut tree, root);
        color_picker_popup::build_color_picker_popup(
            &mut scope,
            &params,
            &mut is_color_picker_open,
        );
    }
    let _ = LayoutEngine::new().compute_layout(&mut tree, Size::new(800.0, 600.0));

    let close_rect = tree
        .node_rect_tagged(
            color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_CLOSE,
            Point::ZERO,
        )
        .expect("Close button must carry TAG_INSPECTOR_COLOR_PICKER_CLOSE");
    let click_pt = Point::new(close_rect.x + 2.0, close_rect.y + 2.0);

    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        point: click_pt,
        button: MouseButton::Left,
    });
    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        point: click_pt,
        button: MouseButton::Left,
    });

    assert_eq!(
        tree.event_state.frame_clicked_tag,
        Some(color_picker_popup::TAG_INSPECTOR_COLOR_PICKER_CLOSE),
        "Close button click must record tag in event state"
    );

    tree.clear();
    let root2 = tree.create_root().expect("root2");
    {
        let mut scope = UiScope::new(&mut tree, root2);
        color_picker_popup::build_color_picker_popup(
            &mut scope,
            &params,
            &mut is_color_picker_open,
        );
    }

    assert!(
        !is_color_picker_open,
        "Clicking close button must set is_color_picker_open to false"
    );
}