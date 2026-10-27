// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestPanel {
    Viewport,
    Hierarchy,
    Inspector,
    Console,
}

#[test]
fn test_cold_start_initial_evaluation_forces_full_shell_rebuild() {
    let mut lifecycle = WorkbenchLifecycle::<TestPanel>::new();
    let inputs = LifecycleInputs::new((1920.0, 1080.0), 1.0, 0);

    assert!(lifecycle.is_shell_dirty(&inputs));
    assert_eq!(
        lifecycle.peek_branch(&inputs),
        LifecycleBranch::FullShellRebuild
    );

    let branch = lifecycle.evaluate(inputs);
    assert_eq!(branch, LifecycleBranch::FullShellRebuild);
}

#[test]
fn test_identical_consecutive_inputs_yields_dormant_branch() {
    let mut lifecycle = WorkbenchLifecycle::<TestPanel>::new();
    let inputs = LifecycleInputs::new((1920.0, 1080.0), 1.0, 0);

    // Initial pass clears cold-start dirty flags
    let _ = lifecycle.evaluate(inputs);

    // Second turn with identical environmental inputs
    let second_turn = lifecycle.evaluate(inputs);
    assert_eq!(second_turn, LifecycleBranch::Dormant);
    assert!(!lifecycle.is_shell_dirty(&inputs));
}

#[test]
fn test_window_resize_triggers_full_shell_rebuild() {
    let mut lifecycle = WorkbenchLifecycle::<TestPanel>::new();
    let inputs_initial = LifecycleInputs::new((1920.0, 1080.0), 1.0, 0);
    let _ = lifecycle.evaluate(inputs_initial);

    let inputs_resized = LifecycleInputs::new((1600.0, 900.0), 1.0, 0);
    assert!(lifecycle.is_shell_dirty(&inputs_resized));

    let branch = lifecycle.evaluate(inputs_resized);
    assert_eq!(branch, LifecycleBranch::FullShellRebuild);

    // Re-evaluating with same resized dimensions returns to Dormant
    assert_eq!(lifecycle.evaluate(inputs_resized), LifecycleBranch::Dormant);
}

#[test]
fn test_dpi_scale_factor_alteration_triggers_full_shell_rebuild() {
    let mut lifecycle = WorkbenchLifecycle::<TestPanel>::new();
    let inputs_1x = LifecycleInputs::new((1920.0, 1080.0), 1.0, 0);
    let _ = lifecycle.evaluate(inputs_1x);

    let inputs_1_25x = LifecycleInputs::new((1920.0, 1080.0), 1.25, 0);
    assert!(lifecycle.is_shell_dirty(&inputs_1_25x));

    let branch = lifecycle.evaluate(inputs_1_25x);
    assert_eq!(branch, LifecycleBranch::FullShellRebuild);
}

#[test]
fn test_dock_revision_and_floating_count_mutations() {
    let mut lifecycle = WorkbenchLifecycle::<TestPanel>::new();
    let inputs_base = LifecycleInputs::new((1920.0, 1080.0), 1.0, 0);
    let _ = lifecycle.evaluate(inputs_base);

    // 1. Revision bump (tab split/close)
    let inputs_split = LifecycleInputs::new((1920.0, 1080.0), 1.0, 1);
    assert!(lifecycle.is_shell_dirty(&inputs_split));
    assert_eq!(
        lifecycle.evaluate(inputs_split),
        LifecycleBranch::FullShellRebuild
    );

    // 2. Floating window created
    let inputs_floating = LifecycleInputs::new((1920.0, 1080.0), 1.0, 1).with_floating_count(1);
    assert!(lifecycle.is_shell_dirty(&inputs_floating));
    assert_eq!(
        lifecycle.evaluate(inputs_floating),
        LifecycleBranch::FullShellRebuild
    );
}

#[test]
fn test_menu_modal_and_dock_interaction_transitions() {
    let mut lifecycle = WorkbenchLifecycle::<TestPanel>::new();
    let inputs_base = LifecycleInputs::new((1920.0, 1080.0), 1.0, 0);
    let _ = lifecycle.evaluate(inputs_base);

    // 1. Menu open transition
    let inputs_menu = inputs_base.with_menu_active(true);
    assert!(lifecycle.is_shell_dirty(&inputs_menu));
    assert_eq!(
        lifecycle.evaluate(inputs_menu),
        LifecycleBranch::FullShellRebuild
    );

    // 2. Menu close transition
    assert!(lifecycle.is_shell_dirty(&inputs_base));
    assert_eq!(
        lifecycle.evaluate(inputs_base),
        LifecycleBranch::FullShellRebuild
    );

    // 3. Modal open transition
    let inputs_modal = inputs_base.with_modal_active(true);
    assert!(lifecycle.is_shell_dirty(&inputs_modal));
    assert_eq!(
        lifecycle.evaluate(inputs_modal),
        LifecycleBranch::FullShellRebuild
    );

    // 4. Modal close transition
    assert!(lifecycle.is_shell_dirty(&inputs_base));
    assert_eq!(
        lifecycle.evaluate(inputs_base),
        LifecycleBranch::FullShellRebuild
    );

    // 5. Active dock interaction (splitter dragging)
    let inputs_drag = inputs_base.with_dock_interacting(true);
    assert!(lifecycle.is_shell_dirty(&inputs_drag));
    assert_eq!(
        lifecycle.evaluate(inputs_drag),
        LifecycleBranch::FullShellRebuild
    );
}

#[test]
fn test_granular_panel_dirty_isolation() {
    let mut lifecycle = WorkbenchLifecycle::<TestPanel>::new();
    let inputs = LifecycleInputs::new((1920.0, 1080.0), 1.0, 0);
    let _ = lifecycle.evaluate(inputs);

    // Mark single panel dirty
    lifecycle.mark_tab_dirty(TestPanel::Inspector);
    assert!(lifecycle.is_tab_dirty(&TestPanel::Inspector));
    assert!(!lifecycle.is_tab_dirty(&TestPanel::Hierarchy));
    assert!(!lifecycle.is_shell_dirty(&inputs));

    let branch = lifecycle.evaluate(inputs);
    match branch {
        LifecycleBranch::GranularPanels(panels) => {
            assert_eq!(panels, vec![TestPanel::Inspector]);
        }
        other => panic!("Expected GranularPanels, got: {other:?}"),
    }

    // After evaluation, dirty tabs are drained and steady state returns to Dormant
    assert_eq!(lifecycle.evaluate(inputs), LifecycleBranch::Dormant);

    // Mark multiple panels dirty using remaining variants
    lifecycle.mark_tabs_dirty([TestPanel::Viewport, TestPanel::Hierarchy]);
    assert!(lifecycle.is_tab_dirty(&TestPanel::Viewport));
    assert!(lifecycle.is_tab_dirty(&TestPanel::Hierarchy));

    let multi_branch = lifecycle.evaluate(inputs);
    match multi_branch {
        LifecycleBranch::GranularPanels(panels) => {
            assert_eq!(panels.len(), 2);
            assert!(panels.contains(&TestPanel::Viewport));
            assert!(panels.contains(&TestPanel::Hierarchy));
        }
        other => panic!("Expected GranularPanels, got: {other:?}"),
    }
}

#[test]
fn test_full_shell_clears_pending_granular_tabs() {
    let mut lifecycle = WorkbenchLifecycle::<TestPanel>::new();
    let inputs_base = LifecycleInputs::new((1920.0, 1080.0), 1.0, 0);
    let _ = lifecycle.evaluate(inputs_base);

    // Mark panels dirty and simultaneously resize window
    lifecycle.mark_tab_dirty(TestPanel::Inspector);
    lifecycle.mark_tab_dirty(TestPanel::Console);

    let inputs_resized = LifecycleInputs::new((1600.0, 900.0), 1.0, 0);
    let branch = lifecycle.evaluate(inputs_resized);

    // Full shell takes precedence and drains dirty tabs
    assert_eq!(branch, LifecycleBranch::FullShellRebuild);
    assert!(lifecycle.dirty_tabs().is_empty());
}

#[test]
fn test_in_place_volatile_update_refresh() {
    let mut lifecycle = WorkbenchLifecycle::<TestPanel>::new();
    let inputs_base = LifecycleInputs::new((1920.0, 1080.0), 1.0, 0);
    let _ = lifecycle.evaluate(inputs_base);

    let inputs_volatile = inputs_base.with_volatile_updates(true);
    assert!(!lifecycle.is_shell_dirty(&inputs_volatile));

    let branch = lifecycle.evaluate(inputs_volatile);
    assert_eq!(branch, LifecycleBranch::InPlaceRefresh);
}