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

/// Builds the right-click entity context menu directly into the active [`UiScope`] if active.
///
/// Dispatches context menu actions directly via declarative [`WidgetResponse::on_click`].
///
/// Returns the computed bounding [`Rect`] of the active context menu card if rendered.
pub fn draw_hierarchy_context_menu(
    scope: &mut UiScope<'_>,
    params: &HierarchyPanelParams<'_>,
    actions: &mut Vec<HierarchyAction>,
) -> Option<Rect> {
    params
        .active_context_menu
        .map(|(target_entity, click_pos)| {
            scope
                .dropdown_menu_card_named(
                    "HierarchyContextMenu",
                    click_pos.x,
                    click_pos.y,
                    160.0,
                    |card| {
                        card.dropdown_item(HIERARCHY_CTX_DELETE, "🗑", "Delete Entity", None, true)
                            .on_click(|| {
                                actions.push(HierarchyAction::DeleteSelected);
                                actions.push(HierarchyAction::CloseContextMenu);
                            });

                        card.dropdown_item(
                            HIERARCHY_CTX_VISIBILITY,
                            "👁",
                            "Toggle Visibility",
                            None,
                            true,
                        )
                        .on_click(|| {
                            actions.push(HierarchyAction::ToggleVisibility(target_entity));
                            actions.push(HierarchyAction::CloseContextMenu);
                        });
                    },
                )
                .response
                .rect
        })
}