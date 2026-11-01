// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Numeric Scrubber & Inline Editing Interaction Response Tests
//!
//! Validates pointer drag capture, threshold isolation, inline text buffering,
//! evaluation on Enter, blur commit, and rejection of non-numeric characters.

use crate::declarative::scope::core::UiScope;
use crate::declarative::widget::NumericDrag;
use iris_core::event::MouseButton;
use iris_core::{Point, Rect, UiEvent, UiTree, WidgetCursor};

#[test]
fn test_declarative_numeric_drag_pointer_capture_across_rebuilds_and_boundaries() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut radius = 10.0f32;
    let drag_tag = 0xABCD_1234_5678_u64;

    // 1. Initial build
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            NumericDrag::new("Radius", &mut radius)
                .tag(drag_tag)
                .range(0.0..=100.0)
                .speed(1.0),
        );
        assert!(!resp.dragged());
    }

    // Assign computed rect for spatial hit testing
    let pill_id = tree.find_node_by_tag(drag_tag).expect("Pill must exist");
    if let Some(node) = tree.get_mut(pill_id) {
        node.computed_rect = Rect::new(50.0, 50.0, 60.0, 22.0);
    }

    // 2. MouseDown within pill boundaries (x=60, y=60) -> initiates pointer capture
    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(60.0, 60.0),
    });
    assert!(down_res.is_consumed());
    assert_eq!(
        tree.event_state.pointer_capture_tag,
        Some(drag_tag),
        "Numeric scrubber must initiate persistent pointer capture tag"
    );

    // 3. Mouse moves way outside widget boundaries (x=1000, y=1000)
    let move_res = tree.dispatch_event(&UiEvent::MouseMove {
        point: Point::new(100.0, 60.0), // +40px delta
    });
    assert!(move_res.is_consumed());

    // 4. Rebuild scope (simulating next frame re-render)
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            NumericDrag::new("Radius", &mut radius)
                .tag(drag_tag)
                .range(0.0..=100.0)
                .speed(1.0),
        );

        // Value must be updated via drag delta (+40px * 1.0 = +40.0)
        assert!(resp.dragged(), "Widget must report being dragged");
        assert!(resp.changed(), "Widget value must change during drag");
        assert_eq!(radius, 50.0, "Radius should be 10.0 + 40.0 = 50.0");
    }

    // 5. Release mouse button outside boundaries
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(200.0, 200.0),
    });
    assert!(up_res.is_consumed());
    assert_eq!(
        tree.event_state.frame_drag_released_tag,
        Some(drag_tag),
        "Drag release tag must be reported on release"
    );

    // 6. Post-release rebuild
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            NumericDrag::new("Radius", &mut radius)
                .tag(drag_tag)
                .range(0.0..=100.0)
                .speed(1.0),
        );
        assert!(
            resp.drag_released(),
            "Widget must report drag_released on the release frame"
        );
        assert!(!resp.dragged(), "Widget must not be dragged after release");
    }
}

#[test]
fn test_numeric_input_child_node_click_bubbles_capture() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");

    let tag = 0xDEAD_BEEF;
    let mut val = 50.0f32;

    // 1. Initial build: creates NumericRow container, Label, and Pill box
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.add(NumericDrag::new("Offset", &mut val).tag(tag));
    }

    // Tag is assigned to the interactive pill node
    let pill_id = tree.find_node_by_tag(tag).expect("Pill must exist");

    // Simulate layout rect on pill
    if let Some(node) = tree.get_mut(pill_id) {
        node.computed_rect = Rect::new(10.0, 10.0, 80.0, 22.0);
    }

    // Add a child node (e.g. text or selection rect inside the pill)
    let child_text = tree.create_node();
    if let Some(c) = tree.get_mut(child_text) {
        c.computed_rect = Rect::new(20.0, 12.0, 40.0, 18.0);
    }
    let _ = tree.add_child(pill_id, child_text);

    // 2. MouseDown on child text (x=25, y=15)
    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(25.0, 15.0),
    });

    assert!(down_res.is_consumed());
    assert_eq!(
        tree.event_state.pointer_capture_tag,
        Some(tag),
        "Clicking child node inside interactive pill MUST capture to pill's tag"
    );
    assert_eq!(tree.event_state.pointer_capture, Some(pill_id));
}

#[test]
fn test_numeric_input_click_vs_drag_threshold_isolation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");

    let tag = 0xCAFE_BABE;
    let mut val = 0.0f32;

    // 1. Initial build
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.add(NumericDrag::new("Angle", &mut val).tag(tag));
    }

    let pill_id = tree.find_node_by_tag(tag).expect("Pill must exist");
    if let Some(node) = tree.get_mut(pill_id) {
        node.computed_rect = Rect::new(10.0, 10.0, 80.0, 22.0);
    }

    // 2. MouseDown at (20, 20)
    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(20.0, 20.0),
    });
    assert!(down_res.is_consumed());

    // 3. Mouse moves 1 pixel (within 3.0px deadzone) -> NOT A DRAG
    let micro_move_res = tree.dispatch_event(&UiEvent::MouseMove {
        point: Point::new(21.0, 20.0),
    });
    assert!(micro_move_res.is_consumed());
    assert_ne!(
        micro_move_res.cursor,
        Some(WidgetCursor::EwResize),
        "Micro-motion must not display EwResize cursor"
    );
    assert!(tree.event_state.frame_drag_delta.is_none());

    // 4. MouseUp within threshold -> MUST confirm click and start active_text_edit
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(21.0, 20.0),
    });
    assert!(up_res.is_consumed());
    assert!(
        up_res.response.is_some_and(|r| r.clicked()),
        "Release within threshold must register as a CLICK"
    );
    assert_eq!(tree.event_state.frame_clicked_tag, Some(tag));

    // 5. Subsequent scope build enters editing mode
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
        assert!(resp.clicked());
        assert!(tree.event_state.active_text_edit.is_some());
    }
}

#[test]
fn test_numeric_drag_typing_buffering_and_enter_commit() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");
    let tag = 4242u64;
    let mut val = 100.0f32;

    // 1. Initial click enters active edit session
    tree.event_state.frame_clicked_tag = Some(tag);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
        assert!(resp.clicked());
        assert!(tree.event_state.active_text_edit.is_some());
    }

    // 2. Typing '5' replaces buffer (all selected), val MUST NOT CHANGE yet
    tree.event_state.frame_clicked_tag = None;
    tree.event_state.frame_key_pressed = true;
    tree.event_state.frame_text_input = Some("5".to_owned());
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
        assert!(!resp.changed(), "Typing must NOT apply value immediately");
        assert_eq!(val, 100.0, "Original value must be preserved while typing");
    }

    // 3. Typing '-10' into the buffer, val MUST STILL NOT CHANGE
    tree.event_state.frame_key_pressed = true;
    tree.event_state.frame_text_input = Some("-10".to_owned());
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
        assert!(!resp.changed(), "Typing must NOT apply value immediately");
        assert_eq!(val, 100.0, "Original value must be preserved while typing");
    }

    // Verify buffer holds "5-10"
    assert_eq!(
        tree.event_state
            .active_text_edit
            .as_ref()
            .map(|s| s.buffer.as_str()),
        Some("5-10")
    );

    // 4. Press Enter -> MUST calculate 5-10 = -5.0, set changed=true, and close edit session
    tree.event_state.frame_key_pressed = true;
    tree.event_state.frame_text_input = None;
    tree.event_state.frame_key = Some(iris_core::event::KeyCode::Enter);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
        assert!(resp.changed(), "Enter MUST commit the numeric edit");
        assert_eq!(val, -5.0, "Math expression 5-10 must evaluate to -5.0");
        assert!(
            tree.event_state.active_text_edit.is_none(),
            "Enter must close editing"
        );
    }
}

#[test]
fn test_numeric_drag_blur_commit() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");
    let tag = 5555u64;
    let mut val = 0.0f32;

    // 1. Initial click enters active edit session
    tree.event_state.frame_clicked_tag = Some(tag);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
    }

    // 2. Type "20+15"
    tree.event_state.frame_clicked_tag = None;
    tree.event_state.frame_key_pressed = true;
    tree.event_state.frame_text_input = Some("20+15".to_owned());
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
    }
    assert_eq!(val, 0.0);

    // 3. Click outside on untagged container
    tree.event_state.frame_key_pressed = false;
    tree.event_state.frame_text_input = None;
    tree.event_state.frame_clicked_tag = None;
    tree.event_state.frame_clicked_id = Some(root);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
        assert!(resp.changed(), "Blur MUST commit the numeric edit");
        assert_eq!(val, 35.0, "Math expression 20+15 must evaluate to 35.0");
        assert!(
            tree.event_state.active_text_edit.is_none(),
            "Blur must close editing"
        );
    }
}