// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Declarative Button and Checkbox Interaction Responses (`iris-widgets::declarative::tests::response_tests`)
//!
//! Verifies immediate reactive interaction states ([`WidgetResponse`]) produced by
//! declarative buttons and checkboxes during autonomous event dispatch cycles.

use crate::declarative::UiScope;
use iris_core::event::{MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect};
use iris_core::tree::UiTree;

#[test]
fn test_declarative_button_click_response() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    // 1. Initial build: render button in tree without any events
    let btn_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button("Kaydet");
        assert!(
            !resp.clicked(),
            "New button must not report clicked on clean tree"
        );
        assert!(
            !resp.hovered(),
            "New button must not report hovered on clean tree"
        );
        assert!(
            !resp.pressed(),
            "New button must not report pressed on clean tree"
        );
        resp.id()
    };

    // Assign a valid bounding box to the button for spatial hit-testing
    let btn_rect = Rect::new(50.0, 50.0, 100.0, 30.0);
    if let Some(node) = tree.get_mut(btn_id) {
        node.computed_rect = btn_rect;
    }

    let click_pt = Point::new(60.0, 60.0);

    // 2. MouseMove over button -> Autonomous hover detection
    let move_res = tree.dispatch_event(&UiEvent::MouseMove { point: click_pt });
    assert!(move_res.target == Some(btn_id));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button("Kaydet");
        assert!(
            resp.hovered(),
            "Button must report hovered() == true when pointer is over it"
        );
        assert!(
            !resp.pressed(),
            "Button must not report pressed on simple mouse move"
        );
        assert!(
            !resp.clicked(),
            "Button must not report clicked on mouse move"
        );
    }

    // 3. MouseDown on button -> Autonomous active pressed state
    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: click_pt,
    });
    assert!(down_res.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button("Kaydet");
        assert!(resp.hovered(), "Button must remain hovered during press");
        assert!(
            resp.pressed(),
            "Button must report pressed() == true during active mouse press"
        );
        assert!(
            !resp.clicked(),
            "Button must not report clicked before button release"
        );
    }

    // 4. MouseUp on button -> Confirmed click gesture!
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: click_pt,
    });
    assert!(up_res.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button("Kaydet");
        assert!(
            resp.clicked(),
            "Button MUST report clicked() == true upon mouse release confirmation"
        );
        assert!(
            !resp.pressed(),
            "Button must clear pressed state on release"
        );
    }
}

#[test]
fn test_declarative_checkbox_click_and_toggle_response() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    let mut is_active = false;

    // 1. Initial build: unchecked checkbox
    let chk_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.checkbox("Otomatik Kaydet", &mut is_active);
        assert!(!resp.clicked());
        assert!(!resp.changed());
        assert!(!is_active);
        resp.id()
    };

    let chk_rect = Rect::new(10.0, 10.0, 140.0, 24.0);
    if let Some(node) = tree.get_mut(chk_id) {
        node.computed_rect = chk_rect;
    }

    let click_pt = Point::new(20.0, 20.0);

    // 2. Perform click sequence on checkbox: MouseDown -> MouseUp
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: click_pt,
    });
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: click_pt,
    });
    assert!(up_res.is_consumed());

    // 3. Render scope with checkbox: must toggle boolean in-place and report clicked & changed
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.checkbox("Otomatik Kaydet", &mut is_active);
        assert!(resp.clicked(), "Checkbox must report clicked() == true");
        assert!(resp.changed(), "Checkbox must report changed() == true");
        assert!(
            is_active,
            "Checkbox underlying boolean MUST be toggled to true in-place"
        );
    }

    // 4. Click again: should toggle back to false
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: click_pt,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: click_pt,
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.checkbox("Otomatik Kaydet", &mut is_active);
        assert!(
            resp.clicked(),
            "Checkbox must report clicked() == true on second click"
        );
        assert!(
            resp.changed(),
            "Checkbox must report changed() == true on second click"
        );
        assert!(
            !is_active,
            "Checkbox underlying boolean MUST be toggled back to false"
        );
    }
}

#[test]
fn test_declarative_numeric_drag_changed_response() {
    use iris_core::WidgetCursor;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    let mut scale = 1.0f32;

    // 1. Initial build: render numeric scrubber in tree without interaction
    let drag_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.numeric_drag("Scale", &mut scale);
        assert!(
            !resp.changed(),
            "Clean drag scrubber must not report changed"
        );
        assert!(
            !resp.dragged(),
            "Clean drag scrubber must not report dragged"
        );
        assert!(
            !resp.hovered(),
            "Clean drag scrubber must not report hovered"
        );
        assert_eq!(scale, 1.0);
        resp.id()
    };

    // Assign computed rect for spatial hit-testing
    let drag_rect = Rect::new(20.0, 20.0, 100.0, 30.0);
    if let Some(node) = tree.get_mut(drag_id) {
        node.computed_rect = drag_rect;
    }

    let start_pt = Point::new(30.0, 25.0);

    // 2. MouseMove over scrubber -> Hovered and EwResize cursor
    let move_res = tree.dispatch_event(&UiEvent::MouseMove { point: start_pt });
    assert_eq!(move_res.target, Some(drag_id));
    assert_eq!(move_res.cursor, Some(WidgetCursor::EwResize));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.numeric_drag("Scale", &mut scale);
        assert!(resp.hovered(), "Scrubber must report hovered == true");
        assert!(!resp.dragged());
        assert!(!resp.changed());
        assert_eq!(scale, 1.0);
    }

    // 3. MouseDown on scrubber -> Initiates pointer capture for  scrubbing
    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: start_pt,
    });
    assert!(down_res.is_consumed());
    assert_eq!(
        tree.event_state.pointer_capture,
        Some(drag_id),
        "Numeric scrubber must capture pointer on MouseDown"
    );

    // 4. MouseMove displacing pointer horizontally by +30.0px
    let moved_pt = Point::new(60.0, 25.0);
    let drag_move_res = tree.dispatch_event(&UiEvent::MouseMove { point: moved_pt });
    assert!(drag_move_res.is_consumed());
    assert_eq!(drag_move_res.cursor, Some(WidgetCursor::EwResize));

    // Next frame declarative rendering: scope.numeric_drag must mutate scale and report changed()
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.numeric_drag("Scale", &mut scale);
        assert!(
            resp.dragged(),
            "Numeric scrubber must report dragged() == true while active"
        );
        assert!(
            resp.changed(),
            "Numeric scrubber must report changed() == true upon pointer displacement"
        );
        // Default speed is 0.1: delta_x = 30.0 * 0.1 = +3.0 -> scale becomes 4.0
        assert!(
            (scale - 4.0).abs() < 1e-4,
            "Scale value should have incremented by delta_x * speed (expected 4.0, got {})",
            scale
        );
    }

    // 5. MouseUp releasing pointer capture and finalizing scrubber interaction
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: moved_pt,
    });
    assert!(up_res.is_consumed());
    assert_eq!(
        tree.event_state.pointer_capture, None,
        "Pointer capture must be released on MouseUp"
    );
    assert_eq!(
        tree.event_state.last_pointer_pos, None,
        "Last pointer position must be reset on MouseUp"
    );

    // Frame after release: no longer dragging or changing
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.numeric_drag("Scale", &mut scale);
        assert!(!resp.dragged(), "Should no longer be dragged after MouseUp");
        assert!(!resp.changed(), "Should not report changed without delta");
        assert_eq!(scale, 4.0, "Scale must retain its updated value");
    }
}