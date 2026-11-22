// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy `➕` Add Menu Lifecycle & Event Integration Tests
//!
//! Verifies the end-to-end integration of the Scene Hierarchy Add Entity toolbar button,
//! `UiTree` SSOT popup/submenu activation, zero-allocation static cascading entries,
//! direct [`ActionBus`] entity spawn dispatching, and outside-click dismissal.

use super::header::build_hierarchy_header;
use super::types::{AddSubmenuId, HIERARCHY_TAG_ADD_BUTTON};
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

#[test]
fn test_hierarchy_add_button_toggle_actions() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node must be created");
    let mut search_input = TextInputState::default();
    let mut bus = ActionBus::new();

    // 1. Clicking '+' activates HIERARCHY_TAG_ADD_BUTTON in UiTree SSOT
    tree.event_state.frame_clicked_tag = Some(HIERARCHY_TAG_ADD_BUTTON.raw());
    {
        let mut scope = UiScope::new(&mut tree, root_id);
        build_hierarchy_header(&mut scope, &mut search_input, None, &mut bus);
    }
    assert!(tree.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON.raw()));

    // 2. Clicking '+' again toggles HIERARCHY_TAG_ADD_BUTTON off
    tree.event_state.frame_clicked_tag = Some(HIERARCHY_TAG_ADD_BUTTON.raw());
    {
        let mut scope = UiScope::new(&mut tree, root_id);
        build_hierarchy_header(&mut scope, &mut search_input, None, &mut bus);
    }
    assert!(!tree.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON.raw()));
}

#[test]
fn test_hierarchy_add_menu_cascading_click_emits_directly_to_action_bus() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut bus = ActionBus::new();

    // Activate Add Menu and open 3D Objects submenu in UiTree SSOT
    tree.open_popup(HIERARCHY_TAG_ADD_BUTTON.raw());
    tree.set_active_submenu(
        irisui::widgets::declarative::hash_label("HierarchyAddMenu"),
        AddSubmenuId::Objects3D.tag().raw(),
    );
    tree.event_state.frame_clicked_tag = Some(UiTag::from_str("Cube").raw());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let rect = super::add_menu::draw_hierarchy_add_menu(&mut scope, false, &mut bus);
        assert!(rect.is_some());
    }

    let drained: Vec<_> = bus.drain().collect();
    assert_eq!(drained.len(), 1);
    assert!(matches!(
        drained[0],
        EngineUiAction::SpawnShape(ae_core::ecs::Shape::Cube)
    ));
    assert!(
        !tree.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON.raw()),
        "Clicking leaf menu item must auto-close Add Menu popup tag in UiTree"
    );
}

#[test]
fn test_hierarchy_add_menu_outside_click_resets_tree_ssot() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut bus = ActionBus::new();
    let panel_rect = Rect::new(0.0, 0.0, 300.0, 600.0);
    let add_btn_rect = Rect::new(240.0, 4.0, 24.0, 24.0);

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
            HIERARCHY_TAG_ADD_BUTTON.raw(),
            |_btn| {},
        );
        scope.finish_layout(panel_rect);
    }

    tree.open_popup(HIERARCHY_TAG_ADD_BUTTON.raw());
    {
        let mut scope = UiScope::new(&mut tree, root);
        super::add_menu::draw_hierarchy_add_menu(&mut scope, false, &mut bus);
    }

    assert!(tree.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON.raw()));

    // Click inside anchor button does not dismiss
    let btn_click = Point::new(250.0, 10.0);
    assert!(tree.is_point_inside_popup(btn_click));
    assert!(!tree.dismiss_popups_on_outside_click(btn_click));
    assert!(tree.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON.raw()));

    // Click outside dismisses popups and submenus directly in UiTree SSOT
    let outside_point = Point::new(800.0, 800.0);
    assert!(!tree.is_point_inside_popup(outside_point));
    assert!(tree.dismiss_popups_on_outside_click(outside_point));
    assert!(!tree.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON.raw()));
    assert!(tree.active_submenus.is_empty());
    assert!(tree.popups_dismissed());
}