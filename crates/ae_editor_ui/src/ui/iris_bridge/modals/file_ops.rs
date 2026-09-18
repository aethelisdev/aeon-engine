// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # File Operations Modal Dialogues
//!
//! Renders hardware-accelerated GPU SDF modal cards for file system operations
//! including permanent delete confirmations, new folder creation, and file renaming.

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

/// Hit testing targets for the Delete Confirmation modal dialogue.
#[derive(Debug, Clone)]
pub struct DeleteModalTargets {
    /// Full bounding box of the modal card.
    pub dialog_rect: Rect,
    /// Hit target of the top-right '✖' close icon.
    pub header_close_rect: Rect,
    /// Hit target of the '🗑 Delete Permanently' button.
    pub confirm_btn_rect: Rect,
    /// Hit target of the 'Cancel' push button.
    pub cancel_btn_rect: Rect,
}

/// Hit testing targets for the New Folder creation modal dialogue.
#[derive(Debug, Clone)]
pub struct NewFolderModalTargets {
    /// Full bounding box of the modal card.
    pub dialog_rect: Rect,
    /// Hit target of the top-right '✖' close icon.
    pub header_close_rect: Rect,
    /// Hit target of the folder name text input box.
    pub input_rect: Rect,
    /// Hit target of the 'Create Folder' button.
    pub confirm_btn_rect: Rect,
    /// Hit target of the 'Cancel' push button.
    pub cancel_btn_rect: Rect,
}

/// Hit testing targets for the Rename asset/folder modal dialogue.
#[derive(Debug, Clone)]
pub struct RenameModalTargets {
    /// Full bounding box of the modal card.
    pub dialog_rect: Rect,
    /// Hit target of the top-right '✖' close icon.
    pub header_close_rect: Rect,
    /// Hit target of the rename text input box.
    pub input_rect: Rect,
    /// Hit target of the 'Apply Rename' button.
    pub confirm_btn_rect: Rect,
    /// Hit target of the 'Cancel' push button.
    pub cancel_btn_rect: Rect,
}

/// Constructs the centered 'Delete Confirmation' modal in the UI tree.
pub fn build_delete_modal(
    tree: &mut UiTree,
    target_path: &Path,
    screen_width: f32,
    screen_height: f32,
    cursor_pos: Point,
) -> (WidgetId, DeleteModalTargets) {
    let frame = ModalDialogBuilder::new("⚠️  Confirm Deletion")
        .size(DELETE_MODAL_WIDTH, DELETE_MODAL_HEIGHT)
        .center_on_screen(screen_width, screen_height)
        .cursor_pos(cursor_pos)
        .scrim(true)
        .close_button(true)
        .cancel_button("Cancel")
        .cancel_button_width(75.0)
        .confirm_button(
            "🗑 Delete Permanently",
            Some(Color::rgba(0.63, 0.14, 0.14, 1.0)),
        )
        .confirm_button_width(150.0)
        .build(tree);

    let left = frame.dialog_rect.x;
    let top = frame.dialog_rect.y;

    // Warning Question Label
    let warn_label = tree.create_node();
    if let Some(node) = tree.get_mut(warn_label) {
        node.set_name("DeleteWarnLabel");
        node.set_text("Are you sure you want to permanently delete this item?");
        node.font_size = 12.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(1.0, 0.45, 0.45, 1.0);
        node.computed_rect = Rect::new(left + 18.0, top + 46.0, DELETE_MODAL_WIDTH - 36.0, 18.0);
    }
    let _ = tree.add_child(frame.card_id, warn_label);

    // Target File Path Label
    let path_label = tree.create_node();
    if let Some(node) = tree.get_mut(path_label) {
        node.set_name("DeletePathLabel");
        node.set_text(target_path.display().to_string());
        node.font_size = 11.5;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.70, 0.72, 0.78, 1.0);
        node.computed_rect = Rect::new(left + 18.0, top + 70.0, DELETE_MODAL_WIDTH - 36.0, 36.0);
    }
    let _ = tree.add_child(frame.card_id, path_label);

    let targets = DeleteModalTargets {
        dialog_rect: frame.dialog_rect,
        header_close_rect: frame.header_close_rect.unwrap_or_default(),
        confirm_btn_rect: frame.confirm_btn_rect.unwrap_or_default(),
        cancel_btn_rect: frame.cancel_btn_rect.unwrap_or_default(),
    };

    (frame.scrim_id.unwrap_or(frame.card_id), targets)
}

/// Parameters for constructing the 'Create New Folder' modal.
pub struct FolderModalParams<'a> {
    /// Target parent directory where the new subfolder will be created.
    pub parent_path: &'a Path,
    /// Currently typed folder name string.
    pub input_text: &'a str,
    /// Measured horizontal text width in physical pixels.
    pub text_width: f32,
    /// Whether the text editing caret should be drawn this frame (blink cycle).
    pub cursor_blink_visible: bool,
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
    /// Currently typed new name string.
    pub input_text: &'a str,
    /// Measured horizontal text width in physical pixels.
    pub text_width: f32,
    /// Whether the target is a folder (true) or an asset file (false).
    pub is_folder: bool,
    /// Whether the text editing caret should be drawn this frame (blink cycle).
    pub cursor_blink_visible: bool,
    /// Viewport width in physical pixels.
    pub screen_width: f32,
    /// Viewport height in physical pixels.
    pub screen_height: f32,
    /// Current mouse cursor coordinates.
    pub cursor_pos: Point,
}

/// Constructs the centered 'Create New Folder' modal in the UI tree.
pub fn build_new_folder_modal(
    tree: &mut UiTree,
    params: FolderModalParams<'_>,
) -> (WidgetId, NewFolderModalTargets) {
    let parent_path = params.parent_path;
    let input_text = params.input_text;
    let text_width = params.text_width;
    let cursor_blink_visible = params.cursor_blink_visible;
    let screen_width = params.screen_width;
    let screen_height = params.screen_height;
    let cursor_pos = params.cursor_pos;

    let frame = ModalDialogBuilder::new("Create New Folder")
        .size(INPUT_MODAL_WIDTH, INPUT_MODAL_HEIGHT)
        .center_on_screen(screen_width, screen_height)
        .cursor_pos(cursor_pos)
        .scrim(true)
        .close_button(true)
        .cancel_button("Cancel")
        .cancel_button_width(75.0)
        .confirm_button("Create Folder", Some(Color::rgba(0.12, 0.16, 0.24, 1.0)))
        .confirm_button_width(112.0)
        .build(tree);

    let left = frame.dialog_rect.x;
    let top = frame.dialog_rect.y;

    // Header Icon (GPU SDF Texture Array ICON_FOLDER)
    let icon_node = tree.create_node();
    if let Some(node) = tree.get_mut(icon_node) {
        node.set_name("NewFolderIcon");
        node.set_role(WidgetRole::ModalWindow);
        node.set_texture_uv(crate::ui::iris_bridge::icons::ICON_FOLDER);
        node.computed_rect = Rect::new(left + 14.0, top + 9.0, 14.0, 14.0);
        node.set_texture_tint(Color::rgba(0.95, 0.76, 0.28, 1.0));
    }
    let _ = tree.add_child(frame.header_id, icon_node);

    // Location Subtitle
    let loc_label = tree.create_node();
    if let Some(node) = tree.get_mut(loc_label) {
        node.set_name("NewFolderLocation");
        node.set_text(format!("Location: {}", parent_path.display()));
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.60, 0.62, 0.70, 1.0);
        node.computed_rect = Rect::new(left + 18.0, top + 46.0, INPUT_MODAL_WIDTH - 36.0, 18.0);
    }
    let _ = tree.add_child(frame.card_id, loc_label);

    // Input Field Box
    let input_rect = Rect::new(left + 18.0, top + 74.0, INPUT_MODAL_WIDTH - 36.0, 28.0);
    let input_box = tree.create_node();
    if let Some(node) = tree.get_mut(input_box) {
        node.set_name("NewFolderInputBox");
        node.computed_rect = input_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.0, 0.85, 0.95, 0.80))
            .background(Color::rgba(0.05, 0.05, 0.07, 1.0));
    }
    let _ = tree.add_child(frame.card_id, input_box);

    // Input Text Content
    let input_text_node = tree.create_node();
    if let Some(node) = tree.get_mut(input_text_node) {
        node.set_name("NewFolderInputText");
        let display = if input_text.is_empty() {
            "Enter folder name..."
        } else {
            input_text
        };
        node.set_text(display);
        node.font_size = 12.0;
        node.line_height = 28.0;
        node.text_color = if input_text.is_empty() {
            Color::rgba(0.45, 0.45, 0.52, 1.0)
        } else {
            Color::WHITE
        };
        node.computed_rect = Rect::new(
            input_rect.x + 8.0,
            input_rect.y,
            input_rect.width - 16.0,
            28.0,
        );
    }
    let _ = tree.add_child(input_box, input_text_node);

    // Blinking Caret Cursor (530ms cycle)
    if cursor_blink_visible {
        let caret_x = if input_text.is_empty() {
            input_rect.x + 8.0
        } else {
            (input_rect.x + 8.0 + text_width + 1.0).min(input_rect.x + input_rect.width - 12.0)
        };
        let caret_node = tree.create_node();
        if let Some(node) = tree.get_mut(caret_node) {
            node.set_name("NewFolderCaret");
            node.computed_rect = Rect::new(caret_x, input_rect.y + 6.0, 1.5, 16.0);
            node.style = Style::new()
                .background(Color::rgba(0.0, 0.90, 1.0, 0.95))
                .border_radius(0.75);
        }
        let _ = tree.add_child(input_box, caret_node);
    }

    let targets = NewFolderModalTargets {
        dialog_rect: frame.dialog_rect,
        header_close_rect: frame.header_close_rect.unwrap_or_default(),
        input_rect,
        confirm_btn_rect: frame.confirm_btn_rect.unwrap_or_default(),
        cancel_btn_rect: frame.cancel_btn_rect.unwrap_or_default(),
    };

    (frame.scrim_id.unwrap_or(frame.card_id), targets)
}

/// Constructs the centered 'Rename Asset / Folder' modal in the UI tree.
pub fn build_rename_modal(
    tree: &mut UiTree,
    params: RenameModalParams<'_>,
) -> (WidgetId, RenameModalTargets) {
    let target_path = params.target_path;
    let input_text = params.input_text;
    let text_width = params.text_width;
    let is_folder = params.is_folder;
    let cursor_blink_visible = params.cursor_blink_visible;
    let screen_width = params.screen_width;
    let screen_height = params.screen_height;
    let cursor_pos = params.cursor_pos;

    let frame = ModalDialogBuilder::new(if is_folder {
        "🔄  Rename Folder"
    } else {
        "🔄  Rename Asset"
    })
    .size(INPUT_MODAL_WIDTH, INPUT_MODAL_HEIGHT)
    .center_on_screen(screen_width, screen_height)
    .cursor_pos(cursor_pos)
    .scrim(true)
    .close_button(true)
    .cancel_button("Cancel")
    .cancel_button_width(75.0)
    .confirm_button("Apply Rename", Some(Color::rgba(0.12, 0.16, 0.24, 1.0)))
    .confirm_button_width(112.0)
    .build(tree);

    let left = frame.dialog_rect.x;
    let top = frame.dialog_rect.y;

    // Target Subtitle
    let target_label = tree.create_node();
    if let Some(node) = tree.get_mut(target_label) {
        node.set_name("RenameTarget");
        node.set_text(format!("Target: {}", target_path.display()));
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.60, 0.62, 0.70, 1.0);
        node.computed_rect = Rect::new(left + 18.0, top + 46.0, INPUT_MODAL_WIDTH - 36.0, 18.0);
    }
    let _ = tree.add_child(frame.card_id, target_label);

    // Input Field Box
    let input_rect = Rect::new(left + 18.0, top + 74.0, INPUT_MODAL_WIDTH - 36.0, 28.0);
    let input_box = tree.create_node();
    if let Some(node) = tree.get_mut(input_box) {
        node.set_name("RenameInputBox");
        node.computed_rect = input_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.0, 0.85, 0.95, 0.80))
            .background(Color::rgba(0.05, 0.05, 0.07, 1.0));
    }
    let _ = tree.add_child(frame.card_id, input_box);

    // Input Text Content
    let input_text_node = tree.create_node();
    if let Some(node) = tree.get_mut(input_text_node) {
        node.set_name("RenameInputText");
        let display = if input_text.is_empty() {
            "Enter new name..."
        } else {
            input_text
        };
        node.set_text(display);
        node.font_size = 12.0;
        node.line_height = 28.0;
        node.text_color = if input_text.is_empty() {
            Color::rgba(0.45, 0.45, 0.52, 1.0)
        } else {
            Color::WHITE
        };
        node.computed_rect = Rect::new(
            input_rect.x + 8.0,
            input_rect.y,
            input_rect.width - 16.0,
            28.0,
        );
    }
    let _ = tree.add_child(input_box, input_text_node);

    // Blinking Caret Cursor (530ms cycle)
    if cursor_blink_visible {
        let caret_x = if input_text.is_empty() {
            input_rect.x + 8.0
        } else {
            (input_rect.x + 8.0 + text_width + 1.0).min(input_rect.x + input_rect.width - 12.0)
        };
        let caret_node = tree.create_node();
        if let Some(node) = tree.get_mut(caret_node) {
            node.set_name("RenameCaret");
            node.computed_rect = Rect::new(caret_x, input_rect.y + 6.0, 1.5, 16.0);
            node.style = Style::new()
                .background(Color::rgba(0.0, 0.90, 1.0, 0.95))
                .border_radius(0.75);
        }
        let _ = tree.add_child(input_box, caret_node);
    }

    let targets = RenameModalTargets {
        dialog_rect: frame.dialog_rect,
        header_close_rect: frame.header_close_rect.unwrap_or_default(),
        input_rect,
        confirm_btn_rect: frame.confirm_btn_rect.unwrap_or_default(),
        cancel_btn_rect: frame.cancel_btn_rect.unwrap_or_default(),
    };

    (frame.scrim_id.unwrap_or(frame.card_id), targets)
}