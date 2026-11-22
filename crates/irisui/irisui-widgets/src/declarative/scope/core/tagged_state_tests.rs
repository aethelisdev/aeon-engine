// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for [`UiScope`] tagged state, popup dismissal, focus queries, and text input auto-sync.

use super::context::UiScope;
use irisui_core::geometry::{Point, Rect};
use irisui_core::node::{UiLayer, WidgetRole};
use irisui_core::tree::UiTree;

#[test]
fn test_ui_scope_popup_introspection_and_dismissal() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");

    let mut scope = UiScope::new(&mut tree, root);
    assert!(!scope.has_open_popup());
    assert!(!scope.is_popup_tag_active(42));

    scope.set_popup_tag_active(42, true);
    assert!(scope.is_popup_tag_active(42));

    // Create a popup node
    let popup_node = scope.tree.create_node();
    if let Some(node) = scope.tree.get_mut(popup_node) {
        node.set_layer(UiLayer::Popup);
        node.set_role(WidgetRole::DropdownPopup);
        node.computed_rect = Rect::new(100.0, 100.0, 200.0, 150.0);
        node.interactive = true;
    }
    let _ = scope.tree.add_child(root, popup_node);

    assert!(scope.has_open_popup());

    // Inside click should not dismiss
    let inside_click = Point::new(150.0, 150.0);
    assert!(!scope.dismiss_popups_on_outside_click(inside_click));
    assert!(scope.has_open_popup());
    assert!(scope.is_popup_tag_active(42));

    // Outside click should dismiss popups and clear active tags
    let outside_click = Point::new(500.0, 500.0);
    assert!(scope.dismiss_popups_on_outside_click(outside_click));
    assert!(!scope.has_open_popup());
    assert!(!scope.is_popup_tag_active(42));
    assert!(scope.popups_dismissed());
}

#[test]
fn test_ui_scope_close_all_popups() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");

    let mut scope = UiScope::new(&mut tree, root);
    assert!(!scope.has_open_popup());

    scope.set_popup_tag_active(99, true);
    assert!(scope.has_open_popup());
    assert!(scope.close_all_popups());
    assert!(!scope.has_open_popup());
    assert!(!scope.close_all_popups());
}

#[test]
fn test_ui_scope_focus_queries_ssot() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");

    let mut scope = UiScope::new(&mut tree, root);
    assert!(!scope.is_tag_focused(55));
    assert!(!scope.has_text_input_focus());

    // 1. Direct tag focus on UiTree
    scope.set_focus_tag(55);
    assert!(scope.is_tag_focused(55));

    // 2. Start text edit marks text input focus active
    scope
        .tree
        .event_state
        .start_text_edit(None, 55, "abc", 3, None);
    assert!(scope.is_tag_focused(55));
    assert!(scope.has_text_input_focus());

    // 3. Clearing text edit drops text input focus
    scope.clear_text_edit();
    assert!(!scope.has_text_input_focus());

    // 4. Clearing focus releases focused tag and state
    scope.clear_focus();
    assert!(!scope.is_tag_focused(55));

    // 5. Setting tag 0 clears focus cleanly
    scope.set_focus_tag(55);
    assert!(scope.is_tag_focused(55));
    scope.set_focus_tag(0);
    assert!(!scope.is_tag_focused(55));
}

#[test]
fn test_ui_scope_auto_sync_text_input() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");

    let mut scope = UiScope::new(&mut tree, root);
    let mut input_state = crate::input::TextInputState::new("initial");

    // 1. Inactive text edit does nothing
    assert!(!scope.auto_sync_text_input(100, Some(101), &mut input_state));
    assert_eq!(input_state.text(), "initial");

    // 2. Active text edit syncs text and cursor
    scope
        .tree
        .event_state
        .start_text_edit(None, 100, "updated query", 13, None);
    assert!(scope.auto_sync_text_input(100, Some(101), &mut input_state));
    assert_eq!(input_state.text(), "updated query");
    assert_eq!(input_state.cursor_byte_idx, 13);

    // 3. Clear button click clears state, focus, and active text edit in one pass
    scope.tree.event_state.frame_clicked_tag = Some(101);
    assert!(scope.auto_sync_text_input(100, Some(101), &mut input_state));
    assert_eq!(input_state.text(), "");
    assert_eq!(input_state.cursor_byte_idx, 0);
    assert!(!scope.has_text_input_focus());
    assert!(!scope.is_tag_focused(100));
}

#[test]
fn test_ui_scope_modifiers_persist_across_pointer_events() {
    use irisui_core::event::{InputModifiers, MouseButton, UiEvent};

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");

    assert_eq!(
        UiScope::new(&mut tree, root).modifiers(),
        InputModifiers::default()
    );

    tree.dispatch_event(&UiEvent::ModifiersChanged {
        ctrl_held: true,
        shift_held: true,
    });
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(10.0, 10.0),
    });
    assert!(UiScope::new(&mut tree, root).modifiers().ctrl());
    assert!(UiScope::new(&mut tree, root).modifiers().shift());

    tree.event_state.clear_frame_clicks();
    assert!(UiScope::new(&mut tree, root).modifiers().ctrl());
    assert!(UiScope::new(&mut tree, root).modifiers().shift());
}