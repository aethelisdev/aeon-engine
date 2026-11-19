// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Panel Autonomous Builder
//!
//! Autonomous builder coordinating hierarchy entity tree rendering, live search input
//! buffer synchronization with [`EventState`], caret blinking, and action handling.

use super::panel::build_hierarchy_panel;
use super::types::{
    HIERARCHY_TAG_SEARCH_CLEAR, HIERARCHY_TAG_SEARCH_INPUT, HierarchyPanelParams,
    HierarchyPanelState,
};
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use irisui::prelude::*;

impl HierarchyPanelState {
    /// Builds the Scene Hierarchy panel directly using typed hierarchy state and `UiScope`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        (cursor_pos, blink_caret): (Point, bool),
    ) {
        if scope.is_collapsed() {
            return;
        }

        if scope.popups_dismissed() {
            self.active_submenu = None;
            self.active_sub_submenu = None;
            self.active_context_menu = None;
        }

        scope.auto_sync_text_input(
            HIERARCHY_TAG_SEARCH_INPUT,
            Some(HIERARCHY_TAG_SEARCH_CLEAR),
            &mut self.search_input,
        );

        build_hierarchy_panel(
            scope,
            &HierarchyPanelParams {
                panel_rect: scope.parent_rect(),
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                search_query: self.search_input.text(),
                is_editing: params.context.is_editing,
                is_2d: params.context.is_2d_mode,
                active_submenu: self.active_submenu,
                active_sub_submenu: self.active_sub_submenu,
                active_context_menu: self.active_context_menu,
                cursor_pos: scope.resolved_cursor_pos(cursor_pos),
                blink_caret,
                cursor_byte_idx: self.search_input.cursor_byte_idx,
            },
            &mut self.rows_cache,
            &mut self.actions,
            &mut self.tree_state,
        );

        if self
            .actions
            .iter()
            .any(|action| matches!(action, super::types::HierarchyAction::ClearSearchQuery))
        {
            self.search_input.clear();
            scope.clear_focus();
            scope.clear_text_edit();
        } else {
            self.actions
                .iter()
                .filter_map(|action| match action {
                    super::types::HierarchyAction::SetSearchQuery(query) => Some(query.as_str()),
                    _ => None,
                })
                .next_back()
                .into_iter()
                .for_each(|query| self.search_input.set_text(query));
        }
    }
}