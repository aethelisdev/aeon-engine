// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy `➕` Add Menu Lifecycle & Event Integration Tests
//!
//! Verifies the end-to-end integration of the Scene Hierarchy Add Entity toolbar button,
//! cascading menu activation, submenu transitions, entity spawn dispatching, and overlay dismissal.

use super::header::build_hierarchy_header;
use super::types::{
    AddSubmenuId, HIERARCHY_TAG_ADD_BUTTON, HierarchyAction, HierarchyPanelParams,
    HierarchyPanelState,
};
use hecs::World;
use irisui::prelude::*;
use irisui::widgets::controls::FocusState;

#[test]
fn test_hierarchy_add_button_toggle_actions() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node must be created");
    let world = World::new();
    let focus_state = FocusState::new();

    // 1. When menu is closed, clicking '+' activates HIERARCHY_TAG_ADD_BUTTON in UiTree
    tree.event_state.frame_clicked_tag = Some(HIERARCHY_TAG_ADD_BUTTON);
    let mut actions = Vec::new();
    {
        let params = HierarchyPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: None,
            search_query: "",
            is_editing: true,
            is_2d: false,
            active_submenu: None,
            active_sub_submenu: None,
            active_context_menu: None,
            cursor_pos: Point::new(10.0, 10.0),
            blink_caret: false,
            ctrl_held: false,
            shift_held: false,
            focus_state: &focus_state,
            cursor_byte_idx: 0,
        };
        let mut scope = UiScope::new(&mut tree, root_id);
        build_hierarchy_header(&mut scope, &params, &mut actions);
    }
    assert!(tree.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON));

    // 2. When menu is open, clicking '+' again toggles HIERARCHY_TAG_ADD_BUTTON off
    tree.event_state.frame_clicked_tag = Some(HIERARCHY_TAG_ADD_BUTTON);
    actions.clear();
    {
        let params = HierarchyPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: None,
            search_query: "",
            is_editing: true,
            is_2d: false,
            active_submenu: None,
            active_sub_submenu: None,
            active_context_menu: None,
            cursor_pos: Point::new(10.0, 10.0),
            blink_caret: false,
            ctrl_held: false,
            shift_held: false,
            focus_state: &focus_state,
            cursor_byte_idx: 0,
        };
        let mut scope = UiScope::new(&mut tree, root_id);
        build_hierarchy_header(&mut scope, &params, &mut actions);
    }
    assert!(!tree.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON));
}

#[test]
fn test_hierarchy_state_open_close_and_submenu_flow() {
    let mut state = HierarchyPanelState::default();
    assert!(!state.is_add_menu_open);

    // Simulate OpenAddMenu processing
    state.is_add_menu_open = true;
    state.active_submenu = None;
    state.active_sub_submenu = None;
    assert!(state.is_add_menu_open);

    // Hover or click on 3D Objects submenu
    assert!(state.activate_submenu(AddSubmenuId::Objects3D));
    assert_eq!(state.active_submenu, Some(AddSubmenuId::Objects3D));

    // Resolve tag 101 into SpawnShape(Cube)
    let action = super::add_menu::get_hierarchy_add_menu_action(101);
    assert_eq!(
        action,
        Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Cube))
    );

    // On item click, menu closes
    state.is_add_menu_open = false;
    state.active_submenu = None;
    state.active_sub_submenu = None;
    assert!(!state.is_add_menu_open);
    assert_eq!(state.active_submenu, None);
}

#[test]
fn test_hierarchy_add_menu_outside_click() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let world = World::new();
    let focus_state = FocusState::new();

    let mut actions = Vec::new();
    let panel_rect = Rect::new(0.0, 0.0, 300.0, 600.0);
    let add_btn_rect = Rect::new(240.0, 4.0, 24.0, 24.0);

    // 1. Create Hierarchy Add button anchor in tree
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.container_tagged(
            "HierarchyAddButton",
            Style::new()
                .position_absolute()
                .left(add_btn_rect.x)
                .top(add_btn_rect.y)
                .width(add_btn_rect.width)
                .height(add_btn_rect.height),
            WidgetRole::Button,
            HIERARCHY_TAG_ADD_BUTTON,
            |_btn| {},
        );
        scope.finish_layout(panel_rect);
    }

    let params = HierarchyPanelParams {
        panel_rect,
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: false,
        active_submenu: None,
        active_sub_submenu: None,
        active_context_menu: None,
        cursor_pos: Point::new(100.0, 100.0),
        blink_caret: false,
        ctrl_held: false,
        shift_held: false,
        focus_state: &focus_state,
        cursor_byte_idx: 0,
    };

    tree.open_popup(HIERARCHY_TAG_ADD_BUTTON);
    // 2. Build cascading add menu
    {
        let mut scope = UiScope::new(&mut tree, root);
        super::add_menu::draw_hierarchy_add_menu(&mut scope, &params, &mut actions);
    }

    // 3. Verify HIERARCHY_TAG_ADD_BUTTON is registered as active popup tag
    assert!(
        tree.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON),
        "Hierarchy add button tag must be registered as active popup tag"
    );

    // 4. Click on trigger button: inside popup check
    let btn_click = Point::new(250.0, 10.0);
    assert!(
        tree.is_point_inside_popup(btn_click),
        "Click on Add button must be recognized as inside popup"
    );
    assert!(
        !tree.dismiss_popups_on_outside_click(btn_click),
        "Click on Add button must not dismiss popup"
    );
    assert!(tree.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON));

    // 5. Click outside: dismiss popups
    let outside_point = Point::new(800.0, 800.0);
    assert!(
        !tree.is_point_inside_popup(outside_point),
        "Point (800, 800) must be outside popup"
    );
    assert!(
        tree.dismiss_popups_on_outside_click(outside_point),
        "Outside click must dismiss popup"
    );
    assert!(
        !tree.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON),
        "Active popup tag must be cleared on dismissal"
    );
    assert!(tree.popups_dismissed());

    // 6. Next frame: scope.popups_dismissed() resets hierarchy panel state
    {
        let scope = UiScope::new(&mut tree, root);
        assert!(scope.popups_dismissed());
        let mut hierarchy_state = super::types::HierarchyState {
            is_add_menu_open: true,
            active_submenu: Some(AddSubmenuId::Objects3D),
            active_sub_submenu: Some(AddSubmenuId::Objects3D),
            ..Default::default()
        };

        if scope.popups_dismissed() {
            hierarchy_state.is_add_menu_open = false;
            hierarchy_state.active_submenu = None;
            hierarchy_state.active_sub_submenu = None;
            hierarchy_state.active_context_menu = None;
        }

        assert!(!hierarchy_state.is_add_menu_open);
        assert!(hierarchy_state.active_submenu.is_none());
        assert!(hierarchy_state.active_sub_submenu.is_none());
    }
}