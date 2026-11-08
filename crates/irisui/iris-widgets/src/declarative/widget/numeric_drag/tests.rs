// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # NumericDrag Unit Tests

use super::*;
use iris_core::UiTree;

#[test]
fn test_numeric_drag_builder_and_response() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let mut scope = UiScope::new(&mut tree, root);

    let mut val = 42.5;
    let resp = scope.add(
        NumericDrag::new("Speed", &mut val)
            .speed(0.2)
            .range(0.0..=100.0)
            .decimals(1)
            .unit("m/s")
            .label_width(70.0)
            .box_width(50.0),
    );

    assert!(!resp.changed());
    assert!(!resp.clicked());
    assert_eq!(resp.value(), 42.5);
    assert_eq!(val, 42.5);
}

#[test]
fn test_numeric_drag_clamping() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let mut scope = UiScope::new(&mut tree, root);

    let mut val = 150.0;
    let _ = scope.add(NumericDrag::new("Clamped", &mut val).range(0.0..=100.0));
    assert_eq!(val, 100.0);

    let mut under = -50.0;
    let _ = scope.add(NumericDrag::new("Under", &mut under).range(0.0..=100.0));
    assert_eq!(under, 0.0);
}

#[test]
fn test_numeric_drag_enter_confirmation_and_submitted_response() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");

    let tag = 0x998877;
    let mut val = 10.0;

    // Simulate active text editing session on tag
    tree.event_state.start_text_edit(None, tag, "25.5", 4, None);
    tree.event_state.focused_tag = Some(tag);
    tree.event_state.frame_key = Some(iris_core::event::KeyCode::Enter);
    tree.event_state.frame_key_pressed = true;

    let mut scope = UiScope::new(&mut tree, root);
    let resp = scope.add(NumericDrag::new("", &mut val).tag(tag).range(0.0..=100.0));

    assert_eq!(val, 25.5);
    assert!(resp.changed());
    assert!(resp.submitted());
    assert!(resp.committed());
    assert!(tree.event_state.active_text_edit.is_none());
}

#[test]
fn test_numeric_drag_clipboard_and_letter_filter() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let tag = 0x112233;
    let mut val = 5.0;

    // 1. Start active edit session with 5.0
    tree.event_state
        .start_text_edit(None, tag, "5.0", 3, Some((0, 3)));
    tree.event_state.focused_tag = Some(tag);

    // 2. Try pasting non-numeric letters "scale_factor"
    crate::input::copy_to_clipboard("scale_factor");
    tree.event_state.frame_key = Some(iris_core::event::KeyCode::Character('v'));
    tree.event_state.frame_key_pressed = true;
    tree.event_state.frame_ctrl_held = true;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.add(NumericDrag::new("Scale", &mut val).tag(tag));
    }

    // Letters should be completely rejected, session text stays "5.0"
    let session = tree.event_state.active_text_edit.as_ref().unwrap();
    assert_eq!(session.buffer, "5.0");

    // 3. Move cursor to end to unselect all, then paste valid numeric expression " + 2.5"
    tree.clear_frame_clicks();
    tree.event_state.frame_key = Some(iris_core::event::KeyCode::ArrowRight);
    tree.event_state.frame_key_pressed = true;
    tree.event_state.frame_ctrl_held = false;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.add(NumericDrag::new("Scale", &mut val).tag(tag));
    }

    crate::input::copy_to_clipboard(" + 2.5");
    tree.clear_frame_clicks();
    tree.event_state.frame_key = Some(iris_core::event::KeyCode::Character('v'));
    tree.event_state.frame_key_pressed = true;
    tree.event_state.frame_ctrl_held = true;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.add(NumericDrag::new("Scale", &mut val).tag(tag));
    }

    let session = tree.event_state.active_text_edit.as_ref().unwrap();
    assert_eq!(session.buffer, "5.0 + 2.5");

    // 4. Submit via Enter key
    tree.clear_frame_clicks();
    tree.event_state.frame_key = Some(iris_core::event::KeyCode::Enter);
    tree.event_state.frame_key_pressed = true;
    tree.event_state.frame_ctrl_held = false;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("Scale", &mut val).tag(tag));
        assert!(resp.changed());
        assert!(resp.submitted());
    }

    assert_eq!(val, 7.5);
    assert!(tree.event_state.active_text_edit.is_none());
}

#[test]
fn test_numeric_drag_exponential_and_logarithmic_curves() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let tag = 0x556677;

    // 1. Exponential curve scrubbing test
    let mut val = 10.0;
    tree.event_state.is_pointer_drag_active = true;
    tree.event_state.pointer_capture_tag = Some(tag);
    tree.event_state.frame_drag_delta = Some(iris_core::Point::new(10.0, 0.0));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            NumericDrag::new("Scrub", &mut val)
                .tag(tag)
                .speed(0.1)
                .pixels_per_step(10.0)
                .curve(DragCurve::quadratic())
                .range(0.0..=100.0),
        );
        assert!(resp.changed());
        assert!(resp.dragged());
    }

    // At delta_x = 10.0 and pixels_per_step = 10.0, ratio is 1.0, so delta = 1.0^2 * 0.1 * 10 = 1.0
    assert!((val - 11.0).abs() < 1e-4);

    // 2. Micro-precision at delta_x = 1.0 (sub-linear fine control)
    tree.event_state.frame_drag_delta = Some(iris_core::Point::new(1.0, 0.0));
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            NumericDrag::new("Scrub", &mut val)
                .tag(tag)
                .speed(0.1)
                .pixels_per_step(10.0)
                .curve(DragCurve::quadratic())
                .range(0.0..=100.0),
        );
        assert!(resp.changed());
    }
    // ratio = 0.1, ratio^2 = 0.01, delta = 0.01 * 1.0 = 0.01
    assert!((val - 11.01).abs() < 1e-4);

    // 3. Logarithmic compression curve scrubbing test
    let mut log_val = 50.0;
    tree.event_state.frame_drag_delta = Some(iris_core::Point::new(20.0, 0.0));
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            NumericDrag::new("LogScrub", &mut log_val)
                .tag(tag)
                .speed(0.1)
                .pixels_per_step(10.0)
                .curve(DragCurve::Logarithmic)
                .range(0.0..=100.0),
        );
        assert!(resp.changed());
    }
    // ratio = 2.0, ln(3)/ln(2) ~ 1.5849625, delta = 1.5849625 * 1.0 ~ 1.5849625
    assert!((log_val - 51.58496).abs() < 1e-3);
}

#[test]
fn test_numeric_drag_undo_disabled() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");

    let tag = 8888;
    tree.event_state.pointer_capture_tag = Some(tag);
    tree.event_state.is_pointer_drag_active = true;
    tree.event_state.frame_drag_delta = Some(iris_core::Point::new(10.0, 0.0));

    let mut val = 10.0;
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            NumericDrag::new("NoUndo", &mut val)
                .tag(tag)
                .undo_enabled(false)
                .range(0.0..=100.0),
        );
        assert!(resp.changed());
    }

    // Because undo was disabled on the widget, nothing should have been pushed to numeric_undo
    assert_eq!(tree.event_state.numeric_undo.active_drag_start(tag), None);
    assert!(!tree.event_state.numeric_undo.can_undo());

    // Release drag
    tree.event_state.frame_drag_released_tag = Some(tag);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            NumericDrag::new("NoUndo", &mut val)
                .tag(tag)
                .with_undo(false)
                .range(0.0..=100.0),
        );
        assert!(resp.drag_released());
    }
    assert!(!tree.event_state.numeric_undo.can_undo());
}

#[test]
fn test_numeric_drag_caret_node_creation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");

    let tag = 4455;
    tree.event_state.active_text_edit = Some(iris_core::dispatch::ActiveTextEditSession {
        tag,
        buffer: "123.45".to_string(),
        cursor_byte_idx: 3,
        selection: None,
        undo_stack: iris_core::dispatch::TextUndoStack::new(),
        allow_clipboard_shortcuts: true,
        allow_undo_shortcuts: true,
    });

    let mut val = 123.45;
    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.add(NumericDrag::new("Mass", &mut val).tag(tag))
    };

    let pill_id = resp.id();
    let children = tree.get(pill_id).unwrap().children.clone();
    let caret_node_id = children.into_iter().find(|&cid| {
        tree.get(cid)
            .is_some_and(|n| n.role == iris_core::WidgetRole::TextInputCaret)
    });

    assert!(
        caret_node_id.is_some(),
        "TextInputCaret must exist during active text editing"
    );
    let caret = tree.get(caret_node_id.unwrap()).unwrap();
    assert_eq!(caret.name.as_deref(), Some("TextInputCaret"));
    assert_eq!(caret.role, iris_core::WidgetRole::TextInputCaret);
    assert!(!caret.interactive);
    assert_eq!(caret.style.position, iris_core::Position::Absolute);
    assert!(caret.style.inset_left.is_some());
}

#[test]
fn test_numeric_drag_caret_suppressed_during_active_selection() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");

    let tag = 4456;
    tree.event_state.active_text_edit = Some(iris_core::dispatch::ActiveTextEditSession {
        tag,
        buffer: "123.45".to_string(),
        cursor_byte_idx: 6,
        selection: Some((0, 6)),
        undo_stack: iris_core::dispatch::TextUndoStack::new(),
        allow_clipboard_shortcuts: true,
        allow_undo_shortcuts: true,
    });

    let mut val = 123.45;
    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.add(NumericDrag::new("Mass", &mut val).tag(tag))
    };

    let pill_id = resp.id();
    let children = tree.get(pill_id).unwrap().children.clone();
    let caret_exists = children.iter().any(|&cid| {
        tree.get(cid)
            .is_some_and(|n| n.role == iris_core::WidgetRole::TextInputCaret)
    });
    let selection_exists = children.iter().any(|&cid| {
        tree.get(cid)
            .is_some_and(|n| n.name.as_deref() == Some("Numeric_Sel"))
    });

    assert!(
        !caret_exists,
        "TextInputCaret must NOT be rendered when text is selected"
    );
    assert!(
        selection_exists,
        "Numeric_Sel must be rendered when text is selected"
    );
}