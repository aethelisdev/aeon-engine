// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::routing::{
    EventInvalidationRoute, route_event_invalidation, route_event_invalidation_with_fallback,
};
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::workbench::Workbench;
use iris_core::dispatch::DispatchResult;
use iris_core::event::UiEvent;
use iris_core::geometry::Point;
use iris_core::node::WidgetRole;
use iris_core::tree::UiTree;
use iris_dock::UiNotifier;
use iris_dock::chrome::DockChromeFrame;
use iris_dock::state::DockState;
use iris_dock::tree::DockTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestTab {
    Hierarchy,
    Inspector,
    Console,
}

fn create_test_tree() -> (
    UiTree,
    iris_core::WidgetId,
    iris_core::WidgetId,
    iris_core::WidgetId,
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
        button: iris_core::event::MouseButton::Left,
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
        button: iris_core::event::MouseButton::Left,
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
        key: iris_core::event::KeyCode::Character('a'),
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
        button: iris_core::event::MouseButton::Left,
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
        button: iris_core::event::MouseButton::Left,
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
    workbench.dock_frame = Some(dock_frame);

    let mut dispatch = DispatchResult::ignored();
    dispatch.target = Some(child_id);
    let event = UiEvent::MouseDown {
        button: iris_core::event::MouseButton::Left,
        point: Point::new(100.0, 100.0),
    };

    let mut notifier = UiNotifier::new();
    notifier.clear_all();

    let route =
        workbench.route_event_invalidation_with_notifier(&tree, &event, &dispatch, &mut notifier);

    assert_eq!(route, EventInvalidationRoute::Panel(TestTab::Hierarchy));
    assert!(workbench.lifecycle.is_tab_dirty(&TestTab::Hierarchy));
    assert!(notifier.is_dirty(TestTab::Hierarchy));
}