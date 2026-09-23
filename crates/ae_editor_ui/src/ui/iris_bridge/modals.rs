// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Modal Subsystems & Dialogs
//!
//! Provides retained GPU SDF modal overlays for file system operations
//! (folder creation, renaming, delete confirmations) and asset loading splash screens.

pub mod file_ops;
pub mod loading;
#[cfg(test)]
mod tests;
pub mod types;

pub use file_ops::{
    DELETE_MODAL_HEIGHT, DELETE_MODAL_WIDTH, FolderModalParams, INPUT_MODAL_HEIGHT,
    INPUT_MODAL_WIDTH, RenameModalParams, build_delete_modal, build_new_folder_modal,
    build_rename_modal,
};
pub use loading::{LoadingOverlayParams, build_loading_overlay};
pub use types::ModalsOverlayState;