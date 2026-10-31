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
        Some((tag, text.clone(), text.len(), true))
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
    tree.event_state.active_text_edit = Some((tag, text.clone(), text.len(), true));

    // Step 2: Press ArrowLeft
    tree.event_state.frame_key = Some(iris_core::event::KeyCode::ArrowLeft);
    tree.event_state.frame_key_pressed = true;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _resp = scope.text_input_with_width_tagged("Name", tag, 200.0, &mut text);
    }

    // ArrowLeft on all-selected should set is_all_selected = false and cursor = 0
    assert_eq!(
        tree.event_state.active_text_edit,
        Some((tag, text.clone(), 0, false))
    );

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
    tree.event_state.active_text_edit = Some((tag, text.clone(), text.len(), true));

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
    assert_eq!(
        tree.event_state.active_text_edit,
        Some((tag, "A".to_string(), 1, false))
    );
}

#[test]
fn test_text_input_enter_submits_and_clears_focus() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x9004;
    let mut text = "Dynamic Cube".to_string();

    tree.event_state.focused_tag = Some(tag);
    tree.event_state.active_text_edit = Some((tag, text.clone(), text.len(), false));

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

#[test]
fn test_input_box_tagged_live_typing_and_clear() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x5010;
    let clear_tag = 0x5011;

    // Step 1: Initial render with empty query
    let props = crate::declarative::scope::InputBoxProps::new("", "Search...", 200.0, false, false)
        .with_clear_tag(clear_tag);

    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.input_box_tagged(&props, tag);
    }

    // Step 2: Click on input box to focus and start text edit
    let box_id = tree.find_node_by_tag(tag).expect("Box node exists");
    tree.event_state.frame_clicked_id = Some(box_id);
    tree.event_state.frame_clicked_tag = Some(tag);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.input_box_tagged(&props, tag);
        assert!(resp.clicked());
    }

    assert_eq!(tree.event_state.focused_tag, Some(tag));
    assert!(tree.event_state.active_text_edit.is_some());
    assert_eq!(tree.event_state.text_edit_buffer(tag), Some(""));

    // Step 3: Type character 'A'
    tree.clear_frame_clicks();
    tree.event_state.frame_text_input = Some("A".to_string());
    tree.event_state.frame_key_pressed = true;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.input_box_tagged(&props, tag);
        assert!(resp.changed(), "Typing must emit changed response flag");
    }

    assert_eq!(tree.event_state.text_edit_buffer(tag), Some("A"));
    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(1));

    // Step 4: Type character 's'
    tree.clear_frame_clicks();
    tree.event_state.frame_text_input = Some("s".to_string());
    tree.event_state.frame_key_pressed = true;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.input_box_tagged(&props, tag);
        assert!(resp.changed());
    }

    assert_eq!(tree.event_state.text_edit_buffer(tag), Some("As"));
    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(2));

    // Step 5: Click clear button (clear_tag)
    tree.clear_frame_clicks();
    tree.event_state.frame_clicked_tag = Some(clear_tag);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.input_box_tagged(&props, tag);
        assert!(
            resp.changed(),
            "Clicking clear must wipe buffer and emit changed"
        );
    }

    assert_eq!(tree.event_state.text_edit_buffer(tag), Some(""));
    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(0));
}

#[test]
fn test_input_box_autonomous_clipboard_shortcuts() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let mut text_buffer = "Hello Aeon".to_string();

    let input_id;
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Type here...", &mut text_buffer);
        input_id = resp.id;
    }

    let input_rect = Rect::new(10.0, 10.0, 200.0, 30.0);
    if let Some(node) = tree.get_mut(input_id) {
        node.computed_rect = input_rect;
    }

    // 1. Click input: MouseDown + MouseUp on input -> Gains keyboard focus
    let click_pt = iris_core::Point::new(20.0, 20.0);
    tree.dispatch_event(&iris_core::UiEvent::MouseDown {
        button: iris_core::MouseButton::Left,
        point: click_pt,
    });
    tree.dispatch_event(&iris_core::UiEvent::MouseUp {
        button: iris_core::MouseButton::Left,
        point: click_pt,
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Type here...", &mut text_buffer);
        assert!(resp.clicked());
        assert!(resp.focused());
    }

    // 2. Dispatch Ctrl+A (Select All)
    let ctrl_a_ev = iris_core::UiEvent::KeyboardInput {
        key: iris_core::KeyCode::Character('a'),
        text: Some("a".to_string()),
        is_pressed: true,
        ctrl_held: true,
    };
    let res = tree.dispatch_event(&ctrl_a_ev);
    assert!(res.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input("Type here...", &mut text_buffer);
        let active_edit = tree.event_state.active_text_edit.as_ref().unwrap();
        assert!(active_edit.3, "Ctrl+A must set is_all_selected to true");
    }

    // 3. Dispatch Ctrl+C (Copy)
    let ctrl_c_ev = iris_core::UiEvent::KeyboardInput {
        key: iris_core::KeyCode::Character('c'),
        text: Some("c".to_string()),
        is_pressed: true,
        ctrl_held: true,
    };
    let res_c = tree.dispatch_event(&ctrl_c_ev);
    assert!(res_c.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input("Type here...", &mut text_buffer);
    }
    // Verify clipboard content via crate::input::get_clipboard_text
    assert_eq!(
        crate::input::get_clipboard_text().as_deref(),
        Some("Hello Aeon"),
        "Ctrl+C must copy full text buffer to SystemClipboard"
    );

    // 4. Dispatch Ctrl+X (Cut)
    let ctrl_x_ev = iris_core::UiEvent::KeyboardInput {
        key: iris_core::KeyCode::Character('x'),
        text: Some("x".to_string()),
        is_pressed: true,
        ctrl_held: true,
    };
    let res_x = tree.dispatch_event(&ctrl_x_ev);
    assert!(res_x.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Type here...", &mut text_buffer);
        assert!(resp.changed(), "Ctrl+X must change text buffer");
        assert_eq!(
            text_buffer, "",
            "Ctrl+X must clear buffer when all selected"
        );
    }
    assert_eq!(
        crate::input::get_clipboard_text().as_deref(),
        Some("Hello Aeon"),
        "Ctrl+X must copy cut content to SystemClipboard"
    );

    // 5. Dispatch Ctrl+V (Paste)
    let ctrl_v_ev = iris_core::UiEvent::KeyboardInput {
        key: iris_core::KeyCode::Character('v'),
        text: Some("v".to_string()),
        is_pressed: true,
        ctrl_held: true,
    };
    let res_v = tree.dispatch_event(&ctrl_v_ev);
    assert!(res_v.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Type here...", &mut text_buffer);
        assert!(resp.changed(), "Ctrl+V must change text buffer");
        assert_eq!(
            text_buffer, "Hello Aeon",
            "Ctrl+V must paste copied text from SystemClipboard into buffer"
        );
    }
}

#[test]
fn test_process_text_input_keys_edge_cases_and_utf8() {
    use crate::declarative::scope::input_box::interaction::process_text_input_keys;
    use iris_core::KeyCode;

    let mut text = "Motor Testi".to_string();
    let mut cursor = text.len();
    let mut is_all_selected = false;

    // 1. Submit on Enter
    let res = process_text_input_keys(
        &mut text,
        &mut cursor,
        &mut is_all_selected,
        true,
        Some(KeyCode::Enter),
        None,
        false,
    );
    assert!(res.submitted);
    assert!(!res.changed);

    // 2. Clear focus on Escape
    let res = process_text_input_keys(
        &mut text,
        &mut cursor,
        &mut is_all_selected,
        true,
        Some(KeyCode::Escape),
        None,
        false,
    );
    assert!(res.should_clear_focus);
    assert!(!res.changed);

    // 3. Arrow left / right navigation within UTF-8 boundaries
    let res = process_text_input_keys(
        &mut text,
        &mut cursor,
        &mut is_all_selected,
        true,
        Some(KeyCode::ArrowLeft),
        None,
        false,
    );
    assert_eq!(cursor, "Motor Test".len());
    assert!(!res.changed);

    // 4. Ctrl+A on UTF-8 string -> sets all selected and marks changed for redraw
    let res = process_text_input_keys(
        &mut text,
        &mut cursor,
        &mut is_all_selected,
        true,
        Some(KeyCode::Character('a')),
        Some("a"),
        true,
    );
    assert!(is_all_selected);
    assert!(res.changed);

    // 5. Typing while all selected replaces entire text
    let res = process_text_input_keys(
        &mut text,
        &mut cursor,
        &mut is_all_selected,
        true,
        Some(KeyCode::Character('X')),
        Some("X"),
        false,
    );
    assert_eq!(text, "X");
    assert_eq!(cursor, 1);
    assert!(!is_all_selected);
    assert!(res.changed);
}

#[test]
fn test_input_box_mouse_click_caret_placement_and_double_click_selection() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x5020;
    let props = crate::declarative::scope::InputBoxProps::new(
        "Hello World",
        "Placeholder...",
        200.0,
        false,
        false,
    );

    // Initial pass to mount the node
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.input_box_tagged(&props, tag);
        let bounds = Rect::new(0.0, 0.0, 400.0, 100.0);
        scope.finish_layout(bounds);
    }

    let box_id = tree.find_node_by_tag(tag).expect("Box node exists");
    if let Some(node) = tree.get_mut(box_id) {
        node.computed_rect = Rect::new(0.0, 0.0, 200.0, 24.0);
    }

    // 1. Single click targeting character boundary after "Hel"
    let hel_width = iris_text::measure_text_advance("Hel", 11.0);
    tree.clear_frame_clicks();
    tree.event_state.frame_clicked_id = Some(box_id);
    tree.event_state.frame_clicked_tag = Some(tag);
    tree.event_state.frame_clicked_pos = Some(iris_core::Point::new(8.0 + hel_width, 12.0));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.input_box_tagged(&props, tag);
        assert!(resp.clicked());
    }

    // Caret index must resolve to index 3 ("Hel"), not full length 11
    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(3));
    assert_eq!(tree.event_state.text_edit_all_selected(tag), Some(false));

    // 2. Double click selects the entire buffer
    tree.clear_frame_clicks();
    tree.event_state.frame_clicked_id = Some(box_id);
    tree.event_state.frame_clicked_tag = Some(tag);
    tree.event_state.frame_double_clicked_tag = Some(tag);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.input_box_tagged(&props, tag);
        assert!(resp.clicked());
    }

    assert_eq!(tree.event_state.text_edit_all_selected(tag), Some(true));
}