// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Collapsing Headers & Accordion Section Unit Tests
//!
//! Validates autonomous state persistence, foldout chevron transitions (`▶` / `▼`),
//! child scope execution, in-place click toggles, and builder configurations.

use crate::declarative::scope::UiScope;
use iris_core::{Point, Rect, UiEvent, UiTree, WidgetRole, event::MouseButton};

#[test]
fn test_declarative_collapsing_default_closed_and_toggle() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must be created");

    let mut body_executed = false;

    // 1. Initial frame: collapsing section is closed by default
    let (header_id, tag) = {
        let mut scope = UiScope::new(&mut tree, root);
        let res = scope.collapsing("Transform Options", |body| {
            body_executed = true;
            body.label("Inner content");
        });

        assert!(!res.is_open(), "Section must be closed by default");
        assert!(!res.clicked(), "Header must not be clicked on fresh frame");
        assert!(
            res.body().is_none(),
            "Body response must be None when closed"
        );
        assert!(!body_executed, "Body closure must NOT execute when closed");

        let header_id = res.header_response().id;
        let tag = scope.tree().get(header_id).unwrap().tag;
        (header_id, tag)
    };

    // Verify chevron arrow text is collapsed ▶
    let header_node = tree.get(header_id).expect("Header node exists");
    assert_eq!(header_node.role, WidgetRole::Button);
    let chevron_node = tree
        .get(header_node.children[0])
        .expect("Chevron child exists");
    assert_eq!(chevron_node.text.as_deref(), Some("▶"));

    // 2. Simulate click interaction on the collapsing header
    if let Some(node) = tree.get_mut(header_id) {
        node.computed_rect = Rect::new(10.0, 10.0, 200.0, 24.0);
    }
    let click_pt = Point::new(20.0, 20.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: click_pt,
    });
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: click_pt,
    });
    assert!(up_res.is_consumed());

    // 3. Render next frame: section must now be OPEN on the click frame!
    tree.clear();
    let root = tree.create_root().expect("Root node must be created");
    body_executed = false;
    let (new_header_id, body_node_id) = {
        let mut scope = UiScope::new(&mut tree, root);
        let res = scope.collapsing("Transform Options", |body| {
            body_executed = true;
            body.label("Inner content")
        });

        assert!(res.clicked(), "Header must report clicked() == true");
        assert!(res.is_open(), "Section must be OPEN on toggle");
        assert!(body_executed, "Body closure MUST execute when open");
        assert!(res.body().is_some(), "Body response must be populated");

        let h_id = res.header_response().id;
        let b_id = scope.tree().get(scope.parent).unwrap().children[1];
        (h_id, b_id)
    };

    // Verify chevron arrow text transitioned to expanded ▼
    let header_node = tree.get(new_header_id).unwrap();
    let chevron_node = tree.get(header_node.children[0]).unwrap();
    assert_eq!(chevron_node.text.as_deref(), Some("▼"));

    // Verify body container exists with indentation padding
    let body_node = tree.get(body_node_id).unwrap();
    assert_eq!(body_node.name.as_deref(), Some("CollapsingBody"));
    assert_eq!(body_node.style.padding.left, 14.0);

    // Verify persistent state in UiTree
    assert!(tree.is_collapsing_open(tag, false));

    // 4. Third frame without interaction: state MUST remain open persistently!
    tree.clear();
    tree.event_state.clear_frame_clicks();
    let root = tree.create_root().expect("Root node must be created");
    body_executed = false;
    let third_header_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let res = scope.collapsing("Transform Options", |_body| {
            body_executed = true;
        });

        assert!(!res.clicked(), "Header was not clicked on idle frame");
        assert!(res.is_open(), "Section must REMAIN open persistently");
        assert!(body_executed, "Body closure must continue executing");
        res.header_response().id
    };

    // 5. Fourth frame: Click again to collapse
    if let Some(node) = tree.get_mut(third_header_id) {
        node.computed_rect = Rect::new(10.0, 10.0, 200.0, 24.0);
    }
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: click_pt,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: click_pt,
    });

    tree.clear();
    let root = tree.create_root().expect("Root node must be created");
    body_executed = false;
    {
        let mut scope = UiScope::new(&mut tree, root);
        let res = scope.collapsing("Transform Options", |_body| {
            body_executed = true;
        });

        assert!(res.clicked(), "Header was clicked");
        assert!(
            !res.is_open(),
            "Section must be COLLAPSED after second click"
        );
        assert!(
            !body_executed,
            "Body closure must NOT execute when collapsed"
        );
    }

    assert!(!tree.is_collapsing_open(tag, false));
}

#[test]
fn test_declarative_collapsing_default_open() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must be created");

    let mut body_executed = false;

    // Default open must execute body on initial frame
    {
        let mut scope = UiScope::new(&mut tree, root);
        let res = scope.collapsing_default_open("Hierarchy View", |_body| {
            body_executed = true;
        });
        assert!(res.is_open());
        assert!(body_executed);
    }
}

#[test]
fn test_declarative_collapsing_builder_customization() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must be created");

    let custom_tag = 0xA110_CA7E_0000_0001;

    let mut scope = UiScope::new(&mut tree, root);
    let res = scope
        .collapsing_builder("Custom Section##1")
        .tag(custom_tag)
        .default_open(true)
        .indent(24.0)
        .show(|body| {
            body.label("Custom Indented Item");
            42
        });

    assert!(res.is_open());
    let header_id = res.header_response().id;
    assert_eq!(res.into_inner(), Some(42));

    // Verify custom tag was assigned to header
    let header_node = tree.get(header_id).unwrap();
    assert_eq!(header_node.tag, custom_tag);

    // Verify custom 24px indentation on body
    let root_node = tree.get(root).unwrap();
    let body_node = tree.get(root_node.children[1]).unwrap();
    assert_eq!(body_node.style.padding.left, 24.0);
}