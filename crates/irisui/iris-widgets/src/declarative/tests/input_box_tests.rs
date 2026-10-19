// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Declarative Input Box Layout & Caret Positioning (`iris-widgets::declarative::tests::input_box_tests`)

use crate::declarative::{UiScope, widget::Toolbar};
use iris_core::{Rect, UiTree, WidgetRole};

#[test]
fn test_input_box_tagged_with_icon_and_caret_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let mut scope = UiScope::new(&mut tree, root);
    let tag = 0x5002;

    let _tb_resp = Toolbar::new("TestTb")
        .height(34.0)
        .tag(0x1000)
        .background(iris_core::Color::BLACK)
        .show(&mut scope, |tb| {
            let props = crate::declarative::scope::InputBoxProps::new(
                "",
                "Search logs...",
                210.0,
                true,
                true,
            )
            .with_icon("🔍");
            tb.input_box_tagged(&props, tag);
        });

    let bounds = Rect::new(0.0, 0.0, 400.0, 34.0);
    scope.finish_layout(bounds);

    let box_id = tree.find_node_by_tag(tag).expect("Box node exists");
    let box_node = tree.get(box_id).expect("Box node exists");
    assert_eq!(box_node.computed_rect.width, 210.0);
    assert_eq!(box_node.computed_rect.height, 24.0);
    assert_eq!(box_node.tag, tag);

    // 3 children: Icon, Text, Caret
    assert_eq!(box_node.children.len(), 3);
    let icon_node = tree.get(box_node.children[0]).expect("Icon node");
    assert_eq!(icon_node.role, WidgetRole::TextInputIcon);
    assert_eq!(icon_node.computed_rect.width, 14.0);
    assert_eq!(icon_node.computed_rect.x, box_node.computed_rect.x + 7.0);

    let text_node = tree.get(box_node.children[1]).expect("Text node");
    assert_eq!(text_node.text.as_deref(), Some("Search logs..."));
    // Since caret is at 0.0 and focused, text is slightly adjusted to the right
    assert!(text_node.computed_rect.x > icon_node.computed_rect.right());

    let caret_node = tree.get(box_node.children[2]).expect("Caret node");
    assert_eq!(caret_node.role, WidgetRole::TextInputCaret);
    assert_eq!(caret_node.computed_rect.width, 1.5);
    assert_eq!(caret_node.computed_rect.height, 14.0); // (24.0 - 10.0)
    assert!(caret_node.computed_rect.x <= text_node.computed_rect.x);
}

#[test]
fn test_input_box_tagged_cursor_byte_idx_caret_positioning() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let mut scope = UiScope::new(&mut tree, root);
    let tag = 0x5003;

    let _tb_resp = Toolbar::new("TestTb2")
        .height(34.0)
        .tag(0x1000)
        .background(iris_core::Color::BLACK)
        .show(&mut scope, |tb| {
            let props = crate::declarative::scope::InputBoxProps::new(
                "Hello World",
                "Search...",
                210.0,
                true,
                true,
            )
            .with_icon("🔍")
            .with_cursor_byte_idx(5);
            tb.input_box_tagged(&props, tag);
        });

    let bounds = Rect::new(0.0, 0.0, 400.0, 34.0);
    scope.finish_layout(bounds);

    let box_id = tree.find_node_by_tag(tag).expect("Box node exists");
    let box_node = tree.get(box_id).expect("Box node exists");
    let text_node = tree.get(box_node.children[1]).expect("Text node");
    let caret_node = tree.get(box_node.children[2]).expect("Caret node");

    let expected_offset = crate::declarative::scope::measure_text_advance("Hello", 11.0);
    let text_start = text_node.computed_rect.x;
    assert_eq!(caret_node.computed_rect.x, text_start + expected_offset);
}

#[test]
fn test_input_box_tagged_long_text_clipping_and_visible_slice() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let mut scope = UiScope::new(&mut tree, root);
    let tag = 0x5004;

    let long_text = "fgdgfdhfhfdhhhhfdhfdhfdhfdhfdhfdhfdhfdhfdffdhfdhfdhfdhdf";
    Toolbar::new("TestTb3")
        .height(34.0)
        .tag(0x1000)
        .background(iris_core::Color::BLACK)
        .show(&mut scope, |tb| {
            let props = crate::declarative::scope::InputBoxProps::new(
                long_text,
                "Search...",
                180.0,
                true,
                true,
            )
            .with_icon("🔍")
            .with_clear_tag(0x9999);
            tb.input_box_tagged(&props, tag);
        });

    let bounds = Rect::new(0.0, 0.0, 400.0, 34.0);
    scope.finish_layout(bounds);

    let box_id = tree.find_node_by_tag(tag).expect("Box node exists");
    let box_node = tree.get(box_id).expect("Box node exists");
    assert!(box_node.style.clip_children, "Input box must clip children");

    // Children: Icon (0), Text (1), Caret (2), ClearButton (3)
    assert_eq!(box_node.children.len(), 4);
    let icon_node = tree.get(box_node.children[0]).expect("Icon node");
    let text_node = tree.get(box_node.children[1]).expect("Text node");
    let caret_node = tree.get(box_node.children[2]).expect("Caret node");
    let clear_node = tree.get(box_node.children[3]).expect("Clear node");

    // Max visible text space = 180 - 26 (icon) - 24 (clear) = 130
    let _text_start_x = icon_node.computed_rect.right() + 5.0; // 7 + 14 + 5 = 26.0 relative to box
    assert!(text_node.computed_rect.x >= box_node.computed_rect.x + 25.0);
    assert!(caret_node.computed_rect.x >= text_node.computed_rect.x);
    // Caret must strictly not overflow past the clear button
    assert!(caret_node.computed_rect.right() <= clear_node.computed_rect.x);
    // Displayed text must be sliced so it fits within the visible width
    let displayed_text = text_node.text.as_deref().unwrap_or("");
    assert!(displayed_text.len() < long_text.len());
    assert!(long_text.ends_with(displayed_text));
}

#[test]
fn test_text_input_first_click_selects_all_and_selection_capsule() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x9001;
    let mut text = "Dynamic Cube".to_string();

    // Frame 1: Click on the text input
    tree.event_state.frame_clicked_tag = Some(tag);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Name", tag, 200.0, &mut text);
        assert!(resp.clicked());
    }

    let box_id = tree.find_node_by_tag(tag).expect("Box node must exist");
    let box_node = tree.get(box_id).expect("Box node must exist");

    // Must be in active text edit mode with is_all_selected = true
    assert_eq!(
        tree.event_state.active_text_edit,
        Some((tag, text.len(), true))
    );

    // Must contain a SelectionCapsule child
    let has_selection_capsule = box_node.children.iter().any(|&child_id| {
        tree.get(child_id)
            .is_some_and(|n| n.name.as_deref() == Some("SelectionCapsule"))
    });
    assert!(
        has_selection_capsule,
        "First click must create SelectionCapsule"
    );

    // Caret must be rendered while focused to provide clear typing UX
    let has_caret = box_node.children.iter().any(|&child_id| {
        tree.get(child_id)
            .is_some_and(|n| n.name.as_deref() == Some("TextInputCaret"))
    });
    assert!(
        has_caret,
        "Caret must be rendered when text input is focused on first click"
    );
}

#[test]
fn test_text_input_arrow_keys_switch_to_caret_mode() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x9002;
    let mut text = "Dynamic Cube".to_string();

    // Step 1: Simulate active all-selected state
    tree.event_state.focused_tag = Some(tag);
    tree.event_state.active_text_edit = Some((tag, text.len(), true));

    // Step 2: Press ArrowLeft
    tree.event_state.frame_key = Some(iris_core::event::KeyCode::ArrowLeft);
    tree.event_state.frame_key_pressed = true;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _resp = scope.text_input_with_width_tagged("Name", tag, 200.0, &mut text);
    }

    // ArrowLeft on all-selected should set is_all_selected = false and cursor = 0
    assert_eq!(tree.event_state.active_text_edit, Some((tag, 0, false)));

    let box_id = tree.find_node_by_tag(tag).expect("Box node must exist");
    let box_node = tree.get(box_id).expect("Box node must exist");

    // SelectionCapsule must be gone
    let has_selection_capsule = box_node.children.iter().any(|&child_id| {
        tree.get(child_id)
            .is_some_and(|n| n.name.as_deref() == Some("SelectionCapsule"))
    });
    assert!(
        !has_selection_capsule,
        "SelectionCapsule must disappear after arrow press"
    );

    // TextInputCaret must be visible
    let has_caret = box_node.children.iter().any(|&child_id| {
        tree.get(child_id)
            .is_some_and(|n| n.name.as_deref() == Some("TextInputCaret"))
    });
    assert!(has_caret, "TextInputCaret must appear in cursor mode");
}

#[test]
fn test_text_input_typing_overwrites_all_selected() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x9003;
    let mut text = "Dynamic Cube".to_string();

    // All selected state
    tree.event_state.focused_tag = Some(tag);
    tree.event_state.active_text_edit = Some((tag, text.len(), true));

    // Type character 'A'
    tree.event_state.frame_text_input = Some("A".to_string());
    tree.event_state.frame_key_pressed = true;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Name", tag, 200.0, &mut text);
        assert!(resp.changed(), "Typing must emit changed event");
    }

    // Text must now be overwritten to "A", cursor at 1, is_all_selected = false
    assert_eq!(text, "A");
    assert_eq!(tree.event_state.active_text_edit, Some((tag, 1, false)));
}

#[test]
fn test_text_input_enter_submits_and_clears_focus() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x9004;
    let mut text = "Dynamic Cube".to_string();

    tree.event_state.focused_tag = Some(tag);
    tree.event_state.active_text_edit = Some((tag, text.len(), false));

    // Press Enter
    tree.event_state.frame_key = Some(iris_core::event::KeyCode::Enter);
    tree.event_state.frame_key_pressed = true;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Name", tag, 200.0, &mut text);
        assert!(resp.submitted(), "Pressing Enter must emit submitted flag");
    }

    // Focus and active text edit must be cleared
    assert!(tree.event_state.focused.is_none());
    assert!(tree.event_state.focused_tag.is_none());
    assert!(tree.event_state.active_text_edit.is_none());
}

#[test]
fn test_input_box_custom_caret_color() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let mut scope = UiScope::new(&mut tree, root);
    let tag = 0x5009;
    let custom_color = iris_core::Color::rgba(1.0, 0.5, 0.0, 1.0);

    let props = crate::declarative::scope::InputBoxProps::new(
        "Test text",
        "Placeholder",
        200.0,
        true,
        true,
    )
    .with_caret_color(custom_color);

    scope.input_box_tagged(&props, tag);

    let box_id = tree.find_node_by_tag(tag).expect("Box node exists");
    let box_node = tree.get(box_id).expect("Box node exists");

    let caret_id = box_node
        .children
        .iter()
        .copied()
        .find(|&id| {
            tree.get(id)
                .is_some_and(|n| n.role == WidgetRole::TextInputCaret)
        })
        .expect("Caret node must exist");

    let caret_node = tree.get(caret_id).expect("Caret node exists");
    assert_eq!(caret_node.style.background_color, custom_color);
}