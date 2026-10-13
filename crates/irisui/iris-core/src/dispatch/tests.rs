// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for UI Event Dispatch and Autonomous Scroll Interactivity (`iris-core::dispatch::tests`)

use crate::event::{ImeEvent, KeyCode, MouseButton, UiEvent};
use crate::geometry::{Point, Rect, Size};
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
fn test_ui_tree_dispatch_event_scroll_bubbling_past_interactive_button() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let scroll_container = tree.create_node();
    tree.add_child(root, scroll_container).unwrap();
    {
        let sc = tree.get_mut(scroll_container).unwrap();
        sc.visible = true;
        sc.style.clip_children = true;
        sc.style.scroll_offset_y = 10.0;
        sc.computed_rect = Rect::new(0.0, 0.0, 300.0, 400.0);
        sc.tag = 100;
    }

    let button = tree.create_node();
    tree.add_child(scroll_container, button).unwrap();
    {
        let btn = tree.get_mut(button).unwrap();
        btn.visible = true;
        btn.interactive = true;
        btn.computed_rect = Rect::new(20.0, 20.0, 100.0, 30.0);
        btn.tag = 200;
    }

    let scroll_evt = UiEvent::MouseScroll {
        delta_x: 0.0,
        delta_y: 15.0,
        point: Point::new(30.0, 25.0),
    };

    let res = tree.dispatch_event(&scroll_evt);
    assert!(res.is_consumed(), "Scroll event must be consumed");
    assert_eq!(res.target, Some(scroll_container));
    assert_eq!(res.target_tag, 100);
    assert_eq!(res.scroll_delta, (0.0, 15.0));

    let sc = tree.get(scroll_container).unwrap();
    assert_eq!(sc.style.scroll_offset_y, 0.0);
}

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
fn test_scroll_area_autonomous_dispatch_integration() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    // 1. Build a complete ScrollArea structure: Viewport, Track, Thumb
    let viewport = tree.create_node();
    tree.add_child(root, viewport).unwrap();
    {
        let vp = tree.get_mut(viewport).unwrap();
        vp.visible = true;
        vp.interactive = true;
        vp.role = WidgetRole::ScrollAreaViewport;
        vp.style.clip_children = true;
        vp.style.scroll_offset_y = 0.0;
        vp.computed_rect = Rect::new(0.0, 0.0, 200.0, 200.0); // Viewport H = 200
        vp.content_size = Size::new(200.0, 600.0); // Content H = 600 -> max_scroll = 400
        vp.tag = 1000;
    }

    let track = tree.create_node();
    tree.add_child(viewport, track).unwrap();
    {
        let tr = tree.get_mut(track).unwrap();
        tr.visible = true;
        tr.interactive = true;
        tr.role = WidgetRole::ScrollBarTrack;
        tr.computed_rect = Rect::new(190.0, 0.0, 10.0, 200.0); // Track H = 200
    }

    let thumb = tree.create_node();
    tree.add_child(viewport, thumb).unwrap();
    {
        let th = tree.get_mut(thumb).unwrap();
        th.visible = true;
        th.interactive = true;
        th.role = WidgetRole::ScrollBarThumb;
        th.computed_rect = Rect::new(190.0, 0.0, 10.0, 50.0); // Thumb H = 50 -> travel = 150
    }

    // A. Wheel Scroll on viewport: downwards delta_y = -100.0
    let wheel_evt = UiEvent::MouseScroll {
        delta_x: 0.0,
        delta_y: -100.0,
        point: Point::new(50.0, 50.0),
    };
    let wheel_res = tree.dispatch_event(&wheel_evt);
    assert!(wheel_res.is_consumed());
    assert_eq!(wheel_res.target, Some(viewport));

    // Viewport scroll_offset_y must be 100.0
    let vp = tree.get(viewport).unwrap();
    assert_eq!(vp.style.scroll_offset_y, 100.0);

    // Thumb computed_rect.y must be synchronized autonomously: (100 / 400) * 150 = 37.5
    let th = tree.get(thumb).unwrap();
    assert_eq!(th.computed_rect.y, 37.5);

    // B. Autonomous Thumb Drag: MouseDown on thumb -> MouseMove -> MouseUp
    let thumb_down = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(195.0, 37.5),
    };
    let down_res = tree.dispatch_event(&thumb_down);
    assert!(down_res.is_consumed());
    assert_eq!(tree.event_state.pointer_capture, Some(thumb));

    // Drag thumb down by +75px (travel halfway: 37.5 + 75 = 112.5 -> scroll = 300)
    let drag_move = UiEvent::MouseMove {
        point: Point::new(195.0, 112.5),
    };
    let move_res = tree.dispatch_event(&drag_move);
    assert!(move_res.is_consumed());
    assert_eq!(move_res.target, Some(thumb));

    let vp_dragged = tree.get(viewport).unwrap();
    assert_eq!(vp_dragged.style.scroll_offset_y, 300.0);

    let th_dragged = tree.get(thumb).unwrap();
    assert_eq!(th_dragged.computed_rect.y, 112.5);

    // MouseUp releases pointer capture
    let thumb_up = UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(195.0, 112.5),
    };
    let up_res = tree.dispatch_event(&thumb_up);
    assert!(up_res.is_consumed());
    assert_eq!(tree.event_state.pointer_capture, None);

    // C. Autonomous Track Click-to-Jump: Click at bottom of track (y = 175)
    let track_jump = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(195.0, 175.0),
    };
    let jump_res = tree.dispatch_event(&track_jump);
    assert!(jump_res.is_consumed());
    // click_offset_y = 175 - 0 - 25 = 150 -> ratio = 1.0 -> max_scroll = 400
    let vp_jumped = tree.get(viewport).unwrap();
    assert_eq!(vp_jumped.style.scroll_offset_y, 400.0);

    let th_jumped = tree.get(thumb).unwrap();
    assert_eq!(th_jumped.computed_rect.y, 150.0);
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