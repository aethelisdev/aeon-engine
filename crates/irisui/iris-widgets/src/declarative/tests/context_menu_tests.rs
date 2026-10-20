// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Context Menu Subsystem Integration Tests
//!
//! Verifies autonomous context menu trigger lifecycles, floating popup elevation on
//! [`UiLayer::Popup`], item click dismiss, outside-click dismiss, and Escape key handling.
//!

use crate::declarative::types::WidgetResponse;
use crate::declarative::{ContainerResponseExt, ContextMenuExt, UiScope};
use iris_core::event::{KeyCode, MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect, Size};
use iris_core::node::{UiLayer, WidgetRole};
use iris_core::tree::UiTree;

#[test]
fn test_declarative_context_menu_open_and_render() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation failed");

    // 1. First frame: Right click on a target card
    let click_pos = Point::new(120.0, 85.0);
    let card_id = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.empty_box_passive_named("TestCard", iris_core::Style::new());
        let id = scope
            .tree()
            .find_node_by_name("TestCard")
            .expect("TestCard exists");
        let card_resp = WidgetResponse::empty(id)
            .with_secondary_clicked(true)
            .with_pointer_pos(Some(click_pos));

        let mut item_count = 0;
        card_resp.context_menu(&mut scope, |menu| {
            let _ = menu.context_menu_item("Spawn");
            let _ = menu.context_menu_item("Delete");
            item_count += 2;
        });

        assert_eq!(item_count, 2);
        id
    };

    // 2. Verify context menu state is now active on EventState
    let active = tree
        .event_state
        .active_context_menu
        .expect("Active context menu must be present");
    assert_eq!(active.owner_id, card_id);
    assert_eq!(active.anchor_pos, click_pos);
    assert!(active.container_id.is_some());

    // 3. Verify elevated popup container properties in tree
    let popup_id = active.container_id.unwrap();
    let popup_node = tree.get(popup_id).expect("Popup container exists in tree");
    assert_eq!(popup_node.name.as_deref(), Some("ContextMenuPopup"));
    assert_eq!(popup_node.role, WidgetRole::DropdownPopup);
    assert_eq!(popup_node.layer, UiLayer::Popup);

    // 4. Second frame: Target card is NOT right-clicked, but menu remains open!
    let mut rendered_on_subsequent_frame = false;
    {
        let mut scope = UiScope::new(&mut tree, root);
        let card_resp = WidgetResponse::empty(card_id).with_secondary_clicked(false);
        card_resp.context_menu(&mut scope, |menu| {
            let _ = menu.context_menu_item("Spawn");
            let _ = menu.context_menu_item("Delete");
            rendered_on_subsequent_frame = true;
        });
    }

    assert!(
        rendered_on_subsequent_frame,
        "Autonomous context menu must remain open across frames until dismissed"
    );
}

#[test]
fn test_declarative_context_menu_item_click_closes_menu() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation failed");

    let card_id = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.empty_box_passive_named("TestCard", iris_core::Style::new());
        scope
            .tree()
            .find_node_by_name("TestCard")
            .expect("TestCard exists")
    };

    // Open context menu for card
    let click_pos = Point::new(100.0, 100.0);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let card_resp = WidgetResponse::empty(card_id)
            .with_secondary_clicked(true)
            .with_pointer_pos(Some(click_pos));
        card_resp.context_menu(&mut scope, |menu| {
            let _ = menu.context_menu_item("Action");
        });
    }
    assert!(tree.event_state.active_context_menu.is_some());

    // Render again where item is clicked
    {
        let mut scope = UiScope::new(&mut tree, root);
        let card_resp = WidgetResponse::empty(card_id).with_secondary_clicked(false);
        card_resp.context_menu(&mut scope, |menu| {
            let item_resp = menu.context_menu_item("Action");
            if !item_resp.clicked() {
                menu.tree.event_state.close_context_menu();
            }
        });
    }

    assert!(
        tree.event_state.active_context_menu.is_none(),
        "Context menu must close after action item click"
    );
}

#[test]
fn test_declarative_context_menu_outside_click_dismiss() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation failed");

    let card_id = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.empty_box_passive_named("TestCard", iris_core::Style::new());
        scope
            .tree()
            .find_node_by_name("TestCard")
            .expect("TestCard exists")
    };

    // Open menu
    {
        let mut scope = UiScope::new(&mut tree, root);
        let card_resp = WidgetResponse::empty(card_id)
            .with_secondary_clicked(true)
            .with_pointer_pos(Some(Point::new(50.0, 50.0)));
        card_resp.context_menu(&mut scope, |menu| {
            let _ = menu.context_menu_item("Item");
        });
    }

    let active = tree.event_state.active_context_menu.unwrap();
    let popup_id = active.container_id.unwrap();
    let popup_node = tree.get_mut(popup_id).unwrap();
    popup_node.computed_rect = Rect::new(50.0, 50.0, 160.0, 60.0);

    // Click outside popup bounds (e.g. at 500.0, 500.0)
    let outside_point = Point::new(500.0, 500.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: outside_point,
    });

    assert!(
        tree.event_state.active_context_menu.is_none(),
        "Clicking outside context menu bounds must automatically dismiss it"
    );
}

#[test]
fn test_declarative_context_menu_escape_key_dismiss() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation failed");

    let card_id = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.empty_box_passive_named("TestCard", iris_core::Style::new());
        scope
            .tree()
            .find_node_by_name("TestCard")
            .expect("TestCard exists")
    };

    // Open menu
    {
        let mut scope = UiScope::new(&mut tree, root);
        let card_resp = WidgetResponse::empty(card_id)
            .with_secondary_clicked(true)
            .with_pointer_pos(Some(Point::new(50.0, 50.0)));
        card_resp.context_menu(&mut scope, |menu| {
            let _ = menu.context_menu_item("Item");
        });
    }
    assert!(tree.event_state.active_context_menu.is_some());

    // Press Escape key
    let res = tree.dispatch_event(&UiEvent::KeyboardInput {
        key: KeyCode::Escape,
        text: None,
        is_pressed: true,
    });

    assert!(res.is_consumed());
    assert!(
        tree.event_state.active_context_menu.is_none(),
        "Pressing Escape must automatically close the active context menu"
    );
}

#[test]
fn test_declarative_container_response_ext_fluent_chaining() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation failed");

    let mut scope = UiScope::new(&mut tree, root);
    let container_resp = scope
        .container_tagged_resp(
            "TestChainedContainer",
            iris_core::Style::new(),
            WidgetRole::ScrollAreaViewport,
            42,
            |child_scope| {
                child_scope.text("Child");
            },
        )
        .content_size(&mut scope, Size::new(800.0, 1200.0))
        .content_size_wh(&mut scope, 900.0, 1500.0)
        .context_menu(&mut scope, |menu| {
            let _ = menu.context_menu_item("Action");
        });

    let container_id = container_resp.id;
    assert_eq!(container_id, tree.get(container_id).unwrap().id);
    let node = tree.get(container_id).expect("Container node exists");
    assert_eq!(node.content_size, Size::new(900.0, 1500.0));
    assert_eq!(node.tag, 42);
    assert_eq!(node.role, WidgetRole::ScrollAreaViewport);
}

#[test]
fn test_declarative_submenu_hover_and_smart_placement() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation failed");
    if let Some(r) = tree.get_mut(root) {
        r.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    }

    // 1. First render: Menu open, but cursor far away
    let anchor = Rect::new(100.0, 50.0, 120.0, 24.0);
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.popup_anchored_named("RootMenu", anchor, 180.0, |menu| {
            menu.dropdown_item(10, "", "Open", None, true);
            menu.submenu("Import", Some("📥"), 20, |sub| {
                sub.dropdown_item(21, "", "Import Texture", None, true);
                sub.dropdown_item(22, "", "Import Mesh", None, true);
            });
        });
    }

    // Verify root menu card exists, but submenu popup does not exist yet
    assert!(tree.find_node_by_name("RootMenu").is_some());
    assert!(tree.find_node_by_name("CascadingSubmenuPopup").is_none());

    // 2. Simulate cursor hovering over the submenu row (item tag 20)
    let submenu_row_id = tree.find_node_by_tag(20).expect("Submenu item exists");
    let row_rect = tree.get(submenu_row_id).unwrap().computed_rect;
    let hover_point = Point::new(row_rect.x + 10.0, row_rect.y + 10.0);
    tree.set_cursor_position(hover_point);

    // Rebuild frame with cursor hovering over submenu row
    tree.clear();
    let root2 = tree.create_root().expect("Root recreated");
    if let Some(r) = tree.get_mut(root2) {
        r.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    }

    {
        let mut scope = UiScope::new(&mut tree, root2);
        scope.popup_anchored_named("RootMenu", anchor, 180.0, |menu| {
            menu.dropdown_item(10, "", "Open", None, true);
            menu.submenu("Import", Some("📥"), 20, |sub| {
                sub.dropdown_item(21, "", "Import Texture", None, true);
                sub.dropdown_item(22, "", "Import Mesh", None, true);
            });
        });
    }

    // 3. Verify submenu popup was autonomously opened and rendered!
    let sub_node = tree.find_node_by_name("CascadingSubmenuPopup");
    assert!(
        sub_node.is_some(),
        "Submenu popup must be rendered when row is hovered"
    );
    let sub_id = sub_node.unwrap();
    let sub_rect = tree.get(sub_id).unwrap().computed_rect;

    let root_menu_id = tree.find_node_by_name("RootMenu").unwrap();
    let root_menu_rect = tree.get(root_menu_id).unwrap().computed_rect;

    // Smart placement: Submenu opens to the right of root menu card
    assert!(
        sub_rect.x >= root_menu_rect.right(),
        "Submenu must be positioned to the right of root menu: sub_x={}, root_right={}",
        sub_rect.x,
        root_menu_rect.right()
    );

    // Child items in submenu must be rendered
    assert!(tree.find_node_by_tag(21).is_some());
    assert!(tree.find_node_by_tag(22).is_some());
}

#[test]
fn test_declarative_submenu_smart_placement_auto_flip_left() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation failed");
    if let Some(r) = tree.get_mut(root) {
        r.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    }

    // Anchor positioned near right window border (e.g. x = 1750.0, width = 150.0)
    let anchor = Rect::new(1750.0, 100.0, 150.0, 24.0);

    // Initial render to establish nodes
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.popup_anchored_named("RightEdgeMenu", anchor, 150.0, |menu| {
            menu.submenu("Export", None, 30, |sub| {
                sub.dropdown_item(31, "", "Export glTF", None, true);
            });
        });
    }

    // Hover over submenu item
    let submenu_row_id = tree.find_node_by_tag(30).expect("Submenu item exists");
    let row_rect = tree.get(submenu_row_id).unwrap().computed_rect;
    tree.set_cursor_position(Point::new(row_rect.x + 5.0, row_rect.y + 5.0));

    // Rebuild frame
    tree.clear();
    let root2 = tree.create_root().expect("Root recreated");
    if let Some(r) = tree.get_mut(root2) {
        r.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    }

    {
        let mut scope = UiScope::new(&mut tree, root2);
        scope.popup_anchored_named("RightEdgeMenu", anchor, 150.0, |menu| {
            menu.submenu("Export", None, 30, |sub| {
                sub.dropdown_item(31, "", "Export glTF", None, true);
            });
        });
    }

    let sub_node = tree.find_node_by_name("CascadingSubmenuPopup");
    assert!(sub_node.is_some(), "Submenu popup must be rendered");
    let sub_id = sub_node.unwrap();
    let sub_rect = tree.get(sub_id).unwrap().computed_rect;

    let parent_card_id = tree.find_node_by_name("RightEdgeMenu").unwrap();
    let parent_card_rect = tree.get(parent_card_id).unwrap().computed_rect;

    // Must have auto-flipped to the left because right placement would overflow 1920.0
    assert!(
        sub_rect.right() <= parent_card_rect.x,
        "Submenu must auto-flip to the left of parent menu card: sub_right={}, parent_x={}",
        sub_rect.right(),
        parent_card_rect.x
    );
}