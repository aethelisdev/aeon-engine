// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Panel Autonomous Builder
//!
//! Autonomous builder coordinating hierarchy entity tree rendering, live search input
//! buffer synchronization with [`EventState`], caret blinking, and action handling.
//!

use super::panel::build_hierarchy_panel;
use super::types::{HIERARCHY_TAG_SEARCH_CLEAR, HIERARCHY_TAG_SEARCH_INPUT, HierarchyPanelParams};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds the Scene Hierarchy panel if active in either docked or floating mode.
    ///
    /// Manages the full lifecycle of the Scene Hierarchy tree:
    /// 1. Resolves physical panel bounds directly from the unified [`UiScope`] docking hierarchy.
    /// 2. Assembles declarative entity tree nodes, selection highlights, and folder collapsing.
    /// 3. Processes hierarchy user actions (search filter updates, selection changes).
    pub(crate) fn build_hierarchy_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        let panel_rect = scope.parent_rect();
        if panel_rect.width <= 20.0 || panel_rect.height <= 20.0 {
            return;
        }

        if let Some(buf) = scope
            .tree()
            .event_state
            .text_edit_buffer(HIERARCHY_TAG_SEARCH_INPUT)
        {
            self.hierarchy.interactions.search_input.set_text(buf);
            if let Some(cur) = scope
                .tree()
                .event_state
                .text_edit_cursor(HIERARCHY_TAG_SEARCH_INPUT)
            {
                self.hierarchy.interactions.search_input.cursor_byte_idx = cur;
            }
        } else if scope.tree().event_state.frame_clicked_tag == Some(HIERARCHY_TAG_SEARCH_CLEAR) {
            self.hierarchy.interactions.search_input.clear();
            self.clear_focus();
            scope.tree_mut().event_state.clear_text_edit();
        }

        let cursor_pos = self.cursor_pos();
        let blink_caret = (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2);
        let ctrl_held = self.chrome.ctrl_held;
        let shift_held = self.chrome.shift_held;
        let focus_state = &self.focus_state;
        let cursor_byte_idx = self.hierarchy.interactions.search_input.cursor_byte_idx;

        let hier_params = HierarchyPanelParams {
            panel_rect,
            world: params.scene.world,
            selected_entity: params.scene.selected_entity,
            search_query: self.hierarchy.interactions.search_input.text(),
            is_editing: params.context.is_editing,
            is_2d: params.context.is_2d_mode,
            active_submenu: self.hierarchy.active_submenu,
            active_sub_submenu: self.hierarchy.active_sub_submenu,
            is_add_menu_open: self.hierarchy.is_add_menu_open,
            active_context_menu: self.hierarchy.active_context_menu,
            cursor_pos,
            blink_caret,
            focus_state,
            cursor_byte_idx,
            ctrl_held,
            shift_held,
        };

        build_hierarchy_panel(
            scope,
            &hier_params,
            &mut self.hierarchy.rows_cache,
            &mut self.hierarchy.interactions.actions,
            &mut self.hierarchy.tree_state,
        );

        let mut should_clear = false;
        let mut set_query = None;
        for action in &self.hierarchy.interactions.actions {
            match action {
                super::types::HierarchyAction::ClearSearchQuery => {
                    should_clear = true;
                }
                super::types::HierarchyAction::SetSearchQuery(q) => {
                    set_query = Some(q.clone());
                }
                _ => {}
            }
        }

        if should_clear {
            self.hierarchy.interactions.search_input.clear();
            self.clear_focus();
            scope.tree_mut().event_state.clear_text_edit();
        } else if let Some(q) = set_query {
            self.hierarchy.interactions.search_input.set_text(&q);
        }
    }
}