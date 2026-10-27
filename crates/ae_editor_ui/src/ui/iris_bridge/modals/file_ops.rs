// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # File Operations Modal Dialogues
//!
//! Declaratively renders hardware-accelerated GPU SDF modal cards for file system operations
//! including permanent delete confirmations, new folder creation, and file renaming
//! by delegating to [`iris_workbench`] modal descriptors and [`WorkbenchUiScopeExt`].

use irisui::prelude::*;
use std::path::Path;

/// Width of the delete confirmation modal card in physical pixels.
pub const DELETE_MODAL_WIDTH: f32 = 410.0;
/// Height of the delete confirmation modal card in physical pixels.
pub const DELETE_MODAL_HEIGHT: f32 = 165.0;

/// Width of folder and rename modal cards in physical pixels.
pub const INPUT_MODAL_WIDTH: f32 = 390.0;
/// Height of folder and rename modal cards in physical pixels.
pub const INPUT_MODAL_HEIGHT: f32 = 170.0;

/// Constructs the centered 'Delete Confirmation' modal in the UI tree using declarative [`UiScope`].
pub fn build_delete_modal(
    tree: &mut UiTree,
    target_path: &Path,
    screen_width: f32,
    screen_height: f32,
    cursor_pos: Point,
) {
    tree.set_cursor_position(cursor_pos);
    let parent = tree
        .root()
        .or_else(|| tree.create_root().ok())
        .unwrap_or_default();
    let mut scope = UiScope::new(tree, parent);

    let desc = ConfirmDialogDescriptor::danger(
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
    );

    let viewport = Rect::new(0.0, 0.0, screen_width, screen_height);
    let _ = scope.confirm_modal(&desc, viewport, false);
}

/// Parameters for constructing the 'Create New Folder' modal.
pub struct FolderModalParams<'a> {
    /// Target parent directory where the new subfolder will be created.
    pub parent_path: &'a Path,
    /// Mutable folder name string buffer bound to the declarative input field.
    pub input_text: &'a mut String,
    /// Viewport width in physical pixels.
    pub screen_width: f32,
    /// Viewport height in physical pixels.
    pub screen_height: f32,
    /// Current mouse cursor coordinates.
    pub cursor_pos: Point,
}

/// Parameters for constructing the 'Rename Asset / Folder' modal.
pub struct RenameModalParams<'a> {
    /// Target path of the file or folder being renamed.
    pub target_path: &'a Path,
    /// Mutable rename string buffer bound to the declarative input field.
    pub input_text: &'a mut String,
    /// Whether the target is a folder (true) or an asset file (false).
    pub is_folder: bool,
    /// Viewport width in physical pixels.
    pub screen_width: f32,
    /// Viewport height in physical pixels.
    pub screen_height: f32,
    /// Current mouse cursor coordinates.
    pub cursor_pos: Point,
}

/// Constructs the centered 'Create New Folder' modal in the UI tree using declarative [`UiScope`].
pub fn build_new_folder_modal(tree: &mut UiTree, params: FolderModalParams<'_>) {
    tree.set_cursor_position(params.cursor_pos);
    let parent = tree
        .root()
        .or_else(|| tree.create_root().ok())
        .unwrap_or_default();
    let mut scope = UiScope::new(tree, parent);

    let desc = InputDialogDescriptor::new("Create New Folder", "")
        .with_icon("📁")
        .with_placeholder("Enter folder name...")
        .with_detail(format!("Location: {}", params.parent_path.display()))
        .with_confirm_label("Create Folder")
        .with_cancel_label("Cancel")
        .with_size(INPUT_MODAL_WIDTH, INPUT_MODAL_HEIGHT)
        .with_style(
            ModalStyle::dark_studio()
                .with_confirm_button_width(112.0)
                .with_cancel_button_width(75.0),
        );

    let viewport = Rect::new(0.0, 0.0, params.screen_width, params.screen_height);
    let _ = scope.input_modal(&desc, params.input_text, viewport, false);
}

/// Constructs the centered 'Rename Asset / Folder' modal in the UI tree using declarative [`UiScope`].
pub fn build_rename_modal(tree: &mut UiTree, params: RenameModalParams<'_>) {
    tree.set_cursor_position(params.cursor_pos);
    let parent = tree
        .root()
        .or_else(|| tree.create_root().ok())
        .unwrap_or_default();
    let mut scope = UiScope::new(tree, parent);

    let title = if params.is_folder {
        "🔄  Rename Folder"
    } else {
        "🔄  Rename Asset"
    };
    let placeholder = if params.is_folder {
        "Enter new folder name..."
    } else {
        "Enter new file name..."
    };

    let desc = InputDialogDescriptor::new(title, "")
        .with_placeholder(placeholder)
        .with_detail(format!("Target: {}", params.target_path.display()))
        .with_confirm_label("Apply Rename")
        .with_cancel_label("Cancel")
        .with_size(INPUT_MODAL_WIDTH, INPUT_MODAL_HEIGHT)
        .with_style(
            ModalStyle::dark_studio()
                .with_confirm_button_width(112.0)
                .with_cancel_button_width(75.0),
        );

    let viewport = Rect::new(0.0, 0.0, params.screen_width, params.screen_height);
    let _ = scope.input_modal(&desc, params.input_text, viewport, false);
}