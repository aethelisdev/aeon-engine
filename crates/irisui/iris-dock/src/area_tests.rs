// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use crate::tree::{DockTree, SplitDirection};
use iris_core::DirtyFlags;
use iris_core::UiTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestPanel {
    Hierarchy,
    Inspector,
    Console,
}

struct MockViewer {
    rendered_hierarchy: bool,
    rendered_inspector: bool,
    rendered_console: bool,
}

impl TabViewer<TestPanel> for MockViewer {
    fn title(&self, tab: &TestPanel) -> String {
        match tab {
            TestPanel::Hierarchy => "Scene Hierarchy".into(),
            TestPanel::Inspector => "Entity Inspector".into(),
            TestPanel::Console => "Debug Console".into(),
        }
    }

    fn ui(&mut self, scope: &mut UiScope<'_>, tab: &TestPanel) {
        match tab {
            TestPanel::Hierarchy => {
                self.rendered_hierarchy = true;
                scope.label("Entities List");
            }
            TestPanel::Inspector => {
                self.rendered_inspector = true;
                scope.label("Component Fields");
            }
            TestPanel::Console => {
                self.rendered_console = true;
                scope.label("System Logs");
            }
        }
    }
}

#[test]
fn test_dock_area_orchestration() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    let mut dock_tree = DockTree::new();
    let center = dock_tree.create_leaf(vec![TestPanel::Hierarchy, TestPanel::Console]);
    dock_tree.set_root(center);
    let (_left, _right) = dock_tree
        .split_ordered(
            center,
            SplitDirection::Horizontal,
            0.3,
            vec![TestPanel::Inspector],
            true,
        )
        .expect("Split succeeds");

    let mut dock_state = DockState::new(dock_tree);
    let mut viewer = MockViewer {
        rendered_hierarchy: false,
        rendered_inspector: false,
        rendered_console: false,
    };

    let workspace_rect = Rect::new(0.0, 0.0, 1000.0, 600.0);
    let frame = DockArea::new(&mut dock_state)
        .bounds(workspace_rect)
        .show(&mut scope, &mut viewer);

    // Verify that chrome frame was generated with panels and targets
    assert_eq!(frame.panel_rects.len(), 2);
    assert!(!frame.tab_targets.is_empty());
    assert!(!frame.splitter_targets.is_empty());

    // Verify that viewer.ui was invoked for all active tabs
    assert!(viewer.rendered_hierarchy, "Hierarchy UI must be invoked");
    assert!(viewer.rendered_inspector, "Inspector UI must be invoked");
    assert!(
        !viewer.rendered_console,
        "Console UI must not be invoked as it is not in the tree"
    );
}

#[test]
fn test_dock_area_panel_invalidation_boundary_isolation() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    let mut dock_tree = DockTree::new();
    let center = dock_tree.create_leaf(vec![TestPanel::Hierarchy]);
    dock_tree.set_root(center);
    let (_left, _right) = dock_tree
        .split_ordered(
            center,
            SplitDirection::Horizontal,
            0.5,
            vec![TestPanel::Inspector],
            true,
        )
        .expect("Split succeeds");

    let mut dock_state = DockState::new(dock_tree);
    let mut viewer = MockViewer {
        rendered_hierarchy: false,
        rendered_inspector: false,
        rendered_console: false,
    };

    let workspace_rect = Rect::new(0.0, 0.0, 1000.0, 600.0);
    let _frame = DockArea::new(&mut dock_state)
        .bounds(workspace_rect)
        .show(&mut scope, &mut viewer);

    // Find both DockLeafPanelContent boundary nodes in the tree
    let panel_nodes: Vec<_> = tree
        .iter()
        .filter(|(_, n)| n.name.as_deref() == Some("DockLeafPanelContent"))
        .map(|(id, n)| (id, n.is_boundary, n.children.clone()))
        .collect();

    assert_eq!(panel_nodes.len(), 2, "Must have exactly 2 leaf panels");
    assert!(
        panel_nodes[0].1,
        "Leaf Panel 0 must be marked as an Invalidation Boundary"
    );
    assert!(
        panel_nodes[1].1,
        "Leaf Panel 1 must be marked as an Invalidation Boundary"
    );

    let panel_a_id = panel_nodes[0].0;
    let panel_b_id = panel_nodes[1].0;

    // Simulate clean tree layout pass
    let all_ids: Vec<_> = tree.iter().map(|(id, _)| id).collect();
    for id in all_ids {
        if let Some(node) = tree.get_mut(id) {
            node.dirty = DirtyFlags::empty();
        }
    }
    tree.clear_dirty_boundaries();

    // Now dirty a child widget inside Panel A (e.g. its label node)
    let child_a = panel_nodes[0].2[0];
    let boundary_caught = tree.mark_dirty(child_a, DirtyFlags::LAYOUT);

    // Invalidation must be trapped strictly at Panel A's boundary!
    assert_eq!(
        boundary_caught,
        Some(panel_a_id),
        "Dirty propagation must halt at Panel A boundary"
    );
    assert!(
        tree.dirty_boundaries.contains(&panel_a_id),
        "Panel A must be registered in dirty_boundaries"
    );

    // Root node and Panel B must REMAIN 100% CLEAN (Zero Invalidation Thrashing)
    assert_eq!(
        tree.get(root).unwrap().dirty,
        DirtyFlags::empty(),
        "Root node must remain clean when child panel is dirtied"
    );
    assert_eq!(
        tree.get(panel_b_id).unwrap().dirty,
        DirtyFlags::empty(),
        "Sibling Panel B must remain clean when Panel A is dirtied"
    );
}

#[test]
fn test_dock_area_floating_window_content_orchestration() {
    use crate::floating::FloatingWindow;
    use iris_core::UiLayer;

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    let mut dock_tree = DockTree::new();
    let center = dock_tree.create_leaf(vec![TestPanel::Hierarchy]);
    dock_tree.set_root(center);

    let mut dock_state = DockState::new(dock_tree);
    let floating_win = FloatingWindow::new(
        101,
        "Inspector",
        Rect::new(100.0, 100.0, 300.0, 400.0),
        vec![TestPanel::Inspector],
    );
    dock_state.floating_windows.push(floating_win);

    let mut viewer = MockViewer {
        rendered_hierarchy: false,
        rendered_inspector: false,
        rendered_console: false,
    };

    let workspace_rect = Rect::new(0.0, 0.0, 1000.0, 600.0);
    let frame = DockArea::new(&mut dock_state)
        .bounds(workspace_rect)
        .show(&mut scope, &mut viewer);

    assert!(
        viewer.rendered_hierarchy,
        "Docked hierarchy must be rendered"
    );
    assert!(
        viewer.rendered_inspector,
        "Floating inspector must be rendered"
    );
    assert_eq!(frame.floating_rects.len(), 1);

    // Find the FloatingLeafPanelContent node in the tree
    let (_id, floating_content_node) = tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("FloatingLeafPanelContent"))
        .expect("FloatingLeafPanelContent node must exist");

    // Verify boundary status and layer
    assert!(
        floating_content_node.is_boundary,
        "Floating leaf panel content must be marked as an Invalidation Boundary"
    );
    assert_eq!(
        floating_content_node.layer,
        UiLayer::Floating,
        "Floating panel content must be in UiLayer::Floating"
    );

    // Verify parent is the FloatingWindow container, not the root
    let parent_id = floating_content_node.parent.expect("must have parent");
    let parent_node = tree.get(parent_id).expect("parent node exists");
    assert_eq!(
        parent_node.name.as_deref(),
        Some("FloatingWindow_101"),
        "Floating content must be a child of FloatingWindow_101 container"
    );
}

#[test]
fn test_dock_area_handle_event_floating_priority() {
    use crate::floating::FloatingWindow;

    let mut dock_tree = DockTree::new();
    let center = dock_tree.create_leaf(vec![TestPanel::Hierarchy]);
    dock_tree.set_root(center);

    let mut dock_state = DockState::new(dock_tree);
    // Create floating window right over the center area
    let floating_win = FloatingWindow::new(
        201,
        "Floating Inspector",
        Rect::new(50.0, 50.0, 300.0, 200.0),
        vec![TestPanel::Inspector],
    );
    dock_state.floating_windows.push(floating_win);

    let viewer = MockViewer {
        rendered_hierarchy: false,
        rendered_inspector: false,
        rendered_console: false,
    };

    let workspace_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    let config = DockInteractionConfig::default();

    // 1. Click on empty space of floating title bar (x = 250.0, y = 62.0)
    let click_title = UiEvent::MouseDown {
        button: iris_core::MouseButton::Left,
        point: Point::new(250.0, 62.0),
    };
    let res = DockArea::handle_event(
        &mut dock_state,
        &viewer,
        &click_title,
        workspace_rect,
        &config,
    );
    assert!(res.consumed, "Floating title click must be consumed");
    assert!(
        dock_state.active_floating_drag.is_some(),
        "Title drag must be activated"
    );

    // 2. Move cursor -> window position should update
    let move_cursor = UiEvent::MouseMove {
        point: Point::new(300.0, 112.0),
    };
    let res_move = DockArea::handle_event(
        &mut dock_state,
        &viewer,
        &move_cursor,
        workspace_rect,
        &config,
    );
    assert!(res_move.consumed, "Active title drag motion is consumed");
    assert!(
        dock_state.floating_windows[0].rect.x > 50.0,
        "Floating window must move"
    );

    // 3. Release mouse -> drag finalized
    let release = UiEvent::MouseUp {
        button: iris_core::MouseButton::Left,
        point: Point::new(300.0, 112.0),
    };
    let res_rel =
        DockArea::handle_event(&mut dock_state, &viewer, &release, workspace_rect, &config);
    assert!(res_rel.consumed);
    assert!(dock_state.active_floating_drag.is_none());
}

#[test]
fn test_dock_area_handle_event_tab_selection_and_close() {
    let mut dock_tree = DockTree::new();
    let center = dock_tree.create_leaf(vec![TestPanel::Hierarchy, TestPanel::Console]);
    dock_tree.set_root(center);

    let mut dock_state = DockState::new(dock_tree);
    let viewer = MockViewer {
        rendered_hierarchy: false,
        rendered_inspector: false,
        rendered_console: false,
    };

    let workspace_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    let config = DockInteractionConfig::default();

    // Initially active tab is index 0 (Hierarchy)
    let leaf = dock_state.tree.find_first_leaf().unwrap();
    let initial_tab = match dock_state.tree.get(leaf) {
        Some(DockNode::Leaf { active_tab, .. }) => Some(*active_tab),
        _ => None,
    };
    assert_eq!(initial_tab, Some(0));

    // Click tab pill for Console (second tab starts after ~120px)
    let click_console_tab = UiEvent::MouseDown {
        button: iris_core::MouseButton::Left,
        point: Point::new(160.0, 13.0),
    };
    let res = DockArea::handle_event(
        &mut dock_state,
        &viewer,
        &click_console_tab,
        workspace_rect,
        &config,
    );
    assert!(res.consumed, "Tab selection click must be consumed");
    let switched_tab = match dock_state.tree.get(leaf) {
        Some(DockNode::Leaf { active_tab, .. }) => Some(*active_tab),
        _ => None,
    };
    assert_eq!(switched_tab, Some(1), "Active tab must switch to Console");
}

#[test]
fn test_dock_area_handle_event_splitter_drag() {
    let mut dock_tree = DockTree::new();
    let center = dock_tree.create_leaf(vec![TestPanel::Hierarchy]);
    dock_tree.set_root(center);
    let (_left, _right) = dock_tree
        .split_ordered(
            center,
            SplitDirection::Horizontal,
            0.5,
            vec![TestPanel::Inspector],
            true,
        )
        .expect("Split succeeds");

    let mut dock_state = DockState::new(dock_tree);
    let viewer = MockViewer {
        rendered_hierarchy: false,
        rendered_inspector: false,
        rendered_console: false,
    };

    let workspace_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    let config = DockInteractionConfig::default();

    // Center split is at x = 400.0 (horizontal split line)
    // Hover over splitter (x = 400.0, y = 200.0)
    let hover = UiEvent::MouseMove {
        point: Point::new(400.0, 200.0),
    };
    let res_hover =
        DockArea::handle_event(&mut dock_state, &viewer, &hover, workspace_rect, &config);
    assert_eq!(
        res_hover.cursor_override,
        Some(WidgetCursor::ColResize),
        "Splitter hover must report ColResize"
    );

    // Click down on splitter
    let press = UiEvent::MouseDown {
        button: iris_core::MouseButton::Left,
        point: Point::new(400.0, 200.0),
    };
    let res_press =
        DockArea::handle_event(&mut dock_state, &viewer, &press, workspace_rect, &config);
    assert!(res_press.consumed);
    assert!(
        dock_state.active_splitter.is_some(),
        "Splitter drag must be active"
    );

    // Drag to x = 500.0
    let drag_move = UiEvent::MouseMove {
        point: Point::new(500.0, 200.0),
    };
    let res_move = DockArea::handle_event(
        &mut dock_state,
        &viewer,
        &drag_move,
        workspace_rect,
        &config,
    );
    assert!(res_move.consumed);
    assert!(res_move.layout_changed);

    // Release mouse
    let release = UiEvent::MouseUp {
        button: iris_core::MouseButton::Left,
        point: Point::new(500.0, 200.0),
    };
    let res_rel =
        DockArea::handle_event(&mut dock_state, &viewer, &release, workspace_rect, &config);
    assert!(res_rel.consumed);
    assert!(
        dock_state.active_splitter.is_none(),
        "Splitter drag must terminate"
    );
}

#[test]
fn test_dock_area_handle_event_tab_tear_off_to_floating_window() {
    let mut dock_tree = DockTree::new();
    let left = dock_tree.create_leaf(vec![TestPanel::Hierarchy]);
    dock_tree.set_root(left);
    let (_first, _second) = dock_tree
        .split_ordered(
            left,
            SplitDirection::Horizontal,
            0.5,
            vec![TestPanel::Inspector],
            true,
        )
        .expect("Split succeeds");

    let mut dock_state = DockState::new(dock_tree);
    let viewer = MockViewer {
        rendered_hierarchy: false,
        rendered_inspector: false,
        rendered_console: false,
    };

    let workspace_rect = Rect::new(0.0, 0.0, 1000.0, 800.0);
    let config = DockInteractionConfig::default();

    // 1. Mouse down on Hierarchy tab (x = 50.0, y = 13.0)
    let click_tab = UiEvent::MouseDown {
        button: iris_core::MouseButton::Left,
        point: Point::new(50.0, 13.0),
    };
    let res_press = DockArea::handle_event(
        &mut dock_state,
        &viewer,
        &click_tab,
        workspace_rect,
        &config,
    );
    assert!(res_press.consumed);
    assert!(dock_state.pending_tab_drag.is_some());

    // 2. Drag cursor beyond 4px threshold into open content area away from compass buttons (e.g. x = 200.0, y = 150.0)
    let drag_motion = UiEvent::MouseMove {
        point: Point::new(200.0, 150.0),
    };
    let res_motion = DockArea::handle_event(
        &mut dock_state,
        &viewer,
        &drag_motion,
        workspace_rect,
        &config,
    );
    assert!(res_motion.consumed);
    assert!(dock_state.active_drag.is_some());

    // 3. Release mouse in open space
    let release = UiEvent::MouseUp {
        button: iris_core::MouseButton::Left,
        point: Point::new(200.0, 150.0),
    };
    let res_release =
        DockArea::handle_event(&mut dock_state, &viewer, &release, workspace_rect, &config);
    assert!(res_release.consumed);
    assert!(res_release.layout_changed);
    assert!(dock_state.active_drag.is_none());

    // Must detach as an autonomous floating window instead of snapping into another leaf
    assert_eq!(
        dock_state.floating_windows.len(),
        1,
        "Tear-off tab dropped into empty space must become a floating window"
    );
    assert_eq!(
        dock_state.floating_windows[0]
            .tree
            .find_tab(&TestPanel::Inspector)
            .map(|(_, i)| i),
        Some(0)
    );
}

#[test]
fn test_dock_area_records_leaf_container_widget_ids() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    let mut dock_tree = DockTree::new();
    let center = dock_tree.create_leaf(vec![TestPanel::Hierarchy]);
    dock_tree.set_root(center);
    let (_left, _right) = dock_tree
        .split_ordered(
            center,
            SplitDirection::Horizontal,
            0.5,
            vec![TestPanel::Inspector],
            true,
        )
        .unwrap();

    let mut state = DockState::new(dock_tree);
    let floating = crate::floating::FloatingWindow::new(
        1,
        "Console Window",
        Rect::new(50.0, 50.0, 300.0, 200.0),
        vec![TestPanel::Console],
    );
    state.floating_windows.push(floating);

    let mut viewer = MockViewer {
        rendered_hierarchy: false,
        rendered_inspector: false,
        rendered_console: false,
    };

    let workspace_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    let frame = DockArea::new(&mut state)
        .bounds(workspace_rect)
        .show(&mut scope, &mut viewer);

    // Frame must record leaf container WidgetId for each rendered panel
    assert_eq!(frame.leaf_containers.len(), 3);

    let hier_id = frame
        .leaf_container(TestPanel::Hierarchy)
        .expect("Hierarchy leaf container must be recorded");
    let insp_id = frame
        .leaf_container(TestPanel::Inspector)
        .expect("Inspector leaf container must be recorded");
    let console_id = frame
        .leaf_container(TestPanel::Console)
        .expect("Floating Console leaf container must be recorded");

    // All IDs must be distinct
    assert_ne!(hier_id, insp_id);
    assert_ne!(hier_id, console_id);
    assert_ne!(insp_id, console_id);

    // Verify nodes in the UiTree
    let hier_node = tree
        .get(hier_id)
        .expect("Hierarchy node must exist in tree");
    assert_eq!(hier_node.name.as_deref(), Some("DockLeafPanelContent"));
    assert!(
        hier_node.is_boundary,
        "Leaf node must be an invalidation boundary"
    );

    let insp_node = tree
        .get(insp_id)
        .expect("Inspector node must exist in tree");
    assert_eq!(insp_node.name.as_deref(), Some("DockLeafPanelContent"));
    assert!(
        insp_node.is_boundary,
        "Leaf node must be an invalidation boundary"
    );

    let console_node = tree
        .get(console_id)
        .expect("Console node must exist in tree");
    assert_eq!(
        console_node.name.as_deref(),
        Some("FloatingLeafPanelContent")
    );
    assert!(
        console_node.is_boundary,
        "Floating leaf node must be an invalidation boundary"
    );
}