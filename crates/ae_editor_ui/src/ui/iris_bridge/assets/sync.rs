// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Retained-Mode synchronization and reactive dirty-state management for Iris UI Asset Browser.
//!
//! Maintains persistent UI tree handles across frames and applies fine-grained `DirtyFlags`
//! updates only when asset browser state, layout geometry, or cursor hover states change.
//!

use super::cards::update_retained_card_hover_and_selection;
use super::panel::build_assets_panel_retained;
use super::types::{AssetBrowserRetainedState, AssetsPanelParams};
use irisui::prelude::*;

/// Synchronizes the Content / Asset Browser panel into the `UiTree` using pure Revision-Based Invalidation.
/// Follows Slate and UI Toolkit reactive principles:
/// - If `state.last_revision == params.revision && state.panel_rect == params.panel_rect`,
///   performs an instant O(1) CPU register comparison and exits immediately with zero work,
///   zero allocations, and zero dirty tree marks.
/// - If invalidation occurred (filesystem changes, folder navigation, filter/search updates, or resize),
///   the panel hierarchy is rebuilt cleanly and retained state is updated.
pub fn sync_assets_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    retained_state: &mut Option<AssetBrowserRetainedState>,
    params: &AssetsPanelParams<'_>,
) -> bool {
    if let Some(state) = retained_state.as_mut() {
        if tree.contains_node(state.root_id)
            && state.last_revision == params.revision
            && state.panel_rect == params.panel_rect
        {
            // Guarantee root container retains its valid dock content rect
            if let Some(node) = tree.get(state.root_id)
                && node.computed_rect != params.panel_rect
                && let Some(node_mut) = tree.get_mut(state.root_id)
            {
                node_mut.computed_rect = params.panel_rect;
            }

            // Ensure parent relationship is synchronized (docked vs floating window container)
            if tree.get(state.root_id).and_then(|n| n.parent) != Some(parent_id) {
                let _ = tree.add_child(parent_id, state.root_id);
            }

            // Micro-interaction: Check if cursor hover or selection changed on any active card
            let style_changed = update_retained_card_hover_and_selection(
                tree,
                &mut state.cards,
                params.cursor_pos,
                params.selected_asset,
            );

            return style_changed;
        }
        state.last_revision = params.revision;
        state.panel_rect = params.panel_rect;
    }

    if let Some(prev) = retained_state.take() {
        let _ = tree.remove_node(prev.root_id);
    }

    let mut new_retained = build_assets_panel_retained(tree, parent_id, params);
    new_retained.last_revision = params.revision;
    new_retained.panel_rect = params.panel_rect;
    *retained_state = Some(new_retained);
    true
}