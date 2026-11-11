// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Workbench Shell Lifecycle Integration Test Suite
//!
//! Validates the autonomous application shell rendering and update pipeline:
//! - Full retained shell assembly (`render_shell`) without manual engine-side plumbing.
//! - Autonomous 3-branch lifecycle transitions (`update_shell`) driving $O(M)$ Taffy subtree solves.
//! - Granular dock leaf invalidation routing via [`UiNotifier`].
//! - Deterministic shell rebuild forcing via [`Workbench::force_shell_rebuild`].

use iris_core::tree::UiTree;
use iris_dock::UiNotifier;
use iris_dock::state::DockState;
use iris_dock::tree::DockTree;
use iris_layout::LayoutEngine;
use iris_text::TextSystem;
use iris_workbench::descriptors::panel_registry::WorkbenchPanelRegistry;
use iris_workbench::descriptors::workbench::WorkbenchDescriptor;
use iris_workbench::layout_sync::dual_layout::SubtreeEngines;
use iris_workbench::lifecycle::{LifecycleBranch, LifecycleInputs};
use iris_workbench::workbench::runner::Workbench;

/// Mock panel identifier enum for IDE workspace testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestPanelId {
    Viewport,
    Inspector,
}

/// Mock application state shared across panels.
#[derive(Debug, Default)]
struct TestAppState {
    viewport_draws: usize,
    inspector_draws: usize,
}

/// Mock decoupled action enum emitted by shell widgets.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TestAction {}

#[test]
fn test_autonomous_shell_lifecycle_branches() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut notifier = UiNotifier::<TestPanelId>::new();
    let mut context = TestAppState::default();

    // 1. Initialize workbench with 2 docked tabs
    let mut dock_tree = DockTree::new();
    let root_leaf = dock_tree.create_leaf(vec![TestPanelId::Viewport, TestPanelId::Inspector]);
    dock_tree.set_root(root_leaf);

    let descriptor = WorkbenchDescriptor::new("Test Workbench", DockState::new(dock_tree));
    let mut workbench = Workbench::<TestPanelId, TestAction, TestAppState>::new(descriptor);

    // Register declarative panels with invocation counters
    let mut registry = WorkbenchPanelRegistry::<TestPanelId, TestAppState>::new();
    registry.register_panel(TestPanelId::Viewport, |_scope, ctx| {
        ctx.viewport_draws += 1;
    });
    registry.register_panel(TestPanelId::Inspector, |_scope, ctx| {
        ctx.inspector_draws += 1;
    });
    workbench.panel_registry = registry;

    let inputs = LifecycleInputs::new((1280.0, 720.0), 1.0, 1);

    // 2. Turn 1: Cold start -> Full Shell Rebuild (Branch 1)
    {
        let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
        let branch = workbench
            .update_shell(&mut engines, &mut notifier, &mut context, &inputs, false)
            .expect("Turn 1 shell update must succeed");

        assert_eq!(branch, LifecycleBranch::FullShellRebuild);
        assert!(workbench.active_dock_frame().is_some());
        assert_eq!(context.viewport_draws, 1);
        // Only active tab in the leaf is drawn initially
        assert_eq!(context.inspector_draws, 0);
    }

    // 3. Turn 2: Steady state without changes -> Dormant (Branch 3)
    {
        let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
        let branch = workbench
            .update_shell(&mut engines, &mut notifier, &mut context, &inputs, false)
            .expect("Turn 2 shell update must succeed");

        assert_eq!(branch, LifecycleBranch::Dormant);
        assert_eq!(context.viewport_draws, 1);
        assert_eq!(context.inspector_draws, 0);
    }

    // 4. Turn 3: Mark Viewport dirty via notifier -> Granular Subtree Invalidation (Branch 2)
    notifier.tag_redraw(TestPanelId::Viewport);
    {
        let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
        let branch = workbench
            .update_shell(&mut engines, &mut notifier, &mut context, &inputs, false)
            .expect("Turn 3 shell update must succeed");

        match branch {
            LifecycleBranch::GranularPanels(dirty_set) => {
                assert!(dirty_set.contains(&TestPanelId::Viewport));
            }
            other => panic!("Expected GranularPanels branch, got {other:?}"),
        }
        assert_eq!(context.viewport_draws, 2);
        assert_eq!(context.inspector_draws, 0);
    }

    // 5. Turn 4: Force shell rebuild -> Full Shell Rebuild (Branch 1)
    workbench.force_shell_rebuild();
    {
        let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
        let branch = workbench
            .update_shell(&mut engines, &mut notifier, &mut context, &inputs, false)
            .expect("Turn 4 shell update must succeed");

        assert_eq!(branch, LifecycleBranch::FullShellRebuild);
        assert_eq!(context.viewport_draws, 3);
    }
}