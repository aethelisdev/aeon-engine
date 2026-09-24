// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Right-Click Entity Context Menu Builder
//!
//! Renders the floating context menu for deleting entities or toggling visibility
//! using the unified `ContextMenuBuilder` widget without mutable target bags.
//!

use super::types::{HIERARCHY_CTX_DELETE, HIERARCHY_CTX_VISIBILITY, HierarchyPanelParams};
use irisui::prelude::*;

/// Builds the right-click entity context menu in the `UiTree` if active.
///
/// Returns the computed bounding [`Rect`] of the active context menu card if rendered.
pub fn build_context_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &HierarchyPanelParams<'_>,
) -> Option<Rect> {
    let (_target_entity, click_pos) = params.active_context_menu?;

    let card_rect = ContextMenuBuilder::new(click_pos)
        .cursor_pos(params.cursor_pos)
        .width(160.0)
        .destructive_item_with_icon(
            HIERARCHY_CTX_DELETE,
            ContextMenuIcon::Text("🗑"),
            "Delete Entity",
        )
        .item_with_icon(
            HIERARCHY_CTX_VISIBILITY,
            ContextMenuIcon::Text("👁"),
            "Toggle Visibility",
        )
        .build(tree, parent_id);

    Some(card_rect)
}