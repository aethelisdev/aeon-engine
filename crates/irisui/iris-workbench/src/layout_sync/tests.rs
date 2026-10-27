// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use iris_core::geometry::Rect;
use iris_core::style::Style;
use iris_core::{DirtyFlags, UiTree};
use iris_layout::LayoutEngine;
use iris_text::TextSystem;
use iris_widgets::declarative::UiScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestTab {
    Hierarchy,
    Inspector,
    Assets,
}

fn create_test_leaf(tree: &mut UiTree, rect: Rect) -> iris_core::WidgetId {
    let id = tree.create_node();
    let mut scope = UiScope::new(tree, id);
    scope.configure_container(
        "LeafContainer",
        Style::new().width(rect.width).height(rect.height),
        false,
    );
    if let Some(node) = tree.get_mut(id) {
        node.computed_rect = rect;
    }
    id
}

#[test]
fn test_synchronize_dock_leaves_seals_boundaries_and_marks_paint() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let leaf1 = create_test_leaf(&mut tree, Rect::new(0.0, 0.0, 300.0, 400.0));
    let leaf2 = create_test_leaf(&mut tree, Rect::new(300.0, 0.0, 300.0, 400.0));

    // Populate leaf1 with a child
    {
        let mut scope = UiScope::new(&mut tree, leaf1);
        scope.button("Click Me");
    }

    let leaves = [(TestTab::Hierarchy, leaf1), (TestTab::Inspector, leaf2)];
    let metrics = synchronize_dock_leaves(&mut tree, &mut layout_engine, &mut text_system, &leaves)
        .expect("Layout synchronization should succeed");

    assert_eq!(metrics.synchronized_count, 2);
    assert_eq!(metrics.skipped_count, 0);

    assert!(tree.is_boundary(leaf1));
    assert!(tree.is_boundary(leaf2));
    assert!(tree.get(leaf1).unwrap().dirty.contains(DirtyFlags::PAINT));
    assert!(tree.get(leaf2).unwrap().dirty.contains(DirtyFlags::PAINT));
}

#[test]
fn test_synchronize_dock_leaves_skips_zero_size_containers() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let leaf_valid = create_test_leaf(&mut tree, Rect::new(0.0, 0.0, 300.0, 400.0));
    // Collapsed splitter container with 0.0 width
    let leaf_collapsed = create_test_leaf(&mut tree, Rect::new(300.0, 0.0, 0.0, 400.0));

    let leaves = [
        (TestTab::Hierarchy, leaf_valid),
        (TestTab::Assets, leaf_collapsed),
    ];
    let metrics = synchronize_dock_leaves(&mut tree, &mut layout_engine, &mut text_system, &leaves)
        .expect("Layout synchronization should handle collapsed panels gracefully");

    assert_eq!(metrics.synchronized_count, 1);
    assert_eq!(metrics.skipped_count, 1);
    assert!(tree.is_boundary(leaf_valid));
}

#[test]
fn test_rebuild_leaf_panel_preserves_boundary_and_replaces_content() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let leaf_id = create_test_leaf(&mut tree, Rect::new(0.0, 0.0, 320.0, 480.0));

    // Initial content
    {
        let mut scope = UiScope::new(&mut tree, leaf_id);
        scope.button("Initial Button");
    }

    assert_eq!(tree.get(leaf_id).unwrap().children.len(), 1);

    // Branch 2 In-place rebuild
    DualLayoutEliminator::rebuild_leaf_panel(
        &mut tree,
        &mut layout_engine,
        &mut text_system,
        leaf_id,
        |scope| {
            scope.button("New Button 1");
            scope.button("New Button 2");
        },
    )
    .expect("Rebuilding leaf panel should succeed");

    assert_eq!(tree.get(leaf_id).unwrap().children.len(), 2);
    assert!(tree.is_boundary(leaf_id));
    assert!(tree.get(leaf_id).unwrap().dirty.contains(DirtyFlags::PAINT));
}

#[test]
fn test_dual_layout_coordinate_invariance_zero_jitter() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let leaf_id = create_test_leaf(&mut tree, Rect::new(100.0, 50.0, 300.0, 500.0));

    // Dal 1: Initial creation + DualLayoutEliminator synchronization pass
    {
        let mut scope = UiScope::new(&mut tree, leaf_id);
        scope.button("Invariant Button");
    }

    let leaves = [(TestTab::Inspector, leaf_id)];
    DualLayoutEliminator::synchronize_all_leaves(
        &mut tree,
        &mut layout_engine,
        &mut text_system,
        &leaves,
    )
    .expect("Dal 1 synchronization must succeed");

    let child_id = tree.get(leaf_id).unwrap().children[0];
    let dal1_rect = tree.get(child_id).unwrap().computed_rect;

    // Dal 2: In-place partial subtree rebuild using the exact same declarative UI closure
    DualLayoutEliminator::rebuild_leaf_panel(
        &mut tree,
        &mut layout_engine,
        &mut text_system,
        leaf_id,
        |scope| {
            scope.button("Invariant Button");
        },
    )
    .expect("Dal 2 rebuild must succeed");

    let new_child_id = tree.get(leaf_id).unwrap().children[0];
    let dal2_rect = tree.get(new_child_id).unwrap().computed_rect;

    // Zero-Jitter Invariant Verification (Delta <= 0.001px)
    assert!(
        (dal1_rect.x - dal2_rect.x).abs() < 1e-3,
        "X coordinate jitter detected: Dal1={}, Dal2={}",
        dal1_rect.x,
        dal2_rect.x
    );
    assert!(
        (dal1_rect.y - dal2_rect.y).abs() < 1e-3,
        "Y coordinate jitter detected: Dal1={}, Dal2={}",
        dal1_rect.y,
        dal2_rect.y
    );
    assert!(
        (dal1_rect.width - dal2_rect.width).abs() < 1e-3,
        "Width jitter detected: Dal1={}, Dal2={}",
        dal1_rect.width,
        dal2_rect.width
    );
    assert!(
        (dal1_rect.height - dal2_rect.height).abs() < 1e-3,
        "Height jitter detected: Dal1={}, Dal2={}",
        dal1_rect.height,
        dal2_rect.height
    );
}