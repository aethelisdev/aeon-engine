// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for declarative modal dialogs ([`UiScope::modal`]) and layer portals ([`UiScope::popup_portal`]).

use crate::declarative::UiScope;
use crate::modal::MODAL_TAG_SCRIM;
use iris_core::geometry::Point;
use iris_core::{KeyCode, UiLayer, UiTree};

#[test]
fn test_declarative_modal_lifecycle_open_and_close() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");

    let mut is_open = false;

    // 1. When closed, modal returns None and does not emit modal nodes
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.modal(("Preferences", 500.0, 400.0), &mut is_open, |_body| 42);
        assert!(resp.is_none());
    }

    assert_eq!(tree.get(root).unwrap().children.len(), 0);

    // 2. When open, modal renders scrim on UiLayer::Modal with centered card
    is_open = true;
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.modal(("Preferences", 500.0, 400.0), &mut is_open, |body| {
            body.text("Settings Body Content");
            100
        });
        assert!(resp.is_some());
        assert_eq!(resp.unwrap().inner, 100);
    }

    let root_node = tree.get(root).unwrap();
    assert_eq!(root_node.children.len(), 1);
    let scrim_id = root_node.children[0];
    let scrim_node = tree.get(scrim_id).unwrap();
    assert_eq!(scrim_node.tag, MODAL_TAG_SCRIM);
    assert_eq!(scrim_node.layer, UiLayer::Modal);
}

#[test]
fn test_declarative_modal_escape_dismissal() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");

    let mut is_open = true;

    // Simulate Escape key press in event state
    tree.event_state.frame_key = Some(KeyCode::Escape);
    tree.event_state.frame_key_pressed = true;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.modal(("Alert", 300.0, 200.0), &mut is_open, |_body| {});
        assert!(resp.is_none(), "Escape must immediately dismiss modal");
    }

    assert!(!is_open, "is_open flag must be cleared to false on Escape");
}

#[test]
fn test_declarative_modal_scrim_click_dismissal() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");

    let mut is_open = true;

    // Initial render frame
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.modal(("Dialog", 400.0, 300.0), &mut is_open, |_body| {});
    }
    assert!(is_open);

    // Simulate clicking on the modal backdrop scrim
    tree.event_state.frame_clicked_tag = Some(MODAL_TAG_SCRIM);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.modal(("Dialog", 400.0, 300.0), &mut is_open, |_body| {});
    }

    assert!(!is_open, "Clicking scrim must dismiss modal");
}

#[test]
fn test_declarative_popup_portal_elevation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");

    let panel = tree.create_node();
    tree.add_child(root, panel).unwrap();

    let mut scope = UiScope::new(&mut tree, panel);

    // Even though scope is targeting `panel`, popup_portal elevates child to root's UiLayer::Popup portal
    let portal_resp = scope.popup_portal(|portal| {
        let child_node = portal.button("Elevated Popup Item");
        child_node.id
    });

    let portal_id = portal_resp.response.id;
    let portal_node = scope.tree().get(portal_id).unwrap();
    assert_eq!(portal_node.layer, UiLayer::Popup);

    let item_id = portal_resp.inner;
    let item_node = scope.tree().get(item_id).unwrap();
    assert_eq!(item_node.layer, UiLayer::Popup);
}

#[test]
fn test_declarative_popup_portal_card_positioning() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");

    let mut scope = UiScope::new(&mut tree, root);
    let card_pos = Point::new(150.0, 250.0);

    let resp = scope.popup_portal_card(card_pos, 220.0, |card| {
        let btn = card.button("Action 1");
        btn.id
    });

    let card_id = resp.response.id;
    let card_node = scope.tree().get(card_id).unwrap();
    assert_eq!(card_node.layer, UiLayer::Popup);
}