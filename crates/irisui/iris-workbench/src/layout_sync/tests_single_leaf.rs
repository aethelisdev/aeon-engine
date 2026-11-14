// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests: SubtreeEngines & DualLayoutEliminator rebuild_single_panel_leaf
//!
//! Verifies atomic, generational partial leaf panel reconstruction through [`SubtreeEngines`]
//! and [`DualLayoutEliminator::rebuild_single_panel_leaf`]. Guarantees zero generational node
//! leakage, correct Taffy layout solving, boundary sealing, and notifier state clearing.

use super::dual_layout::{DualLayoutEliminator, SubtreeEngines};
use iris_core::geometry::Rect;
use iris_core::style::Style;
use iris_core::{DirtyFlags, UiTree, WidgetId};
use iris_dock::UiNotifier;
use iris_layout::LayoutEngine;
use iris_text::TextSystem;
use iris_widgets::declarative::UiScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestPanelId {
    Inspector,
}

fn create_test_leaf_container(tree: &mut UiTree, rect: Rect) -> WidgetId {
    let id = tree.create_node();
    let mut scope = UiScope::new(tree, id);
    scope.configure_container(
        "SingleLeafContainer",
        Style::new().width(rect.width).height(rect.height),
        false,
    );
    if let Some(node) = tree.get_mut(id) {
        node.computed_rect = rect;
    }
    id
}

#[test]
fn test_rebuild_single_panel_leaf_success_seals_boundary_and_replaces_children() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let leaf_id = create_test_leaf_container(&mut tree, Rect::new(10.0, 20.0, 320.0, 480.0));

    // Seed initial child nodes to prove they are completely purged during subtree rebuild
    {
        let mut scope = UiScope::new(&mut tree, leaf_id);
        scope.button("Initial Stale Child 1");
        scope.button("Initial Stale Child 2");
    }
    assert_eq!(tree.get(leaf_id).unwrap().children.len(), 2);

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let rebuilt = engines
        .rebuild_single_panel_leaf(leaf_id, |scope| {
            scope.button("Fresh Button 1");
            scope.button("Fresh Button 2");
            scope.button("Fresh Button 3");
        })
        .expect("Subtree reconstruction must succeed");

    assert!(
        rebuilt,
        "Active leaf container must report successful rebuild"
    );

    // Invariant: Invalidation Boundary must be sealed
    assert!(engines.tree.is_boundary(leaf_id));

    // Invariant: DirtyFlags must indicate PAINT and TEXT
    let leaf_node = engines.tree.get(leaf_id).unwrap();
    assert!(
        leaf_node
            .dirty
            .contains(DirtyFlags::PAINT | DirtyFlags::TEXT)
    );

    // Invariant: Children must contain only the 3 freshly rendered buttons
    assert_eq!(leaf_node.children.len(), 3);
    for &child_id in &leaf_node.children {
        let child_rect = engines.tree.get(child_id).unwrap().computed_rect;
        assert!(child_rect.x >= 10.0);
        assert!(child_rect.y >= 20.0);
    }
}

#[test]
fn test_rebuild_single_panel_leaf_missing_or_zero_area_safe_noop() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);

    // 1. Missing node returns Ok(false) without panic
    let bogus_leaf = WidgetId::default();
    let missing_res = engines
        .rebuild_single_panel_leaf(bogus_leaf, |scope| {
            scope.button("Should Not Run");
        })
        .expect("Missing leaf should not trigger layout error");
    assert!(!missing_res);

    // 2. Collapsed zero-area container returns Ok(false) without panic
    let collapsed_id = create_test_leaf_container(engines.tree, Rect::new(0.0, 0.0, 0.0, 0.0));
    let collapsed_res = engines
        .rebuild_single_panel_leaf(collapsed_id, |scope| {
            scope.button("Should Not Run");
        })
        .expect("Collapsed leaf should not trigger layout error");
    assert!(!collapsed_res);
}

#[test]
fn test_rebuild_single_panel_leaf_fallible_error_propagation() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let leaf_id = create_test_leaf_container(&mut tree, Rect::new(0.0, 0.0, 200.0, 300.0));
    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);

    // Fallible success case
    let ok_res = engines
        .rebuild_single_panel_leaf_fallible(leaf_id, |scope| {
            scope.button("Valid Button");
            Ok::<(), &'static str>(())
        })
        .expect("Valid fallible closure must succeed");
    assert!(ok_res);

    // Fallible error case: verifies exact error string propagation
    let err_res = engines.rebuild_single_panel_leaf_fallible(leaf_id, |_scope| {
        Err::<(), &'static str>("SyntheticPanelRenderError")
    });

    assert!(
        err_res.is_err(),
        "Closure error must propagate as LayoutError"
    );
    let err_msg = err_res.unwrap_err().to_string();
    assert!(err_msg.contains("SyntheticPanelRenderError"));
}

#[test]
fn test_rebuild_single_panel_leaf_notified_clears_notifier_and_flags_text_prepare() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut notifier = UiNotifier::<TestPanelId>::clean();
    let mut eliminator = DualLayoutEliminator::new();

    let leaf_id = create_test_leaf_container(&mut tree, Rect::new(0.0, 0.0, 250.0, 350.0));
    notifier.tag_redraw(TestPanelId::Inspector);
    assert!(notifier.is_dirty(TestPanelId::Inspector));
    assert!(!eliminator.needs_text_prepare());

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let rebuilt = eliminator
        .rebuild_single_panel_leaf_notified(
            &mut engines,
            &mut notifier,
            TestPanelId::Inspector,
            leaf_id,
            |scope| {
                scope.button("Inspector Content");
            },
        )
        .expect("Notified single leaf rebuild must succeed");

    assert!(rebuilt);
    assert!(
        !notifier.is_dirty(TestPanelId::Inspector),
        "Dirty notifier tag must be cleared on successful leaf rebuild"
    );
    assert!(
        eliminator.needs_text_prepare(),
        "needs_text_prepare must be flagged on eliminator"
    );
}

#[test]
fn test_rebuild_single_panel_leaf_zero_layout_leak_repeated_cycles() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let leaf_id = create_test_leaf_container(&mut tree, Rect::new(0.0, 0.0, 400.0, 500.0));
    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);

    // Initial rebuild
    engines
        .rebuild_single_panel_leaf(leaf_id, |scope| {
            scope.button("Action 1");
            scope.button("Action 2");
        })
        .unwrap();

    let baseline_len = engines.layout_engine.len();
    let baseline_taffy = engines.layout_engine.taffy_node_count();
    assert_eq!(baseline_len, baseline_taffy);

    // Run 30 consecutive partial rebuild cycles
    for cycle in 0..30 {
        engines
            .rebuild_single_panel_leaf(leaf_id, |scope| {
                scope.button("Action 1");
                scope.button("Action 2");
            })
            .unwrap();

        assert_eq!(
            engines.layout_engine.len(),
            baseline_len,
            "LayoutEngine node count leaked on cycle {cycle}"
        );
        assert_eq!(
            engines.layout_engine.taffy_node_count(),
            baseline_taffy,
            "Taffy physical node count leaked on cycle {cycle}"
        );
    }
}