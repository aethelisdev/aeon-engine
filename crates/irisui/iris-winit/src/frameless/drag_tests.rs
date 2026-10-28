// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Window Drag Host and Double-Click Maximize Coordinator
//!
//! Verifies single-click window dragging, 200 ms and 300 ms double-click maximize detection,
//! sub-4.0 px spatial tolerance boundary cases, native `WindowEvent` / `UiEvent` integration,
//! declarative `TitleBarResponse` action routing, focus resets, and HighDPI scale factor conversion.

use super::drag::{
    DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD, DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD, WindowDragHost,
    WindowDragOutcome, is_double_click, point_distance,
};
use super::geometry::ResizeDirection;
use super::window_trait::{WindowInterface, WindowOpError};
use iris_core::event::{MouseButton as UiMouseButton, UiEvent};
use iris_core::geometry::{Point, Rect, Size};
use iris_core::id::WidgetId;
use iris_core::response::{ResponseFlags, WidgetResponse};
use iris_widgets::declarative::widget::{TitleBarAction, TitleBarResponse};
use std::time::{Duration, Instant};
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, ElementState, MouseButton as WinitMouseButton, WindowEvent};
use winit::window::CursorIcon;

#[derive(Debug, Default)]
struct MockDragWindow {
    drag_window_count: usize,
    toggle_maximize_count: usize,
    minimize_count: usize,
    close_count: usize,
    maximized: bool,
    logical_size: Size,
    scale_factor: f64,
}

impl MockDragWindow {
    fn new(size: Size) -> Self {
        Self {
            drag_window_count: 0,
            toggle_maximize_count: 0,
            minimize_count: 0,
            close_count: 0,
            maximized: false,
            logical_size: size,
            scale_factor: 1.0,
        }
    }
}

impl WindowInterface for MockDragWindow {
    fn set_cursor(&mut self, _icon: CursorIcon) {}

    fn drag_resize_window(&mut self, _direction: ResizeDirection) -> Result<(), WindowOpError> {
        Ok(())
    }

    fn drag_window(&mut self) -> Result<(), WindowOpError> {
        self.drag_window_count += 1;
        Ok(())
    }

    fn is_maximized(&self) -> bool {
        self.maximized
    }

    fn toggle_maximize(&mut self) {
        self.toggle_maximize_count += 1;
        self.maximized = !self.maximized;
    }

    fn minimize(&mut self) {
        self.minimize_count += 1;
    }

    fn close(&mut self) {
        self.close_count += 1;
    }

    fn logical_size(&self) -> Size {
        self.logical_size
    }

    fn scale_factor(&self) -> f64 {
        self.scale_factor
    }
}

#[test]
fn test_point_distance_calculations_and_edge_cases() {
    let p1 = Point::new(10.0, 20.0);
    let p2 = Point::new(10.0, 20.0);
    assert_eq!(point_distance(p1, p2), 0.0);

    let p3 = Point::new(13.0, 24.0);
    // dx = 3, dy = 4 => dist = 5.0
    assert_eq!(point_distance(p1, p3), 5.0);

    // Non-finite values
    let nan_pt = Point::new(f32::NAN, 10.0);
    assert_eq!(point_distance(p1, nan_pt), f32::INFINITY);

    let inf_pt = Point::new(10.0, f32::INFINITY);
    assert_eq!(point_distance(p1, inf_pt), f32::INFINITY);
}

#[test]
fn test_is_double_click_temporal_and_spatial_boundaries() {
    let t0 = Instant::now();
    let p1 = Point::new(100.0, 50.0);

    // Exactly at same point, 0 ms
    assert!(is_double_click(
        p1,
        t0,
        p1,
        t0,
        DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD,
        DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD
    ));

    // 200 ms later, same point -> true
    let t_200 = t0 + Duration::from_millis(200);
    assert!(is_double_click(
        p1,
        t0,
        p1,
        t_200,
        DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD,
        DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD
    ));

    // 300 ms later, same point -> true (inclusive limit)
    let t_300 = t0 + Duration::from_millis(300);
    assert!(is_double_click(
        p1,
        t0,
        p1,
        t_300,
        DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD,
        DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD
    ));

    // 301 ms later, same point -> false (exceeds 300 ms threshold)
    let t_301 = t0 + Duration::from_millis(301);
    assert!(!is_double_click(
        p1,
        t0,
        p1,
        t_301,
        DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD,
        DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD
    ));

    // Reversed time (t2 < t1) -> false
    assert!(!is_double_click(
        p1,
        t_200,
        p1,
        t0,
        DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD,
        DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD
    ));

    // Spatial boundaries at 200 ms:
    // Distance = 3.9 px (< 4.0 px) -> true
    let p_close = Point::new(100.0 + 3.9, 50.0);
    assert!(is_double_click(
        p1,
        t0,
        p_close,
        t_200,
        DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD,
        DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD
    ));

    // Distance = 4.0 px (not < 4.0 px) -> false
    let p_boundary = Point::new(100.0 + 4.0, 50.0);
    assert!(!is_double_click(
        p1,
        t0,
        p_boundary,
        t_200,
        DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD,
        DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD
    ));

    // Distance = 4.1 px -> false
    let p_far = Point::new(100.0 + 4.1, 50.0);
    assert!(!is_double_click(
        p1,
        t0,
        p_far,
        t_200,
        DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD,
        DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD
    ));
}

#[test]
fn test_single_click_initiates_drag_window() {
    let host = WindowDragHost::new();
    let mut win = MockDragWindow::new(Size::new(800.0, 600.0));
    let t0 = Instant::now();
    let click_point = Point::new(200.0, 15.0);

    let outcome = host.process_click_at(&mut win, click_point, t0);
    assert_eq!(outcome, WindowDragOutcome::DragStarted);
    assert!(outcome.is_drag_started());
    assert!(outcome.is_consumed());
    assert_eq!(win.drag_window_count, 1);
    assert_eq!(win.toggle_maximize_count, 0);

    let last = host.last_click();
    assert!(last.is_some());
    let (pt, t) = last.unwrap();
    assert_eq!(pt, click_point);
    assert_eq!(t, t0);
}

#[test]
fn test_double_click_within_200ms_toggles_maximize() {
    let host = WindowDragHost::new();
    let mut win = MockDragWindow::new(Size::new(800.0, 600.0));
    let t0 = Instant::now();
    let pt1 = Point::new(150.0, 16.0);

    // 1. First click -> drag_window
    let outcome1 = host.process_click_at(&mut win, pt1, t0);
    assert_eq!(outcome1, WindowDragOutcome::DragStarted);
    assert_eq!(win.drag_window_count, 1);
    assert_eq!(win.toggle_maximize_count, 0);
    assert!(!win.is_maximized());

    // 2. Second click 200 ms later at nearby coordinate (Delta r = 2.0 px < 4.0 px)
    let t1 = t0 + Duration::from_millis(200);
    let pt2 = Point::new(152.0, 16.0);
    let outcome2 = host.process_click_at(&mut win, pt2, t1);

    assert_eq!(outcome2, WindowDragOutcome::MaximizeToggled);
    assert!(outcome2.is_maximize_toggled());
    assert!(outcome2.is_consumed());
    assert_eq!(win.drag_window_count, 1); // Drag count does not increase on double-click
    assert_eq!(win.toggle_maximize_count, 1);
    assert!(win.is_maximized());

    // Double-click resets state so third click is single click
    assert_eq!(host.last_click(), None);
}

#[test]
fn test_triple_click_and_quadruple_click_cycle() {
    let host = WindowDragHost::new();
    let mut win = MockDragWindow::new(Size::new(800.0, 600.0));
    let t0 = Instant::now();
    let pt = Point::new(250.0, 14.0);

    // Click 1: single click -> drag
    let out1 = host.process_click_at(&mut win, pt, t0);
    assert_eq!(out1, WindowDragOutcome::DragStarted);
    assert_eq!(win.drag_window_count, 1);
    assert_eq!(win.toggle_maximize_count, 0);

    // Click 2 (at 100 ms): double-click -> toggle maximize
    let out2 = host.process_click_at(&mut win, pt, t0 + Duration::from_millis(100));
    assert_eq!(out2, WindowDragOutcome::MaximizeToggled);
    assert_eq!(win.drag_window_count, 1);
    assert_eq!(win.toggle_maximize_count, 1);

    // Click 3 (at 200 ms, 100 ms after click 2): new single click -> drag
    let out3 = host.process_click_at(&mut win, pt, t0 + Duration::from_millis(200));
    assert_eq!(out3, WindowDragOutcome::DragStarted);
    assert_eq!(win.drag_window_count, 2);
    assert_eq!(win.toggle_maximize_count, 1);

    // Click 4 (at 300 ms, 100 ms after click 3): second double-click -> toggle maximize
    let out4 = host.process_click_at(&mut win, pt, t0 + Duration::from_millis(300));
    assert_eq!(out4, WindowDragOutcome::MaximizeToggled);
    assert_eq!(win.drag_window_count, 2);
    assert_eq!(win.toggle_maximize_count, 2);
}

#[test]
fn test_slow_clicks_exceeding_300ms_do_not_trigger_maximize() {
    let host = WindowDragHost::new();
    let mut win = MockDragWindow::new(Size::new(800.0, 600.0));
    let t0 = Instant::now();
    let pt = Point::new(100.0, 10.0);

    // Click 1
    let out1 = host.process_click_at(&mut win, pt, t0);
    assert_eq!(out1, WindowDragOutcome::DragStarted);
    assert_eq!(win.drag_window_count, 1);

    // Click 2 after 350 ms (> 300 ms)
    let t1 = t0 + Duration::from_millis(350);
    let out2 = host.process_click_at(&mut win, pt, t1);
    assert_eq!(out2, WindowDragOutcome::DragStarted);
    assert_eq!(win.drag_window_count, 2);
    assert_eq!(win.toggle_maximize_count, 0);
}

#[test]
fn test_clicks_with_spatial_distance_exceeding_4px_do_not_trigger_maximize() {
    let host = WindowDragHost::new();
    let mut win = MockDragWindow::new(Size::new(800.0, 600.0));
    let t0 = Instant::now();

    // Click 1 at (100.0, 10.0)
    let out1 = host.process_click_at(&mut win, Point::new(100.0, 10.0), t0);
    assert_eq!(out1, WindowDragOutcome::DragStarted);

    // Click 2 at (105.0, 10.0) after 100 ms (Delta r = 5.0 px >= 4.0 px)
    let t1 = t0 + Duration::from_millis(100);
    let out2 = host.process_click_at(&mut win, Point::new(105.0, 10.0), t1);
    assert_eq!(out2, WindowDragOutcome::DragStarted);
    assert_eq!(win.drag_window_count, 2);
    assert_eq!(win.toggle_maximize_count, 0);
}

#[test]
fn test_disabled_host_ignores_all_interactions() {
    let host = WindowDragHost::new().with_enabled(false);
    let mut win = MockDragWindow::new(Size::new(800.0, 600.0));
    let t0 = Instant::now();
    let pt = Point::new(100.0, 10.0);

    let out1 = host.process_click_at(&mut win, pt, t0);
    assert_eq!(out1, WindowDragOutcome::None);
    assert!(!out1.is_consumed());
    assert_eq!(win.drag_window_count, 0);

    let out2 = host.process_click_at(&mut win, pt, t0 + Duration::from_millis(100));
    assert_eq!(out2, WindowDragOutcome::None);
    assert_eq!(win.toggle_maximize_count, 0);
}

#[test]
fn test_reset_clears_click_history_and_cursor() {
    let host = WindowDragHost::new();
    let mut win = MockDragWindow::new(Size::new(800.0, 600.0));
    let t0 = Instant::now();
    let pt = Point::new(100.0, 10.0);

    host.set_cursor_pos(Some(pt));
    let _ = host.process_click_at(&mut win, pt, t0);
    assert!(host.last_click().is_some());
    assert!(host.cursor_pos().is_some());

    host.reset();
    assert_eq!(host.last_click(), None);
    assert_eq!(host.cursor_pos(), None);

    // Subsequent click within 100 ms will now be a single click, not a double click
    let out = host.process_click_at(&mut win, pt, t0 + Duration::from_millis(100));
    assert_eq!(out, WindowDragOutcome::DragStarted);
    assert_eq!(win.toggle_maximize_count, 0);
}

#[test]
fn test_winit_window_event_integration_inside_and_outside_drag_rect() {
    let host = WindowDragHost::new();
    let mut win = MockDragWindow::new(Size::new(800.0, 600.0));
    let drag_rect = Rect::new(0.0, 0.0, 800.0, 32.0);
    let t0 = Instant::now();

    // 1. Move cursor to title bar area (150.0, 16.0)
    let move_ev = WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(150.0, 16.0),
    };
    let out_move = host.process_window_event_at(&mut win, &move_ev, drag_rect, t0);
    assert_eq!(out_move, WindowDragOutcome::None);
    assert_eq!(host.cursor_pos(), Some(Point::new(150.0, 16.0)));

    // 2. Mouse press inside title bar -> drag_window
    let press_ev = WindowEvent::MouseInput {
        device_id: DeviceId::dummy(),
        state: ElementState::Pressed,
        button: WinitMouseButton::Left,
    };
    let out_press = host.process_window_event_at(&mut win, &press_ev, drag_rect, t0);
    assert_eq!(out_press, WindowDragOutcome::DragStarted);
    assert_eq!(win.drag_window_count, 1);

    // 3. Second click within 200 ms inside title bar -> toggle_maximize
    let t1 = t0 + Duration::from_millis(200);
    let out_double = host.process_window_event_at(&mut win, &press_ev, drag_rect, t1);
    assert_eq!(out_double, WindowDragOutcome::MaximizeToggled);
    assert_eq!(win.toggle_maximize_count, 1);

    // 4. Move cursor outside title bar (150.0, 100.0)
    let move_out = WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(150.0, 100.0),
    };
    host.process_window_event_at(&mut win, &move_out, drag_rect, t1);

    // 5. Mouse press outside drag rect -> None
    let out_press_outside = host.process_window_event_at(&mut win, &press_ev, drag_rect, t1);
    assert_eq!(out_press_outside, WindowDragOutcome::None);
    assert_eq!(win.drag_window_count, 1);
}

#[test]
fn test_winit_window_event_scale_factor_and_focus_loss() {
    let host = WindowDragHost::new();
    let mut win = MockDragWindow::new(Size::new(800.0, 600.0));
    let drag_rect = Rect::new(0.0, 0.0, 800.0, 32.0);
    let t0 = Instant::now();

    // ScaleFactorChanged to 2.0x
    host.set_scale_factor(2.0);
    assert_eq!(host.scale_factor(), 2.0);

    // Move to physical (300.0, 40.0) -> logical (150.0, 20.0)
    let move_ev = WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(300.0, 40.0),
    };
    host.process_window_event_at(&mut win, &move_ev, drag_rect, t0);
    assert_eq!(host.cursor_pos(), Some(Point::new(150.0, 20.0)));

    // Window focus loss clears state
    let focus_ev = WindowEvent::Focused(false);
    host.process_window_event_at(&mut win, &focus_ev, drag_rect, t0);
    assert_eq!(host.cursor_pos(), None);
    assert_eq!(host.last_click(), None);
}

#[test]
fn test_iris_ui_event_integration() {
    let host = WindowDragHost::new();
    let mut win = MockDragWindow::new(Size::new(800.0, 600.0));
    let drag_rect = Rect::new(0.0, 0.0, 800.0, 32.0);
    let t0 = Instant::now();

    // MouseDown inside drag_rect
    let down_ev = UiEvent::MouseDown {
        button: UiMouseButton::Left,
        point: Point::new(120.0, 15.0),
    };
    let out1 = host.process_ui_event_at(&mut win, &down_ev, drag_rect, t0);
    assert_eq!(out1, WindowDragOutcome::DragStarted);
    assert_eq!(win.drag_window_count, 1);

    // MouseDown 200 ms later at nearby point -> toggle maximize
    let down_ev2 = UiEvent::MouseDown {
        button: UiMouseButton::Left,
        point: Point::new(122.0, 16.0),
    };
    let out2 = host.process_ui_event_at(
        &mut win,
        &down_ev2,
        drag_rect,
        t0 + Duration::from_millis(200),
    );
    assert_eq!(out2, WindowDragOutcome::MaximizeToggled);
    assert_eq!(win.toggle_maximize_count, 1);

    // MouseDown outside drag_rect -> None
    let down_outside = UiEvent::MouseDown {
        button: UiMouseButton::Left,
        point: Point::new(120.0, 200.0),
    };
    let out3 = host.process_ui_event_at(
        &mut win,
        &down_outside,
        drag_rect,
        t0 + Duration::from_millis(300),
    );
    assert_eq!(out3, WindowDragOutcome::None);
    assert_eq!(win.drag_window_count, 1);
}

#[test]
fn test_title_bar_response_integration_actions_and_drag() {
    let host = WindowDragHost::new();
    let mut win = MockDragWindow::new(Size::new(800.0, 600.0));
    let t0 = Instant::now();

    // 1. Control button Minimize action
    let min_resp = TitleBarResponse {
        response: WidgetResponse::default(),
        action: Some(TitleBarAction::Minimize),
    };
    let out_min = host.process_title_bar_response_at(&mut win, &min_resp, t0);
    assert_eq!(out_min, WindowDragOutcome::Minimized);
    assert!(out_min.is_minimized());
    assert_eq!(win.minimize_count, 1);

    // 2. Control button Maximize action
    let max_resp = TitleBarResponse {
        response: WidgetResponse::default(),
        action: Some(TitleBarAction::Maximize),
    };
    let out_max = host.process_title_bar_response_at(&mut win, &max_resp, t0);
    assert_eq!(out_max, WindowDragOutcome::MaximizeToggled);
    assert_eq!(win.toggle_maximize_count, 1);

    // 3. Control button Close action
    let close_resp = TitleBarResponse {
        response: WidgetResponse::default(),
        action: Some(TitleBarAction::Close),
    };
    let out_close = host.process_title_bar_response_at(&mut win, &close_resp, t0);
    assert_eq!(out_close, WindowDragOutcome::Closed);
    assert!(out_close.is_closed());
    assert_eq!(win.close_count, 1);

    // 4. Drag surface clicked (single click)
    let drag_resp = TitleBarResponse {
        response: WidgetResponse {
            id: WidgetId::default(),
            rect: Rect::new(0.0, 0.0, 800.0, 32.0),
            flags: ResponseFlags::CLICKED,
            pointer_pos: Some(Point::new(300.0, 16.0)),
            drag_delta: None,
        },
        action: None,
    };

    assert!(drag_resp.drag_area_clicked());

    let out_drag1 = host.process_title_bar_response_at(&mut win, &drag_resp, t0);
    assert_eq!(out_drag1, WindowDragOutcome::DragStarted);
    assert_eq!(win.drag_window_count, 1);

    // 5. Drag surface clicked 200 ms later (double click)
    let out_drag2 =
        host.process_title_bar_response_at(&mut win, &drag_resp, t0 + Duration::from_millis(200));
    assert_eq!(out_drag2, WindowDragOutcome::MaximizeToggled);
    assert_eq!(win.toggle_maximize_count, 2);
}

#[test]
fn test_configured_title_bar_height_and_drag_rect() {
    let host = WindowDragHost::new().with_title_bar_height(32.0);
    let size = Size::new(800.0, 600.0);

    assert!(host.is_point_in_drag_area(Point::new(400.0, 16.0), Some(size)));
    assert!(!host.is_point_in_drag_area(Point::new(400.0, 35.0), Some(size)));
    assert!(!host.is_point_in_drag_area(Point::new(850.0, 16.0), Some(size)));

    // With explicit drag rect
    let host_rect = WindowDragHost::new().with_drag_rect(Rect::new(50.0, 0.0, 700.0, 32.0));
    assert!(host_rect.is_point_in_drag_area(Point::new(100.0, 16.0), None));
    assert!(!host_rect.is_point_in_drag_area(Point::new(20.0, 16.0), None));
}

#[test]
fn test_window_drag_host_handle_title_bar_action() {
    let host = WindowDragHost::new();
    let mut win = MockDragWindow::default();

    assert_eq!(
        host.handle_title_bar_action(&mut win, TitleBarAction::Minimize),
        WindowDragOutcome::Minimized
    );
    assert_eq!(win.minimize_count, 1);

    assert_eq!(
        host.handle_title_bar_action(&mut win, TitleBarAction::Maximize),
        WindowDragOutcome::MaximizeToggled
    );
    assert_eq!(win.toggle_maximize_count, 1);

    assert_eq!(
        host.handle_title_bar_action(&mut win, TitleBarAction::Close),
        WindowDragOutcome::Closed
    );
    assert_eq!(win.close_count, 1);
}