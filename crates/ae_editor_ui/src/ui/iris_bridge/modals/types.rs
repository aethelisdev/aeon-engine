// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modal dialog and splash overlay state descriptors for Iris UI.

use irisui::prelude::{InteractionEvent, WidgetId};

/// Persistent interactive state for modal dialogs and splash overlays.
#[derive(Debug, Default, Clone)]
pub struct ModalsOverlayState {
    /// Whether the About dialog is currently open.
    pub is_about_active: bool,
    /// Whether the Delete Confirmation modal is currently open.
    pub is_delete_active: bool,
    /// Whether the New Folder modal is currently open.
    pub is_new_folder_active: bool,
    /// Whether the Rename modal is currently open.
    pub is_rename_active: bool,
    /// Whether the Asset Loading splash screen is currently active.
    pub is_loading_active: bool,
    /// Centralized text input state for new folder modal dialogues.
    pub new_folder_input: irisui::prelude::TextInputState,
    /// Centralized text input state for rename modal dialogues.
    pub rename_input: irisui::prelude::TextInputState,
    /// Last recorded modal visibility state for detecting dialog popups.
    pub last_modal_active: bool,
    /// Pending interaction events for declarative components inside modals.
    pub pending_interaction_events: Vec<(WidgetId, InteractionEvent)>,
}

impl ModalsOverlayState {
    /// Synchronizes active modal flags directly from authority dialog parameters.
    ///
    /// Ensures modal visibility states remain authoritative and retained between frames
    /// without relying on transient build-pass mutations or per-frame wipe cycles.
    pub fn sync_from_dialogs(&mut self, dialogs: &crate::ui::iris_bridge::types::DialogParams<'_>) {
        self.is_about_active = dialogs.show_about;
        self.is_delete_active = dialogs.delete_target.is_some();
        self.is_new_folder_active = dialogs.new_folder_parent.is_some();
        self.is_rename_active = dialogs.rename_target.is_some();
        self.is_loading_active = dialogs.is_loading_assets;
    }

    /// Sets the new folder input text, positioning the cursor at the end.
    pub fn set_new_folder_text(&mut self, text: &str) {
        self.new_folder_input.set_text(text);
    }

    /// Sets the rename input text and selects all text for immediate overwriting.
    pub fn set_rename_text(&mut self, text: &str) {
        self.rename_input.set_text(text);
        self.rename_input.select_all();
    }

    /// Clears the new folder input state.
    pub fn clear_new_folder(&mut self) {
        self.new_folder_input.clear();
    }

    /// Clears the rename input state.
    pub fn clear_rename(&mut self) {
        self.rename_input.clear();
    }
}