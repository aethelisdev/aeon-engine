// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Workbench Preferences Lifecycle Orchestration
//!
//! Delegates opening, focusing, and toggling of the Preferences panel directly to
//! `irisui::workbench::Workbench`.

use super::state::EngineUi;
use crate::ui::PanelId;

impl EngineUi {
    /// Opens or focuses the Preferences panel according to its registered [`PanelId::placement`].
    #[inline]
    pub fn open_preferences(&mut self) {
        self.workbench.open_or_focus_panel(PanelId::Preferences);
    }

    /// Toggles the visibility of the Preferences panel, closing it if open or floating,
    /// or opening it in a centered floating window if hidden.
    #[inline]
    pub fn toggle_preferences(&mut self) {
        self.workbench.toggle_panel(PanelId::Preferences);
    }
}