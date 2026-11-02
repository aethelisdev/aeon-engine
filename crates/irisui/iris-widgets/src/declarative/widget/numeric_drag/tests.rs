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