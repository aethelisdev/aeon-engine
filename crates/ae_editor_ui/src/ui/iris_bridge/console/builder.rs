// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Panel Autonomous Builder
//!
//! Coordinates search text input synchronization with [`UiTree`] and delegates
//! panel construction directly to [`build_console_panel`] without intermediate queues.

use super::panel::build_console_panel;
use super::types::{CONSOLE_TAG_SEARCH_CLEAR, CONSOLE_TAG_SEARCH_INPUT, ConsolePanelState};
use crate::ui::panel_layout::PanelId;
use irisui::prelude::*;

impl ConsolePanelState {
    /// Builds the Developer Console panel directly using typed console state and `UiScope`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        entries: &[crate::ui::types::ConsoleEntry],
        notifier: &mut DeferredNotifier<PanelId>,
        blink_caret: bool,
    ) {
        if scope.is_collapsed() {
            return;
        }

        scope.auto_sync_text_input(
            CONSOLE_TAG_SEARCH_INPUT,
            Some(CONSOLE_TAG_SEARCH_CLEAR),
            &mut self.search_input,
        );

        build_console_panel(scope, self, entries, blink_caret);

        if std::mem::replace(&mut self.dirty_rebuild, false) {
            notifier.tag_redraw(PanelId::Console);
        }
    }
}