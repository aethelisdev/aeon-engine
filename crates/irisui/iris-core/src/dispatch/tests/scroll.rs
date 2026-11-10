// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for scroll wheel dispatching, thumb dragging, track jump, and scroll bubbling.

use crate::event::UiEvent;
use crate::geometry::{Point, Rect, Size};
use crate::node::WidgetRole;
use crate::tree::UiTree;

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
fn test_scroll_area_autonomous_dispatch_integration() {
    use crate::event::MouseButton;

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

/// Verifies that scrolling over an interactive clipped child card (`WidgetRole::Button` with `clip_children: true`)
/// does not trap the scroll event, correctly bubbling to the enclosing `WidgetRole::ScrollAreaViewport`.
#[test]
fn test_scroll_bubbling_past_clipped_button_card_to_viewport() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let viewport = tree.create_node();
    tree.add_child(root, viewport).unwrap();
    {
        let vp = tree.get_mut(viewport).unwrap();
        vp.visible = true;
        vp.role = WidgetRole::ScrollAreaViewport;
        vp.style.clip_children = true;
        vp.style.scroll_offset_y = 50.0;
        vp.computed_rect = Rect::new(0.0, 0.0, 500.0, 500.0);
        vp.tag = 0x1111;
    }

    let card = tree.create_node();
    tree.add_child(viewport, card).unwrap();
    {
        let c = tree.get_mut(card).unwrap();
        c.visible = true;
        c.interactive = true;
        c.role = WidgetRole::Button;
        c.style.clip_children = true; // Simulates rounded border clipping on asset cards
        c.computed_rect = Rect::new(20.0, 20.0, 100.0, 120.0);
        c.tag = 0x2222;
    }

    // Scroll over the card
    let res = tree.dispatch_event(&UiEvent::MouseScroll {
        delta_x: 0.0,
        delta_y: 20.0,
        point: Point::new(40.0, 40.0),
    });

    assert!(
        res.is_consumed(),
        "Scroll event must be consumed by parent viewport"
    );
    assert_eq!(res.target, Some(viewport));
    assert_eq!(res.target_tag, 0x1111);
    assert_eq!(res.scroll_delta, (0.0, 20.0));

    let vp = tree.get(viewport).unwrap();
    assert_eq!(vp.style.scroll_offset_y, 30.0);

    let c = tree.get(card).unwrap();
    assert_eq!(c.style.scroll_offset_y, 0.0);
}

/// Verifies that mouse wheel scrolling directly over nested card elements (e.g. preview box, icons)
/// reliably bubbles up and scrolls the enclosing viewport container.
#[test]
fn test_scroll_over_nested_card_elements_bubbles_to_viewport() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let viewport = tree.create_node();
    tree.add_child(root, viewport).unwrap();
    {
        let vp = tree.get_mut(viewport).unwrap();
        vp.visible = true;
        vp.role = WidgetRole::ScrollAreaViewport;
        vp.style.clip_children = true;
        vp.style.scroll_offset_y = 0.0;
        vp.computed_rect = Rect::new(0.0, 0.0, 400.0, 400.0);
        vp.content_size = Size::new(400.0, 1200.0);
        vp.tag = 0x9000;
    }

    let card = tree.create_node();
    tree.add_child(viewport, card).unwrap();
    {
        let c = tree.get_mut(card).unwrap();
        c.visible = true;
        c.interactive = true;
        c.role = WidgetRole::Button;
        c.style.clip_children = true;
        c.computed_rect = Rect::new(10.0, 10.0, 120.0, 120.0);
        c.tag = 0x9001;
    }

    let preview_box = tree.create_node();
    tree.add_child(card, preview_box).unwrap();
    {
        let pb = tree.get_mut(preview_box).unwrap();
        pb.visible = true;
        pb.interactive = true;
        pb.role = WidgetRole::Default;
        pb.computed_rect = Rect::new(20.0, 20.0, 54.0, 54.0);
        pb.tag = 0;
    }

    // Scroll directly over preview_box inside the card
    let scroll_res = tree.dispatch_event(&UiEvent::MouseScroll {
        delta_x: 0.0,
        delta_y: -40.0,
        point: Point::new(30.0, 30.0),
    });
    assert!(scroll_res.is_consumed());
    assert_eq!(scroll_res.target, Some(viewport));
    assert_eq!(scroll_res.target_tag, 0x9000);

    let vp = tree.get(viewport).unwrap();
    assert_eq!(vp.style.scroll_offset_y, 40.0);
}