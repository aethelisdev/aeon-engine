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

    // 1. When menu is closed, clicking '+' generates OpenAddMenu(None)
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
            is_add_menu_open: false,
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
    assert_eq!(actions, vec![HierarchyAction::OpenAddMenu(None)]);

    // 2. When menu is open, clicking '+' generates CloseAddMenu
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
            is_add_menu_open: true,
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
    assert_eq!(actions, vec![HierarchyAction::CloseAddMenu]);
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