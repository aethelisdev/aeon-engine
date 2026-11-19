// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Panel Autonomous Builder
//!
//! Autonomous builder coordinating logger entry filtering, search input
//! live buffer synchronization with [`EventState`], and log clear actions.

use super::panel::build_console_panel;
use super::types::{
    CONSOLE_TAG_SEARCH_CLEAR, CONSOLE_TAG_SEARCH_INPUT, CONSOLE_TAG_VIEWPORT, ConsoleAction,
    ConsolePanelParams, ConsolePanelState,
};
use crate::ui::panel_layout::PanelId;
use irisui::prelude::*;

impl ConsolePanelState {
    /// Builds the Developer Console panel directly using typed console state and `UiScope`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        entries: &[crate::ui::types::ConsoleEntry],
        notifier: &mut DeferredNotifier<PanelId>,
        (cursor_pos, blink_caret): (Point, bool),
    ) {
        if scope.is_collapsed() {
            return;
        }

        scope.auto_sync_text_input(
            CONSOLE_TAG_SEARCH_INPUT,
            Some(CONSOLE_TAG_SEARCH_CLEAR),
            &mut self.search_input,
        );

        self.actions.clear();
        build_console_panel(
            scope,
            &mut ConsolePanelParams {
                panel_rect: scope.parent_rect(),
                entries,
                filter: self.filter,
                search_query: self.search_input.text(),
                auto_scroll: &mut self.auto_scroll,
                cursor_pos: scope.resolved_cursor_pos(cursor_pos),
                blink_caret,
                cursor_byte_idx: self.search_input.cursor_byte_idx,
                active_tab: self.active_tab,
            },
            &mut self.scratchpad_state,
            &mut self.actions,
        );

        if !self.actions.is_empty() {
            std::mem::take(&mut self.actions)
                .into_iter()
                .for_each(|action| self.dispatch_action(scope, action));
            notifier.tag_redraw(crate::ui::panel_layout::PanelId::Console);
        }
    }

    fn dispatch_action(&mut self, scope: &mut UiScope<'_>, action: ConsoleAction) {
        match action {
            ConsoleAction::ClearLogs => {
                ae_editor::editor_logger::LOGGER.clear();
                scope.set_scroll_offset(CONSOLE_TAG_VIEWPORT, 0.0);
            }
            ConsoleAction::SetFilter(level) => {
                self.filter = level;
            }
            ConsoleAction::ToggleAutoScroll => {
                self.auto_scroll = !self.auto_scroll;
            }
            ConsoleAction::FocusSearch => {
                scope.set_focus_tag(CONSOLE_TAG_SEARCH_INPUT);
            }
            ConsoleAction::ClearSearch => {
                self.search_input.clear();
                scope.clear_focus();
                scope.clear_text_edit();
            }
            ConsoleAction::CopyLog(_) => {}
            ConsoleAction::SelectTab(tab) => {
                self.active_tab = tab;
            }
            ConsoleAction::ClearScratchpad => {
                self.scratchpad_state.clear();
            }
            ConsoleAction::ResetScratchpad => {
                self.scratchpad_state
                    .set_text(super::types::DEFAULT_SCRATCHPAD_TEXT);
            }
        }
    }
}