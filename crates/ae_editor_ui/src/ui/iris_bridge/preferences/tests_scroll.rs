// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Autonomous Scroll Area Integration Verification Suite
//!
//! Validates autonomous scroll frame geometry, mouse wheel scrolling with bounds clamping,
//! scrollbar thumb click-drag-release lifecycle, track jump clicks, and state synchronization.
//!

use super::types::{PreferencesDialogState, preferences_scroll_style};
use irisui::prelude::*;

#[test]
fn test_preferences_scroll_style_and_frame_geometry() {
    let style = preferences_scroll_style();
    assert_eq!(style.thickness, 6.0);
    assert_eq!(style.inset, 3.0);

    let viewport = Rect::new(161.0, 36.0, 599.0, 504.0);
    let total_h = 1800.0;
    let scroll_y = 120.0;

    let frame = ScrollAreaFrame::from_geometry(viewport, total_h, scroll_y, &style);
    assert_eq!(frame.viewport_rect, viewport);
    assert_eq!(frame.content_height, 1800.0);
    assert_eq!(frame.max_scroll_y, 1800.0 - 504.0);

    let geom = frame
        .scrollbar
        .expect("Scrollbar geometry must be computed");
    assert_eq!(geom.track_rect.width, 6.0);
    assert!(geom.thumb_rect.height >= style.thumb_min_length);
    assert_eq!(geom.thumb_rect.width, 6.0);
}

#[test]
fn test_preferences_scroll_state_reset_and_sync() {
    let mut state = PreferencesDialogState::default();
    state.scroll_state.scroll_y = 250.0;
    state.scroll_state.drag_state = Some(irisui::prelude::ScrollbarDragState::new(300.0, 250.0));
    assert!(state.scroll_state.is_dragging());

    state.sync_scroll_y();
    assert_eq!(state.scroll_y, 250.0);

    state.reset_scroll();
    assert_eq!(state.scroll_state.scroll_y, 0.0);
    assert_eq!(state.scroll_y, 0.0);
    assert!(!state.scroll_state.is_dragging());
}

#[test]
fn test_preferences_handle_scroll_area_event_wheel_scrolling() {
    let viewport = Rect::new(161.0, 36.0, 599.0, 504.0);
    let total_h = 1500.0;
    let style = preferences_scroll_style();
    let frame = ScrollAreaFrame::from_geometry(viewport, total_h, 0.0, &style);

    let mut state = PreferencesDialogState::default();
    assert_eq!(state.scroll_state.scroll_y, 0.0);

    // 1. Wheel scroll inside content area
    let cursor_inside = Point::new(300.0, 200.0);
    let wheel_event = UiEvent::MouseScroll {
        point: cursor_inside,
        delta_x: 0.0,
        delta_y: -50.0,
    };

    let resp = handle_scroll_area_event(&mut state.scroll_state, &wheel_event, &frame, None);
    assert!(resp.consumed);
    assert!(resp.scroll_y > 0.0);
    assert_eq!(state.scroll_state.scroll_y, resp.scroll_y);

    state.sync_scroll_y();
    assert_eq!(state.scroll_y, state.scroll_state.scroll_y);

    // 2. Large wheel scroll clamps to max_scroll_y
    let huge_wheel = UiEvent::MouseScroll {
        point: cursor_inside,
        delta_x: 0.0,
        delta_y: -5000.0,
    };
    let resp_clamped = handle_scroll_area_event(&mut state.scroll_state, &huge_wheel, &frame, None);
    assert!(resp_clamped.consumed);
    assert_eq!(resp_clamped.scroll_y, frame.max_scroll_y);
    assert_eq!(state.scroll_state.scroll_y, frame.max_scroll_y);

    // 3. Wheel upwards clamps at 0.0
    let up_wheel = UiEvent::MouseScroll {
        point: cursor_inside,
        delta_x: 0.0,
        delta_y: 10000.0,
    };
    let resp_top = handle_scroll_area_event(&mut state.scroll_state, &up_wheel, &frame, None);
    assert!(resp_top.consumed);
    assert_eq!(resp_top.scroll_y, 0.0);
    assert_eq!(state.scroll_state.scroll_y, 0.0);
}

#[test]
fn test_preferences_handle_scroll_area_event_thumb_drag_lifecycle() {
    let viewport = Rect::new(161.0, 36.0, 599.0, 504.0);
    let total_h = 1800.0;
    let style = preferences_scroll_style();
    let frame = ScrollAreaFrame::from_geometry(viewport, total_h, 0.0, &style);
    let geom = frame.scrollbar.as_ref().expect("Scrollbar must exist");

    let mut state = PreferencesDialogState::default();

    // 1. Mouse down on thumb -> begins continuous drag
    let thumb_center = Point::new(
        geom.thumb_rect.x + geom.thumb_rect.width * 0.5,
        geom.thumb_rect.y + geom.thumb_rect.height * 0.5,
    );
    let down_event = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: thumb_center,
    };

    let resp_down = handle_scroll_area_event(&mut state.scroll_state, &down_event, &frame, None);
    assert!(resp_down.consumed);
    assert!(resp_down.capture_pointer);
    assert!(state.scroll_state.is_dragging());

    // 2. Mouse move during drag -> translates scroll
    let moved_point = Point::new(thumb_center.x, thumb_center.y + 60.0);
    let move_event = UiEvent::MouseMove { point: moved_point };

    let resp_move = handle_scroll_area_event(&mut state.scroll_state, &move_event, &frame, None);
    assert!(resp_move.consumed);
    assert!(resp_move.scroll_y > 0.0);
    assert_eq!(state.scroll_state.scroll_y, resp_move.scroll_y);

    // 3. Mouse release -> clears drag state and requests pointer capture release
    let up_event = UiEvent::MouseUp {
        button: MouseButton::Left,
        point: moved_point,
    };

    let resp_up = handle_scroll_area_event(&mut state.scroll_state, &up_event, &frame, None);
    assert!(resp_up.consumed);
    assert!(resp_up.release_pointer);
    assert!(!state.scroll_state.is_dragging());
}

#[test]
fn test_preferences_wheel_outside_content_is_unconsumed() {
    let viewport = Rect::new(161.0, 36.0, 599.0, 504.0);
    let total_h = 1500.0;
    let style = preferences_scroll_style();
    let frame = ScrollAreaFrame::from_geometry(viewport, total_h, 50.0, &style);

    let mut state = PreferencesDialogState::default();
    state.scroll_state.scroll_y = 50.0;

    // Cursor is in sidebar (e.g. x = 80.0), outside viewport (x = 161.0)
    let sidebar_cursor = Point::new(80.0, 200.0);
    let wheel_event = UiEvent::MouseScroll {
        point: sidebar_cursor,
        delta_x: 0.0,
        delta_y: -20.0,
    };

    let resp = handle_scroll_area_event(&mut state.scroll_state, &wheel_event, &frame, None);
    assert!(
        !resp.consumed,
        "Wheel event outside viewport must not be consumed by ScrollArea"
    );
    assert_eq!(
        state.scroll_state.scroll_y, 50.0,
        "Scroll position must remain unchanged"
    );
}