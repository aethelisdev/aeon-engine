// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
use iris_core::geometry::Rect;
use iris_core::style::Style;
use iris_core::{DirtyFlags, UiTree};
use iris_dock::UiNotifier;
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

#[derive(Debug, Default)]
struct MockLayoutContext {
    rebuild_counter: u32,
}

#[test]
fn test_rebuild_dirty_panels_clean_notifier_is_noop() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut notifier = UiNotifier::<TestTab>::clean();
    let mut registry = WorkbenchPanelRegistry::<TestTab, MockLayoutContext>::new();
    let mut context = MockLayoutContext::default();
    let mut eliminator = DualLayoutEliminator::new();
    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);

    let count = eliminator
        .rebuild_dirty_panels(
            &mut engines,
            &mut notifier,
            &mut registry,
            &mut context,
            |_| None,
        )
        .expect("Clean notifier rebuild must succeed with zero operations");

    assert_eq!(count, 0);
    assert_eq!(context.rebuild_counter, 0);
}

#[test]
fn test_rebuild_dirty_panels_reconstructs_only_dirty_panels_and_clears_notifier() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut notifier = UiNotifier::<TestTab>::clean();
    let mut registry = WorkbenchPanelRegistry::<TestTab, MockLayoutContext>::new();
    let mut context = MockLayoutContext::default();

    let leaf_hier = create_test_leaf(&mut tree, Rect::new(0.0, 0.0, 300.0, 600.0));
    let leaf_insp = create_test_leaf(&mut tree, Rect::new(300.0, 0.0, 300.0, 600.0));
    let leaf_assets = create_test_leaf(&mut tree, Rect::new(600.0, 0.0, 400.0, 600.0));

    // Register 3 panel renderers
    registry.register_panel(TestTab::Hierarchy, |scope, ctx| {
        ctx.rebuild_counter += 1;
        scope.button("Hierarchy Tree");
    });
    registry.register_panel(TestTab::Inspector, |scope, ctx| {
        ctx.rebuild_counter += 10;
        scope.button("Inspector Transform");
    });
    registry.register_panel(TestTab::Assets, |scope, ctx| {
        ctx.rebuild_counter += 100;
        scope.button("Asset Grid");
    });

    // Initial shell synchronization
    let initial_leaves = [
        (TestTab::Hierarchy, leaf_hier),
        (TestTab::Inspector, leaf_insp),
        (TestTab::Assets, leaf_assets),
    ];
    DualLayoutEliminator::synchronize_all_leaves(
        &mut tree,
        &mut layout_engine,
        &mut text_system,
        &initial_leaves,
    )
    .expect("Initial sync must succeed");

    // Only Inspector is flagged dirty
    notifier.tag_redraw(TestTab::Inspector);
    assert!(notifier.is_dirty(TestTab::Inspector));
    assert!(!notifier.is_dirty(TestTab::Hierarchy));
    assert!(!notifier.is_dirty(TestTab::Assets));

    let mut eliminator = DualLayoutEliminator::new();
    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let rebuilt_count = eliminator
        .rebuild_dirty_panels(
            &mut engines,
            &mut notifier,
            &mut registry,
            &mut context,
            |tab| match tab {
                TestTab::Hierarchy => Some(leaf_hier),
                TestTab::Inspector => Some(leaf_insp),
                TestTab::Assets => Some(leaf_assets),
            },
        )
        .expect("Granular rebuild of dirty panel must succeed");

    assert_eq!(rebuilt_count, 1);
    assert_eq!(
        context.rebuild_counter, 10,
        "Only Inspector closure should have been executed"
    );
    assert!(!notifier.is_dirty(TestTab::Inspector));
    assert!(!notifier.is_any_dirty());

    // Verify Inspector was re-sealed as boundary and marked for paint
    assert!(tree.is_boundary(leaf_insp));
    assert!(
        tree.get(leaf_insp)
            .unwrap()
            .dirty
            .contains(DirtyFlags::PAINT)
    );
}

#[test]
fn test_rebuild_dirty_panels_missing_or_collapsed_leaf_gracefully_clears_notifier() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut notifier = UiNotifier::<TestTab>::clean();
    let mut registry = WorkbenchPanelRegistry::<TestTab, MockLayoutContext>::new();
    let mut context = MockLayoutContext::default();

    // Leaf collapsed to 0 width
    let collapsed_leaf = create_test_leaf(&mut tree, Rect::new(0.0, 0.0, 0.0, 400.0));

    registry.register_panel(TestTab::Hierarchy, |scope, _| {
        scope.button("Hierarchy");
    });
    registry.register_panel(TestTab::Assets, |scope, _| {
        scope.button("Assets");
    });

    // Tag both collapsed and unresolvable panels as dirty
    notifier.tag_redraw(TestTab::Hierarchy);
    notifier.tag_redraw(TestTab::Assets);
    assert!(notifier.is_any_dirty());

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let rebuilt_count = DualLayoutEliminator::rebuild_dirty_panels_direct(
        &mut engines,
        &mut notifier,
        &mut registry,
        &mut context,
        |tab| match tab {
            TestTab::Hierarchy => Some(collapsed_leaf),
            TestTab::Assets => None, // Unresolvable / closed tab
            _ => None,
        },
    )
    .expect("Rebuild with collapsed and missing panels must not error");

    // Both panels skipped gracefully and cleared from notifier
    assert_eq!(rebuilt_count, 0);
    assert!(!notifier.is_any_dirty());
}

#[test]
fn test_om_taffy_subtree_bypass_and_sibling_invariance() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut notifier = UiNotifier::<TestTab>::clean();
    let mut registry = WorkbenchPanelRegistry::<TestTab, MockLayoutContext>::new();
    let mut context = MockLayoutContext::default();

    // 3 distinct dock panels across screen space
    let leaf_left = create_test_leaf(&mut tree, Rect::new(0.0, 30.0, 300.0, 700.0));
    let leaf_center = create_test_leaf(&mut tree, Rect::new(300.0, 30.0, 1200.0, 700.0));
    let leaf_right = create_test_leaf(&mut tree, Rect::new(1500.0, 30.0, 420.0, 700.0));

    registry.register_panel(TestTab::Hierarchy, |scope, _| {
        scope.button("Scene Hierarchy Root");
    });
    registry.register_panel(TestTab::Assets, |scope, _| {
        scope.button("Assets Directory Browser");
    });
    registry.register_panel(TestTab::Inspector, |scope, _| {
        scope.button("Transform X");
        scope.button("Transform Y");
    });

    // 1. Initial shell synchronization pass (Dal 1)
    let leaves = [
        (TestTab::Hierarchy, leaf_left),
        (TestTab::Assets, leaf_center),
        (TestTab::Inspector, leaf_right),
    ];
    DualLayoutEliminator::synchronize_all_leaves(
        &mut tree,
        &mut layout_engine,
        &mut text_system,
        &leaves,
    )
    .expect("Initial synchronization pass must succeed");

    // Initial build of all panels to establish baseline subtree geometry
    notifier.tag_redraw(TestTab::Hierarchy);
    notifier.tag_redraw(TestTab::Assets);
    notifier.tag_redraw(TestTab::Inspector);
    let mut eliminator = DualLayoutEliminator::new();
    let mut initial_engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let initial_count = eliminator
        .rebuild_dirty_panels(
            &mut initial_engines,
            &mut notifier,
            &mut registry,
            &mut context,
            |tab| match tab {
                TestTab::Hierarchy => Some(leaf_left),
                TestTab::Assets => Some(leaf_center),
                TestTab::Inspector => Some(leaf_right),
            },
        )
        .expect("Initial build pass must succeed");
    assert_eq!(initial_count, 3);
    assert!(!notifier.is_any_dirty());

    // Capture baseline child geometry of sibling panels
    let left_child_id = tree.get(leaf_left).unwrap().children[0];
    let left_baseline_rect = tree.get(left_child_id).unwrap().computed_rect;

    let center_child_id = tree.get(leaf_center).unwrap().children[0];
    let center_baseline_rect = tree.get(center_child_id).unwrap().computed_rect;

    // 2. Invalidate ONLY the Right Panel (Inspector)
    notifier.tag_redraw(TestTab::Inspector);
    assert!(notifier.is_dirty(TestTab::Inspector));
    assert!(!notifier.is_dirty(TestTab::Hierarchy));
    assert!(!notifier.is_dirty(TestTab::Assets));

    // 3. Execute isolated O(M) subtree rebuild
    let mut eliminator = DualLayoutEliminator::new();
    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let count = eliminator
        .rebuild_dirty_panels(
            &mut engines,
            &mut notifier,
            &mut registry,
            &mut context,
            |tab| match tab {
                TestTab::Hierarchy => Some(leaf_left),
                TestTab::Assets => Some(leaf_center),
                TestTab::Inspector => Some(leaf_right),
            },
        )
        .expect("Subtree rebuild must succeed");

    assert_eq!(count, 1);
    assert!(!notifier.is_any_dirty());

    // 4. O(N) BYPASS VERIFICATION: Sibling containers and children remain 100% untouched
    let left_current_rect = tree.get(left_child_id).unwrap().computed_rect;
    assert_eq!(
        left_baseline_rect, left_current_rect,
        "Left panel children must experience zero mutation during sibling subtree rebuild"
    );

    let center_current_rect = tree.get(center_child_id).unwrap().computed_rect;
    assert_eq!(
        center_baseline_rect, center_current_rect,
        "Center panel children must experience zero mutation during sibling subtree rebuild"
    );

    // 5. O(M) SUBTREE GLOBAL COORDINATE VERIFICATION
    let right_children = &tree.get(leaf_right).unwrap().children;
    assert_eq!(right_children.len(), 2);

    let child0_rect = tree.get(right_children[0]).unwrap().computed_rect;
    let child1_rect = tree.get(right_children[1]).unwrap().computed_rect;

    // Child 0 must start at or past the right leaf screen X origin (1500.0)
    assert!(
        child0_rect.x >= 1500.0,
        "Child 0 global X ({}) must be >= leaf X (1500.0)",
        child0_rect.x
    );
    assert!(
        child0_rect.y >= 30.0,
        "Child 0 global Y ({}) must be >= leaf Y (30.0)",
        child0_rect.y
    );

    // Child 1 must be positioned to the right of Child 0 (Row layout default)
    assert!(
        child1_rect.x >= child0_rect.x + child0_rect.width,
        "Child 1 global X ({}) must be >= Child 0 right ({})",
        child1_rect.x,
        child0_rect.x + child0_rect.width
    );
    assert_eq!(
        child1_rect.y, child0_rect.y,
        "Row children share identical global Y"
    );

    // Both children must stay within the right panel's 420.0px width
    assert!(child0_rect.x + child0_rect.width <= 1500.0 + 420.0 + 1e-2);
    assert!(child1_rect.x + child1_rect.width <= 1500.0 + 420.0 + 1e-2);
}

#[test]
fn test_om_taffy_subtree_nested_hierarchy_global_offsets() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut notifier = UiNotifier::<TestTab>::clean();
    let mut registry = WorkbenchPanelRegistry::<TestTab, MockLayoutContext>::new();
    let mut context = MockLayoutContext::default();

    // Leaf container positioned at non-trivial screen coordinates
    let leaf_offset = create_test_leaf(&mut tree, Rect::new(450.0, 120.0, 500.0, 400.0));

    registry.register_panel(TestTab::Inspector, |scope, _| {
        scope.row(|row| {
            row.button("SubBtn1");
            row.button("SubBtn2");
        });
    });

    notifier.tag_redraw(TestTab::Inspector);

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let count = DualLayoutEliminator::rebuild_dirty_panels_direct(
        &mut engines,
        &mut notifier,
        &mut registry,
        &mut context,
        |tab| match tab {
            TestTab::Inspector => Some(leaf_offset),
            _ => None,
        },
    )
    .expect("Rebuild with nested flexbox hierarchy must succeed");

    assert_eq!(count, 1);

    // Inspect nested hierarchy global coordinates
    let row_container_id = tree.get(leaf_offset).unwrap().children[0];
    let row_rect = tree.get(row_container_id).unwrap().computed_rect;

    assert!(row_rect.x >= 450.0, "Row container X must be >= 450.0");
    assert!(row_rect.y >= 120.0, "Row container Y must be >= 120.0");

    let row_children = &tree.get(row_container_id).unwrap().children;
    assert_eq!(row_children.len(), 2);

    let btn1_rect = tree.get(row_children[0]).unwrap().computed_rect;
    let btn2_rect = tree.get(row_children[1]).unwrap().computed_rect;

    assert!(btn1_rect.x >= 450.0, "Btn1 X must be >= 450.0");
    assert!(btn1_rect.y >= 120.0, "Btn1 Y must be >= 120.0");

    // Horizontal flex-direction: Btn2 must be to the right of Btn1
    assert!(
        btn2_rect.x >= btn1_rect.x + btn1_rect.width,
        "Btn2 X ({}) must follow Btn1 right ({})",
        btn2_rect.x,
        btn1_rect.x + btn1_rect.width
    );
    assert_eq!(
        btn1_rect.y, btn2_rect.y,
        "Horizontally aligned buttons in row must share identical global Y"
    );
}

#[test]
fn test_om_taffy_subtree_scroll_offset_global_coordinate_adjustment() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut notifier = UiNotifier::<TestTab>::clean();
    let mut registry = WorkbenchPanelRegistry::<TestTab, MockLayoutContext>::new();
    let mut context = MockLayoutContext::default();

    // Leaf container at (200.0, 100.0) with a 60px scroll offset
    let leaf_scroll = create_test_leaf(&mut tree, Rect::new(200.0, 100.0, 400.0, 500.0));
    if let Some(node) = tree.get_mut(leaf_scroll) {
        node.style.scroll_offset_y = 60.0;
    }

    registry.register_panel(TestTab::Hierarchy, |scope, _| {
        scope.button("Scrolled Target Item");
    });

    notifier.tag_redraw(TestTab::Hierarchy);

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let count = DualLayoutEliminator::rebuild_dirty_panels_direct(
        &mut engines,
        &mut notifier,
        &mut registry,
        &mut context,
        |_| Some(leaf_scroll),
    )
    .expect("Rebuild with scroll offset must succeed");

    assert_eq!(count, 1);

    let child_id = tree.get(leaf_scroll).unwrap().children[0];
    let child_rect = tree.get(child_id).unwrap().computed_rect;

    // Child global Y must reflect (container_y - scroll_offset_y + local_y)
    // 100.0 - 60.0 = 40.0
    assert!(
        (child_rect.y - 40.0).abs() < 1e-2,
        "Child global Y ({}) must reflect scroll adjustment (expected ~40.0)",
        child_rect.y
    );
    assert!(
        child_rect.x >= 200.0,
        "Child global X must reflect container X (200.0)"
    );
}

#[test]
fn test_om_taffy_subtree_repeated_rebuild_zero_drift() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut notifier = UiNotifier::<TestTab>::clean();
    let mut registry = WorkbenchPanelRegistry::<TestTab, MockLayoutContext>::new();
    let mut context = MockLayoutContext::default();

    let leaf_id = create_test_leaf(&mut tree, Rect::new(120.0, 80.0, 360.0, 480.0));

    registry.register_panel(TestTab::Hierarchy, |scope, ctx| {
        ctx.rebuild_counter += 1;
        scope.button("Drift Invariant Button");
    });

    let mut initial_rect = Rect::default();

    for i in 0..6 {
        notifier.tag_redraw(TestTab::Hierarchy);

        let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
        let count = DualLayoutEliminator::rebuild_dirty_panels_direct(
            &mut engines,
            &mut notifier,
            &mut registry,
            &mut context,
            |_| Some(leaf_id),
        )
        .expect("Repeated subtree rebuild must succeed");

        assert_eq!(count, 1);

        let child_id = tree.get(leaf_id).unwrap().children[0];
        let curr_rect = tree.get(child_id).unwrap().computed_rect;

        if i == 0 {
            initial_rect = curr_rect;
        } else {
            assert!(
                (initial_rect.x - curr_rect.x).abs() < 1e-3,
                "Iteration {}: X drifted: {} vs {}",
                i,
                initial_rect.x,
                curr_rect.x
            );
            assert!(
                (initial_rect.y - curr_rect.y).abs() < 1e-3,
                "Iteration {}: Y drifted: {} vs {}",
                i,
                initial_rect.y,
                curr_rect.y
            );
            assert!(
                (initial_rect.width - curr_rect.width).abs() < 1e-3,
                "Iteration {}: Width drifted",
                i
            );
            assert!(
                (initial_rect.height - curr_rect.height).abs() < 1e-3,
                "Iteration {}: Height drifted",
                i
            );
        }
    }

    assert_eq!(context.rebuild_counter, 6);
}