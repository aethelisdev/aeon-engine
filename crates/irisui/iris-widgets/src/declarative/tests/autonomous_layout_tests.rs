// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Subtree Layout & Scope Lifecycle Unit Tests
//!
//! Verifies autonomous defer and flush of flexbox subtree layouts across
//! `UiScope::scoped`, `UiScope::panel_styled_tagged`, `UiScope::modal_scrim_sized`,
//! and floating dropdown menus without requiring manual calls to `scope.finish_layout(rect)`.
//!

use crate::declarative::{EmptyBox, UiScope};
use iris_core::{Color, Point, Rect, Style, UiTree, WidgetRole};

#[test]
fn test_scoped_autonomous_subtree_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    if let Some(node) = tree.get_mut(root) {
        node.set_style(Style::new().flex_col().width(400.0).height(300.0));
    }
    let bounds = Rect::new(0.0, 0.0, 400.0, 300.0);

    let (id1, id2) = UiScope::scoped(&mut tree, root, bounds, |scope| {
        let b1 = scope.button("Option 1");
        let b2 = scope.button("Option 2");
        (b1.id, b2.id)
    });

    let n1 = tree.get(id1).expect("Button 1 exists");
    let n2 = tree.get(id2).expect("Button 2 exists");

    assert!(n1.computed_rect.width > 0.0);
    assert!(n1.computed_rect.height > 0.0);
    assert!(n2.computed_rect.width > 0.0);
    assert!(n2.computed_rect.height > 0.0);
    // Button 2 should be stacked vertically below Button 1
    assert!(n2.computed_rect.y >= n1.computed_rect.y + n1.computed_rect.height);
}

#[test]
fn test_panel_builder_autonomous_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let panel_rect = Rect::new(50.0, 30.0, 320.0, 500.0);

    let panel_id = {
        let mut outer_scope = UiScope::new(&mut tree, root);
        let resp = outer_scope
            .panel("AutonomousPanel")
            .tagged(0xABCD)
            .bounds(panel_rect)
            .background(Color::rgba(0.1, 0.1, 0.12, 1.0))
            .border(1.0, Color::rgba(0.2, 0.2, 0.25, 1.0))
            .show(|panel_scope| {
                panel_scope.heading("Panel Title");
                panel_scope.label("Description text");
                let _ = panel_scope.button("Apply Changes");
            });
        resp.response.id
    };

    let panel_node = tree.get(panel_id).expect("Panel node exists");
    assert_eq!(panel_node.computed_rect, panel_rect);

    // Verify all children received non-zero geometry without manual finish_layout
    tree.traverse_depth_first(panel_id, &mut |_id, node| {
        assert!(node.computed_rect.width > 0.0);
        assert!(node.computed_rect.height > 0.0);
    });
}

#[test]
fn test_modal_scrim_sized_autonomous_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let screen = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let scrim_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.modal_scrim_sized(screen, Color::rgba(0.0, 0.0, 0.0, 0.6), |scrim| {
            scrim.modal_card(420.0, 200.0, |card| {
                card.heading("Confirm Action");
                card.label("Are you sure you want to proceed?");
                card.row(|row| {
                    row.modal_cancel_button("Cancel", 80.0);
                    row.modal_confirm_button("Confirm", 80.0);
                });
            });
        });
        resp.response.id
    };

    let scrim_node = tree.get(scrim_id).expect("Scrim exists");
    assert_eq!(scrim_node.computed_rect, screen);

    let mut found_card = false;
    tree.traverse_depth_first(scrim_id, &mut |_id, node| {
        if node.role == WidgetRole::ModalWindow && node.computed_rect.width == 420.0 {
            found_card = true;
            // The card should be centered within the 1920x1080 screen
            let expected_x = (1920.0 - 420.0) / 2.0;
            assert!((node.computed_rect.x - expected_x).abs() < 1.0);
            assert!(node.computed_rect.y > 0.0);
        }
    });

    assert!(found_card, "Modal card was properly centered and laid out");
}

#[test]
fn test_dropdown_menu_card_autonomous_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let cursor = Point::new(110.0, 55.0);
    tree.set_cursor_position(cursor);

    let dd_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.dropdown_menu_card(100.0, 40.0, 160.0, |menu| {
            menu.dropdown_item(1, "📁", "Open File", Some("Ctrl+O"), true);
            menu.dropdown_item(2, "💾", "Save File", Some("Ctrl+S"), true);
        });
        resp.response.id
    };

    let dd_node = tree.get(dd_id).expect("Dropdown exists");
    assert_eq!(dd_node.computed_rect.x, 100.0);
    assert_eq!(dd_node.computed_rect.y, 40.0);
    assert_eq!(dd_node.computed_rect.width, 160.0);
    assert!(dd_node.computed_rect.height > 0.0);
}

#[test]
fn test_declarative_flex_wrap_row_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let mut scope = UiScope::new(&mut tree, root);

    let mut item_ids = Vec::new();
    let row_style = iris_core::Style::new()
        .flex_row()
        .flex_wrap(iris_core::FlexWrap::Wrap)
        .gap(6.0);

    scope.container_named("WrapContainer", row_style, |wrap_scope| {
        for i in 0..5 {
            let item_style = iris_core::Style::new().width(20.0).height(16.0);
            let item_resp = wrap_scope.add(EmptyBox::new("WrapItem", item_style).tag(0x9000 + i));
            let id = item_resp.id;
            item_ids.push(id);
        }
    });

    let bounds = Rect::new(0.0, 0.0, 50.0, 200.0);
    scope.finish_layout(bounds);

    let r0 = tree.get(item_ids[0]).unwrap().computed_rect;
    let r1 = tree.get(item_ids[1]).unwrap().computed_rect;
    let r2 = tree.get(item_ids[2]).unwrap().computed_rect;
    let r3 = tree.get(item_ids[3]).unwrap().computed_rect;
    let r4 = tree.get(item_ids[4]).unwrap().computed_rect;

    assert_eq!(r0.x, 0.0);
    assert_eq!(r0.y, 0.0);
    assert_eq!(r1.x, 26.0);
    assert_eq!(r1.y, 0.0);

    assert_eq!(r2.x, 0.0);
    assert_eq!(r2.y, 22.0); // 16.0 + 6.0 = 22.0
    assert_eq!(r3.x, 26.0);
    assert_eq!(r3.y, 22.0);

    assert_eq!(r4.x, 0.0);
    assert_eq!(r4.y, 44.0); // 22.0 + 22.0 = 44.0
}