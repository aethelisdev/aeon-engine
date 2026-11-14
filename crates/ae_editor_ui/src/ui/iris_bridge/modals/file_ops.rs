// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # File Operations Modal Dialogues
//!
//! Declaratively renders hardware-accelerated GPU SDF modal cards for file system operations
//! including permanent delete confirmations, new folder creation, and file renaming
//! by delegating to [`iris_workbench`] modal descriptors and [`WorkbenchUiScopeExt`].

use irisui::prelude::*;
use irisui::workbench::WorkbenchUiScopeExt;
use std::path::Path;

/// Width of the delete confirmation modal card in physical pixels.
pub const DELETE_MODAL_WIDTH: f32 = 410.0;
/// Height of the delete confirmation modal card in physical pixels.
pub const DELETE_MODAL_HEIGHT: f32 = 165.0;

/// Width of folder and rename modal cards in physical pixels.
pub const INPUT_MODAL_WIDTH: f32 = 390.0;
/// Height of folder and rename modal cards in physical pixels.
pub const INPUT_MODAL_HEIGHT: f32 = 170.0;

/// Constructs the centered 'Delete Confirmation' modal using declarative [`UiScope`].
pub fn build_delete_modal(
    scope: &mut UiScope<'_>,
    target_path: &Path,
    viewport: Rect,
    escape_pressed: bool,
) -> ModalOutcome<bool> {
    scope.confirm_modal(
        &ConfirmDialogDescriptor::danger(
            "⚠️  Confirm Deletion",
            "Are you sure you want to permanently delete this item?",
            "🗑 Delete Permanently",
        )
        .with_detail(target_path.display().to_string())
        .with_cancel_label("Cancel")
        .with_size(DELETE_MODAL_WIDTH, DELETE_MODAL_HEIGHT)
        .with_style(
            ModalStyle::dark_studio()
                .with_confirm_button_width(150.0)
                .with_cancel_button_width(75.0),
        ),
        viewport,
        escape_pressed,
    )
}

/// Constructs the centered 'Create New Folder' modal using declarative [`UiScope`].
pub fn build_new_folder_modal(
    scope: &mut UiScope<'_>,
    parent_path: &Path,
    input_text: &mut String,
    viewport: Rect,
    escape_pressed: bool,
) -> ModalOutcome<String> {
    scope.input_modal(
        &InputDialogDescriptor::new("Create New Folder", "")
            .with_icon("📁")
            .with_placeholder("Enter folder name...")
            .with_detail(format!("Location: {}", parent_path.display()))
            .with_confirm_label("Create Folder")
            .with_cancel_label("Cancel")
            .with_size(INPUT_MODAL_WIDTH, INPUT_MODAL_HEIGHT)
            .with_style(
                ModalStyle::dark_studio()
                    .with_confirm_button_width(112.0)
                    .with_cancel_button_width(75.0),
            ),
        input_text,
        viewport,
        escape_pressed,
    )
}

/// Constructs the centered 'Rename Asset / Folder' modal using declarative [`UiScope`].
pub fn build_rename_modal(
    scope: &mut UiScope<'_>,
    target_path: &Path,
    is_folder: bool,
    input_text: &mut String,
    viewport: Rect,
    escape_pressed: bool,
) -> ModalOutcome<String> {
    let title = if is_folder {
        "🔄  Rename Folder"
    } else {
        "🔄  Rename Asset"
    };
    let placeholder = if is_folder {
        "Enter new folder name..."
    } else {
        "Enter new file name..."
    };

    scope.input_modal(
        &InputDialogDescriptor::new(title, "")
            .with_placeholder(placeholder)
            .with_detail(format!("Target: {}", target_path.display()))
            .with_confirm_label("Apply Rename")
            .with_cancel_label("Cancel")
            .with_size(INPUT_MODAL_WIDTH, INPUT_MODAL_HEIGHT)
            .with_style(
                ModalStyle::dark_studio()
                    .with_confirm_button_width(112.0)
                    .with_cancel_button_width(75.0),
            ),
        input_text,
        viewport,
        escape_pressed,
    )
}