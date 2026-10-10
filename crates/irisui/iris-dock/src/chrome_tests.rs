// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit test suite for `DockChrome` retained quad/tab generation, splitter hit targets, and hit testing.

use super::*;
use crate::tree::DockTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MockTab {
    Viewport,
    Hierarchy,
}

struct MockViewer;
impl TabViewer<MockTab> for MockViewer {
    fn title(&self, tab: &MockTab) -> String {
        match tab {
            MockTab::Viewport => "Viewport".to_string(),
            MockTab::Hierarchy => "Hierarchy".to_string(),
        }
    }

    fn closeable(&self, tab: &MockTab) -> bool {
        *tab != MockTab::Viewport
    }
}

#[test]
fn test_build_dock_chrome_single_pane_and_tabs() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut dock_tree = DockTree::new();
    let root_leaf = dock_tree.create_leaf(vec![MockTab::Viewport, MockTab::Hierarchy]);
    dock_tree.set_root(root_leaf);

    let workspace = Rect::new(0.0, 0.0, 800.0, 600.0);
    let params = DockChromeParams {
        dock_tree: &dock_tree,
        workspace_rect: workspace,
        cursor_pos: Point::new(10.0, 10.0),
        is_cursor_occluded: false,
        is_dragging_splitter: false,
        active_splitter_node: None,
        is_dragging_tab: false,
        viewer: &MockViewer,
        style: &DockChromeStyle::default(),
    };
    let frame = build_dock_chrome(&mut tree, root, &params);

    assert_eq!(frame.panel_rects.len(), 1);
    assert_eq!(frame.panel_rects[0].0, MockTab::Viewport);
    assert_eq!(frame.tab_targets.len(), 2);
    assert_eq!(frame.close_targets.len(), 1); // Only Hierarchy is closeable
    assert_eq!(frame.splitter_targets.len(), 0);
}

#[test]
fn test_build_dock_chrome_splitters() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut dock_tree = DockTree::new();
    let root_leaf = dock_tree.create_leaf(vec![MockTab::Viewport]);
    dock_tree.set_root(root_leaf);
    let (_left, _right) = dock_tree
        .split(
            root_leaf,
            SplitDirection::Horizontal,
            0.3,
            vec![MockTab::Hierarchy],
        )
        .unwrap();

    let workspace = Rect::new(0.0, 0.0, 1000.0, 800.0);
    let params = DockChromeParams {
        dock_tree: &dock_tree,
        workspace_rect: workspace,
        cursor_pos: Point::new(50.0, 50.0),
        is_cursor_occluded: false,
        is_dragging_splitter: false,
        active_splitter_node: None,
        is_dragging_tab: false,
        viewer: &MockViewer,
        style: &DockChromeStyle::default(),
    };
    let frame = build_dock_chrome(&mut tree, root, &params);

    assert_eq!(frame.panel_rects.len(), 2);
    assert_eq!(frame.tab_targets.len(), 2);
    assert_eq!(frame.splitter_targets.len(), 1);
    assert_eq!(
        frame.splitter_targets[0].direction,
        SplitDirection::Horizontal
    );
    assert!(frame.panel_rect(MockTab::Hierarchy).is_some());
    assert!(frame.panel_rect(MockTab::Viewport).is_some());
}

#[test]
fn test_dock_panel_quads_are_passive_for_hit_testing() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut dock_tree = DockTree::new();
    let root_leaf = dock_tree.create_leaf(vec![MockTab::Viewport]);
    dock_tree.set_root(root_leaf);

    let workspace = Rect::new(0.0, 0.0, 500.0, 500.0);
    let params = DockChromeParams {
        dock_tree: &dock_tree,
        workspace_rect: workspace,
        cursor_pos: Point::new(100.0, 100.0),
        is_cursor_occluded: false,
        is_dragging_splitter: false,
        active_splitter_node: None,
        is_dragging_tab: false,
        viewer: &MockViewer,
        style: &DockChromeStyle::default(),
    };
    let _frame = build_dock_chrome(&mut tree, root, &params);

    // Hit test over panel area must NOT hit dock background quads
    let hit = tree.hit_test(Point::new(200.0, 200.0));
    assert!(
        hit.is_none(),
        "Dock panel background quads must not intercept cursor hit test"
    );
}

#[test]
fn test_dock_splitter_target_contains_point_with_margin() {
    let target = DockSplitterTarget {
        node: DockNodeId::default(),
        direction: SplitDirection::Horizontal,
        rect: Rect::new(100.0, 50.0, 3.0, 400.0),
        total_dimension: 800.0,
    };

    // Point right on the 3px bar
    assert!(target.contains_point(Point::new(101.0, 100.0)));

    // Points within HIT_MARGIN (4.0px) to left and right
    assert!(target.contains_point(Point::new(96.0, 100.0)));
    assert!(target.contains_point(Point::new(107.0, 100.0)));

    // Points outside HIT_MARGIN
    assert!(!target.contains_point(Point::new(95.0, 100.0)));
    assert!(!target.contains_point(Point::new(108.0, 100.0)));

    // Points outside vertical span
    assert!(!target.contains_point(Point::new(101.0, 49.0)));
    assert!(!target.contains_point(Point::new(101.0, 451.0)));

    // Test Vertical splitter
    let v_target = DockSplitterTarget {
        node: DockNodeId::default(),
        direction: SplitDirection::Vertical,
        rect: Rect::new(50.0, 200.0, 400.0, 3.0),
        total_dimension: 600.0,
    };
    assert!(v_target.contains_point(Point::new(100.0, 201.0)));
    assert!(v_target.contains_point(Point::new(100.0, 196.0)));
    assert!(v_target.contains_point(Point::new(100.0, 207.0)));
    assert!(!v_target.contains_point(Point::new(100.0, 195.0)));
    assert!(!v_target.contains_point(Point::new(100.0, 208.0)));
}