// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Workbench Preferences Lifecycle Orchestration
//!
//! Provides autonomous opening, focusing, and toggling of the Preferences panel
//! within the `iris-dock` docking tree and detached floating window subsystem.

use super::state::EngineUi;
use crate::ui::PanelId;

impl EngineUi {
    /// Opens or focuses the Preferences panel as a detached floating window in `iris-dock`.
    ///
    /// If the Preferences panel is currently docked as a tab in the workspace tree, focuses
    /// that docked leaf and activates the Preferences tab. If it is already hosted within
    /// a floating window, elevates that floating window to the top of the z-order.
    /// Otherwise, creates a new centered floating window using default preference card dimensions.
    pub fn open_preferences(&mut self) {
        let (screen_w, screen_h) = if self.iris_overlay.chrome.last_dimensions.0 > 100.0 {
            self.iris_overlay.chrome.last_dimensions
        } else {
            (1280.0, 720.0)
        };
        self.layout_state
            .open_preferences_floating(screen_w, screen_h);
        self.iris_overlay.notifier.tag_all();
        self.iris_overlay.chrome.needs_layout_rebuild = true;
    }

    /// Toggles the visibility of the Preferences panel, closing it if open or floating,
    /// or opening it in a centered floating window if hidden.
    pub fn toggle_preferences(&mut self) {
        if self
            .layout_state
            .dock_state
            .is_tab_in_floating_window(&PanelId::Preferences)
        {
            if let Some(win_id) = self
                .layout_state
                .dock_state
                .floating_windows
                .iter()
                .find(|w| w.tree.find_tab(&PanelId::Preferences).is_some())
                .map(|w| w.id)
            {
                let _ = self.layout_state.dock_state.close_floating_window(win_id);
            }
            self.iris_overlay.notifier.tag_all();
            self.iris_overlay.chrome.needs_layout_rebuild = true;
        } else if let Some((leaf, idx)) = self
            .layout_state
            .dock_state
            .tree
            .find_tab(&PanelId::Preferences)
        {
            let _ = self.layout_state.dock_state.tree.remove_tab(leaf, idx);
            self.layout_state.dock_state.tree.collapse_empty_leaves();
            self.iris_overlay.notifier.tag_all();
            self.iris_overlay.chrome.needs_layout_rebuild = true;
        } else {
            self.open_preferences();
        }
    }
}