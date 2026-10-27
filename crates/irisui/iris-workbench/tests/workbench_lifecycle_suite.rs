// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris Workbench Isolated Integration Test Suite
//!
//! Comprehensive end-to-end simulation of a professional desktop creative IDE application:
//! - 3-Branch lifecycle state transitions (`FullShellRebuild`, `GranularPanels`, `InPlaceRefresh`, `Dormant`).
//! - Dirty panel tracking and isolated subtree reconstruction via [`DualLayoutEliminator`].
//! - Mathematical coordinate invariance ($\Delta \le 0.001\text{ px}$) between Branch 1 and Branch 2.
//! - Modal dialog interception, Escape dismissal, and child action propagation.
//! - Type-safe cross-panel drag-and-drop payload delivery.
//! - External texture aspect-ratio fitting and HUD overlay execution.

use iris_core::geometry::Rect;
use iris_core::style::Style;
use iris_core::{Color, DirtyFlags, UiTree, WidgetId};
use iris_layout::LayoutEngine;
use iris_text::TextSystem;
use iris_widgets::declarative::UiScope;
use iris_workbench::prelude::*;

/// Mock panel identifier enum representing desktop IDE layout tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MockTab {
    Viewport,
    Inspector,
}

/// Mock application state shared across panels and lifecycle evaluations.
#[derive(Debug, Default)]
struct MockContext {
    camera_fov: f32,
    frame_count: u64,
}

/// Mock asset drag-and-drop payload transferred between panels.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MockAssetPayload {
    asset_id: u64,
    asset_name: String,
    file_path: String,
}

/// Helper creating a mock dock leaf container with explicit computed bounds in the UI tree.
fn create_mock_leaf(tree: &mut UiTree, rect: Rect) -> WidgetId {
    let id = tree.create_node();
    let mut scope = UiScope::new(tree, id);
    scope.configure_container(
        "MockLeaf",
        Style::new().width(rect.width).height(rect.height),
        false,
    );
    if let Some(node) = tree.get_mut(id) {
        node.computed_rect = rect;
    }
    id
}

#[test]
fn test_cold_start_full_shell_assembly() {
    let mut lifecycle = WorkbenchLifecycle::<MockTab>::new();
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut context = MockContext {
        camera_fov: 60.0,
        ..Default::default()
    };

    // 1. Initial inputs: cold start must trigger FullShellRebuild
    let inputs = LifecycleInputs::new((1920.0, 1080.0), 1.0, 1).with_floating_count(0);

    let branch = lifecycle.evaluate(inputs);
    assert_eq!(branch, LifecycleBranch::FullShellRebuild);

    // 2. Build mock shell leaf containers
    let leaf_viewport = create_mock_leaf(&mut tree, Rect::new(0.0, 30.0, 1280.0, 720.0));
    let leaf_inspector = create_mock_leaf(&mut tree, Rect::new(1280.0, 30.0, 640.0, 720.0));

    // Register declarative panel draw functions
    let mut registry = WorkbenchPanelRegistry::<MockTab, MockContext>::new();
    registry.register_panel(MockTab::Viewport, |_scope, ctx| {
        ctx.frame_count += 1;
    });
    registry.register_panel(MockTab::Inspector, |scope, ctx| {
        scope.button("Reset Camera");
        ctx.camera_fov = 45.0;
    });

    // Draw panels into tree
    {
        let mut scope_vp = UiScope::new(&mut tree, leaf_viewport);
        let res = registry.draw(&MockTab::Viewport, &mut scope_vp, &mut context);
        assert!(res.is_ok(), "Viewport panel drawing must succeed");
    }
    {
        let mut scope_insp = UiScope::new(&mut tree, leaf_inspector);
        let res = registry.draw(&MockTab::Inspector, &mut scope_insp, &mut context);
        assert!(res.is_ok(), "Inspector panel drawing must succeed");
    }

    assert_eq!(context.frame_count, 1);
    assert_eq!(context.camera_fov, 45.0);

    // 3. Run automated DualLayoutEliminator synchronization
    let leaves = [
        (MockTab::Viewport, leaf_viewport),
        (MockTab::Inspector, leaf_inspector),
    ];
    let metrics = DualLayoutEliminator::synchronize_all_leaves(
        &mut tree,
        &mut layout_engine,
        &mut text_system,
        &leaves,
    )
    .expect("Cold start leaf synchronization must succeed");

    assert_eq!(metrics.synchronized_count, 2);
    assert_eq!(metrics.skipped_count, 0);

    // Verify boundary sealing and paint dirty marking
    assert!(tree.is_boundary(leaf_viewport));
    assert!(tree.is_boundary(leaf_inspector));
    assert!(
        tree.get(leaf_viewport)
            .unwrap()
            .dirty
            .contains(DirtyFlags::PAINT)
    );
    assert!(
        tree.get(leaf_inspector)
            .unwrap()
            .dirty
            .contains(DirtyFlags::PAINT)
    );
}

#[test]
fn test_steady_state_dormant_frame() {
    let mut lifecycle = WorkbenchLifecycle::<MockTab>::new();

    // Cold start pass
    let inputs1 = LifecycleInputs::new((1920.0, 1080.0), 1.0, 1);
    let _ = lifecycle.evaluate(inputs1);

    // Identical consecutive frame with no mutations
    let inputs2 = LifecycleInputs::new((1920.0, 1080.0), 1.0, 1);
    let branch = lifecycle.evaluate(inputs2);

    assert_eq!(branch, LifecycleBranch::Dormant);
}

#[test]
fn test_granular_panel_invalidation_branch_2() {
    let mut lifecycle = WorkbenchLifecycle::<MockTab>::new();
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    // Establish cold-start baseline
    let _ = lifecycle.evaluate(LifecycleInputs::new((1920.0, 1080.0), 1.0, 1));

    let leaf_inspector = create_mock_leaf(&mut tree, Rect::new(1280.0, 30.0, 640.0, 720.0));
    {
        let mut scope = UiScope::new(&mut tree, leaf_inspector);
        scope.button("Initial Button");
    }

    // Invalidate exclusively Inspector panel
    lifecycle.mark_tab_dirty(MockTab::Inspector);

    let inputs = LifecycleInputs::new((1920.0, 1080.0), 1.0, 1);
    let branch = lifecycle.evaluate(inputs);

    assert_eq!(
        branch,
        LifecycleBranch::GranularPanels(vec![MockTab::Inspector])
    );

    // Execute isolated Branch 2 leaf rebuild
    DualLayoutEliminator::rebuild_leaf_panel(
        &mut tree,
        &mut layout_engine,
        &mut text_system,
        leaf_inspector,
        |scope| {
            scope.button("Refreshed Button");
            scope.button("Another Field");
        },
    )
    .expect("Partial leaf rebuild must succeed");

    assert_eq!(tree.get(leaf_inspector).unwrap().children.len(), 2);
    assert!(tree.is_boundary(leaf_inspector));
    assert!(
        tree.get(leaf_inspector)
            .unwrap()
            .dirty
            .contains(DirtyFlags::PAINT)
    );

    // Next frame is dormant again
    let branch_next = lifecycle.evaluate(LifecycleInputs::new((1920.0, 1080.0), 1.0, 1));
    assert_eq!(branch_next, LifecycleBranch::Dormant);
}

#[test]
fn test_in_place_volatile_refresh_branch_3() {
    let mut lifecycle = WorkbenchLifecycle::<MockTab>::new();

    // Establish cold-start baseline
    let _ = lifecycle.evaluate(LifecycleInputs::new((1920.0, 1080.0), 1.0, 1));

    // Inputs declare high-frequency volatile telemetry update (144Hz HUD)
    let inputs = LifecycleInputs::new((1920.0, 1080.0), 1.0, 1).with_volatile_updates(true);
    let branch = lifecycle.evaluate(inputs);

    assert_eq!(branch, LifecycleBranch::InPlaceRefresh);
}

#[test]
fn test_e2e_zero_jitter_coordinate_invariance() {
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();

    let leaf_id = create_mock_leaf(&mut tree, Rect::new(200.0, 80.0, 400.0, 600.0));

    // Dal 1 construction + automated synchronization
    {
        let mut scope = UiScope::new(&mut tree, leaf_id);
        scope.button("Transform X");
    }

    let leaves = [(MockTab::Inspector, leaf_id)];
    DualLayoutEliminator::synchronize_all_leaves(
        &mut tree,
        &mut layout_engine,
        &mut text_system,
        &leaves,
    )
    .expect("Dal 1 synchronization must succeed");

    let child_id = tree.get(leaf_id).unwrap().children[0];
    let initial_rect = tree.get(child_id).unwrap().computed_rect;

    // Simulate 5 consecutive Dal 2 rebuilds
    for i in 0..5 {
        DualLayoutEliminator::rebuild_leaf_panel(
            &mut tree,
            &mut layout_engine,
            &mut text_system,
            leaf_id,
            |scope| {
                scope.button("Transform X");
            },
        )
        .expect("Dal 2 rebuild must succeed");

        let curr_child_id = tree.get(leaf_id).unwrap().children[0];
        let curr_rect = tree.get(curr_child_id).unwrap().computed_rect;

        assert!(
            (initial_rect.x - curr_rect.x).abs() < 1e-3,
            "Iteration {}: X jitter detected: {:?} vs {:?}",
            i,
            initial_rect.x,
            curr_rect.x
        );
        assert!(
            (initial_rect.y - curr_rect.y).abs() < 1e-3,
            "Iteration {}: Y jitter detected: {:?} vs {:?}",
            i,
            initial_rect.y,
            curr_rect.y
        );
        assert!(
            (initial_rect.width - curr_rect.width).abs() < 1e-3,
            "Iteration {}: Width jitter detected: {:?} vs {:?}",
            i,
            initial_rect.width,
            curr_rect.width
        );
        assert!(
            (initial_rect.height - curr_rect.height).abs() < 1e-3,
            "Iteration {}: Height jitter detected: {:?} vs {:?}",
            i,
            initial_rect.height,
            curr_rect.height
        );
    }
}

#[test]
fn test_e2e_modal_interception_workflow() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    let modal_desc = ModalDescriptor::new("Save Scene", 400.0, 220.0)
        .with_backdrop_color(Color::rgba(0.0, 0.0, 0.0, 0.70))
        .with_dismiss_on_escape(true)
        .with_dismiss_on_backdrop(true);

    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    // Frame 1: Modal active, no dismiss
    let outcome1 = ModalController::show_confirm_dialog(
        &mut scope,
        &modal_desc,
        viewport,
        false, // Escape not pressed
        "Do you want to save current changes?",
        "Save",
        false,
    );
    assert_eq!(outcome1, ModalOutcome::Active);

    // Frame 2: User presses Escape key -> Modal intercepted and dismissed
    let outcome2 = ModalController::show_confirm_dialog(
        &mut scope,
        &modal_desc,
        viewport,
        true, // Escape key pressed
        "Do you want to save current changes?",
        "Save",
        false,
    );
    assert_eq!(
        outcome2,
        ModalOutcome::Dismissed(ModalDismissReason::EscapeKey)
    );
    assert!(outcome2.is_dismissed());
}

#[test]
fn test_e2e_drag_drop_cross_panel_carrier() {
    // 1. Asset Browser packages a drag-and-drop payload
    let original_payload = MockAssetPayload {
        asset_id: 1042,
        asset_name: "HeroModel.glb".to_string(),
        file_path: "assets/models/HeroModel.glb".to_string(),
    };

    let typed_carrier =
        TypedDragDropPayload::new(original_payload.clone(), "HeroModel.glb", "AssetBrowser");

    // Convert into dynamic carrier for cross-panel transport
    let dynamic_carrier = typed_carrier.into_erased();
    assert_eq!(dynamic_carrier.preview_label(), "HeroModel.glb");
    assert_eq!(dynamic_carrier.source_panel(), "AssetBrowser");
    assert!(dynamic_carrier.is::<MockAssetPayload>());
    assert!(!dynamic_carrier.is::<u64>());

    // 2. Viewport receives payload and attempts typed downcast
    let downcast_ref = dynamic_carrier.downcast_ref::<MockAssetPayload>();
    assert_eq!(downcast_ref, Some(&original_payload));

    // 3. Viewport unpacks and recovers full ownership via downcast
    let recovered_box = dynamic_carrier
        .downcast::<MockAssetPayload>()
        .expect("Type-safe recovery must succeed");

    assert_eq!(*recovered_box, original_payload);
}

#[test]
fn test_e2e_external_texture_hook_simulation() {
    // Viewport resizing to ultra-wide (aspect ratio: 32:9)
    let hook_ultra_wide = ExternalTextureHook::new(777, Rect::new(0.0, 0.0, 2560.0, 720.0))
        .with_aspect_ratio(16.0 / 9.0);
    let fitted_pillarbox = hook_ultra_wide.fit_aspect_ratio();

    // Expect pillarbox (black bars on left and right)
    assert_eq!(fitted_pillarbox.height, 720.0);
    assert_eq!(fitted_pillarbox.width, 720.0 * (16.0 / 9.0));
    assert!(fitted_pillarbox.x > 0.0); // centered horizontally
    assert_eq!(fitted_pillarbox.y, 0.0);

    // Viewport resizing to portrait (aspect ratio: 9:16)
    let hook_portrait = ExternalTextureHook::new(777, Rect::new(0.0, 0.0, 720.0, 1280.0))
        .with_aspect_ratio(16.0 / 9.0);
    let fitted_letterbox = hook_portrait.fit_aspect_ratio();

    // Expect letterbox (black bars on top and bottom)
    assert_eq!(fitted_letterbox.width, 720.0);
    assert_eq!(fitted_letterbox.height, 720.0 / (16.0 / 9.0));
    assert_eq!(fitted_letterbox.x, 0.0);
    assert!(fitted_letterbox.y > 0.0); // centered vertically
}