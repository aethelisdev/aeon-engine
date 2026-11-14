// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Layout engine integration and regression test suite.

use super::*;
use iris_core::{DirtyFlags, JustifyContent, Rect, Size, Style, UiLayer, UiTree};

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
    assert_eq!(engine.taffy_node_count(), 7);
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
    assert_eq!(engine.taffy_node_count(), 4);
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
    assert_eq!(engine.taffy_node_count(), 4);
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

#[test]
fn test_layout_untrack_and_rebuild_zero_leak_100_iterations() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let panel = tree.create_node();
    tree.set_boundary(panel, true);
    if let Some(node) = tree.get_mut(panel) {
        node.set_style(Style::new().flex_col().width(400.0).height(600.0));
        node.computed_rect = Rect::new(50.0, 50.0, 400.0, 600.0);
    }
    tree.add_child(root, panel).unwrap();

    let mut engine = LayoutEngine::new();
    assert!(
        engine
            .compute_layout(&mut tree, Size::new(1920.0, 1080.0))
            .is_ok()
    );

    let baseline_nodes = engine.len();
    assert_eq!(baseline_nodes, 2); // root + panel
    assert_eq!(engine.taffy_node_count(), 2);

    for _ in 0..100 {
        // Untrack previous subtree
        engine.untrack_subtree(&tree, panel);
        assert!(tree.clear_children(panel).is_ok());

        // Baseline after untrack: exactly 2 nodes remain in both node_map and Taffy
        assert_eq!(engine.len(), baseline_nodes);
        assert_eq!(engine.taffy_node_count(), baseline_nodes);

        // Synthesize 3 nested child widgets
        let child1 = tree.create_node();
        if let Some(node) = tree.get_mut(child1) {
            node.set_style(Style::new().width(200.0).height(40.0));
        }
        tree.add_child(panel, child1).unwrap();

        let child2 = tree.create_node();
        if let Some(node) = tree.get_mut(child2) {
            node.set_style(Style::new().width(200.0).height(40.0));
        }
        tree.add_child(panel, child2).unwrap();

        let grandchild = tree.create_node();
        if let Some(node) = tree.get_mut(grandchild) {
            node.set_style(Style::new().width(100.0).height(20.0));
        }
        tree.add_child(child1, grandchild).unwrap();

        // Recompute boundary layout in O(M)
        assert!(engine.compute_boundary_subtree(&mut tree, panel).is_ok());

        // Invariant: 2 baseline + 3 children = 5 nodes total
        assert_eq!(engine.len(), 5);
        assert_eq!(engine.taffy_node_count(), 5);
    }

    // Final untrack check confirms absolute cleanup back to baseline
    engine.untrack_subtree(&tree, panel);
    assert_eq!(engine.len(), baseline_nodes);
    assert_eq!(engine.taffy_node_count(), baseline_nodes);
}

#[test]
fn test_compute_boundary_subtree_preserves_modal_overlay_coordinates() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    // Create an offset dock panel boundary at (300.0, 700.0) simulating Assets panel
    let panel = tree.create_node();
    tree.set_boundary(panel, true);
    if let Some(node) = tree.get_mut(panel) {
        let style = Style::new()
            .position_absolute()
            .left(300.0)
            .top(700.0)
            .width(1200.0)
            .height(300.0)
            .flex_col()
            .padding(8.0);
        node.set_style(style);
        node.computed_rect = Rect::new(300.0, 700.0, 1200.0, 300.0);
    }
    tree.add_child(root, panel).unwrap();

    // Regular panel content child
    let content_child = tree.create_node();
    if let Some(node) = tree.get_mut(content_child) {
        node.set_style(Style::new().width(400.0).height(40.0));
    }
    tree.add_child(panel, content_child).unwrap();

    // Modal overlay attached to panel scope, but spanning entire window (1920x1080)
    let modal_scrim = tree.create_node();
    let screen_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    if let Some(node) = tree.get_mut(modal_scrim) {
        node.layer = UiLayer::Modal;
        node.computed_rect = screen_rect;
        node.set_style(
            Style::new()
                .position_absolute()
                .left(0.0)
                .top(0.0)
                .width(1920.0)
                .height(1080.0),
        );
    }
    tree.add_child(panel, modal_scrim).unwrap();

    // Modal dialog card centered in window
    let modal_card = tree.create_node();
    let card_rect = Rect::new(750.0, 420.0, 420.0, 240.0);
    if let Some(node) = tree.get_mut(modal_card) {
        node.layer = UiLayer::Modal;
        node.computed_rect = card_rect;
    }
    tree.add_child(modal_scrim, modal_card).unwrap();

    let mut engine = LayoutEngine::new();
    // Compute boundary subtree for panel
    assert!(engine.compute_boundary_subtree(&mut tree, panel).is_ok());

    // Regular content child received layout offset from boundary
    let content_rect = tree.get(content_child).unwrap().computed_rect;
    assert_eq!(content_rect.x, 308.0);
    assert_eq!(content_rect.y, 708.0);

    // CRITICAL INVARIANT: Modal scrim and card coordinates were PRESERVED and not shifted by panel.y (700)
    let scrim_rect = tree.get(modal_scrim).unwrap().computed_rect;
    assert_eq!(
        scrim_rect, screen_rect,
        "Modal scrim must retain window bounds"
    );

    let final_card_rect = tree.get(modal_card).unwrap().computed_rect;
    assert_eq!(
        final_card_rect, card_rect,
        "Modal card must retain centered screen coordinates"
    );
}

#[test]
fn test_compute_layout_preserves_modal_overlay_coordinates_under_flex_root() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    // Configure root as flex column with SpaceBetween, typical of desktop IDE shell
    if let Some(node) = tree.get_mut(root) {
        node.set_style(
            Style::new()
                .flex_col()
                .justify_content(JustifyContent::SpaceBetween)
                .width(1920.0)
                .height(1080.0),
        );
    }

    // Top menubar child
    let menubar = tree.create_node();
    if let Some(node) = tree.get_mut(menubar) {
        node.set_style(Style::new().width(1920.0).height(36.0));
    }
    tree.add_child(root, menubar).unwrap();

    // Bottom statusbar child
    let statusbar = tree.create_node();
    if let Some(node) = tree.get_mut(statusbar) {
        node.set_style(Style::new().width(1920.0).height(24.0));
    }
    tree.add_child(root, statusbar).unwrap();

    // Modal overlay attached to root
    let modal_scrim = tree.create_node();
    let screen_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    if let Some(node) = tree.get_mut(modal_scrim) {
        node.layer = UiLayer::Modal;
        node.computed_rect = screen_rect;
        node.set_style(
            Style::new()
                .position_absolute()
                .left(0.0)
                .top(0.0)
                .width(1920.0)
                .height(1080.0),
        );
    }
    tree.add_child(root, modal_scrim).unwrap();

    // Modal dialog card centered in window
    let modal_card = tree.create_node();
    let card_rect = Rect::new(720.0, 420.0, 480.0, 240.0);
    if let Some(node) = tree.get_mut(modal_card) {
        node.layer = UiLayer::Modal;
        node.computed_rect = card_rect;
    }
    tree.add_child(modal_scrim, modal_card).unwrap();

    let mut engine = LayoutEngine::new();
    assert!(
        engine
            .compute_layout(&mut tree, Size::new(1920.0, 1080.0))
            .is_ok()
    );

    // Menubar and statusbar are positioned by SpaceBetween
    let menubar_rect = tree.get(menubar).unwrap().computed_rect;
    assert_eq!(menubar_rect.y, 0.0);
    assert_eq!(menubar_rect.height, 36.0);

    let statusbar_rect = tree.get(statusbar).unwrap().computed_rect;
    assert_eq!(statusbar_rect.y, 1056.0);
    assert_eq!(statusbar_rect.height, 24.0);

    // CRITICAL INVARIANT: Modal scrim and card coordinates were NOT pushed to bottom by SpaceBetween
    let scrim_rect = tree.get(modal_scrim).unwrap().computed_rect;
    assert_eq!(
        scrim_rect, screen_rect,
        "Modal scrim must retain window bounds and not be flexed to bottom"
    );

    let final_card_rect = tree.get(modal_card).unwrap().computed_rect;
    assert_eq!(
        final_card_rect, card_rect,
        "Modal card must retain centered coordinates (420.0), NOT bottom-of-screen (1000+)"
    );
}