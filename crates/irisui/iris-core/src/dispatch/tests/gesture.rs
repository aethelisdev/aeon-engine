// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for gesture recognition: double click, right/middle click isolation, and drag capture.

use crate::event::{MouseButton, UiEvent};
use crate::geometry::{Point, Rect};
use crate::node::WidgetRole;
use crate::tree::UiTree;

#[test]
fn test_ui_tree_dispatch_event_double_click() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let child = tree.create_node();
    tree.add_child(root, child).unwrap();

    let child_node = tree.get_mut(child).unwrap();
    child_node.visible = true;
    child_node.interactive = true;
    child_node.computed_rect = Rect::new(50.0, 50.0, 100.0, 40.0);

    let pt = Point::new(60.0, 60.0);

    // 1st click: down + up
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: pt,
    });
    let res1 = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: pt,
    });
    assert!(res1.response.map(|r| r.clicked()).unwrap_or(false));
    assert!(!res1.response.map(|r| r.double_clicked()).unwrap_or(false));

    // 2nd quick click: down + up
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: pt,
    });
    let res2 = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: pt,
    });
    assert!(res2.response.map(|r| r.clicked()).unwrap_or(false));
    assert!(
        res2.response.map(|r| r.double_clicked()).unwrap_or(false),
        "Second consecutive click must emit DOUBLE_CLICKED"
    );
}

#[test]
fn test_ui_tree_dispatch_event_right_and_middle_click_isolation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let button = tree.create_node();
    tree.add_child(root, button).unwrap();

    {
        let node = tree.get_mut(button).unwrap();
        node.visible = true;
        node.interactive = true;
        node.computed_rect = Rect::new(50.0, 50.0, 100.0, 40.0);
        node.tag = 777;
    }

    let pt = Point::new(60.0, 60.0);

    let down_right = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Right,
        point: pt,
    });
    assert!(down_right.response.map(|r| r.pressed()).unwrap_or(false));
    assert!(
        !down_right
            .response
            .map(|r| r.secondary_clicked())
            .unwrap_or(false)
    );

    let up_right = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Right,
        point: pt,
    });
    assert!(
        up_right
            .response
            .map(|r| r.secondary_clicked())
            .unwrap_or(false)
    );
    assert!(
        !up_right.response.map(|r| r.clicked()).unwrap_or(false),
        "Right click MUST NOT emit primary CLICKED flag"
    );

    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Middle,
        point: pt,
    });
    let up_middle = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Middle,
        point: pt,
    });
    assert!(
        up_middle
            .response
            .map(|r| r.middle_clicked())
            .unwrap_or(false)
    );
    assert!(
        !up_middle.response.map(|r| r.clicked()).unwrap_or(false),
        "Middle click MUST NOT emit primary CLICKED flag"
    );
}

#[test]
fn test_autonomous_drag_capture_and_release() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let card = tree.create_node();
    tree.add_child(root, card).unwrap();
    {
        let c = tree.get_mut(card).unwrap();
        c.visible = true;
        c.interactive = true;
        c.role = WidgetRole::Button;
        c.computed_rect = Rect::new(10.0, 10.0, 100.0, 100.0);
        c.tag = 0xAA01;
    }

    // 1. MouseDown on card
    let down = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(20.0, 20.0),
    };
    let res_down = tree.dispatch_event(&down);
    assert!(res_down.is_consumed());
    assert_eq!(tree.event_state.pressed, Some(card));
    assert_eq!(tree.event_state.pointer_capture, None); // Capture not established before threshold

    // 2. MouseMove past drag threshold (> 3.0 px) -> Autonomous drag capture establishes
    let motion = UiEvent::MouseMove {
        point: Point::new(25.0, 25.0),
    };
    let res_motion = tree.dispatch_event(&motion);
    assert!(res_motion.is_consumed());
    assert_eq!(tree.event_state.pointer_capture, Some(card));
    assert_eq!(tree.event_state.pointer_capture_tag, Some(0xAA01));
    assert!(res_motion.response.map(|r| r.dragged()).unwrap_or(false));

    // 3. Move far away outside card bounds (e.g. over 3D viewport canvas)
    let motion_far = UiEvent::MouseMove {
        point: Point::new(400.0, 300.0),
    };
    let res_far = tree.dispatch_event(&motion_far);
    assert!(res_far.is_consumed());
    assert_eq!(res_far.target, Some(card)); // Still captures target!
    assert!(res_far.response.map(|r| r.dragged()).unwrap_or(false));

    // 4. MouseUp outside card bounds -> Drag gesture release (not a normal click)
    let up = UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(400.0, 300.0),
    };
    let res_up = tree.dispatch_event(&up);
    assert!(res_up.is_consumed());
    let resp = res_up.response.unwrap();
    assert!(resp.drag_released());
    assert!(!resp.clicked()); // Drag release must NOT be considered a click
    assert_eq!(tree.event_state.frame_drag_released_id, Some(card));
    assert_eq!(tree.event_state.frame_drag_released_tag, Some(0xAA01));
    assert_eq!(tree.event_state.pointer_capture, None);
}

#[test]
fn test_double_click_preserved_across_tree_rebuild_via_tag_and_bounds() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let card1 = tree.create_node();
    tree.add_child(root, card1).unwrap();
    {
        let c = tree.get_mut(card1).unwrap();
        c.visible = true;
        c.interactive = true;
        c.role = WidgetRole::Button;
        c.computed_rect = Rect::new(10.0, 10.0, 100.0, 100.0);
        c.tag = 0xABCD;
    }

    // Click 1 on card1
    let pt = Point::new(30.0, 30.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: pt,
    });
    let up1 = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: pt,
    });
    assert!(up1.is_consumed());
    assert_eq!(tree.event_state.frame_clicked_tag, Some(0xABCD));
    assert_eq!(tree.event_state.last_click_tag, Some(0xABCD));

    // Simulate subtree rebuild (clear children, untrack widgets, rebuild card2 with new WidgetId but same tag and rect)
    let _ = tree.clear_children(root);
    assert_eq!(tree.event_state.last_click_tag, Some(0xABCD));

    let card2 = tree.create_node();
    tree.add_child(root, card2).unwrap();
    {
        let c = tree.get_mut(card2).unwrap();
        c.visible = true;
        c.interactive = true;
        c.role = WidgetRole::Button;
        c.computed_rect = Rect::new(10.0, 10.0, 100.0, 100.0);
        c.tag = 0xABCD;
    }

    // Click 2 within double-click delay threshold
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: pt,
    });
    let up2 = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: pt,
    });
    assert!(up2.is_consumed());
    assert!(
        up2.response.unwrap().double_clicked(),
        "Double click must be detected across tree rebuilds via persistent tag and bounds"
    );
    assert_eq!(tree.event_state.frame_double_clicked_tag, Some(0xABCD));
    assert_eq!(up2.target, Some(card2));
}