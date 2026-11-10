// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for mouse click dispatching, hit-testing, cross-frame tag persistence, and child event bubbling.

use crate::event::{MouseButton, UiEvent};
use crate::geometry::{Point, Rect};
use crate::node::WidgetRole;
use crate::tree::UiTree;

#[test]
fn test_ui_tree_dispatch_event_hit_test_and_capture() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let child = tree.create_node();
    tree.add_child(root, child).unwrap();

    let child_node = tree.get_mut(child).unwrap();
    child_node.visible = true;
    child_node.interactive = true;
    child_node.computed_rect = Rect::new(100.0, 100.0, 50.0, 50.0);
    child_node.tag = 42;

    // 1. MouseMove over child -> Hover detected
    let move_evt = UiEvent::MouseMove {
        point: Point::new(120.0, 120.0),
    };
    let res = tree.dispatch_event(&move_evt);
    assert!(!res.is_consumed());
    assert_eq!(res.target, Some(child));
    assert_eq!(res.target_tag, 42);
    assert!(res.response.map(|r| r.hovered()).unwrap_or(false));
    assert_eq!(tree.event_state.hovered, Some(child));

    // 2. MouseDown on child -> Press & Focus captured
    let down_evt = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(120.0, 120.0),
    };
    let res_down = tree.dispatch_event(&down_evt);
    assert!(res_down.is_consumed());
    assert_eq!(res_down.target, Some(child));
    assert!(res_down.response.map(|r| r.pressed()).unwrap_or(false));
    assert_eq!(tree.event_state.focused, Some(child));
    assert_eq!(tree.event_state.focused_tag, Some(42));

    // 3. MouseUp on child -> Click detected!
    let up_evt = UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(120.0, 120.0),
    };
    let res_up = tree.dispatch_event(&up_evt);
    assert!(res_up.is_consumed());
    assert_eq!(res_up.target, Some(child));
    assert!(res_up.response.map(|r| r.clicked()).unwrap_or(false));

    // 4. Pointer capture test: capture moves away from child
    tree.event_state.pointer_capture = Some(child);
    let capture_move = UiEvent::MouseMove {
        point: Point::new(500.0, 500.0),
    };
    let res_capture = tree.dispatch_event(&capture_move);
    assert!(res_capture.is_consumed());
    assert_eq!(res_capture.target, Some(child));
    assert!(res_capture.response.map(|r| r.dragged()).unwrap_or(false));

    // Release capture
    let capture_up = UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(500.0, 500.0),
    };
    let res_cap_up = tree.dispatch_event(&capture_up);
    assert!(res_cap_up.is_consumed());
    assert_eq!(tree.event_state.pointer_capture, None);
}

#[test]
fn test_child_event_consumption_and_parent_isolation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    // 1. Container serving as scroll viewport
    let container = tree.create_node();
    tree.add_child(root, container).unwrap();
    let container_node = tree.get_mut(container).unwrap();
    container_node.role = WidgetRole::ScrollAreaViewport;
    container_node.tag = 1000;
    container_node.interactive = true;
    container_node.computed_rect = Rect::new(0.0, 0.0, 500.0, 500.0);

    // 2. Child card item inside container
    let card = tree.create_node();
    tree.add_child(container, card).unwrap();
    let card_node = tree.get_mut(card).unwrap();
    card_node.role = WidgetRole::Button;
    card_node.tag = 2000;
    card_node.interactive = true;
    card_node.computed_rect = Rect::new(10.0, 10.0, 100.0, 100.0);

    // 3. Grandchild label inside card
    let label = tree.create_node();
    tree.add_child(card, label).unwrap();
    let label_node = tree.get_mut(label).unwrap();
    label_node.role = WidgetRole::Default;
    label_node.tag = 0;
    label_node.interactive = false;
    label_node.computed_rect = Rect::new(20.0, 20.0, 50.0, 20.0);

    // Click on grandchild label
    let click_point = Point::new(30.0, 25.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Right,
        point: click_point,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Right,
        point: click_point,
    });

    let hit = tree.event_state.frame_secondary_clicked_id.unwrap();
    assert_eq!(hit, card);

    // Verification: Card owns the click
    assert!(tree.is_direct_interaction_ancestor(hit, card));

    // Verification: Container does NOT claim the click because card consumed it
    assert!(!tree.is_direct_interaction_ancestor(hit, container));

    // Direct click on container empty area (e.g. x = 300, y = 300)
    let empty_point = Point::new(300.0, 300.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Right,
        point: empty_point,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Right,
        point: empty_point,
    });

    let empty_hit = tree.event_state.frame_secondary_clicked_id.unwrap();
    assert_eq!(empty_hit, container);
    assert!(tree.is_direct_interaction_ancestor(empty_hit, container));
}

#[test]
fn test_dispatch_frame_clicked_pos_recording() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let child = tree.create_node();
    tree.add_child(root, child).unwrap();

    let child_node = tree.get_mut(child).unwrap();
    child_node.visible = true;
    child_node.interactive = true;
    child_node.computed_rect = Rect::new(50.0, 50.0, 100.0, 100.0);

    let click_point = Point::new(75.0, 80.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Right,
        point: click_point,
    });
    // MouseDown clears transient clicked pos
    assert_eq!(tree.event_state.frame_clicked_pos, None);

    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Right,
        point: click_point,
    });

    // MouseUp saves clicked pos
    assert_eq!(tree.event_state.frame_clicked_pos, Some(click_point));

    // Clear frame clicks resets it
    tree.event_state.clear_frame_clicks();
    assert_eq!(tree.event_state.frame_clicked_pos, None);
}

#[test]
fn test_dispatch_cross_frame_tag_click_with_tree_clear() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let child = tree.create_node();
    tree.add_child(root, child).unwrap();

    let child_node = tree.get_mut(child).unwrap();
    child_node.visible = true;
    child_node.interactive = true;
    child_node.tag = 0xABCD;
    child_node.computed_rect = Rect::new(10.0, 10.0, 50.0, 50.0);

    let click_point = Point::new(25.0, 25.0);
    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: click_point,
    });
    assert!(down_res.is_consumed());
    assert_eq!(tree.event_state.pressed_tag, Some(0xABCD));

    // Simulate render frame: tree is cleared and reconstructed with new generational WidgetIds!
    tree.clear();
    let new_root = tree.create_root().unwrap();
    let new_child = tree.create_node();
    assert_ne!(child, new_child); // Generational key difference verified
    tree.add_child(new_root, new_child).unwrap();

    let new_child_node = tree.get_mut(new_child).unwrap();
    new_child_node.visible = true;
    new_child_node.interactive = true;
    new_child_node.tag = 0xABCD;
    new_child_node.computed_rect = Rect::new(10.0, 10.0, 50.0, 50.0);

    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: click_point,
    });
    assert!(up_res.is_consumed());
    assert_eq!(tree.event_state.frame_clicked_tag, Some(0xABCD));
    assert_eq!(tree.event_state.frame_clicked_id, Some(new_child));
}

#[test]
fn test_dispatch_cross_frame_secondary_click_with_tree_clear() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let child = tree.create_node();
    tree.add_child(root, child).unwrap();

    let child_node = tree.get_mut(child).unwrap();
    child_node.visible = true;
    child_node.interactive = true;
    child_node.tag = 0x9999;
    child_node.computed_rect = Rect::new(10.0, 10.0, 50.0, 50.0);

    let click_point = Point::new(25.0, 25.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Right,
        point: click_point,
    });
    assert_eq!(tree.event_state.pressed_tag, Some(0x9999));

    // Rebuild tree
    tree.clear();
    let new_root = tree.create_root().unwrap();
    let new_child = tree.create_node();
    tree.add_child(new_root, new_child).unwrap();

    let new_child_node = tree.get_mut(new_child).unwrap();
    new_child_node.visible = true;
    new_child_node.interactive = true;
    new_child_node.tag = 0x9999;
    new_child_node.computed_rect = Rect::new(10.0, 10.0, 50.0, 50.0);

    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Right,
        point: click_point,
    });
    assert!(up_res.is_consumed());
    assert_eq!(tree.event_state.frame_secondary_clicked_tag, Some(0x9999));
    assert_eq!(tree.event_state.frame_secondary_clicked_id, Some(new_child));
}

#[test]
fn test_dispatch_cross_frame_bounds_click_untagged_widget() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let child = tree.create_node();
    tree.add_child(root, child).unwrap();

    let child_node = tree.get_mut(child).unwrap();
    child_node.visible = true;
    child_node.interactive = true;
    child_node.role = WidgetRole::Button;
    child_node.computed_rect = Rect::new(40.0, 40.0, 80.0, 30.0);

    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(50.0, 50.0),
    });

    // Rebuild tree
    tree.clear();
    let new_root = tree.create_root().unwrap();
    let new_child = tree.create_node();
    tree.add_child(new_root, new_child).unwrap();

    let new_child_node = tree.get_mut(new_child).unwrap();
    new_child_node.visible = true;
    new_child_node.interactive = true;
    new_child_node.role = WidgetRole::Button;
    new_child_node.computed_rect = Rect::new(40.0, 40.0, 80.0, 30.0);

    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(60.0, 55.0),
    });
    assert!(up_res.is_consumed());
    assert_eq!(tree.event_state.frame_clicked_id, Some(new_child));
}

/// Verifies that mouse movement occurring right after a click does not prematurely clear transient click state.
#[test]
fn test_transient_clicks_preserved_across_mouse_move() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let button = tree.create_node();
    tree.add_child(root, button).unwrap();
    {
        let btn = tree.get_mut(button).unwrap();
        btn.visible = true;
        btn.interactive = true;
        btn.role = WidgetRole::Button;
        btn.computed_rect = Rect::new(10.0, 10.0, 80.0, 30.0);
        btn.tag = 0x3333;
    }

    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(20.0, 20.0),
    });

    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(20.0, 20.0),
    });
    assert!(up_res.is_consumed());
    assert_eq!(tree.event_state.frame_clicked_tag, Some(0x3333));

    // Subsequent micro mouse movement before next render frame must NOT clear click state
    let _ = tree.dispatch_event(&UiEvent::MouseMove {
        point: Point::new(21.0, 20.0),
    });
    assert_eq!(
        tree.event_state.frame_clicked_tag,
        Some(0x3333),
        "Click state must be preserved across mouse movement"
    );
}

/// Verifies that clicking on an untagged, passive child container inside a card button
/// correctly delegates the click to the enclosing card button.
#[test]
fn test_click_on_child_container_bubbles_to_parent_card_button() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let card = tree.create_node();
    tree.add_child(root, card).unwrap();
    {
        let c = tree.get_mut(card).unwrap();
        c.visible = true;
        c.interactive = true;
        c.role = WidgetRole::Button;
        c.computed_rect = Rect::new(50.0, 50.0, 120.0, 120.0);
        c.tag = 0x5555;
    }

    // Passive untagged child container (like CardPreviewBox)
    let child_box = tree.create_node();
    tree.add_child(card, child_box).unwrap();
    {
        let b = tree.get_mut(child_box).unwrap();
        b.visible = true;
        b.interactive = true;
        b.role = WidgetRole::Default;
        b.computed_rect = Rect::new(60.0, 60.0, 54.0, 54.0);
        b.tag = 0;
    }

    assert!(
        tree.is_direct_interaction_ancestor(child_box, card),
        "Card must be direct interaction ancestor of untagged child box"
    );

    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(70.0, 70.0),
    });

    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(70.0, 70.0),
    });
    assert!(up_res.is_consumed());
    assert_eq!(tree.event_state.frame_clicked_tag, Some(0x5555));
}

#[test]
fn test_click_on_child_container_bubbles_target_to_ancestor_button() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let button = tree.create_node();
    tree.add_child(root, button).unwrap();
    {
        let b = tree.get_mut(button).unwrap();
        b.visible = true;
        b.interactive = true;
        b.role = WidgetRole::Button;
        b.computed_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        b.tag = 0x9999;
    }

    let child_box = tree.create_node();
    tree.add_child(button, child_box).unwrap();
    {
        let c = tree.get_mut(child_box).unwrap();
        c.visible = true;
        c.interactive = true;
        c.role = WidgetRole::Default;
        c.computed_rect = Rect::new(10.0, 10.0, 80.0, 80.0);
        c.tag = 0;
    }

    let pt = Point::new(20.0, 20.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: pt,
    });
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: pt,
    });

    assert_eq!(
        up_res.target,
        Some(button),
        "DispatchResult target must bubble to the ancestor Button widget ID"
    );
    assert_eq!(
        up_res.response.map(|r| r.id),
        Some(button),
        "WidgetResponse id must bubble to the ancestor Button widget ID"
    );
    assert_eq!(tree.event_state.frame_clicked_tag, Some(0x9999));
}