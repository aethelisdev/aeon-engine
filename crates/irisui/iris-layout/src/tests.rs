// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Layout engine integration and regression test suite.

use super::*;
use iris_core::{DirtyFlags, Rect, Size, Style, UiTree};

#[test]
fn test_layout_computation_simple_column() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let child1 = tree.create_node();
    let child2 = tree.create_node();

    if let Some(node) = tree.get_mut(root) {
        node.set_style(Style::new().flex_col().padding(10.0).gap(5.0));
    }

    if let Some(node) = tree.get_mut(child1) {
        node.set_style(Style::new().width(100.0).height(40.0));
    }

    if let Some(node) = tree.get_mut(child2) {
        node.set_style(Style::new().width(100.0).height(60.0));
    }

    assert!(tree.add_child(root, child1).is_ok());
    assert!(tree.add_child(root, child2).is_ok());

    let mut engine = LayoutEngine::new();
    let result = engine.compute_layout(&mut tree, Size::new(800.0, 600.0));
    assert!(result.is_ok());

    let c1_rect = tree.get(child1).unwrap().computed_rect;
    let c2_rect = tree.get(child2).unwrap().computed_rect;

    // child1 starts at padding (10, 10)
    assert_eq!(c1_rect.x, 10.0);
    assert_eq!(c1_rect.y, 10.0);
    assert_eq!(c1_rect.width, 100.0);
    assert_eq!(c1_rect.height, 40.0);

    // child2 starts after child1 + gap (10 + 40 + 5 = 55)
    assert_eq!(c2_rect.x, 10.0);
    assert_eq!(c2_rect.y, 55.0);
    assert_eq!(c2_rect.width, 100.0);
    assert_eq!(c2_rect.height, 60.0);
}

#[test]
fn test_layout_computation_with_scroll_offset_y() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let child = tree.create_node();

    if let Some(node) = tree.get_mut(root) {
        let mut style = Style::new().flex_col().padding(0.0);
        style.scroll_offset_y = 25.0;
        node.set_style(style);
    }

    if let Some(node) = tree.get_mut(child) {
        node.set_style(Style::new().width(100.0).height(40.0));
    }

    assert!(tree.add_child(root, child).is_ok());

    let mut engine = LayoutEngine::new();
    assert!(
        engine
            .compute_layout(&mut tree, Size::new(800.0, 600.0))
            .is_ok()
    );

    let c_rect = tree.get(child).unwrap().computed_rect;
    // child starts at y = 0.0 - scroll_offset_y (25.0) = -25.0
    assert_eq!(c_rect.y, -25.0);
}

#[test]
fn test_layout_computation_invalidation_boundary_isolation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    // Root row holding Panel A and Panel B
    if let Some(node) = tree.get_mut(root) {
        node.set_style(Style::new().flex_row().width(800.0).height(600.0));
    }

    // Panel A (Boundary)
    let panel_a = tree.create_node();
    tree.set_boundary(panel_a, true);
    if let Some(node) = tree.get_mut(panel_a) {
        node.set_style(Style::new().flex_col().width(300.0).height(600.0));
    }
    tree.add_child(root, panel_a).unwrap();

    let a_child = tree.create_node();
    if let Some(node) = tree.get_mut(a_child) {
        node.set_style(Style::new().width(100.0).height(50.0));
    }
    tree.add_child(panel_a, a_child).unwrap();

    // Panel B (Sibling Boundary)
    let panel_b = tree.create_node();
    tree.set_boundary(panel_b, true);
    if let Some(node) = tree.get_mut(panel_b) {
        node.set_style(Style::new().flex_col().width(500.0).height(600.0));
    }
    tree.add_child(root, panel_b).unwrap();

    let b_child = tree.create_node();
    if let Some(node) = tree.get_mut(b_child) {
        node.set_style(Style::new().width(200.0).height(50.0));
    }
    tree.add_child(panel_b, b_child).unwrap();

    let mut engine = LayoutEngine::new();
    assert!(
        engine
            .compute_layout(&mut tree, Size::new(800.0, 600.0))
            .is_ok()
    );

    // Initial assertions
    assert_eq!(
        tree.get(panel_a).unwrap().computed_rect,
        Rect::new(0.0, 0.0, 300.0, 600.0)
    );
    assert_eq!(
        tree.get(panel_b).unwrap().computed_rect,
        Rect::new(300.0, 0.0, 500.0, 600.0)
    );
    assert_eq!(tree.get(a_child).unwrap().computed_rect.height, 50.0);
    assert_eq!(tree.get(b_child).unwrap().computed_rect.height, 50.0);

    // Mutate a_child inside Panel A and mark layout dirty
    if let Some(node) = tree.get_mut(a_child) {
        node.set_style(Style::new().width(100.0).height(80.0));
    }
    let boundary = tree.mark_dirty(a_child, DirtyFlags::LAYOUT);
    assert_eq!(boundary, Some(panel_a));
    assert!(tree.dirty_boundaries.contains(&panel_a));
    assert!(!tree.dirty_boundaries.contains(&panel_b));

    // Recompute layout: Fast path 2 should kick in!
    assert!(
        engine
            .compute_layout(&mut tree, Size::new(800.0, 600.0))
            .is_ok()
    );

    // a_child inside Panel A must have updated height (80.0)
    assert_eq!(tree.get(a_child).unwrap().computed_rect.height, 80.0);

    // Panel B and its children must remain completely preserved
    assert_eq!(
        tree.get(panel_b).unwrap().computed_rect,
        Rect::new(300.0, 0.0, 500.0, 600.0)
    );
    assert_eq!(tree.get(b_child).unwrap().computed_rect.height, 50.0);
    assert_eq!(tree.get(b_child).unwrap().computed_rect.x, 300.0);

    // dirty_boundaries set must now be clean
    assert!(!tree.has_dirty_boundaries());
    assert!(!tree.has_dirty_nodes(DirtyFlags::LAYOUT));
}

#[test]
fn test_layout_untrack_subtree_removes_taffy_nodes() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    // Panel A (Boundary)
    let panel_a = tree.create_node();
    tree.set_boundary(panel_a, true);
    if let Some(node) = tree.get_mut(panel_a) {
        node.set_style(Style::new().flex_col().width(300.0).height(400.0));
    }
    tree.add_child(root, panel_a).unwrap();

    let a_child1 = tree.create_node();
    if let Some(node) = tree.get_mut(a_child1) {
        node.set_style(Style::new().width(200.0).height(50.0));
    }
    tree.add_child(panel_a, a_child1).unwrap();

    let a_child2 = tree.create_node();
    if let Some(node) = tree.get_mut(a_child2) {
        node.set_style(Style::new().width(200.0).height(50.0));
    }
    tree.add_child(panel_a, a_child2).unwrap();

    let a_grandchild = tree.create_node();
    if let Some(node) = tree.get_mut(a_grandchild) {
        node.set_style(Style::new().width(100.0).height(20.0));
    }
    tree.add_child(a_child1, a_grandchild).unwrap();

    // Panel B (Sibling)
    let panel_b = tree.create_node();
    tree.set_boundary(panel_b, true);
    if let Some(node) = tree.get_mut(panel_b) {
        node.set_style(Style::new().flex_col().width(500.0).height(400.0));
    }
    tree.add_child(root, panel_b).unwrap();

    let b_child = tree.create_node();
    if let Some(node) = tree.get_mut(b_child) {
        node.set_style(Style::new().width(400.0).height(100.0));
    }
    tree.add_child(panel_b, b_child).unwrap();

    let mut engine = LayoutEngine::new();
    assert!(
        engine
            .compute_layout(&mut tree, Size::new(800.0, 600.0))
            .is_ok()
    );

    // Before untracking: 7 total nodes tracked in engine
    assert_eq!(engine.len(), 7);
    assert!(engine.is_tracked(root));
    assert!(engine.is_tracked(panel_a));
    assert!(engine.is_tracked(a_child1));
    assert!(engine.is_tracked(a_child2));
    assert!(engine.is_tracked(a_grandchild));
    assert!(engine.is_tracked(panel_b));
    assert!(engine.is_tracked(b_child));

    // Untrack subtree of Panel A
    engine.untrack_subtree(&tree, panel_a);

    // Panel A itself, Root, Panel B, and b_child MUST remain tracked
    assert!(engine.is_tracked(root));
    assert!(engine.is_tracked(panel_a));
    assert!(engine.is_tracked(panel_b));
    assert!(engine.is_tracked(b_child));

    // Descendants of Panel A MUST be completely removed from Taffy tracking
    assert!(!engine.is_tracked(a_child1));
    assert!(!engine.is_tracked(a_child2));
    assert!(!engine.is_tracked(a_grandchild));

    // Exactly 4 nodes remain tracked: root, panel_a, panel_b, b_child
    assert_eq!(engine.len(), 4);
}

#[test]
fn test_layout_untrack_subtree_after_clear_children() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    // Panel A (Boundary)
    let panel_a = tree.create_node();
    tree.set_boundary(panel_a, true);
    if let Some(node) = tree.get_mut(panel_a) {
        node.set_style(Style::new().flex_col().width(300.0).height(400.0));
    }
    tree.add_child(root, panel_a).unwrap();

    let a_child1 = tree.create_node();
    if let Some(node) = tree.get_mut(a_child1) {
        node.set_style(Style::new().width(200.0).height(50.0));
    }
    tree.add_child(panel_a, a_child1).unwrap();

    let a_child2 = tree.create_node();
    if let Some(node) = tree.get_mut(a_child2) {
        node.set_style(Style::new().width(200.0).height(50.0));
    }
    tree.add_child(panel_a, a_child2).unwrap();

    let a_grandchild = tree.create_node();
    if let Some(node) = tree.get_mut(a_grandchild) {
        node.set_style(Style::new().width(100.0).height(20.0));
    }
    tree.add_child(a_child1, a_grandchild).unwrap();

    // Panel B (Sibling)
    let panel_b = tree.create_node();
    tree.set_boundary(panel_b, true);
    if let Some(node) = tree.get_mut(panel_b) {
        node.set_style(Style::new().flex_col().width(500.0).height(400.0));
    }
    tree.add_child(root, panel_b).unwrap();

    let b_child = tree.create_node();
    if let Some(node) = tree.get_mut(b_child) {
        node.set_style(Style::new().width(400.0).height(100.0));
    }
    tree.add_child(panel_b, b_child).unwrap();

    let mut engine = LayoutEngine::new();
    assert!(
        engine
            .compute_layout(&mut tree, Size::new(800.0, 600.0))
            .is_ok()
    );

    assert_eq!(engine.len(), 7);

    // Call UiTree::clear_children FIRST (so parent_a.children becomes empty in UiTree)
    assert!(tree.clear_children(panel_a).is_ok());
    assert!(tree.get(panel_a).unwrap().children.is_empty());

    // Then call engine.untrack_subtree - it MUST still purge all Taffy descendants directly!
    engine.untrack_subtree(&tree, panel_a);

    // Panel A itself, Root, Panel B, and b_child MUST remain tracked
    assert!(engine.is_tracked(root));
    assert!(engine.is_tracked(panel_a));
    assert!(engine.is_tracked(panel_b));
    assert!(engine.is_tracked(b_child));

    // Descendants of Panel A MUST be completely removed from Taffy tracking
    assert!(!engine.is_tracked(a_child1));
    assert!(!engine.is_tracked(a_child2));
    assert!(!engine.is_tracked(a_grandchild));

    // Exactly 4 nodes remain tracked: root, panel_a, panel_b, b_child
    assert_eq!(engine.len(), 4);
}

#[test]
fn test_compute_boundary_subtree_zero_coordinate_accumulation_across_rebuilds() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    // Create an offset panel with non-zero X and Y (e.g. Inspector or floating dock panel)
    let panel = tree.create_node();
    tree.set_boundary(panel, true);
    if let Some(node) = tree.get_mut(panel) {
        let style = Style::new()
            .position_absolute()
            .left(350.0)
            .top(120.0)
            .width(400.0)
            .height(600.0)
            .flex_col()
            .padding(10.0);
        node.set_style(style);
        node.computed_rect = Rect::new(350.0, 120.0, 400.0, 600.0);
    }
    tree.add_child(root, panel).unwrap();

    let child = tree.create_node();
    if let Some(node) = tree.get_mut(child) {
        node.set_style(Style::new().width(200.0).height(50.0));
    }
    tree.add_child(panel, child).unwrap();

    let mut engine = LayoutEngine::new();
    assert!(
        engine
            .compute_layout(&mut tree, Size::new(1920.0, 1080.0))
            .is_ok()
    );

    let initial_panel_rect = tree.get(panel).unwrap().computed_rect;
    let initial_child_rect = tree.get(child).unwrap().computed_rect;

    assert_eq!(initial_panel_rect, Rect::new(350.0, 120.0, 400.0, 600.0));
    // Child starts at panel.x + padding (350 + 10 = 360), panel.y + padding (120 + 10 = 130)
    assert_eq!(initial_child_rect.x, 360.0);
    assert_eq!(initial_child_rect.y, 130.0);

    // Simulate 10 successive interactive rebuilds / touches on the boundary panel
    for iteration in 1..=10 {
        tree.mark_dirty(child, DirtyFlags::LAYOUT);
        assert!(
            engine.compute_boundary_subtree(&mut tree, panel).is_ok(),
            "compute_boundary_subtree failed on iteration {iteration}"
        );

        let current_panel_rect = tree.get(panel).unwrap().computed_rect;
        let current_child_rect = tree.get(child).unwrap().computed_rect;

        assert_eq!(
            current_panel_rect, initial_panel_rect,
            "Panel computed_rect must NOT drift on iteration {iteration}"
        );
        assert_eq!(
            current_child_rect.x, initial_child_rect.x,
            "Child X coordinate must NOT drift on iteration {iteration}"
        );
        assert_eq!(
            current_child_rect.y, initial_child_rect.y,
            "Child Y coordinate must NOT drift on iteration {iteration}"
        );
    }
}