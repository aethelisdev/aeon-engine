// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Autonomous Workbench Orchestrator
//!
//! Validates event delegation, hit-testing, frameless border resizing,
//! titlebar dragging, and modal keyboard event interception in isolation.

use super::runner::Workbench;
use crate::descriptors::workbench::WorkbenchDescriptor;
use irisui_core::event::{KeyCode, MouseButton, UiEvent};
use irisui_core::geometry::{Point, Rect, Size};
use irisui_dock::state::DockState;
use irisui_dock::tree::DockTree;
use irisui_winit::CursorIcon;
use irisui_winit::frameless::{ResizeDirection, WindowInterface, WindowOpError};

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
    assert!(wb.wants_pointer_at([100.0, 15.0], &dock_state));

    // 2. Inside interactive 3D viewport canvas
    assert!(!wb.wants_pointer_at([500.0, 500.0], &dock_state));

    // 3. Outside viewport canvas (docked panel area)
    assert!(wb.wants_pointer_at([100.0, 500.0], &dock_state));

    // 4. Test wants_pointer_input with tracked cursor position
    assert!(wb.wants_pointer_input(Some(Point::new(100.0, 15.0)), &dock_state));
    assert!(!wb.wants_pointer_input(Some(Point::new(500.0, 500.0)), &dock_state));

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
    assert!(wb.wants_pointer_at([500.0, 1060.0], &dock_state));
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

    let root = wb.tree.create_root().expect("Root node creation succeeds");
    let btn = wb.tree.create_node();
    wb.tree.add_child(root, btn).expect("Append child succeeds");
    if let Some(node) = wb.tree.get_mut(btn) {
        node.role = irisui_core::node::WidgetRole::Button;
        node.computed_rect = Rect::new(100.0, 100.0, 80.0, 30.0);
        node.cursor = Some(irisui_core::node::WidgetCursor::Pointer);
    }

    // Mouse click over button
    let click_event = UiEvent::MouseDown {
        point: Point::new(140.0, 115.0),
        button: MouseButton::Left,
    };
    let outcome = wb.handle_event_full(&mut win, &click_event, &mut dock_state);
    assert!(outcome.consumed);
}

#[test]
fn test_workbench_wants_pointer_at_full_layered_tree() {
    let mut wb = create_test_workbench();
    let dock_state = wb.descriptor.dock_state.clone();
    let vp_rect = Rect::new(200.0, 30.0, 1000.0, 800.0);

    let root = wb.tree.create_root().expect("Root node creation succeeds");
    let popup = wb.tree.create_node();
    wb.tree
        .add_child(root, popup)
        .expect("Append child succeeds");
    if let Some(node) = wb.tree.get_mut(popup) {
        node.role = irisui_core::node::WidgetRole::DropdownPopup;
        node.layer = irisui_core::node::UiLayer::Popup;
        node.computed_rect = Rect::new(300.0, 100.0, 150.0, 200.0);
    }

    // Point over popup that is positioned directly within the 3D viewport canvas
    assert!(wb.wants_pointer_at_full([350.0, 150.0], Some(vp_rect), &dock_state));

    // Point in viewport but outside popup
    assert!(!wb.wants_pointer_at_full([600.0, 400.0], Some(vp_rect), &dock_state));
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
            Some(irisui_core::node::WidgetCursor::Pointer),
            vec![MockAction::ContentAction("Clicked".into())],
        )
    });
    assert!(outcome.consumed);
    assert_eq!(
        outcome.cursor,
        Some(irisui_core::node::WidgetCursor::Pointer)
    );
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

struct MockViewportPanel;

impl crate::descriptors::panel::WorkbenchPanel<MockTab, ()> for MockViewportPanel {
    fn id(&self) -> MockTab {
        MockTab::Viewport
    }

    fn title(&self) -> &str {
        "Viewport"
    }

    fn render(&mut self, scope: &mut irisui_widgets::declarative::UiScope<'_>, _context: &mut ()) {
        scope.button("3D Viewport Render Target");
    }
}

#[test]
fn test_workbench_rebuild_single_panel_leaf_and_registered() {
    let mut wb = create_test_workbench();
    wb.panel_registry.register(MockViewportPanel);

    let leaf_id = wb.tree.create_node();
    {
        let mut scope = irisui_widgets::declarative::UiScope::new(&mut wb.tree, leaf_id);
        scope.configure_container(
            "ViewportLeaf",
            irisui_core::style::Style::new().width(800.0).height(600.0),
            false,
        );
    }
    if let Some(node) = wb.tree.get_mut(leaf_id) {
        node.computed_rect = irisui_core::geometry::Rect::new(0.0, 0.0, 800.0, 600.0);
    }

    // 1. Direct leaf rebuilding with custom closure
    let rebuilt_leaf = wb
        .rebuild_single_panel_leaf(leaf_id, |scope| {
            scope.button("Dynamic Direct Button");
        })
        .expect("rebuild single panel leaf must succeed");
    assert!(rebuilt_leaf);
    assert!(wb.tree.is_boundary(leaf_id));
    let leaf_node = wb.tree.get(leaf_id).expect("node exists");
    assert_eq!(leaf_node.children.len(), 1);

    // 2. Rebuild via leaf resolver with panel registry
    wb.notifier.clear_all();
    wb.notifier.tag_redraw(MockTab::Viewport);
    let rebuilt_panel = wb
        .rebuild_single_panel(&mut (), MockTab::Viewport, |_| Some(leaf_id))
        .expect("rebuild single panel must succeed");
    assert!(rebuilt_panel);
    assert!(!wb.notifier.is_dirty(MockTab::Viewport));
    let child = wb
        .tree
        .get(wb.tree.get(leaf_id).unwrap().children[0])
        .expect("child exists");
    assert_eq!(child.text.as_deref(), Some("3D Viewport Render Target"));
}

#[test]
fn test_pointer_state_event_tracking() {
    let mut state = crate::workbench::PointerState::new();
    assert_eq!(state.position(), None);
    assert!(!state.is_primary_down());
    assert!(!state.is_secondary_down());
    assert!(!state.is_middle_down());
    assert!(!state.is_any_down());

    // 1. MouseMove updates position
    let move_ev = UiEvent::MouseMove {
        point: Point::new(150.0, 250.0),
    };
    state.update_from_event(&move_ev);
    assert_eq!(state.position(), Some(Point::new(150.0, 250.0)));
    assert!(!state.is_any_down());

    // 2. MouseDown (Left) updates position and primary_down
    let down_left = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(160.0, 260.0),
    };
    state.update_from_event(&down_left);
    assert_eq!(state.position(), Some(Point::new(160.0, 260.0)));
    assert!(state.is_primary_down());
    assert!(state.is_any_down());

    // 3. MouseDown (Right) updates secondary_down
    let down_right = UiEvent::MouseDown {
        button: MouseButton::Right,
        point: Point::new(170.0, 270.0),
    };
    state.update_from_event(&down_right);
    assert!(state.is_secondary_down());

    // 4. MouseUp (Left) clears primary_down but keeps secondary_down
    let up_left = UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(175.0, 275.0),
    };
    state.update_from_event(&up_left);
    assert!(!state.is_primary_down());
    assert!(state.is_secondary_down());
    assert!(state.is_any_down());

    // 5. Release all buttons
    state.release_all_buttons();
    assert!(!state.is_any_down());
    assert_eq!(state.position(), Some(Point::new(175.0, 275.0)));

    // 6. MouseScroll and FileHovered update position
    let scroll_ev = UiEvent::MouseScroll {
        delta_x: 0.0,
        delta_y: 1.0,
        point: Point::new(200.0, 300.0),
    };
    state.update_from_event(&scroll_ev);
    assert_eq!(state.position(), Some(Point::new(200.0, 300.0)));

    let file_hover = UiEvent::FileHovered {
        path: std::path::PathBuf::from("/test.txt"),
        point: Point::new(220.0, 320.0),
    };
    state.update_from_event(&file_hover);
    assert_eq!(state.position(), Some(Point::new(220.0, 320.0)));
}

#[test]
fn test_workbench_outcome_cursor_pos_export() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };
    let mut dock_state = wb.descriptor.dock_state.clone();

    // 1. Initial state has no cursor_pos
    assert_eq!(wb.cursor_pos(), None);
    assert_eq!(wb.pointer_state().position(), None);

    // 2. MouseMove updates workbench cursor_pos and outcome.cursor_pos
    let move_ev = UiEvent::MouseMove {
        point: Point::new(450.0, 350.0),
    };
    let outcome = wb.handle_event_with_dock(&mut win, &move_ev, &mut dock_state);
    assert_eq!(outcome.cursor_pos, Some(Point::new(450.0, 350.0)));
    assert_eq!(wb.cursor_pos(), Some(Point::new(450.0, 350.0)));
    assert_eq!(
        wb.pointer_state().position(),
        Some(Point::new(450.0, 350.0))
    );

    // 3. Non-pointer event (e.g. keyboard) preserves last known cursor_pos in outcome
    let key_ev = UiEvent::KeyboardInput {
        key: KeyCode::Character('a'),
        text: Some("a".into()),
        is_pressed: true,
        ctrl_held: false,
        shift_held: false,
    };
    let outcome_key = wb.handle_event_with_dock(&mut win, &key_ev, &mut dock_state);
    assert_eq!(outcome_key.cursor_pos, Some(Point::new(450.0, 350.0)));

    // 4. Content-consumed event in handle_event_with_content retains cursor_pos
    let click_ev = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(500.0, 400.0),
    };
    let outcome_content =
        wb.handle_event_with_content(&mut win, &click_ev, &mut dock_state, |_| {
            (true, Some(irisui_core::node::WidgetCursor::Pointer), vec![])
        });
    assert!(outcome_content.consumed);
    assert_eq!(
        outcome_content.cursor,
        Some(irisui_core::node::WidgetCursor::Pointer)
    );
    assert_eq!(outcome_content.cursor_pos, Some(Point::new(500.0, 400.0)));
    assert_eq!(wb.cursor_pos(), Some(Point::new(500.0, 400.0)));
    assert!(wb.pointer_state().is_primary_down());

    // 5. MouseUp clears button state while preserving coordinate
    let up_ev = UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(505.0, 405.0),
    };
    let outcome_up = wb.handle_event_with_dock(&mut win, &up_ev, &mut dock_state);
    assert_eq!(outcome_up.cursor_pos, Some(Point::new(505.0, 405.0)));
    assert!(!wb.pointer_state().is_primary_down());
}

#[test]
fn test_workbench_sync_caret_blink_updates_visibility() {
    let mut wb = create_test_workbench();
    let root = wb.tree.create_root().expect("root node");
    wb.text_system.set_caret_blink_interval_ms(120);

    let input_id = wb.tree.create_node();
    if let Some(node) = wb.tree.get_mut(input_id) {
        node.role = irisui_core::node::WidgetRole::TextInput;
        node.tag = 888;
    }
    wb.tree.add_child(root, input_id).expect("child");

    // 1. No focus -> caret not active/visible
    let u0 = wb.sync_caret_blink_with_delta(std::time::Duration::from_millis(16));
    assert!(!u0.has_text_focus);
    assert!(!wb.is_caret_visible());

    // 2. Focus -> caret visible and focused node marked PAINT dirty
    wb.tree.event_state.focused = Some(input_id);
    wb.tree.event_state.focused_tag = Some(888);
    if let Some(node) = wb.tree.get_mut(input_id) {
        node.clear_dirty(irisui_core::DirtyFlags::ALL);
    }
    let u1 = wb.sync_caret_blink_with_delta(std::time::Duration::from_millis(16));
    assert!(u1.has_text_focus);
    assert!(wb.is_caret_visible());
    assert_eq!(wb.caret_blink_state(), u1);
    assert!(
        wb.tree
            .get(input_id)
            .expect("node")
            .dirty
            .contains(irisui_core::DirtyFlags::PAINT)
    );

    // 3. Advance past interval -> caret toggles hidden and marks PAINT dirty again
    if let Some(node) = wb.tree.get_mut(input_id) {
        node.clear_dirty(irisui_core::DirtyFlags::ALL);
    }
    let u2 = wb.sync_caret_blink_with_delta(std::time::Duration::from_millis(130));
    assert!(u2.blink_changed);
    assert!(!wb.is_caret_visible());
    assert!(
        wb.tree
            .get(input_id)
            .expect("node")
            .dirty
            .contains(irisui_core::DirtyFlags::PAINT)
    );
}

#[test]
fn test_workbench_evaluate_dock_lifecycle_tracks_dock_notifier_and_host_flags() {
    let mut wb = create_test_workbench();
    wb.set_screen_size((1280.0, 720.0));
    let dock = DockState::new(DockTree::new());

    // 1. Cold start -> FullShellRebuild, then Dormant on steady state
    wb.notifier.clear_all();
    assert_eq!(
        wb.evaluate_dock_lifecycle(&dock, 1.0, |i| i),
        crate::lifecycle::LifecycleBranch::FullShellRebuild
    );
    assert_eq!(
        wb.evaluate_dock_lifecycle(&dock, 1.0, |i| i),
        crate::lifecycle::LifecycleBranch::Dormant
    );

    // 2. Targeted panel redraw via notifier -> GranularPanels
    wb.notifier.tag_redraw(MockTab::Viewport);
    assert_eq!(
        wb.evaluate_dock_lifecycle(&dock, 1.0, |i| i),
        crate::lifecycle::LifecycleBranch::GranularPanels(vec![MockTab::Viewport])
    );
    wb.notifier.clear_panel(MockTab::Viewport);

    // 3. Host flag change (e.g. popup active) -> FullShellRebuild
    assert_eq!(
        wb.evaluate_dock_lifecycle(&dock, 1.0, |i| i.with_popup_active(true)),
        crate::lifecycle::LifecycleBranch::FullShellRebuild
    );

    // 4. Workbench tree menu/modal flags are ingested automatically without host overrides
    wb.tree.active_menu_bar_tag = Some(0x1001);
    assert_eq!(
        wb.evaluate_dock_lifecycle(&dock, 1.0, |i| i),
        crate::lifecycle::LifecycleBranch::FullShellRebuild
    );
    wb.tree.active_menu_bar_tag = None;
    assert_eq!(
        wb.evaluate_dock_lifecycle(&dock, 1.0, |i| i),
        crate::lifecycle::LifecycleBranch::FullShellRebuild
    );
    assert_eq!(
        wb.evaluate_dock_lifecycle(&dock, 1.0, |i| i),
        crate::lifecycle::LifecycleBranch::Dormant
    );
}

#[test]
fn test_workbench_sync_selection_and_domain_revision_dirty_domain_panels() {
    let mut wb = create_test_workbench();
    wb.panel_registry
        .register_domain_panel(MockTab::Viewport, |_scope, _ctx| {});

    // 1. Same selection (None) -> no-op
    assert!(!wb.sync_selection_with_notifier(None));
    assert!(!wb.lifecycle.is_panel_dirty());
    assert!(wb.notifier.dirty_panels().is_empty());

    // 2. Changing selection -> marks registered domain panel dirty in lifecycle and notifier
    assert!(wb.sync_selection_with_notifier(Some(42)));
    assert_eq!(wb.last_selection_id(), Some(42));
    assert!(wb.lifecycle.is_tab_dirty(&MockTab::Viewport));
    assert!(wb.notifier.dirty_panels().contains(&MockTab::Viewport));

    // 3. Idempotent on repeated selection
    wb.lifecycle.clear_all_dirty_tabs();
    wb.notifier.clear_all();
    assert!(!wb.sync_selection(Some(42)));
    assert!(!wb.lifecycle.is_panel_dirty());

    // 4. Same domain revision (0) -> no-op; bumped revision -> marks domain panel dirty
    assert!(!wb.sync_domain_revision_with_notifier(0));
    assert!(wb.sync_domain_revision_with_notifier(7));
    assert_eq!(wb.last_domain_revision(), 7);
    assert!(wb.lifecycle.is_tab_dirty(&MockTab::Viewport));
    assert!(wb.notifier.dirty_panels().contains(&MockTab::Viewport));
}

#[test]
fn test_workbench_sync_frame_state_unifies_blink_selection_domain_and_lifecycle() {
    let mut wb = create_test_workbench();
    wb.set_screen_size((1280.0, 720.0));
    wb.panel_registry
        .register_domain_panel(MockTab::Viewport, |_scope, _ctx| {});

    let _root = wb.tree.create_root().expect("root");
    let dock = DockState::new(DockTree::new());

    // 1. Cold start -> FullShellRebuild and tag_all
    let s0 = wb.sync_frame_state(
        super::shell::WorkbenchFrameSyncParams {
            dock: &dock,
            scale_factor: 1.0,
            selection_id: None,
            domain_revision: 0,
        },
        |i| i,
    );
    assert_eq!(
        s0.lifecycle_branch,
        crate::lifecycle::LifecycleBranch::FullShellRebuild
    );
    assert!(wb.notifier.is_global_dirty());

    // 2. Steady frame -> Dormant
    wb.notifier.clear_all();
    let s1 = wb.sync_frame_state(
        super::shell::WorkbenchFrameSyncParams {
            dock: &dock,
            scale_factor: 1.0,
            selection_id: None,
            domain_revision: 0,
        },
        |i| i,
    );
    assert_eq!(
        s1.lifecycle_branch,
        crate::lifecycle::LifecycleBranch::Dormant
    );
    assert!(!s1.selection_changed);
    assert!(!s1.domain_revision_changed);

    // 3. Selection change -> GranularPanels and notifier dirty
    let s2 = wb.sync_frame_state(
        super::shell::WorkbenchFrameSyncParams {
            dock: &dock,
            scale_factor: 1.0,
            selection_id: Some(99),
            domain_revision: 0,
        },
        |i| i,
    );
    assert!(s2.selection_changed);
    assert_eq!(
        s2.lifecycle_branch,
        crate::lifecycle::LifecycleBranch::GranularPanels(vec![MockTab::Viewport])
    );
    assert!(wb.notifier.dirty_panels().contains(&MockTab::Viewport));
}

#[test]
fn test_workbench_sync_window_cursor_deduplicates_os_ipc() {
    let mut wb = create_test_workbench();
    let mut win = MockWindow::default();
    let dock = wb.descriptor.dock_state.clone();
    let root = wb.tree.create_root().expect("root");
    let btn = wb.tree.create_node();
    wb.tree.add_child(root, btn).expect("child");
    if let Some(node) = wb.tree.get_mut(btn) {
        node.role = irisui_core::node::WidgetRole::Button;
        node.computed_rect = Rect::new(40.0, 40.0, 80.0, 30.0);
        node.cursor = Some(irisui_core::node::WidgetCursor::Pointer);
    }

    // 1. Cursor over button resolves to Pointer and updates MockWindow once
    wb.pointer_state.set_position(Some(Point::new(60.0, 50.0)));
    wb.last_cursor_pos = Some(Point::new(60.0, 50.0));
    assert_eq!(
        wb.resolve_window_cursor(&dock),
        Some(irisui_core::node::WidgetCursor::Pointer)
    );
    wb.sync_window_cursor(&mut win, &dock);
    assert_eq!(win.cursor, Some(CursorIcon::Pointer));

    // 2. Clear MockWindow cursor manually and re-sync same state -> deduplicated (no OS call)
    win.cursor = None;
    wb.sync_window_cursor(&mut win, &dock);
    assert_eq!(win.cursor, None);

    // 3. Cursor moves to unoccluded non-button space over popup overlay -> resolves Default
    let popup = wb.tree.create_node();
    wb.tree.add_child(root, popup).expect("popup");
    if let Some(node) = wb.tree.get_mut(popup) {
        node.layer = irisui_core::node::UiLayer::Popup;
        node.computed_rect = Rect::new(200.0, 100.0, 160.0, 120.0);
    }
    wb.pointer_state
        .set_position(Some(Point::new(250.0, 150.0)));
    wb.last_cursor_pos = Some(Point::new(250.0, 150.0));
    assert_eq!(
        wb.resolve_window_cursor(&dock),
        Some(irisui_core::node::WidgetCursor::Default)
    );
    wb.sync_window_cursor(&mut win, &dock);
    assert_eq!(win.cursor, Some(CursorIcon::Default));

    // 4. Moving into unoccluded neutral space returns None without forcing an IPC override
    wb.pointer_state
        .set_position(Some(Point::new(600.0, 400.0)));
    wb.last_cursor_pos = Some(Point::new(600.0, 400.0));
    assert_eq!(wb.resolve_window_cursor(&dock), None);
}