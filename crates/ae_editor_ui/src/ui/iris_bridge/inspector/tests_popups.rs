// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Floating Popups Unit Tests
//!
//! Verifies 100% declarative [`UiScope`] floating popup construction, layout isolation,
//! and $O(1)$ semantic tag hit-testing for ComboBox dropdowns, Cascading Add Component menus,
//! and 2D HSV Color Pickers.
//!

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
        hovered_tag: None,
    }
}

#[test]
fn test_inspector_dropdown_popup_declarative_scope_and_hit_testing() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut world = hecs::World::new();
    let entity = world.spawn((Position::default(),));

    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let mut params = create_default_test_params(&world, Some(entity), &euler, &swatches);
    params.active_dropdown = Some(InspectorDropdownId::RigidBodyType);

    // Build trigger anchor via declarative UiScope
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.container_tagged(
            "AnchorDropdownTrigger",
            Style::new()
                .position_absolute()
                .left(100.0)
                .top(100.0)
                .width(120.0)
                .height(24.0),
            WidgetRole::Button,
            encode_inspector_dropdown_tag(InspectorDropdownId::RigidBodyType),
            |btn| {
                btn.label_styled_passive(
                    "DropdownLabel",
                    "Dynamic",
                    11.0,
                    Color::WHITE,
                    TextAlign::Left,
                    Style::new().height(20.0),
                );
            },
        );
        scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));
    }

    // Build the dropdown popup via declarative UiScope
    dropdown_popup::build_inspector_dropdown_popup(&mut tree, root, &params);

    // Option 0: "Dynamic" (tag = TAG_INSPECTOR_DROPDOWN_ITEM_BASE)
    let opt0_point = Point::new(110.0, 100.0 + 24.0 + 6.0 + 10.0);
    let hit0 = tree
        .hit_test_target(opt0_point)
        .expect("Must hit dropdown item 0");
    assert_eq!(hit0.layer, UiLayer::Popup);
    assert_eq!(hit0.role, WidgetRole::DropdownItem);
    assert_eq!(hit0.tag, tags::TAG_INSPECTOR_DROPDOWN_ITEM_BASE);

    // Option 1: "Kinematic" (tag = TAG_INSPECTOR_DROPDOWN_ITEM_BASE + 1)
    let opt1_point = Point::new(110.0, 100.0 + 24.0 + 6.0 + 22.0 + 10.0);
    let hit1 = tree
        .hit_test_target(opt1_point)
        .expect("Must hit dropdown item 1");
    assert_eq!(hit1.layer, UiLayer::Popup);
    assert_eq!(hit1.role, WidgetRole::DropdownItem);
    assert_eq!(hit1.tag, tags::TAG_INSPECTOR_DROPDOWN_ITEM_BASE + 1);
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

    // Build Add Component button anchor via declarative UiScope
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
    add_menu::build_add_component_menu(&mut tree, root, &params);

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

    // Build Swatch trigger anchor via declarative UiScope
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
    color_picker_popup::build_color_picker_popup(&mut tree, root, &params);

    // Verify popup elements exist with UiLayer::Popup
    let mut sv_box_found = false;
    let mut hue_bar_found = false;
    let mut close_btn_found = false;

    for (id, node) in tree.iter() {
        if tree.effective_layer(id) == UiLayer::Popup {
            if node.tag == tags::TAG_INSPECTOR_COLOR_PICKER_SV_BOX {
                sv_box_found = true;
                assert!(node.computed_rect.width >= 150.0);
                assert!(node.computed_rect.height >= 120.0);
            } else if node.tag == tags::TAG_INSPECTOR_COLOR_PICKER_HUE_BAR {
                hue_bar_found = true;
                assert!(node.computed_rect.width >= 15.0);
                assert!(node.computed_rect.height >= 120.0);
            } else if node.tag == tags::TAG_INSPECTOR_COLOR_PICKER_CLOSE {
                close_btn_found = true;
            }
        }
    }

    assert!(sv_box_found, "2D SV Box must exist in Popup layer");
    assert!(hue_bar_found, "Hue Bar must exist in Popup layer");
    assert!(close_btn_found, "Close Button must exist in Popup layer");
}