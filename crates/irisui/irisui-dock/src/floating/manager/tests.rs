// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use crate::tab_viewer::SimpleTabViewer;

#[test]
fn test_floating_window_clamping_on_drag_outside_screen() {
    let clamp_bounds = FloatingWindowClampBounds::new(1920.0, 1080.0)
        .with_min_y(32.0)
        .with_bottom_inset(24.0)
        .with_min_size(240.0, 160.0)
        .with_min_visible_margin(60.0);

    let mut manager = FloatingWindowManager::new(clamp_bounds);
    let win_id = manager.open_window(
        "Test Window",
        Rect::new(100.0, 100.0, 300.0, 200.0),
        vec!["Tab1"],
    );

    // 1. Drag far beyond top-left corner (-1000, -1000)
    manager.start_title_drag(win_id, Point::new(20.0, 10.0));
    assert!(manager.is_dragging());
    let changed = manager.update_drag(Point::new(-1000.0, -1000.0));
    assert!(changed);

    let win = manager.get(win_id).expect("window exists");
    // Y coordinate cannot breach min_y (32.0)
    assert_eq!(win.rect.y, 32.0);
    // X coordinate clamps so at least min_visible_margin (60px) stays on screen: min_x = 60 - 300 = -240.0
    assert_eq!(win.rect.x, -240.0);

    // 2. Drag far beyond bottom-right corner (5000, 5000)
    let changed = manager.update_drag(Point::new(5000.0, 5000.0));
    assert!(changed);

    let win = manager.get(win_id).expect("window exists");
    // X coordinate clamps so at least 60px stays on screen: max_x = 1920 - 60 = 1860.0
    assert_eq!(win.rect.x, 1860.0);
    // Y coordinate clamps so at least MIN_VISIBLE_TITLE_BAR (32.0px) stays visible above bottom inset (24.0px):
    // max_y = 1080 - 24 - 32 = 1024.0
    assert_eq!(win.rect.y, 1024.0);

    // 3. Release drag
    manager.end_drag();
    assert!(!manager.is_dragging());
}

#[test]
fn test_floating_window_z_order_elevation_on_click() {
    let clamp_bounds = FloatingWindowClampBounds::new(1920.0, 1080.0);
    let mut manager = FloatingWindowManager::new(clamp_bounds);

    // Add Window 1 at (100, 100)
    let id1 = manager.open_window("Win 1", Rect::new(100.0, 100.0, 300.0, 200.0), vec!["Tab1"]);
    // Add Window 2 overlapping Window 1 at (150, 150)
    let id2 = manager.open_window("Win 2", Rect::new(150.0, 150.0, 300.0, 200.0), vec!["Tab2"]);

    // Initially Window 2 was added last, so it's at the end (topmost) and focused
    assert_eq!(manager.windows().len(), 2);
    assert_eq!(manager.windows()[0].id, id1);
    assert_eq!(manager.windows()[1].id, id2);
    assert!(!manager.get(id1).expect("w1").is_focused);
    assert!(manager.get(id2).expect("w2").is_focused);

    // Click on Window 1's non-overlapping title area at (120, 110)
    let viewer = SimpleTabViewer;
    let click_event = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(120.0, 110.0),
    };

    let response = manager.handle_event(&viewer, &click_event);
    assert!(response.layout_changed);

    // Window 1 must now be at the end of the collection (highest Z-order layer) and focused!
    assert_eq!(manager.windows()[0].id, id2);
    assert_eq!(manager.windows()[1].id, id1);
    assert!(manager.get(id1).expect("w1").is_focused);
    assert!(!manager.get(id2).expect("w2").is_focused);

    // Click inside Window 2's visible non-overlapping content body at (420, 320)
    let body_click = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(420.0, 320.0),
    };
    let body_response = manager.handle_event(&viewer, &body_click);
    assert!(body_response.layout_changed);
    // consumed: false so child widgets can receive the click
    assert!(!body_response.consumed);

    // Window 2 must now be back at the end (topmost) and focused!
    assert_eq!(manager.windows()[0].id, id1);
    assert_eq!(manager.windows()[1].id, id2);
    assert!(manager.get(id2).expect("w2").is_focused);
    assert!(!manager.get(id1).expect("w1").is_focused);
}

#[test]
fn test_floating_window_close_restores_focus_to_next_topmost() {
    let mut manager = FloatingWindowManager::default();
    let id1 = manager.open_window("Win 1", Rect::new(50.0, 50.0, 200.0, 150.0), vec!["Tab1"]);
    let id2 = manager.open_window("Win 2", Rect::new(100.0, 100.0, 200.0, 150.0), vec!["Tab2"]);

    assert!(manager.get(id2).expect("w2").is_focused);

    // Close topmost window
    let closed = manager.close_window(id2);
    assert!(closed.is_some());
    assert_eq!(manager.len(), 1);

    // Remaining window must now be focused
    assert!(manager.get(id1).expect("w1").is_focused);
}

#[test]
fn test_floating_window_resize_drag_and_escape_cancel() {
    let clamp_bounds = FloatingWindowClampBounds::new(1920.0, 1080.0).with_min_size(200.0, 150.0);
    let mut manager = FloatingWindowManager::new(clamp_bounds);
    let win_id = manager.open_window(
        "Resizable",
        Rect::new(100.0, 100.0, 300.0, 200.0),
        vec!["Tab"],
    );

    // Start right-edge resize
    manager.start_resize(win_id, FloatingResizeEdge::Right);
    assert!(manager.is_dragging());

    // Drag cursor to 500.0
    let changed = manager.update_drag(Point::new(500.0, 150.0));
    assert!(changed);
    assert_eq!(manager.get(win_id).expect("win").rect.width, 400.0);

    // Press Escape to cancel
    let esc = UiEvent::KeyboardInput {
        key: KeyCode::Escape,
        is_pressed: true,
        text: None,
        ctrl_held: false,
        shift_held: false,
    };
    let viewer = SimpleTabViewer;
    let resp = manager.handle_event(&viewer, &esc);
    assert!(resp.layout_changed);
    assert!(!manager.is_dragging());
}