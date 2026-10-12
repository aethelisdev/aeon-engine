// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Declarative Input Box Layout & Caret Positioning (`iris-widgets::declarative::tests::input_box_tests`)

use crate::declarative::UiScope;
use iris_core::{Rect, UiTree, WidgetRole};

#[test]
fn test_input_box_tagged_with_icon_and_caret_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let mut scope = UiScope::new(&mut tree, root);
    let tag = 0x5002;

    let mut box_id_opt = None;
    scope.toolbar_tagged("TestTb", 34.0, 0x1000, iris_core::Color::BLACK, |tb| {
        let props =
            crate::declarative::scope::InputBoxProps::new("", "Search logs...", 210.0, true, true)
                .with_icon("🔍");
        let id = tb.input_box_tagged(&props, tag);
        box_id_opt = Some(id);
    });

    let bounds = Rect::new(0.0, 0.0, 400.0, 34.0);
    scope.finish_layout(bounds);

    let box_id = box_id_opt.expect("Box exists");
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

    let mut box_id_opt = None;
    scope.toolbar_tagged("TestTb2", 34.0, 0x1000, iris_core::Color::BLACK, |tb| {
        let props = crate::declarative::scope::InputBoxProps::new(
            "Hello World",
            "Search...",
            210.0,
            true,
            true,
        )
        .with_icon("🔍")
        .with_cursor_byte_idx(5);
        let id = tb.input_box_tagged(&props, tag);
        box_id_opt = Some(id);
    });

    let bounds = Rect::new(0.0, 0.0, 400.0, 34.0);
    scope.finish_layout(bounds);

    let box_id = box_id_opt.expect("Box exists");
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
    let mut box_id_opt = None;
    scope.toolbar_tagged("TestTb3", 34.0, 0x1000, iris_core::Color::BLACK, |tb| {
        let props = crate::declarative::scope::InputBoxProps::new(
            long_text,
            "Search...",
            180.0,
            true,
            true,
        )
        .with_icon("🔍")
        .with_clear_tag(0x9999);
        let id = tb.input_box_tagged(&props, tag);
        box_id_opt = Some(id);
    });

    let bounds = Rect::new(0.0, 0.0, 400.0, 34.0);
    scope.finish_layout(bounds);

    let box_id = box_id_opt.expect("Box exists");
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