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