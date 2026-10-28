// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Workbench Event Dispatching and WindowInterface Integration
//!
//! Validates frameless window edge resize hit-testing, native title bar dragging,
//! double-click window maximization, and cursor icon synchronization using headless mock windows.

use iris_winit::frameless::{FramelessWindowHost, ResizeDirection, WindowInterface, WindowOpError};
use irisui::prelude::{MouseButton, Point, Rect, Size, UiEvent};
use winit::window::CursorIcon;

#[derive(Debug)]
struct MockWindow {
    last_cursor: Option<CursorIcon>,
    last_resize_direction: Option<ResizeDirection>,
    drag_window_called: bool,
    maximized: bool,
    minimized: bool,
    closed: bool,
    cursor_call_count: usize,
    resize_call_count: usize,
    logical_size: Size,
    scale_factor: f64,
}

impl Default for MockWindow {
    fn default() -> Self {
        Self {
            last_cursor: None,
            last_resize_direction: None,
            drag_window_called: false,
            maximized: false,
            minimized: false,
            closed: false,
            cursor_call_count: 0,
            resize_call_count: 0,
            logical_size: Size::new(1920.0, 1080.0),
            scale_factor: 1.0,
        }
    }
}

impl WindowInterface for MockWindow {
    fn set_cursor(&mut self, icon: CursorIcon) {
        self.last_cursor = Some(icon);
        self.cursor_call_count += 1;
    }

    fn drag_resize_window(&mut self, direction: ResizeDirection) -> Result<(), WindowOpError> {
        self.last_resize_direction = Some(direction);
        self.resize_call_count += 1;
        Ok(())
    }

    fn drag_window(&mut self) -> Result<(), WindowOpError> {
        self.drag_window_called = true;
        Ok(())
    }

    fn is_maximized(&self) -> bool {
        self.maximized
    }

    fn toggle_maximize(&mut self) {
        self.maximized = !self.maximized;
    }

    fn minimize(&mut self) {
        self.minimized = true;
    }

    fn close(&mut self) {
        self.closed = true;
    }

    fn logical_size(&self) -> Size {
        self.logical_size
    }

    fn scale_factor(&self) -> f64 {
        self.scale_factor
    }
}

#[test]
fn test_frameless_window_host_resize_hover_with_window_interface() {
    let host = FramelessWindowHost::new();
    let mut win = MockWindow::default();

    // 1. Move to left border (x = 2.0 <= 6.0)
    let left_move = UiEvent::MouseMove {
        point: Point::new(2.0, 500.0),
    };
    assert!(host.handle_ui_event_for_window(&mut win, &left_move));
    assert_eq!(win.last_cursor, Some(CursorIcon::WResize));

    // 2. Move to top-right corner
    let tr_move = UiEvent::MouseMove {
        point: Point::new(1918.0, 2.0),
    };
    assert!(host.handle_ui_event_for_window(&mut win, &tr_move));
    assert_eq!(win.last_cursor, Some(CursorIcon::NeResize));

    // 3. Move to safe interior restores cursor to default and does not consume as resize
    let interior_move = UiEvent::MouseMove {
        point: Point::new(960.0, 540.0),
    };
    assert!(!host.handle_ui_event_for_window(&mut win, &interior_move));
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));
}

#[test]
fn test_frameless_window_host_resize_drag_with_window_interface() {
    let host = FramelessWindowHost::new();
    let mut win = MockWindow::default();

    // MouseDown on bottom border triggers drag-resize
    let bottom_down = UiEvent::MouseDown {
        point: Point::new(960.0, 1078.0),
        button: MouseButton::Left,
    };
    assert!(host.handle_ui_event_for_window(&mut win, &bottom_down));
    assert_eq!(win.last_resize_direction, Some(ResizeDirection::Bottom));
    assert_eq!(win.resize_call_count, 1);
}

#[test]
fn test_frameless_window_host_maximized_suppresses_resize() {
    let host = FramelessWindowHost::new();
    let mut win = MockWindow {
        maximized: true,
        ..Default::default()
    };

    // Hover on boundary margin
    let edge_move = UiEvent::MouseMove {
        point: Point::new(2.0, 500.0),
    };
    assert!(!host.handle_ui_event_for_window(&mut win, &edge_move));
    assert_ne!(win.last_cursor, Some(CursorIcon::WResize));

    // Click on boundary margin
    let edge_click = UiEvent::MouseDown {
        point: Point::new(2.0, 500.0),
        button: MouseButton::Left,
    };
    assert!(!host.handle_ui_event_for_window(&mut win, &edge_click));
    assert_eq!(win.resize_call_count, 0);
}

#[test]
fn test_window_interface_methods_on_mock() {
    let mut win = MockWindow::default();

    assert_eq!(win.logical_size(), Size::new(1920.0, 1080.0));
    assert_eq!(win.scale_factor(), 1.0);

    assert!(!win.is_maximized());
    win.toggle_maximize();
    assert!(win.is_maximized());
    win.toggle_maximize();
    assert!(!win.is_maximized());

    assert!(!win.minimized);
    win.minimize();
    assert!(win.minimized);

    assert!(!win.closed);
    win.close();
    assert!(win.closed);

    assert!(!win.drag_window_called);
    assert!(win.drag_window().is_ok());
    assert!(win.drag_window_called);

    win.set_widget_cursor(irisui::prelude::WidgetCursor::Pointer);
    assert_eq!(win.last_cursor, Some(CursorIcon::Pointer));
}

#[test]
fn test_window_drag_host_single_click_and_double_click_maximize() {
    let host = iris_winit::frameless::WindowDragHost::new().with_title_bar_height(36.0);
    let mut win = MockWindow::default();
    let drag_rect = Rect::new(0.0, 0.0, 1000.0, 36.0);

    // 1. Single click inside title bar (y = 18.0 <= 36.0) initiates native drag_window
    let click1 = UiEvent::MouseDown {
        point: Point::new(500.0, 18.0),
        button: MouseButton::Left,
    };
    let outcome1 = host.process_ui_event(&mut win, &click1, drag_rect);
    assert_eq!(
        outcome1,
        iris_winit::frameless::WindowDragOutcome::DragStarted
    );
    assert!(win.drag_window_called);
    assert!(!win.is_maximized());

    // 2. Second click within 100 ms at identical coordinate toggles window maximization
    let click2 = UiEvent::MouseDown {
        point: Point::new(500.0, 18.0),
        button: MouseButton::Left,
    };
    let outcome2 = host.process_ui_event(&mut win, &click2, drag_rect);
    assert_eq!(
        outcome2,
        iris_winit::frameless::WindowDragOutcome::MaximizeToggled
    );
    assert!(win.is_maximized());
}

#[test]
fn test_window_drag_host_handle_title_bar_actions_on_mock_window() {
    use irisui::prelude::TitleBarAction;

    let host = iris_winit::frameless::WindowDragHost::new();
    let mut win = MockWindow::default();

    // 1. Minimize action
    let out_min = host.handle_title_bar_action(&mut win, TitleBarAction::Minimize);
    assert_eq!(out_min, iris_winit::frameless::WindowDragOutcome::Minimized);
    assert!(win.minimized);

    // 2. Maximize action
    let out_max = host.handle_title_bar_action(&mut win, TitleBarAction::Maximize);
    assert_eq!(
        out_max,
        iris_winit::frameless::WindowDragOutcome::MaximizeToggled
    );
    assert!(win.maximized);

    // 3. Close action
    let out_close = host.handle_title_bar_action(&mut win, TitleBarAction::Close);
    assert_eq!(out_close, iris_winit::frameless::WindowDragOutcome::Closed);
    assert!(win.closed);
}