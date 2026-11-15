// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Floating Window Interaction and Event Prioritization Unit Tests
//!
//! Validates autonomous floating window titlebar dragging, 8-directional border edge resizing,
//! close/dock-back button activation, hover feedback, and event precedence over background panels.

use super::runner::Workbench;
use crate::descriptors::workbench::WorkbenchDescriptor;
use iris_core::event::{MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect, Size};
use iris_core::node::WidgetCursor;
use iris_dock::floating::FloatingDragState;
use iris_dock::{DockState, DockTree};
use iris_winit::CursorIcon;
use iris_winit::frameless::{ResizeDirection, WindowInterface, WindowOpError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestTab {
    Viewport,
    Properties,
}

#[derive(Debug, Default)]
struct MockWindow {
    cursor: Option<CursorIcon>,
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
        Ok(())
    }

    fn toggle_maximize(&mut self) {}

    fn is_maximized(&self) -> bool {
        false
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

fn create_floating_test_workbench() -> (Workbench<TestTab, (), ()>, DockState<TestTab>) {
    let mut tree = DockTree::new();
    let root = tree.create_leaf(vec![TestTab::Viewport]);
    tree.set_root(root);
    let mut dock_state = DockState::new(tree);

    // Open a floating window hosting TestTab::Properties
    let float_rect = Rect::new(200.0, 100.0, 400.0, 300.0);
    dock_state.open_or_focus_floating_tab(TestTab::Properties, "Properties", float_rect);

    let mut descriptor = WorkbenchDescriptor::new("Test Workbench", dock_state.clone());
    descriptor.menubar = Some(crate::descriptors::menu::MenuBarDescriptor::new());
    descriptor.statusbar = Some(crate::descriptors::status::StatusBarDescriptor::new());
    let wb = Workbench::new(descriptor);
    (wb, dock_state)
}

#[test]
fn test_workbench_floating_window_title_drag_priority_over_background() {
    let (mut wb, mut dock_state) = create_floating_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };

    // 1. MouseDown on floating window title bar (empty titlebar space at x=350, y=115 inside [200, 100, 400, 32])
    let down_ev = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(350.0, 115.0),
    };

    let outcome_down = wb.handle_event_with_content(&mut win, &down_ev, &mut dock_state, |_| {
        // Background content should NOT consume this event
        (false, None, vec![])
    });

    assert!(outcome_down.consumed);
    assert!(outcome_down.layout_changed);
    assert!(outcome_down.shell_dirty);
    assert!(dock_state.has_active_floating_drag());

    // 2. MouseMove by (50, 35) to (400, 150)
    let move_ev = UiEvent::MouseMove {
        point: Point::new(400.0, 150.0),
    };

    let outcome_move = wb.handle_event_with_content(&mut win, &move_ev, &mut dock_state, |_| {
        // Even if background content claims to consume, active floating drag takes precedence
        (true, None, vec![])
    });

    assert!(outcome_move.consumed);
    assert!(outcome_move.layout_changed);
    assert!(outcome_move.shell_dirty);
    assert_eq!(outcome_move.cursor, Some(WidgetCursor::Grabbing));

    // Verify window moved to (250, 135)
    let win_rect = dock_state.floating_windows[0].rect;
    assert_eq!(win_rect.x, 250.0);
    assert_eq!(win_rect.y, 135.0);

    // 3. MouseUp releases title drag
    let up_ev = UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(400.0, 150.0),
    };

    let outcome_up =
        wb.handle_event_with_content(&mut win, &up_ev, &mut dock_state, |_| (false, None, vec![]));

    assert!(outcome_up.consumed);
    assert!(outcome_up.layout_changed);
    assert!(!dock_state.has_active_floating_drag());
}

#[test]
fn test_workbench_floating_window_resize_priority() {
    let (mut wb, mut dock_state) = create_floating_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };

    // Right border edge is at x = 600.0 (200 + 400), y = 200.0
    let down_resize = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(600.0, 200.0),
    };

    let outcome_down =
        wb.handle_event_with_content(&mut win, &down_resize, &mut dock_state, |_| {
            (false, None, vec![])
        });

    assert!(outcome_down.consumed);
    assert!(outcome_down.layout_changed);
    assert!(dock_state.has_active_floating_drag());

    if let Some((_, FloatingDragState::Resize(edge))) = dock_state.active_floating_drag {
        assert_eq!(edge, iris_dock::floating::FloatingResizeEdge::Right);
    } else {
        panic!("Expected active resize drag along right edge");
    }

    // Drag right edge outwards by +60px to x=660
    let move_resize = UiEvent::MouseMove {
        point: Point::new(660.0, 200.0),
    };

    let outcome_move =
        wb.handle_event_with_content(&mut win, &move_resize, &mut dock_state, |_| {
            (false, None, vec![])
        });

    assert!(outcome_move.consumed);
    assert!(outcome_move.layout_changed);
    assert_eq!(dock_state.floating_windows[0].rect.width, 460.0);

    // Release mouse
    let up_resize = UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(660.0, 200.0),
    };

    let outcome_up = wb.handle_event_with_content(&mut win, &up_resize, &mut dock_state, |_| {
        (false, None, vec![])
    });

    assert!(outcome_up.consumed);
    assert!(!dock_state.has_active_floating_drag());
}

#[test]
fn test_workbench_floating_window_close_button() {
    let (mut wb, mut dock_state) = create_floating_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };

    // Close button is at right - 32px: x in [568..588], y in [102..124]
    let click_close = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(578.0, 112.0),
    };

    let outcome = wb.handle_event_with_content(&mut win, &click_close, &mut dock_state, |_| {
        (false, None, vec![])
    });

    assert!(outcome.consumed);
    assert!(outcome.layout_changed);
    assert!(outcome.shell_dirty);
    assert!(dock_state.floating_windows.is_empty());
}

#[test]
fn test_workbench_floating_window_dock_back_button() {
    let (mut wb, mut dock_state) = create_floating_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };

    // Dock-back button is at right - 56px: x in [544..564], y in [102..124]
    let click_dock_back = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(554.0, 112.0),
    };

    let outcome = wb.handle_event_with_content(&mut win, &click_dock_back, &mut dock_state, |_| {
        (false, None, vec![])
    });

    assert!(outcome.consumed);
    assert!(outcome.layout_changed);
    assert!(outcome.shell_dirty);
    assert!(dock_state.floating_windows.is_empty());

    // Properties tab should now be docked back into the main dock tree
    assert!(dock_state.tree.find_tab(&TestTab::Properties).is_some());
}

#[test]
fn test_workbench_floating_window_hover_cursors() {
    let (mut wb, mut dock_state) = create_floating_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };

    // Hover over right border edge (x = 600.0, y = 200.0)
    let hover_edge = UiEvent::MouseMove {
        point: Point::new(600.0, 200.0),
    };

    let outcome_edge = wb.handle_event_with_content(&mut win, &hover_edge, &mut dock_state, |_| {
        (false, None, vec![])
    });

    assert_eq!(outcome_edge.cursor, Some(WidgetCursor::ColResize));

    // Hover over titlebar (x = 250.0, y = 115.0)
    let hover_title = UiEvent::MouseMove {
        point: Point::new(250.0, 115.0),
    };

    let outcome_title =
        wb.handle_event_with_content(&mut win, &hover_title, &mut dock_state, |_| {
            (false, None, vec![])
        });

    assert_eq!(outcome_title.cursor, Some(WidgetCursor::Pointer));
}

#[test]
fn test_workbench_floating_tab_drag_promotion_to_active_drag() {
    let (mut wb, mut dock_state) = create_floating_test_workbench();
    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };

    // Click on tab pill "Properties" at x=240, y=115 inside [200, 100, 400, 32]
    let down_ev = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(240.0, 115.0),
    };

    let outcome_down = wb.handle_event_with_content(&mut win, &down_ev, &mut dock_state, |_| {
        (false, None, vec![])
    });

    assert!(outcome_down.consumed);
    assert!(
        dock_state.pending_tab_drag.is_some(),
        "pending_tab_drag must be preserved in dock_state!"
    );

    // Move past threshold (dx=10px, dy=0)
    let move_ev = UiEvent::MouseMove {
        point: Point::new(250.0, 115.0),
    };

    let outcome_move = wb.handle_event_with_content(&mut win, &move_ev, &mut dock_state, |_| {
        (false, None, vec![])
    });

    assert!(outcome_move.consumed);
    assert!(
        dock_state.active_drag.is_some(),
        "active_drag must be active after threshold!"
    );
}

#[test]
fn test_workbench_docked_tab_drag_promotion_to_active_drag() {
    let mut tree = DockTree::new();
    let root = tree.create_leaf(vec![TestTab::Viewport, TestTab::Properties]);
    tree.set_root(root);
    let mut dock_state = DockState::new(tree);

    let mut descriptor = WorkbenchDescriptor::new("Test Workbench", dock_state.clone());
    descriptor.menubar = Some(crate::descriptors::menu::MenuBarDescriptor::new());
    descriptor.statusbar = Some(crate::descriptors::status::StatusBarDescriptor::new());

    let mut wb = Workbench::<TestTab, (), ()>::new(descriptor)
        .with_menubar_height(30.0)
        .with_statusbar_height(24.0);

    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };

    // Tab 0 (Viewport) is at x=0..120, y=30..56
    // Tab 1 (Properties) is at x=120..240, y=30..56
    let down_ev = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(150.0, 42.0),
    };

    let outcome_down = wb.handle_event_with_content(&mut win, &down_ev, &mut dock_state, |_| {
        (false, None, vec![])
    });

    assert!(outcome_down.consumed, "Docked tab click must be consumed");
    assert!(
        dock_state.pending_tab_drag.is_some(),
        "pending_tab_drag must be set for docked tab!"
    );

    // Move past threshold (dx=10px, dy=10px)
    let move_ev = UiEvent::MouseMove {
        point: Point::new(160.0, 52.0),
    };

    let outcome_move = wb.handle_event_with_content(&mut win, &move_ev, &mut dock_state, |_| {
        (false, None, vec![])
    });

    assert!(outcome_move.consumed);
    assert!(
        dock_state.active_drag.is_some(),
        "active_drag must be active for docked tab!"
    );
}