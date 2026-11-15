// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Panel Autonomous Builder
//!
//! Autonomous builder coordinating logger entry filtering, search input
//! live buffer synchronization with [`EventState`], and log clear actions.

use super::panel::build_console_panel;
use super::types::{
    CONSOLE_TAG_SEARCH_CLEAR, CONSOLE_TAG_SEARCH_INPUT, CONSOLE_TAG_VIEWPORT, ConsoleAction,
    ConsolePanelParams,
};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds the Developer Console panel if active in either docked or floating mode.
    ///
    /// Manages the full lifecycle of the log viewport:
    /// 1. Resolves physical panel bounds directly from the unified [`UiScope`] docking hierarchy.
    /// 2. Assembles log level filters, search query filtering, and auto-scrolling log table in unified coordinates.
    /// 3. Processes clear logs, search clear, and log copy triggers.
    pub(crate) fn build_console_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        if scope.is_collapsed() {
            return;
        }

        scope.auto_sync_core_focus(&mut self.focus_manager);
        scope.auto_sync_focus_state(&mut self.focus_state);
        scope.auto_sync_text_input(
            CONSOLE_TAG_SEARCH_INPUT,
            Some(CONSOLE_TAG_SEARCH_CLEAR),
            &mut self.console.search_input,
        );

        let mut actions = Vec::new();
        build_console_panel(
            scope,
            &mut ConsolePanelParams {
                panel_rect: scope.parent_rect(),
                entries: params.panel_data.console_entries,
                filter: self.console.filter,
                search_query: self.console.interactions.search_input.text(),
                auto_scroll: &mut self.console.auto_scroll,
                cursor_pos: scope.cursor_pos().unwrap_or(self.chrome.last_cursor_pos),
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
                focus_state: &self.focus_state,
                cursor_byte_idx: self.console.interactions.search_input.cursor_byte_idx,
                active_tab: self.console.active_tab,
            },
            &mut self.console.scratchpad_state,
            &mut actions,
        );

        if !actions.is_empty() {
            for action in actions {
                self.dispatch_console_action(scope, action);
            }
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
        }
    }

    /// Dispatches a console action triggered by interactive controls in the console panel.
    fn dispatch_console_action(&mut self, scope: &mut UiScope<'_>, action: ConsoleAction) {
        match action {
            ConsoleAction::ClearLogs => {
                ae_editor::editor_logger::LOGGER.clear();
                scope.set_scroll_offset(CONSOLE_TAG_VIEWPORT, 0.0);
            }
            ConsoleAction::SetFilter(level) => {
                self.console.filter = level;
            }
            ConsoleAction::ToggleAutoScroll => {
                self.console.auto_scroll = !self.console.auto_scroll;
            }
            ConsoleAction::FocusSearch => {
                self.set_focus_tag(CONSOLE_TAG_SEARCH_INPUT);
            }
            ConsoleAction::ClearSearch => {
                self.console.search_input.clear();
                self.clear_focus();
                scope.clear_text_edit();
            }
            ConsoleAction::CopyLog(_) => {}
            ConsoleAction::SelectTab(tab) => {
                self.console.active_tab = tab;
            }
            ConsoleAction::ClearScratchpad => {
                self.console.scratchpad_state.clear();
            }
            ConsoleAction::ResetScratchpad => {
                self.console
                    .scratchpad_state
                    .set_text(super::types::DEFAULT_SCRATCHPAD_TEXT);
            }
        }
    }
}