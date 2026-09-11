// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Retained-Mode synchronization and reactive state management for Iris UI Stats & Profiler panel.
//!
//! Maintains persistent UI tree handles across frames and applies in-place updates to
//! telemetry labels, bar indicators, and viewport toggles with zero node allocations.
//!

use super::panel::{build_stats_panel, update_stats_panel_values};
use super::types::{
    StatsPanelParams, StatsPanelRetainedState, StatsPanelSnapshot, StatsPanelTargets,
};
use irisui::prelude::*;

/// Synchronizes the Performance Stats & Profiler panel into the `UiTree` in Retained Mode.
/// If no persistent state exists, the root node is no longer present in the arena,
/// or layout geometry (such as panel bounds or scroll offset) has changed:
/// - Any outdated nodes are safely purged.
/// - The full static card hierarchy is constructed once via `build_stats_panel`.
/// - Initial values are applied and the state snapshot is cached.
/// On subsequent calls with unchanged panel geometry and scroll offset:
/// - **Zero** nodes are allocated or freed.
/// - The root container's parent relationship is ensured (supporting docked vs floating transitions).
/// - Dynamic telemetry metrics (frametime, FPS, CPU/GPU pass timings, VRAM distribution, and draw counts)
///   are updated in place directly on existing leaf nodes via `update_stats_panel_values`.
pub fn sync_stats_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    retained_state: &mut Option<StatsPanelRetainedState>,
    params: &StatsPanelParams<'_>,
    targets: &mut StatsPanelTargets,
) {
    let needs_full_rebuild = match retained_state {
        Some(state) => {
            !tree.contains_node(state.nodes.root_id)
                || state.snapshot.panel_rect != params.panel_rect
                || (state.snapshot.scroll_y - params.scroll_y).abs() > 0.001
        }
        None => true,
    };

    if needs_full_rebuild {
        if let Some(prev) = retained_state.take() {
            let _ = tree.remove_node(prev.nodes.root_id);
        }

        let nodes = build_stats_panel(tree, parent_id, params, targets);
        update_stats_panel_values(tree, &nodes, params, targets);

        *retained_state = Some(StatsPanelRetainedState {
            nodes,
            cached_targets: targets.clone(),
            snapshot: StatsPanelSnapshot {
                panel_rect: params.panel_rect,
                scroll_y: params.scroll_y,
            },
        });
        return;
    }

    // Retained Fast Path: Structure and geometry are identical
    if let Some(state) = retained_state {
        // Guarantee root container retains its valid dock content rect
        if let Some(node) = tree.get(state.nodes.root_id)
            && node.computed_rect != params.panel_rect
            && let Some(node_mut) = tree.get_mut(state.nodes.root_id)
        {
            node_mut.computed_rect = params.panel_rect;
        }

        // Ensure parent relationship is synchronized (docked vs floating window container)
        if tree.get(state.nodes.root_id).and_then(|n| n.parent) != Some(parent_id) {
            let _ = tree.add_child(parent_id, state.nodes.root_id);
        }

        // Apply in-place value updates to telemetry widgets (zero node allocations)
        update_stats_panel_values(tree, &state.nodes, params, &state.cached_targets);

        // Preserve cached interaction hit targets without recomputing
        *targets = state.cached_targets.clone();
    }
}