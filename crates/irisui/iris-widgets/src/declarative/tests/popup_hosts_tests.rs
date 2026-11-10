// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Dropdown and Context Menu Host Unit Tests
//!
//! Verifies state machine transitions, elevated [`UiLayer::Popup`] card construction,
//! item selection propagation, outside-click dismissal, and keyboard Escape cancellation.

use crate::declarative::scope::core::UiScope;
use crate::declarative::widget::{ContextMenuHost, DropdownHost, DropdownItem};
use iris_core::event::KeyCode;
use iris_core::{Point, Rect, UiLayer, UiTree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestAction {
    ItemA,
    ItemB,
    Delete,
    DummySeparator,
}

#[test]
fn test_dropdown_host_open_and_toggle_lifecycle() {
    let mut host: DropdownHost<TestAction> = DropdownHost::new();
    assert!(!host.is_open());

    let anchor = Rect::new(50.0, 30.0, 120.0, 24.0);
    host.open(anchor);
    assert!(host.is_open());
    assert_eq!(host.anchor_rect(), anchor);

    // Toggle with same anchor closes it
    host.toggle(anchor);
    assert!(!host.is_open());

    // Toggle again opens it
    host.toggle(anchor);
    assert!(host.is_open());

    host.close();
    assert!(!host.is_open());
}

#[test]
fn test_dropdown_host_show_and_layer_hierarchy() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut host = DropdownHost::new();
    let anchor = Rect::new(100.0, 50.0, 140.0, 26.0);
    host.open(anchor);

    let items = [
        DropdownItem::item_with_icon(TestAction::ItemA, "📁", "Open File"),
        DropdownItem::item(TestAction::ItemB, "Save File").with_shortcut("Ctrl+S"),
        DropdownItem::separator(TestAction::DummySeparator),
        DropdownItem::item(TestAction::Delete, "Delete").destructive(),
    ];

    {
        let mut scope = UiScope::new(&mut tree, root);
        let selected = host.show(&mut scope, &items);
        assert_eq!(selected, None);
    }

    assert!(host.is_open());

    // Verify popup container exists in UiTree on UiLayer::Popup
    let root_node = tree.get(root).expect("Root exists");
    assert_eq!(root_node.children.len(), 1);

    let popup_id = root_node.children[0];
    let popup = tree.get(popup_id).expect("Popup node exists");
    assert_eq!(popup.layer, UiLayer::Popup);
    assert_eq!(popup.name.as_deref(), Some("DropdownHostPopup"));

    // Verify popup contains items + separator
    assert_eq!(popup.children.len(), 4);
}

#[test]
fn test_dropdown_host_escape_cancellation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut host = DropdownHost::new();
    host.open(Rect::new(100.0, 50.0, 140.0, 26.0));

    // Simulate Escape key press in active frame
    tree.event_state.frame_key = Some(KeyCode::Escape);
    tree.event_state.frame_key_pressed = true;

    let items = [DropdownItem::item(TestAction::ItemA, "Action")];

    {
        let mut scope = UiScope::new(&mut tree, root);
        let selected = host.show(&mut scope, &items);
        assert_eq!(selected, None);
    }

    assert!(
        !host.is_open(),
        "Escape key must immediately dismiss open dropdown host"
    );
}

#[test]
fn test_context_menu_host_lifecycle_and_outside_click() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut host: ContextMenuHost<TestAction> =
        ContextMenuHost::new().with_header("Context Actions");
    let click_pos = Point::new(300.0, 400.0);
    host.open(click_pos);
    assert!(host.is_open());
    assert_eq!(host.click_pos(), click_pos);

    let items = [
        DropdownItem::item(TestAction::ItemA, "Inspect"),
        DropdownItem::item(TestAction::Delete, "Remove").destructive(),
    ];

    // Frame 1: show menu without click -> remains open
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = host.show(&mut scope, &items);
    }
    assert!(host.is_open());

    // Frame 2: simulate outside click far away (10.0, 10.0)
    tree.event_state.frame_clicked_pos = Some(Point::new(10.0, 10.0));
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = host.show(&mut scope, &items);
    }
    assert!(
        !host.is_open(),
        "Outside click must immediately dismiss open context menu"
    );
}