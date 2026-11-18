// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for [`VolatileHookRegistry`] and `Workbench::update_shell` `InPlaceRefresh` hooks.

use irisui_core::UiTree;
use irisui_dock::{DockState, DockTree};

use super::overlay::OverlayId;
use super::volatile::{VolatileHookRegistry, VolatileRefreshResult};
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::lifecycle::{LifecycleBranch, LifecycleInputs};
use crate::workbench::Workbench;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct VolatileTab;

#[derive(Default)]
struct VolatileAppState {
    ticks: u32,
}

#[test]
fn test_volatile_hook_registry_deduplication_and_merge() {
    let mut reg = VolatileHookRegistry::<VolatileAppState>::new();
    let mut tree = UiTree::new();
    let mut state = VolatileAppState::default();

    reg.register_fn(OverlayId::new(1), |_tree, s| {
        s.ticks += 1;
        VolatileRefreshResult::quads_only()
    });
    reg.register_fn(OverlayId::new(2), |_tree, s| {
        s.ticks += 10;
        VolatileRefreshResult::text_and_quads()
    });
    // Replacing ID 1 preserves count = 2
    reg.register_fn(OverlayId::new(1), |_tree, s| {
        s.ticks += 2;
        VolatileRefreshResult::quads_only()
    });
    assert_eq!(reg.len(), 2);

    let res = reg.refresh_in_place(&mut tree, &mut state);
    assert_eq!(state.ticks, 12);
    assert!(res.needs_quad_reupload);
    assert!(res.needs_text_prepare);

    assert!(reg.unregister(OverlayId::new(2)));
    assert!(!reg.unregister(OverlayId::new(2)));
    assert_eq!(reg.len(), 1);
}

#[test]
fn test_workbench_update_shell_runs_volatile_hooks_only_on_in_place_refresh() {
    let dock = DockState::new(DockTree::new());
    let mut wb = Workbench::<VolatileTab, (), VolatileAppState>::new(WorkbenchDescriptor::new(
        "Volatile Test",
        dock,
    ));
    wb.register_volatile_fn(OverlayId::new(0x9001), |_tree, app| {
        app.ticks += 1;
        VolatileRefreshResult::text_and_quads()
    });

    let mut state = VolatileAppState::default();

    // 1. Cold start -> FullShellRebuild (volatile hook not invoked)
    let base = LifecycleInputs::new((1280.0, 720.0), 1.0, 0);
    let b0 = wb
        .update_shell(&mut state, &base, false)
        .expect("cold start");
    assert_eq!(b0, LifecycleBranch::FullShellRebuild);
    assert_eq!(state.ticks, 0);

    // 2. Dormant -> volatile hook skipped
    let b1 = wb.update_shell(&mut state, &base, false).expect("dormant");
    assert_eq!(b1, LifecycleBranch::Dormant);
    assert_eq!(state.ticks, 0);

    // 3. InPlaceRefresh -> volatile hook executes and propagates needs_text_prepare
    let volatile_inputs = base.with_volatile_updates(true);
    let _ = wb.take_needs_text_prepare();
    let b2 = wb
        .update_shell(&mut state, &volatile_inputs, false)
        .expect("in-place refresh");
    assert_eq!(b2, LifecycleBranch::InPlaceRefresh);
    assert_eq!(state.ticks, 1);
    assert!(wb.take_needs_text_prepare());
}