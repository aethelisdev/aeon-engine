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

/// Synchronizes the Content / Asset Browser panel into the `UiTree` in Retained Mode.
/// If no persistent state exists or the root node is no longer in the arena, this function
/// performs initial construction, caches node handles, and marks `DirtyFlags::ALL`.
/// On subsequent calls:
/// - If state parameters are completely unchanged, **zero** nodes are allocated or mutated,
///   and no layout or paint dirty flags are marked.
/// - If cursor position changes card hover or selection state, only affected cards have their
///   styles updated and are marked with `DirtyFlags::PAINT`.
/// - If structural parameters (panel rect, scroll offset, directory path, search query, or items)
///   change, the panel is reconciled and relevant subtrees are marked dirty.
pub fn sync_assets_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    retained_state: &mut Option<AssetBrowserRetainedState>,
    params: &AssetsPanelParams<'_>,
) -> bool {
    let needs_full_rebuild = match retained_state {
        Some(state) => {
            if !tree.contains_node(state.root_id) {
                true
            } else if let Some(ref snapshot) = state.snapshot {
                // Check if structural or content parameters have mutated
                snapshot.panel_rect != params.panel_rect
                    || snapshot.current_folder != *params.current_folder
                    || snapshot.search_query != params.search_query
                    || snapshot.active_category != params.active_category
                    || snapshot.view_mode != params.view_mode
                    || snapshot.scroll_y != params.scroll_y
                    || snapshot.tree_scroll_y != params.tree_scroll_y
                    || snapshot.sidebar_width != params.sidebar_width
                    || snapshot.sidebar_collapsed != params.sidebar_collapsed
                    || params.active_context_menu.is_some()
                    || params.active_preview_modal.is_some()
                    || snapshot.revision != params.revision
            } else {
                true
            }
        }
        None => true,
    };

    if needs_full_rebuild {
        if let Some(prev) = retained_state.take() {
            let _ = tree.remove_node(prev.root_id);
        }
        let new_retained = build_assets_panel_retained(tree, parent_id, params);
        *retained_state = Some(new_retained);
        return true;
    }

    // Retained Fast Path: Structure is identical
    if let Some(state) = retained_state {
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

        if style_changed && let Some(ref mut snapshot) = state.snapshot {
            snapshot.selected_asset = params.selected_asset.map(|p| p.to_path_buf());
        }
        style_changed
    } else {
        false
    }
}