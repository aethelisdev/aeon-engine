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
        let panel_rect = scope.parent_rect();
        if panel_rect.width <= 20.0 || panel_rect.height <= 20.0 {
            return;
        }

        if let Some(buf) = scope
            .tree()
            .event_state
            .text_edit_buffer(CONSOLE_TAG_SEARCH_INPUT)
        {
            self.console.search_input.set_text(buf);
            if let Some(cur) = scope
                .tree()
                .event_state
                .text_edit_cursor(CONSOLE_TAG_SEARCH_INPUT)
            {
                self.console.search_input.cursor_byte_idx = cur;
            }
        } else if scope.tree().event_state.frame_clicked_tag == Some(CONSOLE_TAG_SEARCH_CLEAR) {
            self.console.search_input.clear();
            self.clear_focus();
            scope.tree_mut().event_state.clear_text_edit();
        }

        let console_params = ConsolePanelParams {
            panel_rect,
            entries: params.panel_data.console_entries,
            filter: self.console.filter,
            search_query: self.console.search_input.text(),
            auto_scroll: self.console.auto_scroll,
            cursor_pos: self.cursor_pos(),
            blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
            focus_state: &self.focus_state,
            cursor_byte_idx: self.console.search_input.cursor_byte_idx,
        };

        let mut actions = Vec::new();
        build_console_panel(scope, &console_params, &mut actions);

        let has_console_actions = !actions.is_empty();
        for action in actions {
            match action {
                ConsoleAction::ClearLogs => {
                    if let Ok(mut lock) = ae_editor::editor_logger::LOGGER.logs.lock() {
                        lock.clear();
                    }
                    ae_editor::editor_logger::LOGGER
                        .log_count
                        .store(0, std::sync::atomic::Ordering::Relaxed);
                    self.tree.set_scroll_offset(CONSOLE_TAG_VIEWPORT, 0.0);
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
                    scope.tree_mut().event_state.clear_text_edit();
                }
                ConsoleAction::CopyLog(_) => {}
            }
        }
        if has_console_actions {
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
        }
    }
}