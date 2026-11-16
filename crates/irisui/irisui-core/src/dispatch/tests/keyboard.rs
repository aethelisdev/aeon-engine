// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for keyboard navigation, IME events, and text/numeric input focus isolation.

use crate::event::{ImeEvent, KeyCode, MouseButton, UiEvent};
use crate::geometry::{Point, Rect};
use crate::node::WidgetRole;
use crate::tree::UiTree;

#[test]
fn test_ui_tree_dispatch_event_keyboard_and_ime() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let input_box = tree.create_node();
    tree.add_child(root, input_box).unwrap();

    {
        let node = tree.get_mut(input_box).unwrap();
        node.visible = true;
        node.interactive = true;
        node.role = WidgetRole::TextInput;
        node.computed_rect = Rect::new(10.0, 10.0, 200.0, 30.0);
        node.tag = 999;
    }

    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(15.0, 15.0),
    });
    assert_eq!(tree.event_state.focused, Some(input_box));

    let key_evt = UiEvent::KeyboardInput {
        key: KeyCode::Character('A'),
        text: Some("A".to_string()),
        is_pressed: true,
        ctrl_held: false,
        shift_held: false,
    };
    let key_res = tree.dispatch_event(&key_evt);
    assert!(key_res.is_consumed());
    assert_eq!(key_res.target, Some(input_box));
    assert_eq!(key_res.key, Some(KeyCode::Character('A')));
    assert!(key_res.is_key_pressed);
    assert_eq!(key_res.text_input.as_deref(), Some("A"));
    assert!(key_res.response.map(|r| r.changed()).unwrap_or(false));

    let ime_evt = UiEvent::Ime(ImeEvent::Commit("ğ".to_string()));
    let ime_res = tree.dispatch_event(&ime_evt);
    assert!(ime_res.is_consumed());
    assert_eq!(ime_res.target, Some(input_box));
    assert_eq!(ime_res.text_input.as_deref(), Some("ğ"));
    assert!(ime_res.response.map(|r| r.changed()).unwrap_or(false));
}

#[test]
fn test_button_focus_does_not_consume_character_keys_such_as_wasd() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let btn = tree.create_node();
    tree.add_child(root, btn).unwrap();

    {
        let b = tree.get_mut(btn).unwrap();
        b.visible = true;
        b.interactive = true;
        b.role = WidgetRole::Button;
        b.computed_rect = Rect::new(10.0, 10.0, 80.0, 30.0);
        b.tag = 100;
    }

    // 1. Click button -> button becomes focused
    let click = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(20.0, 20.0),
    };
    let res = tree.dispatch_event(&click);
    assert!(res.is_consumed());
    assert_eq!(tree.event_state.focused, Some(btn));
    assert_eq!(tree.event_state.focused_tag, Some(100));

    // Verify tree reports NO text input focus on button
    assert!(!tree.is_text_input_focused());
    assert!(!tree.event_state.has_text_input_focus());

    // 2. Press 'W' key -> Must NOT be consumed by button! (Crucial for 3D camera / game controls)
    let key_w = UiEvent::KeyboardInput {
        key: KeyCode::Character('w'),
        text: Some("w".to_string()),
        is_pressed: true,
        ctrl_held: false,
        shift_held: false,
    };
    let res_w = tree.dispatch_event(&key_w);
    assert!(
        !res_w.is_consumed(),
        "Character keys ('W') must NOT be consumed by non-text widgets like buttons"
    );

    // 3. Press 'A', 'S', 'D' keys -> Must NOT be consumed
    for k in ['a', 's', 'd'] {
        let key_evt = UiEvent::KeyboardInput {
            key: KeyCode::Character(k),
            text: Some(k.to_string()),
            is_pressed: true,
            ctrl_held: false,
            shift_held: false,
        };
        let res_k = tree.dispatch_event(&key_evt);
        assert!(
            !res_k.is_consumed(),
            "Camera navigation key ('{k}') must NOT be swallowed by button focus"
        );
    }

    // 4. Press Space or Enter -> Consumed and triggers Clicked on button
    let key_space = UiEvent::KeyboardInput {
        key: KeyCode::Space,
        text: None,
        is_pressed: true,
        ctrl_held: false,
        shift_held: false,
    };
    let res_space = tree.dispatch_event(&key_space);
    assert!(
        res_space.is_consumed(),
        "Space key on focused button must be consumed"
    );
    assert!(res_space.response.map(|r| r.clicked()).unwrap_or(false));
}

#[test]
fn test_text_input_focus_consumes_all_character_keys() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let input = tree.create_node();
    tree.add_child(root, input).unwrap();

    {
        let inp = tree.get_mut(input).unwrap();
        inp.visible = true;
        inp.interactive = true;
        inp.role = WidgetRole::TextInput;
        inp.computed_rect = Rect::new(10.0, 10.0, 150.0, 30.0);
        inp.tag = 200;
    }

    // 1. Click text input -> input becomes focused
    let click = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(20.0, 20.0),
    };
    let res = tree.dispatch_event(&click);
    assert!(res.is_consumed());
    assert_eq!(tree.event_state.focused, Some(input));

    // Verify tree reports active text input focus
    assert!(tree.is_text_input_focused());

    // 2. Press 'W' key -> MUST be consumed by text input
    let key_w = UiEvent::KeyboardInput {
        key: KeyCode::Character('w'),
        text: Some("w".to_string()),
        is_pressed: true,
        ctrl_held: false,
        shift_held: false,
    };
    let res_w = tree.dispatch_event(&key_w);
    assert!(
        res_w.is_consumed(),
        "Text input MUST consume character typing keys ('w')"
    );
    assert!(res_w.response.map(|r| r.changed()).unwrap_or(false));
}

#[test]
fn test_numeric_input_focus_does_not_swallow_keyboard_unless_editing() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let num_node = tree.create_node();
    tree.add_child(root, num_node).unwrap();

    {
        let node = tree.get_mut(num_node).unwrap();
        node.visible = true;
        node.interactive = true;
        node.role = WidgetRole::NumericInput;
        node.computed_rect = Rect::new(10.0, 10.0, 100.0, 24.0);
        node.tag = 300;
    }

    // 1. Mouse down / dragging on numeric input sets focus to the node
    let click = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(20.0, 20.0),
    };
    let res = tree.dispatch_event(&click);
    assert!(res.is_consumed());
    assert_eq!(tree.event_state.focused, Some(num_node));

    // 2. Crucial Invariant: Scrubber dragging must NOT report text input focus!
    assert!(
        !tree.is_text_input_focused(),
        "Numeric input scrubber must NOT report text input focus when not in active inline text edit mode"
    );

    // 3. Ctrl+Z shortcut dispatched must NOT be consumed
    let key_ctrl_z = UiEvent::KeyboardInput {
        key: KeyCode::Character('z'),
        text: Some("z".to_string()),
        is_pressed: true,
        ctrl_held: true,
        shift_held: false,
    };
    let res_z = tree.dispatch_event(&key_ctrl_z);
    assert!(
        !res_z.is_consumed(),
        "Global shortcuts (Ctrl+Z) must NOT be swallowed when scrubbing numeric inputs"
    );

    // 4. Activating inline text editing mode MUST enable text input focus
    tree.event_state
        .start_text_edit(Some(num_node), 300, "12.5", 4, None);
    assert!(
        tree.is_text_input_focused(),
        "Active inline text editing session on numeric field must report text input focus"
    );

    // 5. Blurring / clearing text edit must immediately release text input focus
    tree.event_state.clear_text_edit();
    assert!(
        !tree.is_text_input_focused(),
        "Clearing text edit on numeric field must immediately release text input focus"
    );
}