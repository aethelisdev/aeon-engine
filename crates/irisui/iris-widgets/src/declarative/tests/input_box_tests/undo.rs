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