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
    let left = ((screen_width - DELETE_MODAL_WIDTH) * 0.5).max(0.0).round();
    let top = ((screen_height - DELETE_MODAL_HEIGHT) * 0.5)
        .max(28.0)
        .round();
    let dialog_rect = Rect::new(left, top, DELETE_MODAL_WIDTH, DELETE_MODAL_HEIGHT);

    let header_close_rect = Rect::new(left + DELETE_MODAL_WIDTH - 32.0, top + 5.0, 24.0, 22.0);
    let confirm_btn_rect = Rect::new(
        left + DELETE_MODAL_WIDTH - 170.0,
        top + DELETE_MODAL_HEIGHT - 42.0,
        150.0,
        28.0,
    );
    let cancel_btn_rect = Rect::new(
        left + DELETE_MODAL_WIDTH - 255.0,
        top + DELETE_MODAL_HEIGHT - 42.0,
        75.0,
        28.0,
    );

    let is_header_close_hovered = header_close_rect.contains_point(cursor_pos);
    let is_confirm_hovered = confirm_btn_rect.contains_point(cursor_pos);
    let is_cancel_hovered = cancel_btn_rect.contains_point(cursor_pos);

    // 1. Semi-transparent backdrop scrim
    let scrim_id = tree.create_node();
    if let Some(node) = tree.get_mut(scrim_id) {
        node.set_name("DeleteModalScrim");
        node.computed_rect = Rect::new(0.0, 0.0, screen_width, screen_height);
        node.style = Style::new().background(Color::rgba(0.0, 0.0, 0.0, 0.55));
    }

    // 2. Main Modal Card
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("DeleteModalCard");
        node.computed_rect = dialog_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.08, 0.10, 0.98))
            .border(1.0, Color::rgba(0.24, 0.22, 0.28, 1.0))
            .border_radius(8.0)
            .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.70));
    }
    let _ = tree.add_child(scrim_id, card_id);

    // 3. Header Bar
    let header_id = tree.create_node();
    if let Some(node) = tree.get_mut(header_id) {
        node.set_name("DeleteHeader");
        node.computed_rect = Rect::new(left, top, DELETE_MODAL_WIDTH, 32.0);
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.06, 0.08, 1.0))
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 1.0))
            .border_radius(8.0);
    }
    let _ = tree.add_child(card_id, header_id);

    // Header Title
    let title_label = tree.create_node();
    if let Some(node) = tree.get_mut(title_label) {
        node.set_name("DeleteHeaderTitle");
        node.set_text("⚠️  Confirm Deletion");
        node.font_size = 12.5;
        node.line_height = 16.0;
        node.text_color = Color::rgba(1.0, 0.45, 0.45, 1.0);
        node.computed_rect = Rect::new(left + 14.0, top + 7.0, 220.0, 18.0);
    }
    let _ = tree.add_child(header_id, title_label);

    // Header Close Button
    let close_x = tree.create_node();
    if let Some(node) = tree.get_mut(close_x) {
        node.set_name("DeleteCloseX");
        node.set_text("✖");
        node.font_size = 11.0;
        node.line_height = 14.0;
        node.text_align = TextAlign::Center;
        node.text_color = if is_header_close_hovered {
            Color::rgba(1.0, 0.4, 0.4, 1.0)
        } else {
            Color::rgba(0.60, 0.60, 0.65, 1.0)
        };
        node.computed_rect = header_close_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .background(if is_header_close_hovered {
                Color::rgba(0.8, 0.15, 0.15, 0.40)
            } else {
                Color::TRANSPARENT
            });
    }
    let _ = tree.add_child(header_id, close_x);

    // 4. Warning Question Label
    let warn_label = tree.create_node();
    if let Some(node) = tree.get_mut(warn_label) {
        node.set_name("DeleteWarnLabel");
        node.set_text("Are you sure you want to permanently delete this item?");
        node.font_size = 12.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(1.0, 0.45, 0.45, 1.0);
        node.computed_rect = Rect::new(left + 18.0, top + 46.0, DELETE_MODAL_WIDTH - 36.0, 18.0);
    }
    let _ = tree.add_child(card_id, warn_label);

    // 5. Target File Path Label
    let path_label = tree.create_node();
    if let Some(node) = tree.get_mut(path_label) {
        node.set_name("DeletePathLabel");
        node.set_text(target_path.display().to_string());
        node.font_size = 11.5;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.70, 0.72, 0.78, 1.0);
        node.computed_rect = Rect::new(left + 18.0, top + 70.0, DELETE_MODAL_WIDTH - 36.0, 36.0);
    }
    let _ = tree.add_child(card_id, path_label);

    // 6. Cancel Button
    let cancel_btn = tree.create_node();
    if let Some(node) = tree.get_mut(cancel_btn) {
        node.set_name("DeleteCancelButton");
        node.set_text("Cancel");
        node.font_size = 11.5;
        node.line_height = 28.0;
        node.text_align = TextAlign::Center;
        node.text_color = if is_cancel_hovered {
            Color::WHITE
        } else {
            Color::rgba(0.75, 0.75, 0.80, 1.0)
        };
        node.computed_rect = cancel_btn_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.24, 0.26, 0.34, 1.0))
            .background(if is_cancel_hovered {
                Color::rgba(0.18, 0.20, 0.26, 1.0)
            } else {
                Color::rgba(0.11, 0.12, 0.16, 1.0)
            });
    }
    let _ = tree.add_child(card_id, cancel_btn);

    // 7. Delete Permanently Button
    let del_btn = tree.create_node();
    if let Some(node) = tree.get_mut(del_btn) {
        node.set_name("DeleteConfirmButton");
        node.set_text("🗑 Delete Permanently");
        node.font_size = 11.5;
        node.line_height = 28.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::WHITE;
        node.computed_rect = confirm_btn_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .border(
                1.0,
                if is_confirm_hovered {
                    Color::rgba(1.0, 0.35, 0.35, 1.0)
                } else {
                    Color::rgba(0.80, 0.20, 0.20, 1.0)
                },
            )
            .background(if is_confirm_hovered {
                Color::rgba(0.80, 0.18, 0.18, 1.0)
            } else {
                Color::rgba(0.63, 0.14, 0.14, 1.0)
            });
    }
    let _ = tree.add_child(card_id, del_btn);

    (
        scrim_id,
        DeleteModalTargets {
            dialog_rect,
            header_close_rect,
            confirm_btn_rect,
            cancel_btn_rect,
        },
    )
}

/// Constructs the centered 'Create New Folder' modal in the UI tree.
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

    let left = ((screen_width - INPUT_MODAL_WIDTH) * 0.5).max(0.0).round();
    let top = ((screen_height - INPUT_MODAL_HEIGHT) * 0.5)
        .max(28.0)
        .round();
    let dialog_rect = Rect::new(left, top, INPUT_MODAL_WIDTH, INPUT_MODAL_HEIGHT);

    let header_close_rect = Rect::new(left + INPUT_MODAL_WIDTH - 32.0, top + 5.0, 24.0, 22.0);
    let input_rect = Rect::new(left + 18.0, top + 74.0, INPUT_MODAL_WIDTH - 36.0, 28.0);
    let confirm_btn_rect = Rect::new(
        left + INPUT_MODAL_WIDTH - 130.0,
        top + INPUT_MODAL_HEIGHT - 42.0,
        112.0,
        28.0,
    );
    let cancel_btn_rect = Rect::new(
        left + INPUT_MODAL_WIDTH - 215.0,
        top + INPUT_MODAL_HEIGHT - 42.0,
        75.0,
        28.0,
    );

    let is_header_close_hovered = header_close_rect.contains_point(cursor_pos);
    let is_confirm_hovered = confirm_btn_rect.contains_point(cursor_pos);
    let is_cancel_hovered = cancel_btn_rect.contains_point(cursor_pos);

    // 1. Blocker Scrim
    let scrim_id = tree.create_node();
    if let Some(node) = tree.get_mut(scrim_id) {
        node.set_name("NewFolderScrim");
        node.computed_rect = Rect::new(0.0, 0.0, screen_width, screen_height);
        node.style = Style::new().background(Color::rgba(0.0, 0.0, 0.0, 0.55));
    }

    // 2. Main Card
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("NewFolderCard");
        node.computed_rect = dialog_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.08, 0.10, 0.98))
            .border(1.0, Color::rgba(0.20, 0.22, 0.28, 1.0))
            .border_radius(8.0)
            .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.70));
    }
    let _ = tree.add_child(scrim_id, card_id);

    // 3. Header Bar
    let header_id = tree.create_node();
    if let Some(node) = tree.get_mut(header_id) {
        node.set_name("NewFolderHeader");
        node.computed_rect = Rect::new(left, top, INPUT_MODAL_WIDTH, 32.0);
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.06, 0.08, 1.0))
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 1.0))
            .border_radius(8.0);
    }
    let _ = tree.add_child(card_id, header_id);

    // Header Title
    let title_label = tree.create_node();
    if let Some(node) = tree.get_mut(title_label) {
        node.set_name("NewFolderTitle");
        node.set_text("📁  Create New Folder");
        node.font_size = 12.5;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.88, 0.88, 0.92, 1.0);
        node.computed_rect = Rect::new(left + 14.0, top + 7.0, 220.0, 18.0);
    }
    let _ = tree.add_child(header_id, title_label);

    // Close ✖
    let close_x = tree.create_node();
    if let Some(node) = tree.get_mut(close_x) {
        node.set_name("NewFolderCloseX");
        node.set_text("✖");
        node.font_size = 11.0;
        node.line_height = 14.0;
        node.text_align = TextAlign::Center;
        node.text_color = if is_header_close_hovered {
            Color::rgba(1.0, 0.4, 0.4, 1.0)
        } else {
            Color::rgba(0.60, 0.60, 0.65, 1.0)
        };
        node.computed_rect = header_close_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .background(if is_header_close_hovered {
                Color::rgba(0.8, 0.15, 0.15, 0.40)
            } else {
                Color::TRANSPARENT
            });
    }
    let _ = tree.add_child(header_id, close_x);

    // 4. Location Subtitle
    let loc_label = tree.create_node();
    if let Some(node) = tree.get_mut(loc_label) {
        node.set_name("NewFolderLocation");
        node.set_text(format!("Location: {}", parent_path.display()));
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.60, 0.62, 0.70, 1.0);
        node.computed_rect = Rect::new(left + 18.0, top + 46.0, INPUT_MODAL_WIDTH - 36.0, 18.0);
    }
    let _ = tree.add_child(card_id, loc_label);

    // 5. Input Field Box
    let input_box = tree.create_node();
    if let Some(node) = tree.get_mut(input_box) {
        node.set_name("NewFolderInputBox");
        node.computed_rect = input_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.0, 0.85, 0.95, 0.80))
            .background(Color::rgba(0.05, 0.05, 0.07, 1.0));
    }
    let _ = tree.add_child(card_id, input_box);

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

    // 6. Cancel Button
    let cancel_btn = tree.create_node();
    if let Some(node) = tree.get_mut(cancel_btn) {
        node.set_name("NewFolderCancelButton");
        node.set_text("Cancel");
        node.font_size = 11.5;
        node.line_height = 28.0;
        node.text_align = TextAlign::Center;
        node.text_color = if is_cancel_hovered {
            Color::WHITE
        } else {
            Color::rgba(0.75, 0.75, 0.80, 1.0)
        };
        node.computed_rect = cancel_btn_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.24, 0.26, 0.34, 1.0))
            .background(if is_cancel_hovered {
                Color::rgba(0.18, 0.20, 0.26, 1.0)
            } else {
                Color::rgba(0.11, 0.12, 0.16, 1.0)
            });
    }
    let _ = tree.add_child(card_id, cancel_btn);

    // 7. Create Folder Confirm Button
    let confirm_btn = tree.create_node();
    if let Some(node) = tree.get_mut(confirm_btn) {
        node.set_name("NewFolderConfirmButton");
        node.set_text("Create Folder");
        node.font_size = 11.5;
        node.line_height = 28.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::WHITE;
        node.computed_rect = confirm_btn_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .border(
                1.0,
                if is_confirm_hovered {
                    Color::rgba(0.0, 0.90, 1.0, 0.90)
                } else {
                    Color::rgba(0.25, 0.35, 0.50, 1.0)
                },
            )
            .background(if is_confirm_hovered {
                Color::rgba(0.16, 0.24, 0.36, 1.0)
            } else {
                Color::rgba(0.12, 0.16, 0.24, 1.0)
            });
    }
    let _ = tree.add_child(card_id, confirm_btn);

    (
        scrim_id,
        NewFolderModalTargets {
            dialog_rect,
            header_close_rect,
            input_rect,
            confirm_btn_rect,
            cancel_btn_rect,
        },
    )
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

    let left = ((screen_width - INPUT_MODAL_WIDTH) * 0.5).max(0.0).round();
    let top = ((screen_height - INPUT_MODAL_HEIGHT) * 0.5)
        .max(28.0)
        .round();
    let dialog_rect = Rect::new(left, top, INPUT_MODAL_WIDTH, INPUT_MODAL_HEIGHT);

    let header_close_rect = Rect::new(left + INPUT_MODAL_WIDTH - 32.0, top + 5.0, 24.0, 22.0);
    let input_rect = Rect::new(left + 18.0, top + 74.0, INPUT_MODAL_WIDTH - 36.0, 28.0);
    let confirm_btn_rect = Rect::new(
        left + INPUT_MODAL_WIDTH - 130.0,
        top + INPUT_MODAL_HEIGHT - 42.0,
        112.0,
        28.0,
    );
    let cancel_btn_rect = Rect::new(
        left + INPUT_MODAL_WIDTH - 215.0,
        top + INPUT_MODAL_HEIGHT - 42.0,
        75.0,
        28.0,
    );

    let is_header_close_hovered = header_close_rect.contains_point(cursor_pos);
    let is_confirm_hovered = confirm_btn_rect.contains_point(cursor_pos);
    let is_cancel_hovered = cancel_btn_rect.contains_point(cursor_pos);

    // 1. Blocker Scrim
    let scrim_id = tree.create_node();
    if let Some(node) = tree.get_mut(scrim_id) {
        node.set_name("RenameScrim");
        node.computed_rect = Rect::new(0.0, 0.0, screen_width, screen_height);
        node.style = Style::new().background(Color::rgba(0.0, 0.0, 0.0, 0.55));
    }

    // 2. Main Card
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("RenameCard");
        node.computed_rect = dialog_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.08, 0.10, 0.98))
            .border(1.0, Color::rgba(0.20, 0.22, 0.28, 1.0))
            .border_radius(8.0)
            .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.70));
    }
    let _ = tree.add_child(scrim_id, card_id);

    // 3. Header Bar
    let header_id = tree.create_node();
    if let Some(node) = tree.get_mut(header_id) {
        node.set_name("RenameHeader");
        node.computed_rect = Rect::new(left, top, INPUT_MODAL_WIDTH, 32.0);
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.06, 0.08, 1.0))
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 1.0))
            .border_radius(8.0);
    }
    let _ = tree.add_child(card_id, header_id);

    // Header Title
    let title_label = tree.create_node();
    if let Some(node) = tree.get_mut(title_label) {
        node.set_name("RenameTitle");
        node.set_text(if is_folder {
            "🔄  Rename Folder"
        } else {
            "🔄  Rename Asset"
        });
        node.font_size = 12.5;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.88, 0.88, 0.92, 1.0);
        node.computed_rect = Rect::new(left + 14.0, top + 7.0, 220.0, 18.0);
    }
    let _ = tree.add_child(header_id, title_label);

    // Close ✖
    let close_x = tree.create_node();
    if let Some(node) = tree.get_mut(close_x) {
        node.set_name("RenameCloseX");
        node.set_text("✖");
        node.font_size = 11.0;
        node.line_height = 14.0;
        node.text_align = TextAlign::Center;
        node.text_color = if is_header_close_hovered {
            Color::rgba(1.0, 0.4, 0.4, 1.0)
        } else {
            Color::rgba(0.60, 0.60, 0.65, 1.0)
        };
        node.computed_rect = header_close_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .background(if is_header_close_hovered {
                Color::rgba(0.8, 0.15, 0.15, 0.40)
            } else {
                Color::TRANSPARENT
            });
    }
    let _ = tree.add_child(header_id, close_x);

    // 4. Target Subtitle
    let target_label = tree.create_node();
    if let Some(node) = tree.get_mut(target_label) {
        node.set_name("RenameTarget");
        node.set_text(format!("Target: {}", target_path.display()));
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.60, 0.62, 0.70, 1.0);
        node.computed_rect = Rect::new(left + 18.0, top + 46.0, INPUT_MODAL_WIDTH - 36.0, 18.0);
    }
    let _ = tree.add_child(card_id, target_label);

    // 5. Input Field Box
    let input_box = tree.create_node();
    if let Some(node) = tree.get_mut(input_box) {
        node.set_name("RenameInputBox");
        node.computed_rect = input_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.0, 0.85, 0.95, 0.80))
            .background(Color::rgba(0.05, 0.05, 0.07, 1.0));
    }
    let _ = tree.add_child(card_id, input_box);

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

    // 6. Cancel Button
    let cancel_btn = tree.create_node();
    if let Some(node) = tree.get_mut(cancel_btn) {
        node.set_name("RenameCancelButton");
        node.set_text("Cancel");
        node.font_size = 11.5;
        node.line_height = 28.0;
        node.text_align = TextAlign::Center;
        node.text_color = if is_cancel_hovered {
            Color::WHITE
        } else {
            Color::rgba(0.75, 0.75, 0.80, 1.0)
        };
        node.computed_rect = cancel_btn_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.24, 0.26, 0.34, 1.0))
            .background(if is_cancel_hovered {
                Color::rgba(0.18, 0.20, 0.26, 1.0)
            } else {
                Color::rgba(0.11, 0.12, 0.16, 1.0)
            });
    }
    let _ = tree.add_child(card_id, cancel_btn);

    // 7. Apply Rename Confirm Button
    let confirm_btn = tree.create_node();
    if let Some(node) = tree.get_mut(confirm_btn) {
        node.set_name("RenameConfirmButton");
        node.set_text("Apply Rename");
        node.font_size = 11.5;
        node.line_height = 28.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::WHITE;
        node.computed_rect = confirm_btn_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .border(
                1.0,
                if is_confirm_hovered {
                    Color::rgba(0.0, 0.90, 1.0, 0.90)
                } else {
                    Color::rgba(0.25, 0.35, 0.50, 1.0)
                },
            )
            .background(if is_confirm_hovered {
                Color::rgba(0.16, 0.24, 0.36, 1.0)
            } else {
                Color::rgba(0.12, 0.16, 0.24, 1.0)
            });
    }
    let _ = tree.add_child(card_id, confirm_btn);

    (
        scrim_id,
        RenameModalTargets {
            dialog_rect,
            header_close_rect,
            input_rect,
            confirm_btn_rect,
            cancel_btn_rect,
        },
    )
}