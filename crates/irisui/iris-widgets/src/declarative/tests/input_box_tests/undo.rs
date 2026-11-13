// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Input Box Autonomous Undo & Redo Shortcuts (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`)
//!
//! Verifies autonomous undo/redo restoration, word boundary coalescing, cross-platform shortcut
//! mappings, and event consumption guarantees preventing editor/scene event propagation.

use crate::declarative::UiScope;
use iris_core::{KeyCode, Point, Rect, UiEvent, UiTree};

#[test]
fn test_input_box_ctrl_z_undo_and_ctrl_y_redo() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x7701u64;
    let mut text_buffer = "Initial".to_string();

    // 1. Initial layout and mounting
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
        let bounds = Rect::new(0.0, 0.0, 400.0, 100.0);
        scope.finish_layout(bounds);
        if let Some(n) = tree.get_mut(resp.id) {
            n.computed_rect = Rect::new(10.0, 10.0, 200.0, 24.0);
        }
    }

    // 2. Click text input at the end of text to focus
    let initial_width = iris_text::measure_text_advance(&text_buffer, 11.0);
    let click = UiEvent::MouseDown {
        button: iris_core::MouseButton::Left,
        point: Point::new(18.0 + initial_width + 5.0, 20.0),
    };
    let res = tree.dispatch_event(&click);
    assert!(res.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
    }
    assert!(tree.is_text_input_focused());

    // 3. Type " Word" -> triggers word boundary
    let space_ev = UiEvent::KeyboardInput {
        key: KeyCode::Space,
        text: Some(" ".to_string()),
        is_pressed: true,
        ctrl_held: false,
        shift_held: false,
    };
    tree.dispatch_event(&space_ev);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
        assert!(resp.changed());
        assert_eq!(text_buffer, "Initial ");
    }

    // Type 'A'
    let a_ev = UiEvent::KeyboardInput {
        key: KeyCode::Character('A'),
        text: Some("A".to_string()),
        is_pressed: true,
        ctrl_held: false,
        shift_held: false,
    };
    tree.dispatch_event(&a_ev);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
        assert!(resp.changed());
        assert_eq!(text_buffer, "Initial A");
    }

    // 4. Press Ctrl+Z (Undo) -> Should revert 'A', returning to "Initial "
    let undo_ev = UiEvent::KeyboardInput {
        key: KeyCode::Character('z'),
        text: Some("z".to_string()),
        is_pressed: true,
        ctrl_held: true,
        shift_held: false,
    };
    let undo_res = tree.dispatch_event(&undo_ev);
    assert!(
        undo_res.is_consumed(),
        "Ctrl+Z on focused input must be consumed"
    );

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
        assert!(resp.changed(), "Undo must trigger changed flag");
        assert_eq!(text_buffer, "Initial ");
    }

    // 5. Press Ctrl+Z again -> Should revert space, returning to "Initial"
    let undo_res2 = tree.dispatch_event(&undo_ev);
    assert!(undo_res2.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
        assert!(resp.changed());
        assert_eq!(text_buffer, "Initial");
    }

    // 6. Press Ctrl+Y (Redo - Windows/Linux) -> Should redo forward to "Initial "
    let redo_y_ev = UiEvent::KeyboardInput {
        key: KeyCode::Character('y'),
        text: Some("y".to_string()),
        is_pressed: true,
        ctrl_held: true,
        shift_held: false,
    };
    let redo_res = tree.dispatch_event(&redo_y_ev);
    assert!(
        redo_res.is_consumed(),
        "Ctrl+Y on focused input must be consumed"
    );

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
        assert!(resp.changed());
        assert_eq!(text_buffer, "Initial ");
    }

    // Redo forward to "Initial A"
    let redo_res2 = tree.dispatch_event(&redo_y_ev);
    assert!(redo_res2.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
        assert!(resp.changed());
        assert_eq!(text_buffer, "Initial A");
    }
}

#[test]
fn test_input_box_ctrl_shift_z_redo() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x7702u64;
    let mut text_buffer = "Scene".to_string();

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
        let bounds = Rect::new(0.0, 0.0, 400.0, 100.0);
        scope.finish_layout(bounds);
        if let Some(n) = tree.get_mut(resp.id) {
            n.computed_rect = Rect::new(10.0, 10.0, 200.0, 24.0);
        }
    }

    // Focus input at the end of text
    let scene_width = iris_text::measure_text_advance(&text_buffer, 11.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: iris_core::MouseButton::Left,
        point: Point::new(18.0 + scene_width + 5.0, 20.0),
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
    }

    // Type character '1'
    tree.dispatch_event(&UiEvent::KeyboardInput {
        key: KeyCode::Character('1'),
        text: Some("1".to_string()),
        is_pressed: true,
        ctrl_held: false,
        shift_held: false,
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
        assert_eq!(text_buffer, "Scene1");
    }

    // Undo via Ctrl+Z
    tree.dispatch_event(&UiEvent::KeyboardInput {
        key: KeyCode::Character('z'),
        text: Some("z".to_string()),
        is_pressed: true,
        ctrl_held: true,
        shift_held: false,
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
        assert!(resp.changed());
        assert_eq!(text_buffer, "Scene");
    }

    // Redo via macOS / Modern IDE standard: Ctrl+Shift+Z
    let redo_shift_z = UiEvent::KeyboardInput {
        key: KeyCode::Character('z'),
        text: Some("Z".to_string()),
        is_pressed: true,
        ctrl_held: true,
        shift_held: true,
    };
    let redo_res = tree.dispatch_event(&redo_shift_z);
    assert!(
        redo_res.is_consumed(),
        "Ctrl+Shift+Z must be consumed by focused input box"
    );

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Search...", tag, 200.0, &mut text_buffer);
        assert!(resp.changed());
        assert_eq!(text_buffer, "Scene1");
    }
}

#[test]
fn test_input_box_undo_event_consumption() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x7703u64;
    let mut text_buffer = "".to_string();

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp =
            scope.text_input_with_width_tagged("Placeholder...", tag, 200.0, &mut text_buffer);
        let bounds = Rect::new(0.0, 0.0, 400.0, 100.0);
        scope.finish_layout(bounds);
        if let Some(n) = tree.get_mut(resp.id) {
            n.computed_rect = Rect::new(10.0, 10.0, 200.0, 24.0);
        }
    }

    // 1. Unfocused state: Ctrl+Z must NOT be consumed by tree
    let undo_ev = UiEvent::KeyboardInput {
        key: KeyCode::Character('z'),
        text: Some("z".to_string()),
        is_pressed: true,
        ctrl_held: true,
        shift_held: false,
    };
    let unfocused_res = tree.dispatch_event(&undo_ev);
    assert!(
        !unfocused_res.is_consumed(),
        "Unfocused Ctrl+Z must NOT be consumed so scene editor can handle it"
    );

    // 2. Click text input to focus
    tree.dispatch_event(&UiEvent::MouseDown {
        button: iris_core::MouseButton::Left,
        point: Point::new(20.0, 20.0),
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input_with_width_tagged("Placeholder...", tag, 200.0, &mut text_buffer);
    }
    assert!(tree.is_text_input_focused());

    // 3. Focused empty input: Ctrl+Z MUST be consumed (swallowing scene undo) even though undo stack is empty!
    let focused_undo_res = tree.dispatch_event(&undo_ev);
    assert!(
        focused_undo_res.is_consumed(),
        "Focused input MUST consume Ctrl+Z unconditionally to shield editor undo"
    );

    // 4. Focused empty input: Ctrl+Y MUST be consumed unconditionally
    let redo_y_ev = UiEvent::KeyboardInput {
        key: KeyCode::Character('y'),
        text: Some("y".to_string()),
        is_pressed: true,
        ctrl_held: true,
        shift_held: false,
    };
    let focused_redo_res = tree.dispatch_event(&redo_y_ev);
    assert!(
        focused_redo_res.is_consumed(),
        "Focused input MUST consume Ctrl+Y unconditionally to shield editor redo"
    );

    // 5. Focused empty input: Ctrl+Shift+Z MUST be consumed unconditionally
    let redo_shift_z = UiEvent::KeyboardInput {
        key: KeyCode::Character('z'),
        text: Some("Z".to_string()),
        is_pressed: true,
        ctrl_held: true,
        shift_held: true,
    };
    let focused_shift_z_res = tree.dispatch_event(&redo_shift_z);
    assert!(
        focused_shift_z_res.is_consumed(),
        "Focused input MUST consume Ctrl+Shift+Z unconditionally to shield editor redo"
    );
}

#[test]
fn test_numeric_filter_typing_and_pasting() {
    let _guard = crate::input::CLIPBOARD_TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    use crate::declarative::scope::input_box::interaction::{
        TextInputFilter, TextInputKeyContext, process_text_input_keys,
    };
    use iris_core::dispatch::TextUndoStack;

    let mut text = String::new();
    let mut cursor = 0;
    let mut selection = None;
    let mut undo_stack = TextUndoStack::new();

    // 1. Try typing alphabetic letters into a numeric-filtered input
    let ctx_alpha =
        TextInputKeyContext::new(true, Some(KeyCode::Character('a')), Some("a"), false, false)
            .with_filter(TextInputFilter::NumericExpression);

    let res = process_text_input_keys(
        &mut text,
        &mut cursor,
        &mut selection,
        ctx_alpha,
        &mut undo_stack,
    );
    assert!(
        !res.changed,
        "Alphabetic character 'a' MUST NOT mutate text in NumericExpression mode"
    );
    assert_eq!(text, "");
    assert_eq!(cursor, 0);

    // 2. Type valid numeric digits and decimal point
    let ctx_num1 =
        TextInputKeyContext::new(true, Some(KeyCode::Character('4')), Some("4"), false, false)
            .with_filter(TextInputFilter::NumericExpression);
    let res = process_text_input_keys(
        &mut text,
        &mut cursor,
        &mut selection,
        ctx_num1,
        &mut undo_stack,
    );
    assert!(res.changed);
    assert_eq!(text, "4");
    assert_eq!(cursor, 1);

    let ctx_dot =
        TextInputKeyContext::new(true, Some(KeyCode::Character('.')), Some("."), false, false)
            .with_filter(TextInputFilter::NumericExpression);
    let _ = process_text_input_keys(
        &mut text,
        &mut cursor,
        &mut selection,
        ctx_dot,
        &mut undo_stack,
    );

    let ctx_num2 =
        TextInputKeyContext::new(true, Some(KeyCode::Character('2')), Some("2"), false, false)
            .with_filter(TextInputFilter::NumericExpression);
    let _ = process_text_input_keys(
        &mut text,
        &mut cursor,
        &mut selection,
        ctx_num2,
        &mut undo_stack,
    );
    assert_eq!(text, "4.2");

    // 3. Paste via clipboard: simulate clipboard containing invalid characters
    crate::input::copy_to_clipboard("letters_only");
    let ctx_paste_invalid =
        TextInputKeyContext::new(true, Some(KeyCode::Character('v')), Some("v"), true, false)
            .with_filter(TextInputFilter::NumericExpression);
    let res_paste = process_text_input_keys(
        &mut text,
        &mut cursor,
        &mut selection,
        ctx_paste_invalid,
        &mut undo_stack,
    );
    assert!(
        !res_paste.changed,
        "Pasting purely alphabetic string MUST NOT change numeric buffer"
    );
    assert_eq!(text, "4.2");

    // 4. Paste valid numeric expression with some letters mixed in
    crate::input::copy_to_clipboard(" + 5.0kg");
    let ctx_paste_mixed =
        TextInputKeyContext::new(true, Some(KeyCode::Character('v')), Some("v"), true, false)
            .with_filter(TextInputFilter::NumericExpression);
    let res_paste2 = process_text_input_keys(
        &mut text,
        &mut cursor,
        &mut selection,
        ctx_paste_mixed,
        &mut undo_stack,
    );
    assert!(res_paste2.changed);
    assert_eq!(text, "4.2 + 5.0");

    // 5. Test undo in numeric input
    let ctx_undo =
        TextInputKeyContext::new(true, Some(KeyCode::Character('z')), Some("z"), true, false)
            .with_filter(TextInputFilter::NumericExpression);
    let res_undo = process_text_input_keys(
        &mut text,
        &mut cursor,
        &mut selection,
        ctx_undo,
        &mut undo_stack,
    );
    assert!(res_undo.changed);
    assert_eq!(text, "4.2");
}