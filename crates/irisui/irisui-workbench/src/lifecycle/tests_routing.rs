// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::routing::{
    EventInvalidationRoute, route_event_invalidation, route_event_invalidation_with_fallback,
};
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::workbench::Workbench;
use irisui_core::dispatch::DispatchResult;
use irisui_core::event::UiEvent;
use irisui_core::geometry::{Point, Size};
use irisui_core::node::WidgetRole;
use irisui_core::tree::UiTree;
use irisui_dock::UiNotifier;
use irisui_dock::chrome::DockChromeFrame;
use irisui_dock::state::DockState;
use irisui_dock::tree::DockTree;
use irisui_winit::CursorIcon;
use irisui_winit::frameless::{ResizeDirection, WindowInterface, WindowOpError};

#[derive(Debug, Default)]
struct MockWindow {
    cursor: Option<CursorIcon>,
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
        if self.logical_size.width > 0.0 {
            self.logical_size
        } else {
            Size::new(1024.0, 768.0)
        }
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestTab {
    Hierarchy,
    Inspector,
    Console,
}

fn create_test_tree() -> (
    UiTree,
    irisui_core::WidgetId,
    irisui_core::WidgetId,
    irisui_core::WidgetId,
) {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation succeeds");

    // Leaf container with invalidation boundary
    let boundary_id = tree.create_node();
    tree.set_boundary(boundary_id, true);
    let _ = tree.add_child(root, boundary_id);

    // Child widget inside boundary
    let child_id = tree.create_node();
    if let Some(node) = tree.get_mut(child_id) {
        node.tag = 0x1001;
    }
    let _ = tree.add_child(boundary_id, child_id);

    // Unbounded widget outside leaf (e.g. MenuBar item)
    let shell_id = tree.create_node();
    if let Some(node) = tree.get_mut(shell_id) {
        node.tag = 0x9001;
    }
    let _ = tree.add_child(root, shell_id);

    (tree, boundary_id, child_id, shell_id)
}

#[test]
fn test_route_event_invalidation_direct_boundary() {
    let (tree, boundary_id, child_id, _) = create_test_tree();
    let mut dock_frame = DockChromeFrame::default();
    dock_frame
        .leaf_containers
        .push((TestTab::Inspector, boundary_id));

    let mut dispatch = DispatchResult::ignored();
    dispatch.target = Some(child_id);
    let event = UiEvent::MouseDown {
        button: irisui_core::event::MouseButton::Left,
        point: Point::new(100.0, 100.0),
    };

    let route = route_event_invalidation(&tree, Some(&dock_frame), &event, &dispatch);
    assert_eq!(route, EventInvalidationRoute::Panel(TestTab::Inspector));
    assert!(route.is_panel());
    assert_eq!(route.panel(), Some(&TestTab::Inspector));
}

#[test]
fn test_route_event_invalidation_semantic_tag_fallback() {
    let (tree, boundary_id, _, _) = create_test_tree();
    let mut dock_frame = DockChromeFrame::default();
    dock_frame
        .leaf_containers
        .push((TestTab::Inspector, boundary_id));

    let mut dispatch = DispatchResult::ignored();
    dispatch.target = None;
    dispatch.target_tag = 0x1001; // Matches child_node tag inside boundary
    let event = UiEvent::MouseDown {
        button: irisui_core::event::MouseButton::Left,
        point: Point::new(100.0, 100.0),
    };

    let route = route_event_invalidation(&tree, Some(&dock_frame), &event, &dispatch);
    assert_eq!(route, EventInvalidationRoute::Panel(TestTab::Inspector));
}

#[test]
fn test_route_event_invalidation_focused_tag_fallback() {
    let (mut tree, boundary_id, child_id, _) = create_test_tree();
    let mut dock_frame = DockChromeFrame::default();
    dock_frame
        .leaf_containers
        .push((TestTab::Inspector, boundary_id));

    // Simulate active text input focused on child_id
    if let Some(node) = tree.get_mut(child_id) {
        node.role = WidgetRole::TextInput;
    }
    tree.event_state.focused = Some(child_id);
    tree.event_state.focused_tag = Some(0x1001);

    let mut dispatch = DispatchResult::ignored();
    dispatch.target = None;
    dispatch.target_tag = 0;
    let event = UiEvent::KeyboardInput {
        key: irisui_core::event::KeyCode::Character('a'),
        text: Some("a".to_string()),
        is_pressed: true,
        ctrl_held: false,
        shift_held: false,
    };

    let route = route_event_invalidation(&tree, Some(&dock_frame), &event, &dispatch);
    assert_eq!(route, EventInvalidationRoute::Panel(TestTab::Inspector));
}

#[test]
fn test_route_event_invalidation_shell_click_outside_leaf() {
    let (tree, boundary_id, _, shell_id) = create_test_tree();
    let mut dock_frame = DockChromeFrame::default();
    dock_frame
        .leaf_containers
        .push((TestTab::Inspector, boundary_id));

    let mut dispatch = DispatchResult::ignored();
    dispatch.target = Some(shell_id);
    let event = UiEvent::MouseDown {
        button: irisui_core::event::MouseButton::Left,
        point: Point::new(10.0, 10.0),
    };

    let route = route_event_invalidation(&tree, Some(&dock_frame), &event, &dispatch);
    assert_eq!(route, EventInvalidationRoute::Shell);
    assert!(route.is_shell());
}

#[test]
fn test_route_event_invalidation_continuous_motion_is_dormant() {
    let (tree, boundary_id, _, shell_id) = create_test_tree();
    let mut dock_frame = DockChromeFrame::default();
    dock_frame
        .leaf_containers
        .push((TestTab::Inspector, boundary_id));

    let mut dispatch = DispatchResult::ignored();
    dispatch.target = Some(shell_id);
    let event = UiEvent::MouseMove {
        point: Point::new(12.0, 12.0),
    };

    let route = route_event_invalidation(&tree, Some(&dock_frame), &event, &dispatch);
    assert_eq!(route, EventInvalidationRoute::None);
    assert!(route.is_none());
}

#[test]
fn test_route_event_invalidation_pointer_drag_motion_escalates_to_shell() {
    let (mut tree, boundary_id, _, shell_id) = create_test_tree();
    let mut dock_frame = DockChromeFrame::default();
    dock_frame
        .leaf_containers
        .push((TestTab::Inspector, boundary_id));

    // Active pointer dragging in progress (e.g. moving floating window or splitter)
    tree.event_state.is_pointer_drag_active = true;

    let mut dispatch = DispatchResult::ignored();
    dispatch.target = Some(shell_id);
    let event = UiEvent::MouseMove {
        point: Point::new(15.0, 15.0),
    };

    let route = route_event_invalidation(&tree, Some(&dock_frame), &event, &dispatch);
    assert_eq!(route, EventInvalidationRoute::Shell);
}

#[test]
fn test_route_event_invalidation_with_fallback_closure() {
    let (tree, boundary_id, _, shell_id) = create_test_tree();
    let mut dock_frame = DockChromeFrame::default();
    dock_frame
        .leaf_containers
        .push((TestTab::Inspector, boundary_id));

    let mut dispatch = DispatchResult::ignored();
    dispatch.target = Some(shell_id);
    dispatch.target_tag = 0x7777;

    let event = UiEvent::MouseDown {
        button: irisui_core::event::MouseButton::Left,
        point: Point::new(50.0, 50.0),
    };

    let route = route_event_invalidation_with_fallback(
        &tree,
        Some(&dock_frame),
        &event,
        &dispatch,
        |disp, _| {
            if disp.target_tag == 0x7777 {
                Some(TestTab::Console)
            } else {
                None
            }
        },
    );

    assert_eq!(route, EventInvalidationRoute::Panel(TestTab::Console));
}

#[test]
fn test_workbench_route_event_invalidation_lifecycle_and_notifier() {
    let (tree, boundary_id, child_id, _) = create_test_tree();
    let mut dock_frame = DockChromeFrame::default();
    dock_frame
        .leaf_containers
        .push((TestTab::Hierarchy, boundary_id));

    let descriptor = WorkbenchDescriptor::new("Test Workbench", DockState::new(DockTree::new()));
    let mut workbench: Workbench<TestTab, (), ()> = Workbench::new(descriptor);
    workbench.tree = tree;
    workbench.dock_frame = Some(dock_frame);

    let mut dispatch = DispatchResult::ignored();
    dispatch.target = Some(child_id);
    let event = UiEvent::MouseDown {
        button: irisui_core::event::MouseButton::Left,
        point: Point::new(100.0, 100.0),
    };

    workbench.notifier.clear_all();

    let route = workbench.route_event_invalidation_with_notifier(&event, &dispatch);

    assert_eq!(route, EventInvalidationRoute::Panel(TestTab::Hierarchy));
    assert!(workbench.lifecycle.is_tab_dirty(&TestTab::Hierarchy));
    assert!(workbench.notifier.is_dirty(TestTab::Hierarchy));
}

#[test]
fn test_workbench_handle_event_full_routes_invalidation_outcome_to_panel() {
    let (mut tree, boundary_id, child_id, _) = create_test_tree();
    if let Some(node) = tree.get_mut(child_id) {
        node.computed_rect = irisui_core::geometry::Rect::new(50.0, 50.0, 100.0, 50.0);
    }
    let mut dock_frame = DockChromeFrame::default();
    dock_frame
        .leaf_containers
        .push((TestTab::Hierarchy, boundary_id));

    let descriptor = WorkbenchDescriptor::new("Test Workbench", DockState::new(DockTree::new()));
    let mut workbench: Workbench<TestTab, (), ()> = Workbench::new(descriptor);
    workbench.tree = tree;
    workbench.dock_frame = Some(dock_frame);

    let mut win = MockWindow::default();
    let mut dock_state = workbench.descriptor.dock_state.clone();
    let event = UiEvent::MouseDown {
        button: irisui_core::event::MouseButton::Left,
        point: Point::new(60.0, 60.0),
    };

    let outcome = workbench.handle_event_full(&mut win, &event, &mut dock_state);

    assert_eq!(outcome.invalidated_panel(), Some(&TestTab::Hierarchy));
    assert!(!outcome.is_shell_dirty());
    assert_eq!(
        outcome.invalidation_route(),
        Some(&EventInvalidationRoute::Panel(TestTab::Hierarchy))
    );

    let mut notifier = UiNotifier::new();
    notifier.clear_all();
    outcome.apply_to_notifier(&mut notifier);
    assert!(notifier.is_dirty(TestTab::Hierarchy));
    assert!(!notifier.is_shell_dirty());
}

#[test]
fn test_workbench_handle_event_full_routes_shell_invalidation() {
    let (mut tree, _, _, shell_id) = create_test_tree();
    if let Some(node) = tree.get_mut(shell_id) {
        node.computed_rect = irisui_core::geometry::Rect::new(200.0, 10.0, 100.0, 30.0);
    }

    let descriptor = WorkbenchDescriptor::new("Test Workbench", DockState::new(DockTree::new()));
    let mut workbench: Workbench<TestTab, (), ()> = Workbench::new(descriptor);
    workbench.tree = tree;
    let mut win = MockWindow::default();
    let mut dock_state = workbench.descriptor.dock_state.clone();

    let event = UiEvent::MouseDown {
        button: irisui_core::event::MouseButton::Left,
        point: Point::new(210.0, 20.0),
    };

    let outcome = workbench.handle_event_full(&mut win, &event, &mut dock_state);

    assert_eq!(outcome.invalidated_panel(), None);
    assert!(outcome.is_shell_dirty());
    assert_eq!(
        outcome.invalidation_route(),
        Some(&EventInvalidationRoute::Shell)
    );

    let mut notifier = UiNotifier::new();
    notifier.clear_all();
    outcome.apply_to_notifier(&mut notifier);
    assert!(notifier.is_shell_dirty());
}

#[test]
fn test_workbench_handle_event_with_tree_routes_invalidation() {
    let (mut tree, boundary_id, child_id, _) = create_test_tree();
    if let Some(node) = tree.get_mut(child_id) {
        node.computed_rect = irisui_core::geometry::Rect::new(50.0, 50.0, 100.0, 50.0);
    }
    let mut dock_frame = DockChromeFrame::default();
    dock_frame
        .leaf_containers
        .push((TestTab::Inspector, boundary_id));

    let descriptor = WorkbenchDescriptor::new("Test Workbench", DockState::new(DockTree::new()));
    let mut workbench: Workbench<TestTab, (), ()> = Workbench::new(descriptor);
    workbench.tree = tree;
    workbench.dock_frame = Some(dock_frame);

    let mut win = MockWindow::default();
    let event = UiEvent::MouseDown {
        button: irisui_core::event::MouseButton::Left,
        point: Point::new(60.0, 60.0),
    };

    let outcome = workbench.handle_event(&mut win, &event);

    assert_eq!(outcome.invalidated_panel(), Some(&TestTab::Inspector));
    assert_eq!(
        outcome.invalidation_route(),
        Some(&EventInvalidationRoute::Panel(TestTab::Inspector))
    );
}

#[test]
fn test_workbench_handle_event_with_content_tree_routes_invalidation() {
    let (mut tree, boundary_id, child_id, _) = create_test_tree();
    if let Some(node) = tree.get_mut(child_id) {
        node.computed_rect = irisui_core::geometry::Rect::new(50.0, 50.0, 100.0, 50.0);
    }
    let mut dock_frame = DockChromeFrame::default();
    dock_frame
        .leaf_containers
        .push((TestTab::Console, boundary_id));

    let descriptor = WorkbenchDescriptor::new("Test Workbench", DockState::new(DockTree::new()));
    let mut workbench: Workbench<TestTab, (), ()> = Workbench::new(descriptor);
    workbench.tree = tree;
    workbench.dock_frame = Some(dock_frame);

    let mut win = MockWindow::default();
    let mut dock_state = workbench.descriptor.dock_state.clone();
    let event = UiEvent::MouseDown {
        button: irisui_core::event::MouseButton::Left,
        point: Point::new(60.0, 60.0),
    };

    let outcome = workbench.handle_event_with_content(&mut win, &event, &mut dock_state, |_| {
        (true, None, Vec::new())
    });

    assert_eq!(outcome.invalidated_panel(), Some(&TestTab::Console));
    assert_eq!(
        outcome.invalidation_route(),
        Some(&EventInvalidationRoute::Panel(TestTab::Console))
    );
}

#[test]
fn test_workbench_outcome_into_action_only() {
    let outcome: crate::workbench::WorkbenchOutcome<(), TestTab> =
        crate::workbench::WorkbenchOutcome::empty()
            .with_invalidated_panel(Some(TestTab::Inspector))
            .with_shell_dirty(true);

    assert_eq!(outcome.invalidated_panel(), Some(&TestTab::Inspector));
    assert!(outcome.is_shell_dirty());

    let action_only = outcome.into_action_only();
    assert_eq!(action_only.invalidated_panel(), None);
    assert!(action_only.is_shell_dirty());
}