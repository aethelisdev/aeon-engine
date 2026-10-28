// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Frameless Window Host and Event Dispatcher
//!
//! Verifies 8-way resize boundary hit-testing, dynamic hardware cursor management,
//! cursor icon clean restoration to default, HighDPI scale factor lifecycle across
//! window state changes, robust event suppression, and decoupled window operations.

use super::geometry::ResizeDirection;
use super::host::FramelessWindowHost;
use super::window_trait::{WindowInterface, WindowOpError};
use iris_core::event::UiEvent;
use iris_core::geometry::{Point, Size};
use iris_core::node::WidgetCursor;
use std::sync::Arc;
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, ElementState, MouseButton, WindowEvent};
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
    fail_drag_resize: bool,
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
            fail_drag_resize: false,
            logical_size: Size::new(1000.0, 700.0),
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
        if self.fail_drag_resize {
            Err(WindowOpError::PlatformRejected)
        } else {
            Ok(())
        }
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

#[inline]
fn make_move_event(x: f64, y: f64) -> WindowEvent {
    WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(x, y),
    }
}

#[inline]
fn make_mouse_event(state: ElementState, button: MouseButton) -> WindowEvent {
    WindowEvent::MouseInput {
        device_id: DeviceId::dummy(),
        state,
        button,
    }
}

#[test]
fn test_frameless_host_cursor_movement_all_edges_and_corners() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // 1. Four cardinal edges
    // Left edge
    let ev = make_move_event(2.0, 300.0);
    assert!(host.handle_window_event(&mut win, &ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::WResize));

    // Right edge
    let ev = make_move_event(798.0, 300.0);
    assert!(host.handle_window_event(&mut win, &ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::EResize));

    // Top edge
    let ev = make_move_event(400.0, 2.0);
    assert!(host.handle_window_event(&mut win, &ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::NResize));

    // Bottom edge
    let ev = make_move_event(400.0, 598.0);
    assert!(host.handle_window_event(&mut win, &ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::SResize));

    // 2. Four corners
    // TopLeft
    let ev = make_move_event(1.0, 1.0);
    assert!(host.handle_window_event(&mut win, &ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::NwResize));

    // TopRight
    let ev = make_move_event(799.0, 1.0);
    assert!(host.handle_window_event(&mut win, &ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::NeResize));

    // BottomLeft
    let ev = make_move_event(1.0, 599.0);
    assert!(host.handle_window_event(&mut win, &ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::SwResize));

    // BottomRight
    let ev = make_move_event(799.0, 599.0);
    assert!(host.handle_window_event(&mut win, &ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::SeResize));
}

#[test]
fn test_frameless_host_interior_cursor_motion_not_consumed() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    let ev = make_move_event(400.0, 300.0);
    assert!(!host.handle_window_event(&mut win, &ev, size));
    assert_eq!(win.cursor_call_count, 0);
    assert_eq!(win.last_cursor, None);
}

#[test]
fn test_frameless_host_mouse_click_triggers_drag_resize() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // Move to TopLeft corner then click
    let move_ev = make_move_event(0.0, 0.0);
    assert!(host.handle_window_event(&mut win, &move_ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::NwResize));

    let click_ev = make_mouse_event(ElementState::Pressed, MouseButton::Left);
    assert!(host.handle_window_event(&mut win, &click_ev, size));
    assert_eq!(win.last_resize_direction, Some(ResizeDirection::TopLeft));
    assert_eq!(win.resize_call_count, 1);

    // Move to Right edge then click
    let move_ev = make_move_event(798.0, 300.0);
    assert!(host.handle_window_event(&mut win, &move_ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::EResize));

    let click_ev = make_mouse_event(ElementState::Pressed, MouseButton::Left);
    assert!(host.handle_window_event(&mut win, &click_ev, size));
    assert_eq!(win.last_resize_direction, Some(ResizeDirection::Right));
    assert_eq!(win.resize_call_count, 2);
}

#[test]
fn test_frameless_host_other_mouse_buttons_and_release_ignored() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // Position cursor on left border
    let move_ev = make_move_event(2.0, 300.0);
    assert!(host.handle_window_event(&mut win, &move_ev, size));

    // Right click should not trigger resize
    let right_click = make_mouse_event(ElementState::Pressed, MouseButton::Right);
    assert!(!host.handle_window_event(&mut win, &right_click, size));
    assert_eq!(win.resize_call_count, 0);

    // Middle click should not trigger resize
    let middle_click = make_mouse_event(ElementState::Pressed, MouseButton::Middle);
    assert!(!host.handle_window_event(&mut win, &middle_click, size));
    assert_eq!(win.resize_call_count, 0);

    // Button release should not trigger resize
    let release_ev = make_mouse_event(ElementState::Released, MouseButton::Left);
    assert!(!host.handle_window_event(&mut win, &release_ev, size));
    assert_eq!(win.resize_call_count, 0);
}

#[test]
fn test_frameless_host_interior_click_not_consumed() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // Position in center
    let move_ev = make_move_event(400.0, 300.0);
    assert!(!host.handle_window_event(&mut win, &move_ev, size));

    let click_ev = make_mouse_event(ElementState::Pressed, MouseButton::Left);
    assert!(!host.handle_window_event(&mut win, &click_ev, size));
    assert_eq!(win.resize_call_count, 0);
}

#[test]
fn test_frameless_host_disabled_and_maximized_guards() {
    let size = Size::new(800.0, 600.0);

    // 1. Host disabled
    let host = FramelessWindowHost::new().with_enabled(false);
    let mut win = MockWindow::default();

    let move_ev = make_move_event(1.0, 1.0);
    assert!(!host.handle_window_event(&mut win, &move_ev, size));
    let click_ev = make_mouse_event(ElementState::Pressed, MouseButton::Left);
    assert!(!host.handle_window_event(&mut win, &click_ev, size));
    assert_eq!(win.cursor_call_count, 0);
    assert_eq!(win.resize_call_count, 0);

    // 2. Window maximized
    let host = FramelessWindowHost::new();
    let mut max_win = MockWindow {
        maximized: true,
        ..Default::default()
    };
    assert!(!host.handle_window_event(&mut max_win, &move_ev, size));
    assert!(!host.handle_window_event(&mut max_win, &click_ev, size));
    assert_eq!(max_win.cursor_call_count, 0);
    assert_eq!(max_win.resize_call_count, 0);
}

#[test]
fn test_frameless_host_highdpi_scaling() {
    // 2.0x HighDPI scale factor
    let host = FramelessWindowHost::new().with_scale_factor(2.0);
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // Physical position (1596.0, 600.0) -> logical (798.0, 300.0) -> Right edge
    let move_ev = make_move_event(1596.0, 600.0);
    assert!(host.handle_window_event(&mut win, &move_ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::EResize));

    let click_ev = make_mouse_event(ElementState::Pressed, MouseButton::Left);
    assert!(host.handle_window_event(&mut win, &click_ev, size));
    assert_eq!(win.last_resize_direction, Some(ResizeDirection::Right));

    // Dynamic scale factor update
    host.set_scale_factor(1.5);
    assert_eq!(host.scale_factor(), 1.5);
}

#[test]
fn test_frameless_host_cursor_left_clears_position() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // Move to border
    let move_ev = make_move_event(2.0, 300.0);
    assert!(host.handle_window_event(&mut win, &move_ev, size));
    assert!(host.cursor_pos().is_some());

    // Cursor leaves window
    let leave_ev = WindowEvent::CursorLeft {
        device_id: DeviceId::dummy(),
    };
    assert!(!host.handle_window_event(&mut win, &leave_ev, size));
    assert!(host.cursor_pos().is_none());

    // Click after cursor left should not trigger resize
    let click_ev = make_mouse_event(ElementState::Pressed, MouseButton::Left);
    assert!(!host.handle_window_event(&mut win, &click_ev, size));
    assert_eq!(win.resize_call_count, 0);
}

#[test]
fn test_mock_window_interface_methods() {
    let mut win = MockWindow::default();
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
}

#[test]
fn test_frameless_host_builder_and_robustness() {
    let host = FramelessWindowHost::default();
    assert_eq!(host.border_thickness, 6.0);
    assert_eq!(host.corner_size, 12.0);
    assert!(host.enabled);
    assert_eq!(host.scale_factor(), 1.0);
    assert!(host.cursor_pos().is_none());

    let custom = FramelessWindowHost::new()
        .with_border_thickness(8.0)
        .with_corner_size(16.0)
        .with_enabled(false)
        .with_scale_factor(0.0); // should clamp to 1.0

    assert_eq!(custom.border_thickness, 8.0);
    assert_eq!(custom.corner_size, 16.0);
    assert!(!custom.enabled);
    assert_eq!(custom.scale_factor(), 1.0);

    custom.set_scale_factor(-2.5);
    assert_eq!(custom.scale_factor(), 1.0);

    // Non-finite scale factors should default/clamp to 1.0
    let inf_host = FramelessWindowHost::new().with_scale_factor(f64::INFINITY);
    assert_eq!(inf_host.scale_factor(), 1.0);

    let nan_host = FramelessWindowHost::new().with_scale_factor(f64::NAN);
    assert_eq!(nan_host.scale_factor(), 1.0);

    let neg_inf_host = FramelessWindowHost::new().with_scale_factor(f64::NEG_INFINITY);
    assert_eq!(neg_inf_host.scale_factor(), 1.0);

    inf_host.set_scale_factor(f64::INFINITY);
    assert_eq!(inf_host.scale_factor(), 1.0);

    inf_host.set_scale_factor(f64::NAN);
    assert_eq!(inf_host.scale_factor(), 1.0);

    custom.set_cursor_pos(Some(Point::new(10.0, 20.0)));
    assert_eq!(custom.cursor_pos(), Some(Point::new(10.0, 20.0)));

    // Non-finite cursor pos should clear
    custom.set_cursor_pos(Some(Point::new(f32::NAN, 20.0)));
    assert!(custom.cursor_pos().is_none());
}

#[test]
fn test_scale_factor_changed_while_window_is_maximized_or_disabled() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut max_win = MockWindow {
        maximized: true,
        ..Default::default()
    };

    // Updating scale factor must work and normalize coordinates correctly even if window is maximized
    host.set_scale_factor(2.0);
    assert_eq!(host.scale_factor(), 2.0);

    let move_ev = make_move_event(200.0, 100.0);
    assert!(!host.handle_window_event(&mut max_win, &move_ev, size));
    assert_eq!(host.cursor_pos(), Some(Point::new(100.0, 50.0)));

    // Scale factor updates and HighDPI normalization must work even when host is disabled
    let disabled_host = FramelessWindowHost::new().with_enabled(false);
    let mut win = MockWindow::default();
    disabled_host.set_scale_factor(1.5);
    assert_eq!(disabled_host.scale_factor(), 1.5);
    let move_ev2 = make_move_event(150.0, 75.0);
    assert!(!disabled_host.handle_window_event(&mut win, &move_ev2, size));
    assert_eq!(disabled_host.cursor_pos(), Some(Point::new(100.0, 50.0)));
}

#[test]
fn test_cursor_left_while_window_is_maximized_or_disabled() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // Hover border
    let move_ev = make_move_event(2.0, 300.0);
    assert!(host.handle_window_event(&mut win, &move_ev, size));
    assert!(host.cursor_pos().is_some());

    // Window maximized
    win.maximized = true;

    // Cursor leaves window
    let leave_ev = WindowEvent::CursorLeft {
        device_id: DeviceId::dummy(),
    };
    assert!(!host.handle_window_event(&mut win, &leave_ev, size));
    assert!(host.cursor_pos().is_none());
    assert!(!host.is_hovering_border());
}

#[test]
fn test_cursor_restoration_when_moving_border_to_interior() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // 1. Move to left border -> WResize applied
    let move_border = make_move_event(2.0, 300.0);
    assert!(host.handle_window_event(&mut win, &move_border, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::WResize));
    assert!(host.is_hovering_border());
    let initial_calls = win.cursor_call_count;

    // 2. Move to interior (400.0, 300.0) -> CursorIcon::Default cleanly restored
    let move_interior = make_move_event(400.0, 300.0);
    assert!(!host.handle_window_event(&mut win, &move_interior, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));
    assert!(!host.is_hovering_border());
    assert_eq!(win.cursor_call_count, initial_calls + 1);

    // 3. Move again inside interior -> not hovering border, zero redundant cursor calls
    let move_interior2 = make_move_event(410.0, 300.0);
    assert!(!host.handle_window_event(&mut win, &move_interior2, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));
    assert_eq!(win.cursor_call_count, initial_calls + 1);
}

#[test]
fn test_cursor_restoration_on_cursor_left() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // Move to border
    let move_border = make_move_event(2.0, 300.0);
    assert!(host.handle_window_event(&mut win, &move_border, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::WResize));

    // Cursor leaves window -> restored to Default
    let leave_ev = WindowEvent::CursorLeft {
        device_id: DeviceId::dummy(),
    };
    assert!(!host.handle_window_event(&mut win, &leave_ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));
    assert!(!host.is_hovering_border());
}

#[test]
fn test_cursor_restoration_when_maximized_while_hovering() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // Move to border
    let move_border = make_move_event(2.0, 300.0);
    assert!(host.handle_window_event(&mut win, &move_border, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::WResize));

    // Window maximized
    win.maximized = true;

    // Pointer moves along edge -> resize is suppressed and cursor restored to Default
    let move_ev2 = make_move_event(3.0, 305.0);
    assert!(!host.handle_window_event(&mut win, &move_ev2, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));
    assert!(!host.is_hovering_border());
}

#[test]
fn test_non_finite_coordinates_handling() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // Move to border first
    let move_border = make_move_event(2.0, 300.0);
    assert!(host.handle_window_event(&mut win, &move_border, size));

    // Non-finite position event
    let nan_move = make_move_event(f64::NAN, 300.0);
    assert!(!host.handle_window_event(&mut win, &nan_move, size));
    assert!(host.cursor_pos().is_none());
    assert!(!host.is_hovering_border());

    let inf_move = make_move_event(100.0, f64::INFINITY);
    assert!(!host.handle_window_event(&mut win, &inf_move, size));
    assert!(host.cursor_pos().is_none());

    // Coordinate that is finite in f64 but overflows f32 (e.g. 1e39)
    let overflow_move = make_move_event(1e39, 300.0);
    assert!(!host.handle_window_event(&mut win, &overflow_move, size));
    assert!(host.cursor_pos().is_none());
}

#[test]
fn test_frameless_host_handle_ui_event() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // 1. MouseMove on left border
    let move_ui = UiEvent::MouseMove {
        point: Point::new(2.0, 300.0),
    };
    assert!(host.handle_ui_event(&mut win, &move_ui, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::WResize));
    assert!(host.is_hovering_border());

    // 2. MouseDown on border -> drag_resize_window triggered
    let click_ui = UiEvent::MouseDown {
        point: Point::new(2.0, 300.0),
        button: iris_core::event::MouseButton::Left,
    };
    assert!(host.handle_ui_event(&mut win, &click_ui, size));
    assert_eq!(win.last_resize_direction, Some(ResizeDirection::Left));

    // 3. MouseMove to interior -> restored to Default
    let interior_ui = UiEvent::MouseMove {
        point: Point::new(400.0, 300.0),
    };
    assert!(!host.handle_ui_event(&mut win, &interior_ui, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));
    assert!(!host.is_hovering_border());

    // 4. Reset hover cursor explicitly
    host.set_cursor_pos(None);
    host.reset_hover_cursor(&mut win);
    assert!(host.cursor_pos().is_none());
    assert!(!host.is_hovering_border());
}

#[test]
fn test_window_interface_arc_and_blanket_mut() {
    let mut win = MockWindow::default();
    // Test &mut W blanket implementation
    let ref_win = &mut win;
    ref_win.set_widget_cursor(WidgetCursor::Crosshair);
    assert_eq!(ref_win.last_cursor, Some(CursorIcon::Crosshair));

    // Test Arc<Window> typecheck and compilation
    fn _assert_window_interface<W: WindowInterface>() {}
    _assert_window_interface::<winit::window::Window>();
    _assert_window_interface::<&winit::window::Window>();
    _assert_window_interface::<Arc<winit::window::Window>>();
    _assert_window_interface::<&mut MockWindow>();
}

#[test]
fn test_cursor_restoration_on_focus_lost() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // Move to border
    let move_border = make_move_event(2.0, 300.0);
    assert!(host.handle_window_event(&mut win, &move_border, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::WResize));
    assert!(host.is_hovering_border());
    assert!(host.cursor_pos().is_some());

    // Window loses focus -> restored to Default, cursor_pos cleared, hovering_border cleared
    let unfocus_ev = WindowEvent::Focused(false);
    assert!(!host.handle_window_event(&mut win, &unfocus_ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));
    assert!(!host.is_hovering_border());
    assert!(host.cursor_pos().is_none());

    // Window gains focus -> does not crash or falsely set cursor
    let focus_ev = WindowEvent::Focused(true);
    assert!(!host.handle_window_event(&mut win, &focus_ev, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));
}

#[test]
fn test_scale_factor_changed_resets_cursor_and_stale_coordinates() {
    let host = FramelessWindowHost::new();
    let mut win = MockWindow::default();

    // Move to border
    let size = Size::new(800.0, 600.0);
    let move_border = make_move_event(2.0, 300.0);
    assert!(host.handle_window_event(&mut win, &move_border, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::WResize));
    assert!(host.is_hovering_border());
    assert!(host.cursor_pos().is_some());

    // ScaleFactorChanged via handle_scale_factor_changed
    host.handle_scale_factor_changed(&mut win, 2.0);
    assert_eq!(host.scale_factor(), 2.0);
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));
    assert!(!host.is_hovering_border());
    assert!(host.cursor_pos().is_none());

    // Non-finite scale factor clamped to 1.0 and resets cursor
    let move_border2 = make_move_event(2.0, 300.0);
    assert!(host.handle_window_event(&mut win, &move_border2, size));
    assert_eq!(win.last_cursor, Some(CursorIcon::WResize));
    assert!(host.is_hovering_border());

    host.handle_scale_factor_changed(&mut win, f64::INFINITY);
    assert_eq!(host.scale_factor(), 1.0);
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));
    assert!(!host.is_hovering_border());
    assert!(host.cursor_pos().is_none());
}

#[test]
fn test_handle_ui_event_mouse_down_coordinates_and_finiteness() {
    let host = FramelessWindowHost::new();
    let size = Size::new(800.0, 600.0);
    let mut win = MockWindow::default();

    // 1. MouseDown on interior updates cursor_pos even if not on resize border
    let interior_down = UiEvent::MouseDown {
        point: Point::new(200.0, 150.0),
        button: iris_core::event::MouseButton::Left,
    };
    assert!(!host.handle_ui_event(&mut win, &interior_down, size));
    assert_eq!(host.cursor_pos(), Some(Point::new(200.0, 150.0)));
    assert_eq!(win.resize_call_count, 0);

    // 2. MouseDown on border updates cursor_pos and triggers resize
    let border_down = UiEvent::MouseDown {
        point: Point::new(2.0, 300.0),
        button: iris_core::event::MouseButton::Left,
    };
    assert!(host.handle_ui_event(&mut win, &border_down, size));
    assert_eq!(host.cursor_pos(), Some(Point::new(2.0, 300.0)));
    assert_eq!(win.last_resize_direction, Some(ResizeDirection::Left));
    assert_eq!(win.resize_call_count, 1);

    // 3. MouseDown with NaN coordinate clears cursor_pos, resets hover cursor, returns false
    let nan_down = UiEvent::MouseDown {
        point: Point::new(f32::NAN, 300.0),
        button: iris_core::event::MouseButton::Left,
    };
    assert!(!host.handle_ui_event(&mut win, &nan_down, size));
    assert!(host.cursor_pos().is_none());
    assert!(!host.is_hovering_border());
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));

    // 4. MouseDown with INFINITY coordinate clears cursor_pos, resets hover cursor, returns false
    // First hover border
    let move_ui = UiEvent::MouseMove {
        point: Point::new(2.0, 300.0),
    };
    assert!(host.handle_ui_event(&mut win, &move_ui, size));
    assert!(host.is_hovering_border());

    let inf_down = UiEvent::MouseDown {
        point: Point::new(2.0, f32::INFINITY),
        button: iris_core::event::MouseButton::Left,
    };
    assert!(!host.handle_ui_event(&mut win, &inf_down, size));
    assert!(host.cursor_pos().is_none());
    assert!(!host.is_hovering_border());
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));
}

#[test]
fn test_handle_ui_event_for_window_and_window_event_for_window() {
    let host = FramelessWindowHost::new();
    let mut win = MockWindow {
        logical_size: Size::new(800.0, 600.0),
        ..Default::default()
    };

    // 1. handle_ui_event_for_window delegates to handle_ui_event using win.logical_size
    let border_move = UiEvent::MouseMove {
        point: Point::new(2.0, 300.0),
    };
    assert!(host.handle_ui_event_for_window(&mut win, &border_move));
    assert_eq!(win.last_cursor, Some(CursorIcon::WResize));

    // 2. handle_window_event_for_window delegates using win.logical_size
    let interior_winit = make_move_event(400.0, 300.0);
    assert!(!host.handle_window_event_for_window(&mut win, &interior_winit));
    assert_eq!(win.last_cursor, Some(CursorIcon::Default));

    // 3. WindowInterface logical_size and scale_factor inspection
    assert_eq!(win.logical_size(), Size::new(800.0, 600.0));
    assert_eq!(win.scale_factor(), 1.0);
}