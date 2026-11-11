// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests: Encapsulated SubtreeSession & Atomic Panel Rebuilding
//!
//! Verifies atomic lifecycle management for isolated subtree invalidation within
//! [`DualLayoutEliminator`], ensuring that external application layers do not directly
//! manipulate raw low-level [`SubtreeSession`] begin/finish operations or extract tree
//! structures via `std::mem::take`.

use super::dual_layout::{DualLayoutEliminator, SubtreeEngines};
use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
use iris_core::geometry::Rect;
use iris_core::style::Style;
use iris_core::{DirtyFlags, UiTree, WidgetId};
use iris_dock::UiNotifier;
use iris_layout::LayoutEngine;
use iris_text::TextSystem;
use iris_widgets::declarative::UiScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SessionTestTab {
    Viewport,
    Inspector,
    Console,
}

#[derive(Default)]
struct SessionMockContext {
    pub render_count: usize,
}

fn create_session_test_leaf(tree: &mut UiTree, rect: Rect) -> WidgetId {
    let id = tree.create_node();
    let mut scope = UiScope::new(tree, id);
    scope.configure_container(
        "SessionLeafContainer",
        Style::new().width(rect.width).height(rect.height),
        false,
    );
    if let Some(node) = tree.get_mut(id) {
        node.computed_rect = rect;
    }
    id
}

#[test]
fn test_atomic_rebuild_panel_success_and_boundary_sealing() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut registry = WorkbenchPanelRegistry::<SessionTestTab, SessionMockContext>::new();
    let mut context = SessionMockContext::default();

    let leaf_id = create_session_test_leaf(&mut tree, Rect::new(50.0, 50.0, 400.0, 600.0));

    registry.register_panel(SessionTestTab::Inspector, |scope, ctx| {
        ctx.render_count += 1;
        scope.button("Inspector Header");
        scope.button("Inspector Property");
    });

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let success = DualLayoutEliminator::rebuild_panel(
        &mut engines,
        &mut registry,
        &mut context,
        SessionTestTab::Inspector,
        leaf_id,
    )
    .expect("Atomic rebuild_panel must succeed");

    assert!(success, "Panel should have been successfully rebuilt");
    assert_eq!(context.render_count, 1);

    // Verify boundary sealing and paint flagging via engines.tree
    assert!(engines.tree.is_boundary(leaf_id));
    assert!(
        engines
            .tree
            .get(leaf_id)
            .unwrap()
            .dirty
            .contains(DirtyFlags::PAINT)
    );

    // Verify child generation under leaf_id
    let children = &engines.tree.get(leaf_id).unwrap().children;
    assert_eq!(children.len(), 2);
    let child0_rect = engines.tree.get(children[0]).unwrap().computed_rect;
    assert!(child0_rect.x >= 50.0);
    assert!(child0_rect.y >= 50.0);
}

#[test]
fn test_atomic_rebuild_panel_missing_or_zero_area_leaf() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut registry = WorkbenchPanelRegistry::<SessionTestTab, SessionMockContext>::new();
    let mut context = SessionMockContext::default();

    registry.register_panel(SessionTestTab::Console, |scope, _| {
        scope.button("Log Output");
    });

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);

    // 1. Missing node: Null/default WidgetId that does not exist in tree
    let missing_id = WidgetId::default();
    let missing_result = DualLayoutEliminator::rebuild_panel(
        &mut engines,
        &mut registry,
        &mut context,
        SessionTestTab::Console,
        missing_id,
    )
    .expect("Missing leaf query must not panic and must return Ok(false)");
    assert!(!missing_result);

    // 2. Collapsed zero-area node: width = 0.0, height = 0.0
    let collapsed_id = create_session_test_leaf(engines.tree, Rect::new(0.0, 0.0, 0.0, 0.0));
    let collapsed_result = DualLayoutEliminator::rebuild_panel(
        &mut engines,
        &mut registry,
        &mut context,
        SessionTestTab::Console,
        collapsed_id,
    )
    .expect("Zero-area leaf query must safely return Ok(false)");
    assert!(!collapsed_result);
    assert_eq!(context.render_count, 0);
}

#[test]
fn test_atomic_rebuild_panel_propagates_registry_draw_error() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut registry = WorkbenchPanelRegistry::<SessionTestTab, SessionMockContext>::new();
    let mut context = SessionMockContext::default();

    let leaf_id = create_session_test_leaf(&mut tree, Rect::new(10.0, 10.0, 300.0, 300.0));

    // Notice: SessionTestTab::Viewport is NOT registered in registry!
    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let result = DualLayoutEliminator::rebuild_panel(
        &mut engines,
        &mut registry,
        &mut context,
        SessionTestTab::Viewport,
        leaf_id,
    );

    assert!(
        result.is_err(),
        "Unregistered panel draw must propagate error"
    );
}

#[test]
fn test_rebuild_leaf_panel_fallible_success_and_error_handling() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let leaf_id = create_session_test_leaf(&mut tree, Rect::new(0.0, 0.0, 500.0, 500.0));
    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);

    // 1. Successful fallible rebuild
    let ok_res = DualLayoutEliminator::rebuild_leaf_panel_fallible(
        &mut engines,
        leaf_id,
        |scope| -> Result<(), &'static str> {
            scope.button("Valid Item");
            Ok(())
        },
    );
    assert!(ok_res.is_ok());
    assert!(engines.tree.is_boundary(leaf_id));

    // 2. Aborted fallible rebuild returning an error
    let err_res = DualLayoutEliminator::rebuild_leaf_panel_fallible(
        &mut engines,
        leaf_id,
        |_scope| -> Result<(), &'static str> {
            Err("Simulated business validation failure during render")
        },
    );
    assert!(err_res.is_err());
}

#[test]
fn test_subtree_engines_convenience_methods() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut registry = WorkbenchPanelRegistry::<SessionTestTab, SessionMockContext>::new();
    let mut context = SessionMockContext::default();

    let leaf_id = create_session_test_leaf(&mut tree, Rect::new(100.0, 100.0, 400.0, 400.0));

    registry.register_panel(SessionTestTab::Inspector, |scope, ctx| {
        ctx.render_count += 1;
        scope.button("Inspector Tree Item");
    });

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);

    // Test engines.rebuild_panel
    let rebuilt = engines
        .rebuild_panel(
            &mut registry,
            &mut context,
            SessionTestTab::Inspector,
            leaf_id,
        )
        .expect("SubtreeEngines::rebuild_panel must succeed");
    assert!(rebuilt);
    assert_eq!(context.render_count, 1);

    // Test engines.rebuild_leaf
    let leaf_rebuilt = engines.rebuild_leaf(leaf_id, |scope| {
        scope.button("Direct Rebuilt Item");
    });
    assert!(leaf_rebuilt.is_ok());
}

#[test]
fn test_atomic_rebuild_panel_clears_old_children_completely() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut registry = WorkbenchPanelRegistry::<SessionTestTab, SessionMockContext>::new();
    let mut context = SessionMockContext::default();

    let leaf_id = create_session_test_leaf(&mut tree, Rect::new(0.0, 0.0, 400.0, 500.0));

    // First generation: 3 buttons
    registry.register_panel(SessionTestTab::Console, |scope, _| {
        scope.button("Log 1");
        scope.button("Log 2");
        scope.button("Log 3");
    });

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let first_run = engines
        .rebuild_panel(
            &mut registry,
            &mut context,
            SessionTestTab::Console,
            leaf_id,
        )
        .expect("First run must succeed");
    assert!(first_run);

    let initial_children: Vec<WidgetId> = engines.tree.get(leaf_id).unwrap().children.clone();
    assert_eq!(initial_children.len(), 3);

    // Second generation: Replace registration with only 1 button
    registry.register_panel(SessionTestTab::Console, |scope, _| {
        scope.button("Fresh Single Log");
    });

    let second_run = engines
        .rebuild_panel(
            &mut registry,
            &mut context,
            SessionTestTab::Console,
            leaf_id,
        )
        .expect("Second run must succeed");
    assert!(second_run);

    let current_children = &engines.tree.get(leaf_id).unwrap().children;
    assert_eq!(current_children.len(), 1);

    // Verify all old children were untracked and purged from tree arena
    for old_child_id in initial_children {
        assert!(
            engines.tree.get(old_child_id).is_none(),
            "Old child {old_child_id:?} must be purged from tree arena"
        );
        assert!(
            !engines.layout_engine.is_tracked(old_child_id),
            "Old child {old_child_id:?} must be untracked from layout engine"
        );
    }

    assert_eq!(
        engines.layout_engine.taffy_node_count(),
        engines.layout_engine.len(),
        "Taffy physical node count must match layout engine tracked count exactly"
    );
}

#[test]
fn test_dual_layout_needs_text_prepare_lifecycle() {
    let mut eliminator = DualLayoutEliminator::new();
    assert!(
        !eliminator.needs_text_prepare(),
        "Initial state must not need text prepare"
    );

    eliminator.mark_needs_text_prepare();
    assert!(eliminator.needs_text_prepare());

    eliminator.clear_needs_text_prepare();
    assert!(!eliminator.needs_text_prepare());

    eliminator.mark_needs_text_prepare();
    assert!(eliminator.take_needs_text_prepare());
    assert!(
        !eliminator.needs_text_prepare(),
        "take_needs_text_prepare must reset flag to false"
    );
}

#[test]
fn test_dual_layout_rebuild_dirty_panels_triggers_needs_text_prepare() {
    use iris_core::DirtyFlags;
    use iris_dock::UiNotifier;

    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let leaf_id = create_session_test_leaf(&mut tree, Rect::new(0.0, 0.0, 300.0, 200.0));

    // Initial Branch 1 layout synchronization
    let initial_leaves = [(SessionTestTab::Console, leaf_id)];
    let init_metrics = DualLayoutEliminator::synchronize_all_leaves(
        &mut tree,
        &mut layout_engine,
        &mut text_system,
        &initial_leaves,
    )
    .expect("Initial synchronization must succeed");
    assert!(init_metrics.needs_text_prepare);

    let mut registry = WorkbenchPanelRegistry::<SessionTestTab, SessionMockContext>::new();
    let mut context = SessionMockContext { render_count: 0 };
    registry.register_panel(SessionTestTab::Console, |scope, ctx| {
        ctx.render_count += 1;
        scope.button("Text Element");
    });

    let mut notifier = UiNotifier::<SessionTestTab>::clean();
    let mut eliminator = DualLayoutEliminator::new();
    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);

    // 1. Clean run: no panels dirty -> 0 rebuilt, needs_text_prepare remains false
    let zero_count = eliminator
        .rebuild_dirty_panels(
            &mut engines,
            &mut notifier,
            &mut registry,
            &mut context,
            |_tab| Some(leaf_id),
        )
        .expect("Rebuild with clean notifier must succeed");
    assert_eq!(zero_count, 0);
    assert!(!eliminator.needs_text_prepare());

    // 2. Mark panel dirty in notifier -> 1 rebuilt, needs_text_prepare becomes true
    notifier.tag_redraw(SessionTestTab::Console);
    assert!(notifier.is_dirty(SessionTestTab::Console));

    let rebuilt_count = eliminator
        .rebuild_dirty_panels(
            &mut engines,
            &mut notifier,
            &mut registry,
            &mut context,
            |_tab| Some(leaf_id),
        )
        .expect("Rebuild with dirty notifier must succeed");
    assert_eq!(rebuilt_count, 1);
    assert_eq!(context.render_count, 1);
    assert!(!notifier.is_dirty(SessionTestTab::Console));

    // Verify autonomous text prepare flag is raised
    assert!(
        eliminator.needs_text_prepare(),
        "Rebuilding dirty panels must autonomously flag needs_text_prepare"
    );

    // Verify leaf node has both PAINT and TEXT dirty flags
    let leaf_node = engines.tree.get(leaf_id).expect("Leaf node must exist");
    assert!(leaf_node.dirty.contains(DirtyFlags::PAINT));
    assert!(leaf_node.dirty.contains(DirtyFlags::TEXT));
    assert!(engines.tree.is_boundary(leaf_id));

    // Consuming flag resets it
    assert!(eliminator.take_needs_text_prepare());
    assert!(!eliminator.needs_text_prepare());
}

#[test]
fn test_subtree_engines_rebuild_panel_with_eliminator_and_text_flag() {
    use iris_core::DirtyFlags;

    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let leaf_id = create_session_test_leaf(&mut tree, Rect::new(0.0, 0.0, 400.0, 300.0));

    let mut registry = WorkbenchPanelRegistry::<SessionTestTab, SessionMockContext>::new();
    let mut context = SessionMockContext { render_count: 0 };
    registry.register_panel(SessionTestTab::Console, |scope, ctx| {
        ctx.render_count += 1;
        scope.button("Log Viewer");
    });

    let mut eliminator = DualLayoutEliminator::new();
    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);

    let success = engines
        .rebuild_panel_with_eliminator(
            &mut eliminator,
            &mut registry,
            &mut context,
            SessionTestTab::Console,
            leaf_id,
        )
        .expect("rebuild_panel_with_eliminator must succeed");

    assert!(success);
    assert!(eliminator.needs_text_prepare());
    assert!(engines.tree.is_boundary(leaf_id));
    assert!(
        engines
            .tree
            .get(leaf_id)
            .unwrap()
            .dirty
            .contains(DirtyFlags::TEXT)
    );
}

#[test]
fn test_synchronize_all_leaves_reports_needs_text_prepare() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let leaf_id = create_session_test_leaf(&mut tree, Rect::new(0.0, 0.0, 500.0, 400.0));

    let leaves = [(SessionTestTab::Console, leaf_id)];
    let metrics = DualLayoutEliminator::synchronize_all_leaves(
        &mut tree,
        &mut layout_engine,
        &mut text_system,
        &leaves,
    )
    .expect("synchronize_all_leaves must succeed");

    assert_eq!(metrics.synchronized_count, 1);
    assert!(
        metrics.needs_text_prepare,
        "Synchronizing leaves must raise needs_text_prepare in LayoutSyncMetrics"
    );
}

#[test]
fn test_rebuild_panel_taffy_node_cleanup_zero_leak_50_iterations() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut registry = WorkbenchPanelRegistry::<SessionTestTab, SessionMockContext>::new();
    let mut context = SessionMockContext::default();

    let leaf_id = create_session_test_leaf(&mut tree, Rect::new(0.0, 0.0, 600.0, 800.0));

    // Register a panel that constructs nested structure: rows with buttons
    registry.register_panel(SessionTestTab::Inspector, |scope, _| {
        scope.row(|row| {
            row.button("Btn 1");
            row.button("Btn 2");
        });
        scope.row(|row| {
            row.button("Btn 3");
            row.button("Btn 4");
        });
    });

    let mut previous_children: Vec<WidgetId> = Vec::new();
    let mut expected_node_count = 0;

    for iteration in 0..50 {
        let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
        let success = DualLayoutEliminator::rebuild_panel(
            &mut engines,
            &mut registry,
            &mut context,
            SessionTestTab::Inspector,
            leaf_id,
        )
        .expect("rebuild_panel must succeed");
        assert!(success);

        // Verify that all children from the previous iteration were completely purged
        for old_id in &previous_children {
            assert!(
                engines.tree.get(*old_id).is_none(),
                "Old node {old_id:?} must be purged from tree on iteration {iteration}"
            );
            assert!(
                !engines.layout_engine.is_tracked(*old_id),
                "Old node {old_id:?} must be untracked from layout engine on iteration {iteration}"
            );
        }

        // Mathematical invariant: physical Taffy node count must exactly equal tracked layout nodes
        let current_len = engines.layout_engine.len();
        let current_taffy = engines.layout_engine.taffy_node_count();
        assert_eq!(
            current_taffy, current_len,
            "Taffy physical count ({current_taffy}) must equal engine len ({current_len}) on iteration {iteration}"
        );

        if iteration == 0 {
            expected_node_count = current_len;
            assert!(
                expected_node_count > 1,
                "Must have tracked leaf plus nested children"
            );
        } else {
            // Absolute zero-leak invariant: tracked node count must remain constant across all iterations
            assert_eq!(
                current_len, expected_node_count,
                "Tracked layout nodes leaked! Expected {expected_node_count}, got {current_len} on iteration {iteration}"
            );
        }

        // Record current children for next iteration's verification
        previous_children.clear();
        if let Some(leaf_node) = engines.tree.get(leaf_id) {
            for &child_id in &leaf_node.children {
                previous_children.push(child_id);
                engines
                    .tree
                    .collect_subtree(child_id, &mut previous_children);
            }
        }
    }
}

#[test]
fn test_rebuild_dirty_panels_multi_panel_taffy_leak_free() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut registry = WorkbenchPanelRegistry::<SessionTestTab, SessionMockContext>::new();
    let mut context = SessionMockContext::default();
    let mut eliminator = DualLayoutEliminator::new();
    let mut notifier = UiNotifier::<SessionTestTab>::clean();

    let leaf_inspector = create_session_test_leaf(&mut tree, Rect::new(0.0, 0.0, 400.0, 600.0));
    let leaf_console = create_session_test_leaf(&mut tree, Rect::new(400.0, 0.0, 400.0, 600.0));

    registry.register_panel(SessionTestTab::Inspector, |scope, _| {
        scope.row(|row| {
            row.button("Insp A");
            row.button("Insp B");
        });
    });

    registry.register_panel(SessionTestTab::Console, |scope, _| {
        scope.row(|row| {
            row.button("Console Line 1");
            row.button("Console Line 2");
            row.button("Console Line 3");
        });
    });

    // Initial rebuild of both panels
    notifier.tag_redraw(SessionTestTab::Inspector);
    notifier.tag_redraw(SessionTestTab::Console);

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let initial_rebuilt = eliminator
        .rebuild_dirty_panels(
            &mut engines,
            &mut notifier,
            &mut registry,
            &mut context,
            |tab| match tab {
                SessionTestTab::Inspector => Some(leaf_inspector),
                SessionTestTab::Console => Some(leaf_console),
                _ => None,
            },
        )
        .expect("initial rebuild must succeed");
    assert_eq!(initial_rebuilt, 2);

    let baseline_len = engines.layout_engine.len();
    assert_eq!(engines.layout_engine.taffy_node_count(), baseline_len);

    // Run 20 cycles alternating between marking Inspector, Console, or both
    for cycle in 0..20 {
        match cycle % 3 {
            0 => notifier.tag_redraw(SessionTestTab::Inspector),
            1 => notifier.tag_redraw(SessionTestTab::Console),
            _ => {
                notifier.tag_redraw(SessionTestTab::Inspector);
                notifier.tag_redraw(SessionTestTab::Console);
            }
        }

        let rebuilt = eliminator
            .rebuild_dirty_panels(
                &mut engines,
                &mut notifier,
                &mut registry,
                &mut context,
                |tab| match tab {
                    SessionTestTab::Inspector => Some(leaf_inspector),
                    SessionTestTab::Console => Some(leaf_console),
                    _ => None,
                },
            )
            .expect("rebuild must succeed");
        assert!(rebuilt > 0);

        // Verification: Layout engine tracked nodes and physical Taffy nodes remain invariant and leak-free
        assert_eq!(
            engines.layout_engine.taffy_node_count(),
            engines.layout_engine.len(),
            "Taffy physical count must equal engine len on cycle {cycle}"
        );
        assert_eq!(
            engines.layout_engine.len(),
            baseline_len,
            "Layout engine node count leaked on cycle {cycle}"
        );
    }
}