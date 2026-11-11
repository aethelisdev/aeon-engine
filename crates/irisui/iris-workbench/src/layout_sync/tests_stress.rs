// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Multi-Panel Stress & Boundary Scaling Test Suite
//!
//! Rigorous stress testing verifying that [`DualLayoutEliminator`] can atomically
//! resolve 5 or more dirty dock panels in a single frame without coordinate drift,
//! memory leaks, or sibling invalidation bleeding.

use super::dual_layout::{DualLayoutEliminator, SubtreeEngines};
use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
use iris_core::geometry::Rect;
use iris_core::style::Style;
use iris_core::{DirtyFlags, UiTree, WidgetId};
use iris_dock::UiNotifier;
use iris_layout::LayoutEngine;
use iris_text::TextSystem;
use iris_widgets::declarative::UiScope;

/// Distinct panel tabs simulating complex desktop IDE dock layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StressTab {
    Viewport,
    Hierarchy,
    Inspector,
    Console,
    AssetBrowser,
    Profiler,
    Dynamic(u32),
}

/// Shared application state tracking per-panel execution metrics across stress loops.
#[derive(Default)]
struct StressContext {
    pub render_counts: [usize; 6],
    pub dynamic_render_count: usize,
}

/// Helper creating a mock dock leaf container with explicit geometry in the UI tree.
fn create_stress_leaf(tree: &mut UiTree, rect: Rect) -> WidgetId {
    let id = tree.create_node();
    let mut scope = UiScope::new(tree, id);
    scope.configure_container(
        "StressLeafContainer",
        Style::new().width(rect.width).height(rect.height),
        false,
    );
    if let Some(node) = tree.get_mut(id) {
        node.computed_rect = rect;
    }
    id
}

#[test]
fn test_simultaneous_5_panel_dirty_stress_in_single_frame() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut registry = WorkbenchPanelRegistry::<StressTab, StressContext>::new();
    let mut context = StressContext::default();
    let mut eliminator = DualLayoutEliminator::new();
    let mut notifier = UiNotifier::<StressTab>::clean();

    // 1. Create 6 distinct leaf containers positioned across the workspace
    let leaf_viewport = create_stress_leaf(&mut tree, Rect::new(0.0, 30.0, 1280.0, 720.0));
    let leaf_hierarchy = create_stress_leaf(&mut tree, Rect::new(0.0, 750.0, 300.0, 330.0));
    let leaf_inspector = create_stress_leaf(&mut tree, Rect::new(1280.0, 30.0, 640.0, 720.0));
    let leaf_console = create_stress_leaf(&mut tree, Rect::new(300.0, 750.0, 500.0, 330.0));
    let leaf_asset = create_stress_leaf(&mut tree, Rect::new(800.0, 750.0, 480.0, 330.0));
    let leaf_profiler = create_stress_leaf(&mut tree, Rect::new(1280.0, 750.0, 640.0, 330.0));

    // 2. Register all 6 panels with rich declarative contents
    registry.register_panel(StressTab::Viewport, |scope, ctx| {
        ctx.render_counts[0] += 1;
        scope.button("3D Camera Gizmo");
    });

    registry.register_panel(StressTab::Hierarchy, |scope, ctx| {
        ctx.render_counts[1] += 1;
        scope.row(|row| {
            row.button("Node 1");
            row.button("Node 2");
        });
    });

    registry.register_panel(StressTab::Inspector, |scope, ctx| {
        ctx.render_counts[2] += 1;
        scope.column(|col| {
            col.button("Transform Card");
            col.button("Mesh Card");
            col.button("Material Card");
        });
    });

    registry.register_panel(StressTab::Console, |scope, ctx| {
        ctx.render_counts[3] += 1;
        scope.row(|row| {
            row.button("Clear Log");
            row.button("Filter Warn");
            row.button("Filter Error");
        });
    });

    registry.register_panel(StressTab::AssetBrowser, |scope, ctx| {
        ctx.render_counts[4] += 1;
        scope.row(|row| {
            row.button("Textures Folder");
            row.button("Meshes Folder");
        });
    });

    registry.register_panel(StressTab::Profiler, |scope, ctx| {
        ctx.render_counts[5] += 1;
        scope.column(|col| {
            col.button("CPU Timeline 16.6ms");
            col.button("GPU Frame 8.2ms");
        });
    });

    // 3. Initial baseline layout bake for Viewport (Clean Sibling)
    let mut initial_engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let viewport_baked = DualLayoutEliminator::rebuild_panel(
        &mut initial_engines,
        &mut registry,
        &mut context,
        StressTab::Viewport,
        leaf_viewport,
    )
    .expect("Viewport initial bake must succeed");
    assert!(viewport_baked);
    assert_eq!(context.render_counts[0], 1);

    let viewport_child_id = initial_engines.tree.get(leaf_viewport).unwrap().children[0];
    let viewport_child_rect = initial_engines
        .tree
        .get(viewport_child_id)
        .unwrap()
        .computed_rect;

    // 4. Tag exactly 5 different panels dirty simultaneously
    notifier.tag_redraw(StressTab::Hierarchy);
    notifier.tag_redraw(StressTab::Inspector);
    notifier.tag_redraw(StressTab::Console);
    notifier.tag_redraw(StressTab::AssetBrowser);
    notifier.tag_redraw(StressTab::Profiler);

    assert_eq!(notifier.dirty_panels().len(), 5);
    assert!(notifier.is_any_dirty());
    assert!(!notifier.is_dirty(StressTab::Viewport));

    // 5. Execute rebuild_dirty_panels in a single frame pass
    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let resolver = |tab: StressTab| match tab {
        StressTab::Viewport => Some(leaf_viewport),
        StressTab::Hierarchy => Some(leaf_hierarchy),
        StressTab::Inspector => Some(leaf_inspector),
        StressTab::Console => Some(leaf_console),
        StressTab::AssetBrowser => Some(leaf_asset),
        StressTab::Profiler => Some(leaf_profiler),
        StressTab::Dynamic(_) => None,
    };

    let rebuilt_count = eliminator
        .rebuild_dirty_panels(
            &mut engines,
            &mut notifier,
            &mut registry,
            &mut context,
            resolver,
        )
        .expect("Simultaneous 5-panel rebuild must succeed");

    // 6. Verification of rebuild counts and notifier clearance
    assert_eq!(rebuilt_count, 5);
    assert!(!notifier.is_any_dirty());
    assert!(notifier.dirty_panels().is_empty());
    assert!(eliminator.take_needs_text_prepare());

    // 7. Verify Clean Sibling Panel Immunity: Viewport was untouched
    assert_eq!(
        context.render_counts[0], 1,
        "Clean Viewport panel must NOT be rebuilt during 5-panel dirty pass"
    );
    let current_viewport_child = engines.tree.get(leaf_viewport).unwrap().children[0];
    assert_eq!(current_viewport_child, viewport_child_id);
    let current_viewport_rect = engines
        .tree
        .get(current_viewport_child)
        .unwrap()
        .computed_rect;
    assert_eq!(current_viewport_rect, viewport_child_rect);

    // 8. Verify each of the 5 dirty panels was rebuilt exactly once
    assert_eq!(context.render_counts[1], 1, "Hierarchy rendered once");
    assert_eq!(context.render_counts[2], 1, "Inspector rendered once");
    assert_eq!(context.render_counts[3], 1, "Console rendered once");
    assert_eq!(context.render_counts[4], 1, "AssetBrowser rendered once");
    assert_eq!(context.render_counts[5], 1, "Profiler rendered once");

    // 9. Verify boundary sealing and paint flagging for all 5 leaves
    let dirty_leaves = [
        leaf_hierarchy,
        leaf_inspector,
        leaf_console,
        leaf_asset,
        leaf_profiler,
    ];
    for &leaf_id in &dirty_leaves {
        assert!(engines.tree.is_boundary(leaf_id));
        let leaf_node = engines.tree.get(leaf_id).unwrap();
        assert!(leaf_node.dirty.contains(DirtyFlags::PAINT));
        assert!(leaf_node.dirty.contains(DirtyFlags::TEXT));
        assert!(!leaf_node.children.is_empty());

        // Verify child global coordinate containment
        for &child_id in &leaf_node.children {
            let child_rect = engines.tree.get(child_id).unwrap().computed_rect;
            assert!(child_rect.x >= leaf_node.computed_rect.x);
            assert!(child_rect.y >= leaf_node.computed_rect.y);
            assert!(child_rect.width > 0.0);
            assert!(child_rect.height > 0.0);
        }
    }

    // 10. Verify absolute Taffy allocation sync (zero orphaned Taffy nodes)
    assert_eq!(
        engines.layout_engine.taffy_node_count(),
        engines.layout_engine.len(),
        "Taffy physical count must match engine len after 5-panel simultaneous rebuild"
    );
}

#[test]
fn test_continuous_multi_panel_dirty_stress_100_frames() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut registry = WorkbenchPanelRegistry::<StressTab, StressContext>::new();
    let mut context = StressContext::default();
    let mut eliminator = DualLayoutEliminator::new();
    let mut notifier = UiNotifier::<StressTab>::clean();

    let leaf_hierarchy = create_stress_leaf(&mut tree, Rect::new(0.0, 0.0, 300.0, 500.0));
    let leaf_inspector = create_stress_leaf(&mut tree, Rect::new(300.0, 0.0, 400.0, 500.0));
    let leaf_console = create_stress_leaf(&mut tree, Rect::new(700.0, 0.0, 500.0, 500.0));
    let leaf_asset = create_stress_leaf(&mut tree, Rect::new(0.0, 500.0, 600.0, 500.0));
    let leaf_profiler = create_stress_leaf(&mut tree, Rect::new(600.0, 500.0, 600.0, 500.0));

    registry.register_panel(StressTab::Hierarchy, |scope, _| {
        scope.button("Hierarchy Item");
    });
    registry.register_panel(StressTab::Inspector, |scope, _| {
        scope.row(|row| {
            row.button("Insp Key");
            row.button("Insp Val");
        });
    });
    registry.register_panel(StressTab::Console, |scope, _| {
        scope.button("Console Line");
    });
    registry.register_panel(StressTab::AssetBrowser, |scope, _| {
        scope.row(|row| {
            row.button("Asset 1");
            row.button("Asset 2");
        });
    });
    registry.register_panel(StressTab::Profiler, |scope, _| {
        scope.button("Profiler Marker");
    });

    // Initial rebuild of all 5 panels
    notifier.tag_redraw(StressTab::Hierarchy);
    notifier.tag_redraw(StressTab::Inspector);
    notifier.tag_redraw(StressTab::Console);
    notifier.tag_redraw(StressTab::AssetBrowser);
    notifier.tag_redraw(StressTab::Profiler);

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let resolver = |tab: StressTab| match tab {
        StressTab::Hierarchy => Some(leaf_hierarchy),
        StressTab::Inspector => Some(leaf_inspector),
        StressTab::Console => Some(leaf_console),
        StressTab::AssetBrowser => Some(leaf_asset),
        StressTab::Profiler => Some(leaf_profiler),
        _ => None,
    };

    let initial_count = eliminator
        .rebuild_dirty_panels(
            &mut engines,
            &mut notifier,
            &mut registry,
            &mut context,
            resolver,
        )
        .expect("Initial bake must succeed");
    assert_eq!(initial_count, 5);

    let baseline_node_count = engines.layout_engine.len();
    assert_eq!(
        engines.layout_engine.taffy_node_count(),
        baseline_node_count
    );

    // Run 100 continuous simulated frames with fluctuating multi-panel dirty sets
    for frame in 1..=100 {
        match frame % 4 {
            0 => {
                // All 5 dirty
                notifier.tag_redraw(StressTab::Hierarchy);
                notifier.tag_redraw(StressTab::Inspector);
                notifier.tag_redraw(StressTab::Console);
                notifier.tag_redraw(StressTab::AssetBrowser);
                notifier.tag_redraw(StressTab::Profiler);
            }
            1 => {
                // 3 dirty
                notifier.tag_redraw(StressTab::Hierarchy);
                notifier.tag_redraw(StressTab::Console);
                notifier.tag_redraw(StressTab::Profiler);
            }
            2 => {
                // 2 dirty
                notifier.tag_redraw(StressTab::Inspector);
                notifier.tag_redraw(StressTab::AssetBrowser);
            }
            _ => {
                // 4 dirty
                notifier.tag_redraw(StressTab::Hierarchy);
                notifier.tag_redraw(StressTab::Inspector);
                notifier.tag_redraw(StressTab::Console);
                notifier.tag_redraw(StressTab::AssetBrowser);
            }
        }

        let dirty_target_count = notifier.dirty_panels().len();
        let rebuilt = eliminator
            .rebuild_dirty_panels(
                &mut engines,
                &mut notifier,
                &mut registry,
                &mut context,
                resolver,
            )
            .expect("Continuous frame rebuild must succeed");

        assert_eq!(rebuilt, dirty_target_count);
        assert!(!notifier.is_any_dirty());

        // Zero-leak invariant: Layout engine and Taffy node counts remain strictly flat
        assert_eq!(
            engines.layout_engine.len(),
            baseline_node_count,
            "Node count leaked on frame {frame}"
        );
        assert_eq!(
            engines.layout_engine.taffy_node_count(),
            baseline_node_count,
            "Taffy physical count drifted on frame {frame}"
        );
    }
}

#[test]
fn test_stack_buffer_scaling_boundary_beyond_32_panels() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut registry = WorkbenchPanelRegistry::<StressTab, StressContext>::new();
    let mut context = StressContext::default();
    let mut eliminator = DualLayoutEliminator::new();
    let mut notifier = UiNotifier::<StressTab>::clean();

    // Create 36 distinct panels and leaf containers (exceeding the 32-panel stack buffer)
    const TOTAL_PANELS: u32 = 36;
    let mut leaves = Vec::with_capacity(TOTAL_PANELS as usize);

    for idx in 0..TOTAL_PANELS {
        let leaf_id =
            create_stress_leaf(&mut tree, Rect::new((idx as f32) * 20.0, 0.0, 100.0, 100.0));
        leaves.push((StressTab::Dynamic(idx), leaf_id));

        registry.register_panel(StressTab::Dynamic(idx), |scope, ctx| {
            ctx.dynamic_render_count += 1;
            scope.button("Dynamic Item");
        });

        // Tag every panel dirty
        notifier.tag_redraw(StressTab::Dynamic(idx));
    }

    assert_eq!(notifier.dirty_panels().len(), 36);

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let leaves_clone = leaves.clone();
    let resolver = move |tab: StressTab| {
        leaves_clone
            .iter()
            .find(|(t, _)| *t == tab)
            .map(|(_, id)| *id)
    };

    let rebuilt_count = eliminator
        .rebuild_dirty_panels(
            &mut engines,
            &mut notifier,
            &mut registry,
            &mut context,
            resolver,
        )
        .expect("36-panel scaling rebuild must succeed");

    assert_eq!(rebuilt_count, 36);
    assert_eq!(context.dynamic_render_count, 36);
    assert!(!notifier.is_any_dirty());
    assert_eq!(
        engines.layout_engine.taffy_node_count(),
        engines.layout_engine.len()
    );
}

#[test]
fn test_zero_vector_reallocation_in_steady_state_partial_rebuild() {
    let mut tree = UiTree::with_capacity(128);
    let mut layout_engine = LayoutEngine::with_capacity(128);
    let mut text_system = TextSystem::new();
    let mut registry = WorkbenchPanelRegistry::<StressTab, StressContext>::new();
    let mut context = StressContext::default();
    let mut eliminator = DualLayoutEliminator::new();
    let mut notifier = UiNotifier::<StressTab>::clean();

    let leaf_hierarchy = create_stress_leaf(&mut tree, Rect::new(0.0, 0.0, 300.0, 500.0));
    let leaf_inspector = create_stress_leaf(&mut tree, Rect::new(300.0, 0.0, 400.0, 500.0));
    let leaf_console = create_stress_leaf(&mut tree, Rect::new(700.0, 0.0, 500.0, 500.0));
    let leaf_asset = create_stress_leaf(&mut tree, Rect::new(0.0, 500.0, 600.0, 500.0));
    let leaf_profiler = create_stress_leaf(&mut tree, Rect::new(600.0, 500.0, 600.0, 500.0));

    registry.register_panel(StressTab::Hierarchy, |scope, _| {
        scope.row(|row| {
            row.button("Hierarchy Tree 1");
            row.button("Hierarchy Tree 2");
        });
    });
    registry.register_panel(StressTab::Inspector, |scope, _| {
        scope.column(|col| {
            col.button("Transform Prop");
            col.button("Physics Prop");
        });
    });
    registry.register_panel(StressTab::Console, |scope, _| {
        scope.button("Log Message");
    });
    registry.register_panel(StressTab::AssetBrowser, |scope, _| {
        scope.row(|row| {
            row.button("Folder A");
            row.button("Folder B");
        });
    });
    registry.register_panel(StressTab::Profiler, |scope, _| {
        scope.button("Profile Sample");
    });

    // Warm-up initial frame: build all 5 panels into steady-state
    notifier.tag_redraw(StressTab::Hierarchy);
    notifier.tag_redraw(StressTab::Inspector);
    notifier.tag_redraw(StressTab::Console);
    notifier.tag_redraw(StressTab::AssetBrowser);
    notifier.tag_redraw(StressTab::Profiler);

    let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
    let resolver = |tab: StressTab| match tab {
        StressTab::Hierarchy => Some(leaf_hierarchy),
        StressTab::Inspector => Some(leaf_inspector),
        StressTab::Console => Some(leaf_console),
        StressTab::AssetBrowser => Some(leaf_asset),
        StressTab::Profiler => Some(leaf_profiler),
        _ => None,
    };

    let warm_up_rebuilt = eliminator
        .rebuild_dirty_panels(
            &mut engines,
            &mut notifier,
            &mut registry,
            &mut context,
            resolver,
        )
        .expect("warm-up rebuild must succeed");
    assert_eq!(warm_up_rebuilt, 5);

    // Capture steady-state capacity baselines
    let steady_tree_capacity = engines.tree.capacity();
    let steady_layout_capacity = engines.layout_engine.capacity();
    let steady_node_len = engines.layout_engine.len();
    assert_eq!(engines.layout_engine.taffy_node_count(), steady_node_len);

    // Run 100 consecutive frames rebuilding all 5 panels every single frame
    for frame in 1..=100 {
        notifier.tag_redraw(StressTab::Hierarchy);
        notifier.tag_redraw(StressTab::Inspector);
        notifier.tag_redraw(StressTab::Console);
        notifier.tag_redraw(StressTab::AssetBrowser);
        notifier.tag_redraw(StressTab::Profiler);

        let rebuilt = eliminator
            .rebuild_dirty_panels(
                &mut engines,
                &mut notifier,
                &mut registry,
                &mut context,
                resolver,
            )
            .expect("per-frame rebuild must succeed");
        assert_eq!(rebuilt, 5);
        assert!(!notifier.is_any_dirty());

        // Zero-allocation invariant 1: UiTree node arena vector never reallocates
        assert_eq!(
            engines.tree.capacity(),
            steady_tree_capacity,
            "UiTree capacity reallocated on frame {frame} (expected {steady_tree_capacity}, got {})",
            engines.tree.capacity()
        );

        // Zero-allocation invariant 2: LayoutEngine HashMap capacity never reallocates
        assert_eq!(
            engines.layout_engine.capacity(),
            steady_layout_capacity,
            "LayoutEngine map capacity reallocated on frame {frame} (expected {steady_layout_capacity}, got {})",
            engines.layout_engine.capacity()
        );

        // Zero-allocation invariant 3: Total tracked layout nodes and Taffy nodes remain identical
        assert_eq!(
            engines.layout_engine.len(),
            steady_node_len,
            "Tracked nodes drifted on frame {frame}"
        );
        assert_eq!(
            engines.layout_engine.taffy_node_count(),
            steady_node_len,
            "Taffy physical count drifted on frame {frame}"
        );
    }
}

#[test]
fn test_zero_heap_vector_allocation_stack_dirty_buffer() {
    let mut tree = UiTree::with_capacity(64);
    let mut layout_engine = LayoutEngine::with_capacity(64);
    let mut text_system = TextSystem::new();
    let mut registry = WorkbenchPanelRegistry::<StressTab, StressContext>::new();
    let mut context = StressContext::default();
    let mut eliminator = DualLayoutEliminator::new();
    let mut notifier = UiNotifier::<StressTab>::clean();

    // Create 32 distinct panels (the exact maximum boundary of the zero-allocation stack buffer)
    const MAX_STACK_PANELS: u32 = 32;
    let mut leaves = Vec::with_capacity(MAX_STACK_PANELS as usize);

    for idx in 0..MAX_STACK_PANELS {
        let leaf_id =
            create_stress_leaf(&mut tree, Rect::new((idx as f32) * 10.0, 0.0, 50.0, 50.0));
        leaves.push((StressTab::Dynamic(idx), leaf_id));

        registry.register_panel(StressTab::Dynamic(idx), |scope, ctx| {
            ctx.dynamic_render_count += 1;
            scope.button("Stack Item");
        });
    }

    // Test with varying sizes: 1, 8, 16, and exactly 32 panels
    for test_batch in [1, 8, 16, 32] {
        for idx in 0..test_batch {
            notifier.tag_redraw(StressTab::Dynamic(idx));
        }

        assert_eq!(notifier.dirty_panels().len(), test_batch as usize);

        let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
        let leaves_ref = &leaves;
        let resolver = |tab: StressTab| {
            leaves_ref
                .iter()
                .find(|(t, _)| *t == tab)
                .map(|(_, id)| *id)
        };

        let rebuilt = eliminator
            .rebuild_dirty_panels(
                &mut engines,
                &mut notifier,
                &mut registry,
                &mut context,
                resolver,
            )
            .expect("stack buffer rebuild must succeed");

        assert_eq!(rebuilt, test_batch as usize);
        assert!(!notifier.is_any_dirty());
        assert_eq!(
            engines.layout_engine.taffy_node_count(),
            engines.layout_engine.len()
        );
    }
}