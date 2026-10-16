// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Comprehensive unit tests for UI tree hierarchy, hit testing, and cursor evaluation.

use super::arena::UiTree;
use crate::geometry::{Point, Rect};
use crate::node::{UiLayer, WidgetCursor, WidgetRole};

#[test]
fn test_hit_test_layered_priority() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 1000.0, 1000.0);
    }

    let content_btn = tree.create_node();
    if let Some(node) = tree.get_mut(content_btn) {
        node.computed_rect = Rect::new(100.0, 100.0, 200.0, 50.0);
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, content_btn).unwrap();

    let popup_item = tree.create_node();
    if let Some(node) = tree.get_mut(popup_item) {
        // Popup overlaps content button directly
        node.computed_rect = Rect::new(150.0, 100.0, 100.0, 100.0);
        node.layer = UiLayer::Popup;
    }
    tree.add_child(root, popup_item).unwrap();

    // Hit point inside both popup and content button: Popup MUST win!
    let hit = tree.hit_test_layered(Point::new(160.0, 120.0));
    assert_eq!(hit, Some(popup_item));

    // Hit point inside content button only
    let hit2 = tree.hit_test_layered(Point::new(110.0, 120.0));
    assert_eq!(hit2, Some(content_btn));
}

#[test]
fn test_hit_test_layered_modal_blocking() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 1000.0, 1000.0);
    }

    let bg_btn = tree.create_node();
    if let Some(node) = tree.get_mut(bg_btn) {
        node.computed_rect = Rect::new(50.0, 50.0, 100.0, 50.0);
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, bg_btn).unwrap();

    let modal_dlg = tree.create_node();
    if let Some(node) = tree.get_mut(modal_dlg) {
        node.computed_rect = Rect::new(300.0, 200.0, 400.0, 300.0);
        node.layer = UiLayer::Modal;
    }
    tree.add_child(root, modal_dlg).unwrap();

    assert!(tree.is_modal_active());

    // Inside modal: hits modal
    let hit_modal = tree.hit_test_layered(Point::new(350.0, 250.0));
    assert_eq!(hit_modal, Some(modal_dlg));

    // Outside modal over bg_btn: blocked by active modal!
    let hit_outside = tree.hit_test_layered(Point::new(60.0, 60.0));
    assert_eq!(hit_outside, None);
}

#[test]
fn test_hit_test_target_metadata_extraction() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    }

    let dropdown_item = tree.create_node();
    if let Some(node) = tree.get_mut(dropdown_item) {
        node.computed_rect = Rect::new(200.0, 150.0, 120.0, 24.0);
        node.role = WidgetRole::DropdownItem;
        node.layer = UiLayer::Popup;
        node.tag = 2; // Option index 2
        node.name = Some("Fps120Option".to_string());
    }
    tree.add_child(root, dropdown_item).unwrap();

    let hit_target = tree
        .hit_test_target(Point::new(250.0, 160.0))
        .expect("Dropdown item must be hit");

    assert_eq!(hit_target.id, dropdown_item);
    assert_eq!(hit_target.layer, UiLayer::Popup);
    assert_eq!(hit_target.role, WidgetRole::DropdownItem);
    assert_eq!(hit_target.tag, 2);
    assert_eq!(hit_target.name.as_deref(), Some("Fps120Option"));
    assert_eq!(hit_target.rect, Rect::new(200.0, 150.0, 120.0, 24.0));
    assert_eq!(hit_target.cursor, Some(WidgetCursor::Pointer));
}

#[test]
fn test_tree_cursor_at_layered_priority() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    }

    // 1. Text input in Content layer
    let input_id = tree.create_node();
    if let Some(node) = tree.get_mut(input_id) {
        node.computed_rect = Rect::new(100.0, 100.0, 200.0, 30.0);
        node.role = WidgetRole::TextInput;
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, input_id).unwrap();

    // 2. Numeric drag pill in Content layer
    let num_id = tree.create_node();
    if let Some(node) = tree.get_mut(num_id) {
        node.computed_rect = Rect::new(320.0, 100.0, 80.0, 30.0);
        node.role = WidgetRole::NumericInput;
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, num_id).unwrap();

    // 3. Button in Content layer with explicit Grab cursor override
    let grab_btn = tree.create_node();
    if let Some(node) = tree.get_mut(grab_btn) {
        node.computed_rect = Rect::new(420.0, 100.0, 80.0, 30.0);
        node.role = WidgetRole::Button;
        node.cursor = Some(WidgetCursor::Grab);
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, grab_btn).unwrap();

    // Test resolved cursors
    assert_eq!(tree.cursor_at(Point::new(150.0, 115.0)), WidgetCursor::Text);
    assert_eq!(
        tree.cursor_at(Point::new(350.0, 115.0)),
        WidgetCursor::EwResize
    );
    assert_eq!(tree.cursor_at(Point::new(450.0, 115.0)), WidgetCursor::Grab);
    assert_eq!(
        tree.cursor_at(Point::new(10.0, 10.0)),
        WidgetCursor::Default
    );
}

#[test]
fn test_hit_test_target_resolves_ancestor_tag_for_untagged_children() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    }

    // Tagged container (e.g. Button or Row)
    let parent_box = tree.create_node();
    if let Some(node) = tree.get_mut(parent_box) {
        node.computed_rect = Rect::new(100.0, 100.0, 200.0, 40.0);
        node.role = WidgetRole::Button;
        node.layer = UiLayer::Content;
        node.tag = 0x4849_4552_0000_0003; // Semantic tag
        node.interactive = true;
    }
    tree.add_child(root, parent_box).unwrap();

    // Untagged child (e.g. label or icon inside the button)
    let child_label = tree.create_node();
    if let Some(node) = tree.get_mut(child_label) {
        node.computed_rect = Rect::new(110.0, 110.0, 80.0, 20.0);
        node.role = WidgetRole::Default;
        node.layer = UiLayer::Content;
        node.tag = 0; // Untagged
        node.interactive = true;
    }
    tree.add_child(parent_box, child_label).unwrap();

    // Hit testing directly on the untagged child label must resolve the parent's semantic tag
    let hit = tree
        .hit_test_target(Point::new(120.0, 120.0))
        .expect("Child label must be hit");
    assert_eq!(hit.id, child_label);
    assert_eq!(hit.tag, 0x4849_4552_0000_0003);
}

#[test]
fn test_wants_pointer_at_evaluation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    }

    let button = tree.create_node();
    if let Some(node) = tree.get_mut(button) {
        node.computed_rect = Rect::new(50.0, 50.0, 100.0, 30.0);
        node.role = WidgetRole::Button;
        node.layer = UiLayer::Content;
        node.tag = 42;
    }
    tree.add_child(root, button).unwrap();

    // Over button -> wants pointer
    assert!(tree.wants_pointer_at(Point::new(60.0, 60.0)));
    // Empty root background -> does not want pointer
    assert!(!tree.wants_pointer_at(Point::new(10.0, 10.0)));
}