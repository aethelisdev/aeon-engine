// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # About Aeon Engine Modal Dialog
//!
//! Renders the premium GPU SDF modal dialogue containing version details,
//! copyright, MPL 2.0 licensing terms, and interactive close buttons directly via Iris UI.
//!
//! Powered by Iris UI's native [`ModalDialogBuilder`].

use irisui::prelude::*;

/// Width of the modal dialogue card in physical pixels.
pub const ABOUT_DIALOG_WIDTH: f32 = 480.0;
/// Height of the modal dialogue card in physical pixels.
pub const ABOUT_DIALOG_HEIGHT: f32 = 250.0;

/// Interactive hit targets returned by the About dialog layout builder.
#[derive(Debug, Clone)]
pub struct AboutDialogTargets {
    /// Full bounding box of the dialog card.
    pub dialog_rect: Rect,
    /// Hit target of the top-right '✖' close icon.
    pub header_close_rect: Rect,
    /// Hit target of the bottom 'Close' push button.
    pub bottom_close_rect: Rect,
    /// Hit target of the MPL-2.0 hyperlink.
    pub link_rect: Rect,
}

/// Opens the provided web URL in the user's default web browser.
pub fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

/// Constructs the centered 'About Aeon Engine' modal dialogue in the UI tree.
///
/// Delegates modal frame construction (scrim, card, header bar, close icon, bottom button)
/// to Iris UI's native [`ModalDialogBuilder`].
pub fn build_about_dialog(
    tree: &mut UiTree,
    screen_width: f32,
    screen_height: f32,
    cursor_pos: Point,
) -> (WidgetId, AboutDialogTargets) {
    let frame = ModalDialogBuilder::new("ℹ  About Aeon Engine")
        .size(ABOUT_DIALOG_WIDTH, ABOUT_DIALOG_HEIGHT)
        .center_on_screen(screen_width, screen_height)
        .cursor_pos(cursor_pos)
        .confirm_button("Close", None)
        .confirm_button_width(100.0)
        .build(tree);

    let left = frame.dialog_rect.x;
    let top = frame.dialog_rect.y;
    let link_rect = Rect::new(left + 40.0, top + 124.0, ABOUT_DIALOG_WIDTH - 80.0, 18.0);
    let is_link_hovered = link_rect.contains_point(cursor_pos);

    let link_color = if is_link_hovered {
        Color::rgba(0.35, 0.95, 1.0, 1.0)
    } else {
        Color::rgba(0.0, 0.85, 0.95, 1.0)
    };

    let mut scope = UiScope::new(tree, frame.content_id);
    scope.column(|col| {
        col.text_colored("Aeon Engine", Color::rgba(0.0, 0.90, 1.0, 1.0));
        col.text_colored(
            "Copyright (C) 2026 AethelisDEV / Aeon Engine",
            Color::rgba(0.65, 0.65, 0.70, 1.0),
        );
        col.separator();
        col.text_colored(
            "This Source Code Form is subject to the terms of the Mozilla Public",
            Color::rgba(0.72, 0.72, 0.78, 1.0),
        );
        col.text_colored(
            "License, v. 2.0 (MPL-2.0). https://mozilla.org/MPL/2.0/",
            link_color,
        );
        col.text_colored(
            "This program comes with ABSOLUTELY NO WARRANTY.",
            Color::rgba(1.0, 0.40, 0.40, 1.0),
        );
    });

    let targets = AboutDialogTargets {
        dialog_rect: frame.dialog_rect,
        header_close_rect: frame.header_close_rect.unwrap_or_default(),
        bottom_close_rect: frame.confirm_btn_rect.unwrap_or_default(),
        link_rect,
    };

    (frame.scrim_id.unwrap_or(frame.card_id), targets)
}