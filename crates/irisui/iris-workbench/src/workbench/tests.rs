// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Autonomous Workbench Orchestrator
//!
//! Validates event delegation, hit-testing, frameless border resizing,
//! titlebar dragging, and modal keyboard event interception in isolation.

use super::runner::Workbench;
use crate::descriptors::workbench::WorkbenchDescriptor;
use iris_core::event::{KeyCode, MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect, Size};
use iris_dock::state::DockState;
use iris_dock::tree::DockTree;
use iris_winit::CursorIcon;
use iris_winit::frameless::{ResizeDirection, WindowInterface, WindowOpError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MockTab {
    Viewport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MockAction {
    DeleteConfirmed,
    FileDropped(Vec<std::path::PathBuf>),
    ZoomChanged(u32),
    OpenPreferences,
    ContentAction(String),
}

#[derive(Debug, Default)]
struct MockWindow {
    cursor: Option<CursorIcon>,
    drag_window_called: bool,
    maximized: bool,
    logical_size: Size,
}

impl WindowInterface for MockWindow {
    fn set_cursor(&mut self, icon: CursorIcon) {
        self.cursor = Some(icon);
    }

    fn drag_resize_window(&mut self, _direction: ResizeDirection) -> Result<(), WindowOpError> {
        Ok(())
    }

    fn drag_window(&mut self) -> Result<(), WindowOpError> {
        self.drag_window_called = true;
        Ok(())
    }

    fn toggle_maximize(&mut self) {
        self.maximized = !self.maximized;
    }

    fn is_maximized(&self) -> bool {
        self.maximized
    }

    fn minimize(&mut self) {}

    fn close(&mut self) {}

    fn logical_size(&self) -> Size {
        self.logical_size
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }
}

fn create_test_workbench() -> Workbench<MockTab, MockAction, ()> {
    let mut tree = DockTree::new();
    let root = tree.create_leaf(vec![MockTab::Viewport]);
    tree.set_root(root);
    let mut descriptor = WorkbenchDescriptor::new("Test Workbench", DockState::new(tree));
    descriptor.menubar = Some(crate::descriptors::menu::MenuBarDescriptor::new());
    descriptor.statusbar = Some(crate::descriptors::status::StatusBarDescriptor::new());
    Workbench::new(descriptor)
}

#[test]
fn test_workbench_workspace_rect_computation() {
    let wb = create_test_workbench();
    let rect = wb.compute_workspace_rect(1920.0, 1080.0);
    assert_eq!(rect.x, 0.0);
    assert_eq!(rect.y, 30.0); // menubar height
    assert_eq!(rect.width, 1920.0);
    assert_eq!(rect.height, 1080.0 - 30.0 - 24.0); // 1026.0
}

#[test]
fn test_workbench_wants_pointer_at() {
    let mut wb = create_test_workbench();
    let vp_rect = Rect::new(300.0, 30.0, 1300.0, 900.0);
    wb.set_interactive_viewport(Some(vp_rect));
    let dock_state = wb.descriptor.dock_state.clone();

    // 1. Inside top menubar
    assert!(wb.wants_pointer_at([100.0, 15.0], &dock_state, None));

    // 2. Inside interactive 3D viewport canvas
    assert!(!wb.wants_pointer_at([500.0, 500.0], &dock_state, None));

    // 3. Outside viewport canvas (docked panel area)
    assert!(wb.wants_pointer_at([100.0, 500.0], &dock_state, None));

    // 4. Test wants_pointer_input with tracked cursor position
    assert!(wb.wants_pointer_input(Some(Point::new(100.0, 15.0)), &dock_state, None));
    assert!(!wb.wants_pointer_input(Some(Point::new(500.0, 500.0)), &dock_state, None));

    // 5. Test compatibility default helper
    assert!(wb.wants_pointer_at_default([100.0, 15.0], Some(vp_rect)));
    assert!(!wb.wants_pointer_at_default([500.0, 500.0], Some(vp_rect)));
}

#[test]
fn test_workbench_cursor_tracking_and_statusbar_hit_testing() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };
    let dock_state = wb.descriptor.dock_state.clone();

    assert_eq!(wb.cursor_pos(), None);

    // Send a mouse move event into the workbench
    let move_event = UiEvent::MouseMove {
        point: Point::new(600.0, 400.0),
    };
    let _ = wb.handle_event(&mut win, &move_event);

    assert_eq!(wb.cursor_pos(), Some(Point::new(600.0, 400.0)));
    assert_eq!(wb.window_size(), Some(Size::new(1920.0, 1080.0)));

    // Statusbar is at bottom 24px (1056.0 to 1080.0)
    assert!(wb.wants_pointer_at([500.0, 1060.0], &dock_state, None));
}

#[test]
fn test_workbench_frameless_window_resize_interception() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };

    // Cursor near top-left window edge (2.0, 2.0)
    let move_event = UiEvent::MouseMove {
        point: Point::new(2.0, 2.0),
    };
    let outcome = wb.handle_event(&mut win, &move_event);
    assert!(outcome.consumed);
    assert_eq!(win.cursor, Some(CursorIcon::NwResize));
}

#[test]
fn test_workbench_titlebar_drag_interception() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };

    // Mouse press on the menubar title bar region (y = 15.0)
    let press_event = UiEvent::MouseDown {
        point: Point::new(500.0, 15.0),
        button: MouseButton::Left,
    };
    let outcome = wb.handle_event(&mut win, &press_event);
    assert!(outcome.consumed);
    assert!(win.drag_window_called);
}

#[test]
fn test_workbench_modal_keyboard_trap_and_action_bus() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow::default();

    // Spawn a confirmation dialog into dialog_host
    wb.dialog_host
        .open_confirm("Delete Item", "Are you sure?", MockAction::DeleteConfirmed);

    assert!(wb.dialog_host.is_active());
    assert!(wb.wants_keyboard_input());

    // Send Enter key to confirm modal
    let enter_event = UiEvent::KeyboardInput {
        key: KeyCode::Enter,
        is_pressed: true,
        text: None,
        ctrl_held: false,
        shift_held: false,
    };
    let outcome = wb.handle_event(&mut win, &enter_event);

    assert!(outcome.consumed);
    assert!(outcome.layout_changed);
    assert_eq!(outcome.actions, vec![MockAction::DeleteConfirmed]);
    assert!(!wb.dialog_host.is_active());
    assert!(!wb.wants_keyboard_input());
}

#[test]
fn test_workbench_file_dropped_interception() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow::default();

    let drop_event = UiEvent::FileDropped {
        paths: vec![std::path::PathBuf::from("/models/character.gltf")],
        point: Point::new(400.0, 300.0),
    };

    let outcome = wb.handle_event(&mut win, &drop_event);
    assert!(outcome.consumed);
    assert!(outcome.layout_changed);
    assert_eq!(
        outcome.dropped_files,
        Some((
            vec![std::path::PathBuf::from("/models/character.gltf")],
            Point::new(400.0, 300.0)
        ))
    );
}

#[test]
fn test_workbench_keyboard_zoom_and_preferences() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow::default();

    // 1. Ctrl + '=' (Zoom In)
    let zoom_in_event = UiEvent::KeyboardInput {
        key: KeyCode::Character('='),
        is_pressed: true,
        text: Some("=".into()),
        ctrl_held: true,
        shift_held: false,
    };
    let outcome = wb.handle_event(&mut win, &zoom_in_event);
    assert!(outcome.consumed);
    assert!(outcome.layout_changed);
    assert_eq!(outcome.zoom_changed, Some(1.10));
    assert_eq!(wb.scale_factor(), 1.10);

    // 2. Ctrl + '0' (Reset Zoom)
    let zoom_reset_event = UiEvent::KeyboardInput {
        key: KeyCode::Character('0'),
        is_pressed: true,
        text: Some("0".into()),
        ctrl_held: true,
        shift_held: false,
    };
    let outcome = wb.handle_event(&mut win, &zoom_reset_event);
    assert!(outcome.consumed);
    assert_eq!(outcome.zoom_changed, Some(1.0));
    assert_eq!(wb.scale_factor(), 1.0);

    // 3. Ctrl + ',' (Toggle Preferences)
    let pref_event = UiEvent::KeyboardInput {
        key: KeyCode::Character(','),
        is_pressed: true,
        text: Some(",".into()),
        ctrl_held: true,
        shift_held: false,
    };
    let outcome = wb.handle_event(&mut win, &pref_event);
    assert!(outcome.consumed);
    assert!(outcome.toggle_preferences);
}

#[test]
fn test_workbench_handle_event_full_with_ui_tree() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow::default();
    let mut dock_state = wb.descriptor.dock_state.clone();

    let mut tree = iris_core::tree::UiTree::new();
    let root = tree.create_root().expect("Root node creation succeeds");
    let btn = tree.create_node();
    tree.add_child(root, btn).expect("Append child succeeds");
    if let Some(node) = tree.get_mut(btn) {
        node.role = iris_core::node::WidgetRole::Button;
        node.computed_rect = Rect::new(100.0, 100.0, 80.0, 30.0);
        node.cursor = Some(iris_core::node::WidgetCursor::Pointer);
    }

    // Mouse click over button
    let click_event = UiEvent::MouseDown {
        point: Point::new(140.0, 115.0),
        button: MouseButton::Left,
    };
    let outcome = wb.handle_event_full(&mut win, &click_event, &mut dock_state, Some(&mut tree));
    assert!(outcome.consumed);
}

#[test]
fn test_workbench_wants_pointer_at_full_layered_tree() {
    let wb = create_test_workbench();
    let dock_state = wb.descriptor.dock_state.clone();
    let vp_rect = Rect::new(200.0, 30.0, 1000.0, 800.0);

    let mut tree = iris_core::tree::UiTree::new();
    let root = tree.create_root().expect("Root node creation succeeds");
    let popup = tree.create_node();
    tree.add_child(root, popup).expect("Append child succeeds");
    if let Some(node) = tree.get_mut(popup) {
        node.role = iris_core::node::WidgetRole::DropdownPopup;
        node.layer = iris_core::node::UiLayer::Popup;
        node.computed_rect = Rect::new(300.0, 100.0, 150.0, 200.0);
    }

    // Point over popup that is positioned directly within the 3D viewport canvas
    assert!(wb.wants_pointer_at_full([350.0, 150.0], Some(vp_rect), &dock_state, Some(&tree)));

    // Point in viewport but outside popup
    assert!(!wb.wants_pointer_at_full([600.0, 400.0], Some(vp_rect), &dock_state, Some(&tree)));
}

#[test]
fn test_workbench_handle_shell_event_respects_content_consumed() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };
    let mut dock_state = wb.descriptor.dock_state.clone();

    // MouseDown at y = 15.0 (inside menubar region)
    let press_event = UiEvent::MouseDown {
        point: Point::new(500.0, 15.0),
        button: MouseButton::Left,
    };

    // 1. If content_consumed is true (e.g. user clicked a menu button), shell skips window drag
    let outcome1 = wb.handle_shell_event(&mut win, &press_event, &mut dock_state, true);
    assert!(outcome1.consumed);
    assert!(!win.drag_window_called);

    // 2. If content_consumed is false (empty menubar area), shell initiates native window drag
    let outcome2 = wb.handle_shell_event(&mut win, &press_event, &mut dock_state, false);
    assert!(outcome2.consumed);
    assert!(win.drag_window_called);
}

#[test]
fn test_workbench_handle_event_with_content_and_action_hooks() {
    let wb = create_test_workbench()
        .with_file_drop_action(|paths, _| MockAction::FileDropped(paths))
        .with_zoom_action(|z| MockAction::ZoomChanged((z * 100.0) as u32))
        .with_preferences_action(|| MockAction::OpenPreferences);

    let mut wb = wb;
    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };
    let mut dock_state = wb.descriptor.dock_state.clone();

    // 1. Content consumes event and produces action
    let click_ev = UiEvent::MouseDown {
        point: Point::new(200.0, 200.0),
        button: MouseButton::Left,
    };
    let outcome = wb.handle_event_with_content(&mut win, &click_ev, &mut dock_state, |_| {
        (
            true,
            Some(iris_core::node::WidgetCursor::Pointer),
            vec![MockAction::ContentAction("Clicked".into())],
        )
    });
    assert!(outcome.consumed);
    assert_eq!(outcome.cursor, Some(iris_core::node::WidgetCursor::Pointer));
    assert_eq!(
        outcome.actions,
        vec![MockAction::ContentAction("Clicked".into())]
    );

    // 2. File drop triggers configured file_drop_action
    let drop_ev = UiEvent::FileDropped {
        paths: vec![std::path::PathBuf::from("/test/asset.png")],
        point: Point::new(100.0, 100.0),
    };
    let outcome_drop = wb.handle_event_with_content(&mut win, &drop_ev, &mut dock_state, |_| {
        (false, None, vec![])
    });
    assert!(outcome_drop.consumed);
    assert_eq!(
        outcome_drop.actions,
        vec![MockAction::FileDropped(vec![std::path::PathBuf::from(
            "/test/asset.png"
        )])]
    );

    // 3. Zoom shortcut triggers configured zoom_action
    let zoom_ev = UiEvent::KeyboardInput {
        key: KeyCode::Character('+'),
        is_pressed: true,
        text: Some("+".into()),
        ctrl_held: true,
        shift_held: false,
    };
    let outcome_zoom = wb.handle_event_with_content(&mut win, &zoom_ev, &mut dock_state, |_| {
        (false, None, vec![])
    });
    assert!(outcome_zoom.consumed);
    assert_eq!(outcome_zoom.actions, vec![MockAction::ZoomChanged(110)]);

    // 4. Preferences shortcut triggers configured preferences_action
    let pref_ev = UiEvent::KeyboardInput {
        key: KeyCode::Character(','),
        is_pressed: true,
        text: Some(",".into()),
        ctrl_held: true,
        shift_held: false,
    };
    let outcome_pref = wb.handle_event_with_content(&mut win, &pref_ev, &mut dock_state, |_| {
        (false, None, vec![])
    });
    assert!(outcome_pref.consumed);
    assert_eq!(outcome_pref.actions, vec![MockAction::OpenPreferences]);
}