// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Right-Click Entity Context Menu Builder
//!
//! Renders the floating context menu for deleting entities or toggling visibility
//! using pure declarative [`UiScope::dropdown_menu_card_named`] and [`UiScope::dropdown_item`].

use super::types::{
    HIERARCHY_CTX_DELETE, HIERARCHY_CTX_VISIBILITY, HierarchyAction, HierarchyPanelParams,
};
use irisui::prelude::*;

/// Builds the right-click entity context menu in the [`UiTree`] if active.
///
/// Dispatches context menu actions directly via [`WidgetResponse::clicked`].
///
/// Returns the computed bounding [`Rect`] of the active context menu card if rendered.
pub fn build_context_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &HierarchyPanelParams<'_>,
    actions: &mut Vec<HierarchyAction>,
) -> Option<Rect> {
    let (target_entity, click_pos) = params.active_context_menu?;

    let card_rect = UiScope::new(tree, parent_id)
        .dropdown_menu_card_named(
            "HierarchyContextMenu",
            click_pos.x,
            click_pos.y,
            160.0,
            |card| {
                let del_resp =
                    card.dropdown_item(HIERARCHY_CTX_DELETE, "🗑", "Delete Entity", None, true);
                if del_resp.clicked() {
                    actions.push(HierarchyAction::DeleteSelected);
                    actions.push(HierarchyAction::CloseContextMenu);
                }

                let vis_resp = card.dropdown_item(
                    HIERARCHY_CTX_VISIBILITY,
                    "👁",
                    "Toggle Visibility",
                    None,
                    true,
                );
                if vis_resp.clicked() {
                    actions.push(HierarchyAction::ToggleVisibility(target_entity));
                    actions.push(HierarchyAction::CloseContextMenu);
                }
            },
        )
        .response
        .rect;

    Some(card_rect)
}