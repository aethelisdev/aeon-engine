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
use irisui::widgets::controls::{FocusState, FocusTarget};

impl HierarchyPanelState {
    /// Builds the Scene Hierarchy panel directly using typed hierarchy state and `UiScope`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        focus_manager: &mut FocusManager,
        focus_state: &mut FocusState<FocusTarget>,
        (ctrl_held, shift_held): (bool, bool),
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

        scope.auto_sync_search_input(
            HIERARCHY_TAG_SEARCH_INPUT,
            Some(HIERARCHY_TAG_SEARCH_CLEAR),
            &mut self.interactions.search_input,
            focus_manager,
            focus_state,
        );

        build_hierarchy_panel(
            scope,
            &HierarchyPanelParams {
                panel_rect: scope.parent_rect(),
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                search_query: self.interactions.search_input.text(),
                is_editing: params.context.is_editing,
                is_2d: params.context.is_2d_mode,
                active_submenu: self.active_submenu,
                active_sub_submenu: self.active_sub_submenu,
                active_context_menu: self.active_context_menu,
                cursor_pos: scope.resolved_cursor_pos(cursor_pos),
                blink_caret,
                focus_state: &*focus_state,
                cursor_byte_idx: self.interactions.search_input.cursor_byte_idx,
                ctrl_held,
                shift_held,
            },
            &mut self.rows_cache,
            &mut self.interactions.actions,
            &mut self.tree_state,
        );

        if self
            .interactions
            .actions
            .iter()
            .any(|action| matches!(action, super::types::HierarchyAction::ClearSearchQuery))
        {
            self.interactions.search_input.clear();
            scope.clear_focus_with(focus_manager, focus_state);
        } else {
            self.interactions
                .actions
                .iter()
                .filter_map(|action| match action {
                    super::types::HierarchyAction::SetSearchQuery(query) => Some(query.as_str()),
                    _ => None,
                })
                .next_back()
                .into_iter()
                .for_each(|query| self.interactions.search_input.set_text(query));
        }
    }
}