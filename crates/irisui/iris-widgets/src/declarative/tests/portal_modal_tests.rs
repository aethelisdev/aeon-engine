// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for declarative modal dialogs ([`UiScope::modal`]) and layer portals ([`UiScope::popup_portal`]).

use crate::declarative::UiScope;
use crate::modal::{MODAL_TAG_CARD, MODAL_TAG_SCRIM};
use iris_core::geometry::{Point, Rect};
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
fn test_declarative_modal_card_click_does_not_dismiss() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");
    tree.get_mut(root).unwrap().computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let mut is_open = true;

    // Simulate clicking inside the modal dialog card bounds (center of screen, e.g. text label, padding, empty card space)
    tree.event_state.frame_clicked_pos = Some(Point::new(960.0, 540.0));
    tree.event_state.frame_clicked_tag = None;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.modal(("Dialog", 400.0, 300.0), &mut is_open, |_body| {});
    }

    assert!(
        is_open,
        "Clicking inside modal card body bounds must NOT dismiss modal; it must remain active"
    );

    // Simulate clicking outside on the backdrop scrim (e.g. at (100, 100))
    tree.event_state.frame_clicked_pos = Some(Point::new(100.0, 100.0));
    tree.event_state.frame_clicked_tag = Some(MODAL_TAG_SCRIM);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.modal(("Dialog", 400.0, 300.0), &mut is_open, |_body| {});
    }

    assert!(
        !is_open,
        "Clicking outside modal card on backdrop scrim must dismiss modal"
    );
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

#[test]
fn test_declarative_modal_tagged_lifecycle() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");

    const MODAL_TAG: u64 = 0x5001;

    // Initially closed
    {
        let mut scope = UiScope::new(&mut tree, root);
        assert!(!scope.is_modal_open(MODAL_TAG));
        assert!(!scope.is_any_modal_open());
        let resp = scope.modal_tagged(MODAL_TAG, ("About", 400.0, 300.0), |_body| {});
        assert!(resp.is_none());
    }

    // Open modal via tag
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.open_modal(MODAL_TAG);
        assert!(scope.is_modal_open(MODAL_TAG));
        assert!(scope.is_any_modal_open());
        let resp = scope.modal_tagged(MODAL_TAG, ("About", 400.0, 300.0), |body| {
            body.button("OK").id
        });
        assert!(resp.is_some());
    }

    // Modal remains open on subsequent frame
    {
        let mut scope = UiScope::new(&mut tree, root);
        assert!(scope.is_modal_open(MODAL_TAG));
        let resp = scope.modal_tagged(MODAL_TAG, ("About", 400.0, 300.0), |_body| {});
        assert!(resp.is_some());
    }

    // Close modal via tag
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.close_modal(MODAL_TAG);
        assert!(!scope.is_modal_open(MODAL_TAG));
        assert!(!scope.is_any_modal_open());
        let resp = scope.modal_tagged(MODAL_TAG, ("About", 400.0, 300.0), |_body| {});
        assert!(resp.is_none());
    }
}

#[test]
fn test_modal_card_hit_test_does_not_hit_scrim() {
    use iris_core::{Color, Rect};

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");

    let screen_w = 1920.0;
    let screen_h = 1080.0;
    let card_w = 400.0;
    let card_h = 300.0;

    tree.get_mut(root).unwrap().computed_rect = Rect::new(0.0, 0.0, screen_w, screen_h);

    let (scrim_id, card_id) = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.modal_scrim_sized(
            Rect::new(0.0, 0.0, screen_w, screen_h),
            Color::rgba(0.0, 0.0, 0.0, 0.65),
            |scrim| {
                let card_resp = scrim.modal_card(card_w, card_h, |card| {
                    card.label("Modal Body Content");
                });
                card_resp.response.id
            },
        );
        (resp.response.id, resp.inner)
    };

    let scrim_node = tree.get(scrim_id).unwrap();
    assert_eq!(scrim_node.tag, MODAL_TAG_SCRIM);

    let card_node = tree.get(card_id).unwrap();
    assert_eq!(card_node.tag, MODAL_TAG_CARD);
    assert!(card_node.interactive);

    // Hit test at center of screen (inside modal card)
    let center_point = Point::new(screen_w * 0.5, screen_h * 0.5);
    let hit_center = tree.hit_test_layered(center_point);
    assert_eq!(
        hit_center,
        Some(card_id),
        "Clicking inside modal card must hit the card, NOT the scrim"
    );

    // Hit test at top-left corner (outside modal card, on backdrop scrim)
    let corner_point = Point::new(10.0, 10.0);
    let hit_corner = tree.hit_test_layered(corner_point);
    assert_eq!(
        hit_corner,
        Some(scrim_id),
        "Clicking outside modal card must hit the backdrop scrim"
    );
}