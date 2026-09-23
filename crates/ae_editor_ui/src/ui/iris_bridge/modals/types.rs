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
    /// Live typing input buffer for the new folder modal.
    pub new_folder_buffer: String,
    /// Live typing input buffer for the rename modal.
    pub rename_buffer: String,
    /// Last recorded modal visibility state for detecting dialog popups.
    pub last_modal_active: bool,
    /// Pending interaction events for declarative components inside modals.
    pub pending_interaction_events: Vec<(WidgetId, InteractionEvent)>,
}