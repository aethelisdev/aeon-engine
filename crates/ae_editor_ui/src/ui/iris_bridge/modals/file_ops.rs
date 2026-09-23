// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # File Operations Modal Dialogues
//!
//! Renders hardware-accelerated GPU SDF modal cards for file system operations
//! including permanent delete confirmations, new folder creation, and file renaming
//! purely via declarative [`UiScope`].
//!

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
) -> WidgetId {
    let parent = tree.root().unwrap_or_default();
    let mut scope = UiScope::new(tree, parent);

    scope.modal_scrim(Color::rgba(0.0, 0.0, 0.0, 0.60), |scrim| {
        scrim.modal_card(DELETE_MODAL_WIDTH, DELETE_MODAL_HEIGHT, |card| {
            // 1. Header Bar: Title and '✕' close button
            card.container(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::SpaceBetween),
                |header| {
                    header.label(
                        "⚠️  Confirm Deletion",
                        12.5,
                        Color::rgba(0.95, 0.95, 0.98, 1.0),
                        TextAlign::Left,
                    );
                    header.modal_close_button();
                },
            );

            // 2. Body Column: Description warning and target file path
            card.column(|col| {
                col.label(
                    "Are you sure you want to permanently delete this item?",
                    12.0,
                    Color::rgba(1.0, 0.45, 0.45, 1.0),
                    TextAlign::Left,
                );
                col.label(
                    target_path.display().to_string(),
                    11.5,
                    Color::rgba(0.70, 0.72, 0.78, 1.0),
                    TextAlign::Left,
                );
            });

            // 3. Footer Bar: Cancel and Danger Delete buttons
            card.container(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::FlexEnd)
                    .gap(12.0),
                |footer| {
                    footer.modal_cancel_button("Cancel", 75.0);
                    footer.modal_danger_button("🗑 Delete Permanently", 150.0);
                },
            );
        });

        scrim
            .finish_layout_with_hover(Rect::new(0.0, 0.0, screen_width, screen_height), cursor_pos);
    })
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

/// Constructs the centered 'Create New Folder' modal in the UI tree using declarative [`UiScope`].
pub fn build_new_folder_modal(tree: &mut UiTree, params: FolderModalParams<'_>) -> WidgetId {
    let parent = tree.root().unwrap_or_default();
    let mut scope = UiScope::new(tree, parent);

    scope.modal_scrim(Color::rgba(0.0, 0.0, 0.0, 0.60), |scrim| {
        scrim.modal_card(INPUT_MODAL_WIDTH, INPUT_MODAL_HEIGHT, |card| {
            // 1. Header Bar: Folder icon + Title on left, '✕' close button on right
            card.container(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::SpaceBetween),
                |header| {
                    header.container(
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .gap(8.0),
                        |title_row| {
                            title_row.icon(
                                crate::ui::iris_bridge::icons::ICON_FOLDER,
                                Color::rgba(0.95, 0.76, 0.28, 1.0),
                                14.0,
                            );
                            title_row.label(
                                "Create New Folder",
                                12.5,
                                Color::rgba(0.95, 0.95, 0.98, 1.0),
                                TextAlign::Left,
                            );
                        },
                    );
                    header.modal_close_button();
                },
            );

            // 2. Body Column: Parent directory path and text input box
            card.column(|col| {
                col.label(
                    format!("Location: {}", params.parent_path.display()),
                    11.0,
                    Color::rgba(0.60, 0.62, 0.70, 1.0),
                    TextAlign::Left,
                );
                col.input_box(
                    params.input_text,
                    "Enter folder name...",
                    params.text_width,
                    params.cursor_blink_visible,
                );
            });

            // 3. Footer Bar: Cancel and Confirm action buttons
            card.container(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::FlexEnd)
                    .gap(12.0),
                |footer| {
                    footer.modal_cancel_button("Cancel", 75.0);
                    footer.modal_confirm_button("Create Folder", 112.0);
                },
            );
        });

        scrim.finish_layout_with_hover(
            Rect::new(0.0, 0.0, params.screen_width, params.screen_height),
            params.cursor_pos,
        );
    })
}

/// Constructs the centered 'Rename Asset / Folder' modal in the UI tree using declarative [`UiScope`].
pub fn build_rename_modal(tree: &mut UiTree, params: RenameModalParams<'_>) -> WidgetId {
    let parent = tree.root().unwrap_or_default();
    let mut scope = UiScope::new(tree, parent);

    let title = if params.is_folder {
        "🔄  Rename Folder"
    } else {
        "🔄  Rename Asset"
    };

    scope.modal_scrim(Color::rgba(0.0, 0.0, 0.0, 0.60), |scrim| {
        scrim.modal_card(INPUT_MODAL_WIDTH, INPUT_MODAL_HEIGHT, |card| {
            // 1. Header Bar: Title on left, '✕' close button on right
            card.container(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::SpaceBetween),
                |header| {
                    header.label(
                        title,
                        12.5,
                        Color::rgba(0.95, 0.95, 0.98, 1.0),
                        TextAlign::Left,
                    );
                    header.modal_close_button();
                },
            );

            // 2. Body Column: Target file path and text input box
            card.column(|col| {
                col.label(
                    format!("Target: {}", params.target_path.display()),
                    11.0,
                    Color::rgba(0.60, 0.62, 0.70, 1.0),
                    TextAlign::Left,
                );
                col.input_box(
                    params.input_text,
                    "Enter new name...",
                    params.text_width,
                    params.cursor_blink_visible,
                );
            });

            // 3. Footer Bar: Cancel and Confirm action buttons
            card.container(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::FlexEnd)
                    .gap(12.0),
                |footer| {
                    footer.modal_cancel_button("Cancel", 75.0);
                    footer.modal_confirm_button("Apply Rename", 112.0);
                },
            );
        });

        scrim.finish_layout_with_hover(
            Rect::new(0.0, 0.0, params.screen_width, params.screen_height),
            params.cursor_pos,
        );
    })
}