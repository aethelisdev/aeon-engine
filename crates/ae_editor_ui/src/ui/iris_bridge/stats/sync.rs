// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Retained-Mode synchronization and reactive state management for Iris UI Stats & Profiler panel.
//!
//! Maintains persistent UI tree handles across frames and applies in-place updates to
//! telemetry labels, bar indicators, and viewport toggles with zero node allocations.
//!

use super::panel::{build_stats_panel, update_stats_panel_values};
use super::types::{StatsPanelParams, StatsPanelRetainedState, StatsPanelTargets};
use irisui::prelude::*;

/// Synchronizes the Performance Stats & Profiler panel into the `UiTree` using pure Revision-Based Invalidation.
/// Follows Slate and UI Toolkit reactive principles:
/// - If `state.last_revision == params.revision && state.panel_rect == params.panel_rect`,
///   performs an instant O(1) CPU register comparison and exits immediately with zero work,
///   zero allocations, and zero dirty tree marks.
/// - In-place text updates for telemetry metrics (FPS, ms, VRAM, CPU/GPU pass timings) are handled
///   independently via `update_stats_panel_text_values` to keep the UI tree pristine.
/// - When structural invalidation occurs (e.g., panel resize or user interaction), the panel is rebuilt.
pub fn sync_stats_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    retained_state: &mut Option<StatsPanelRetainedState>,
    params: &StatsPanelParams<'_>,
    targets: &mut StatsPanelTargets,
) {
    if let Some(state) = retained_state.as_mut() {
        if tree.contains_node(state.nodes.root_id)
            && state.last_revision == params.revision
            && state.panel_rect == params.panel_rect
        {
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
            return;
        }
        state.last_revision = params.revision;
        state.panel_rect = params.panel_rect;
    }

    if let Some(prev) = retained_state.take() {
        let _ = tree.remove_node(prev.nodes.root_id);
    }

    let nodes = build_stats_panel(tree, parent_id, params, targets);
    update_stats_panel_values(tree, &nodes, params, targets);

    *retained_state = Some(StatsPanelRetainedState {
        nodes,
        cached_targets: targets.clone(),
        panel_rect: params.panel_rect,
        last_revision: params.revision,
    });
}