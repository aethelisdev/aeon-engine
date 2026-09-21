// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modal dialog and splash overlay state descriptors for Iris UI.

use super::file_ops::{DeleteModalTargets, NewFolderModalTargets, RenameModalTargets};
use super::loading::LoadingOverlayTargets;
use crate::ui::iris_bridge::about::AboutDialogTargets;

/// Persistent interactive state for modal dialogs and splash overlays.
#[derive(Debug, Default, Clone)]
pub struct ModalsOverlayState {
    /// Cached bounding box and close button hit targets of the active About dialog.
    pub about_targets: Option<AboutDialogTargets>,
    /// Cached bounding box and button targets of the active Delete Confirmation modal.
    pub delete_targets: Option<DeleteModalTargets>,
    /// Cached bounding box and input targets of the active New Folder modal.
    pub new_folder_targets: Option<NewFolderModalTargets>,
    /// Cached bounding box and input targets of the active Rename modal.
    pub rename_targets: Option<RenameModalTargets>,
    /// Cached bounding box targets of the active Asset Loading splash screen.
    pub loading_targets: Option<LoadingOverlayTargets>,
    /// Live typing input buffer for the new folder modal.
    pub new_folder_buffer: String,
    /// Live typing input buffer for the rename modal.
    pub rename_buffer: String,
    /// Last recorded modal visibility state for detecting dialog popups.
    pub last_modal_active: bool,
}