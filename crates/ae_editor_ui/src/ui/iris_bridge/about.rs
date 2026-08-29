// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # About Aeon Engine Modal Dialog
//!
//! Renders the premium GPU SDF modal dialogue containing version details,
//! copyright, MPL 2.0 licensing terms, and interactive close buttons directly via Iris UI.

use irisui::prelude::*;

/// Width of the modal dialogue card in physical pixels.
pub const ABOUT_DIALOG_WIDTH: f32 = 480.0;
/// Height of the modal dialogue card in physical pixels.
pub const ABOUT_DIALOG_HEIGHT: f32 = 250.0;

/// Interactive hit targets returned by the About dialog layout builder.
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
pub fn build_about_dialog(
    tree: &mut UiTree,
    screen_width: f32,
    screen_height: f32,
    cursor_pos: Point,
) -> (WidgetId, AboutDialogTargets) {
    let left = ((screen_width - ABOUT_DIALOG_WIDTH) * 0.5).max(0.0).round();
    let top = ((screen_height - ABOUT_DIALOG_HEIGHT) * 0.5)
        .max(28.0)
        .round();
    let dialog_rect = Rect::new(left, top, ABOUT_DIALOG_WIDTH, ABOUT_DIALOG_HEIGHT);

    let header_close_rect = Rect::new(left + ABOUT_DIALOG_WIDTH - 32.0, top + 5.0, 24.0, 22.0);
    let bottom_close_rect = Rect::new(
        left + (ABOUT_DIALOG_WIDTH - 100.0) * 0.5,
        top + ABOUT_DIALOG_HEIGHT - 44.0,
        100.0,
        28.0,
    );
    let link_rect = Rect::new(left + 40.0, top + 124.0, ABOUT_DIALOG_WIDTH - 80.0, 18.0);

    let is_header_close_hovered = header_close_rect.contains_point(cursor_pos);
    let is_bottom_close_hovered = bottom_close_rect.contains_point(cursor_pos);
    let is_link_hovered = link_rect.contains_point(cursor_pos);

    // 1. Semi-transparent backdrop scrim (full screen modal blocker)
    let scrim_id = tree.create_node();
    if let Some(node) = tree.get_mut(scrim_id) {
        node.set_name("AboutModalScrim");
        node.computed_rect = Rect::new(0.0, 0.0, screen_width, screen_height);
        node.style = Style::new().background(Color::rgba(0.0, 0.0, 0.0, 0.55));
    }

    // 2. Main Dialog Card Container (SDF rounded corners + stroke + dark theme)
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("AboutDialogCard");
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
        node.set_name("AboutHeader");
        node.computed_rect = Rect::new(left, top, ABOUT_DIALOG_WIDTH, 32.0);
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.06, 0.08, 1.0))
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 1.0))
            .border_radius(8.0);
    }
    let _ = tree.add_child(card_id, header_id);

    // Header Title Label
    let title_label = tree.create_node();
    if let Some(node) = tree.get_mut(title_label) {
        node.set_name("AboutHeaderTitle");
        node.set_text("ℹ  About Aeon Engine");
        node.font_size = 12.5;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.88, 0.88, 0.92, 1.0);
        node.computed_rect = Rect::new(left + 14.0, top + 7.0, 240.0, 18.0);
    }
    let _ = tree.add_child(header_id, title_label);

    // Header Close (✖) Button
    let close_x = tree.create_node();
    if let Some(node) = tree.get_mut(close_x) {
        node.set_name("AboutCloseX");
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

    // 4a. Engine Title (Cyan/Aqua Accent)
    let engine_title = tree.create_node();
    if let Some(node) = tree.get_mut(engine_title) {
        node.set_name("AeonEngineTitle");
        node.set_text("Aeon Engine");
        node.font_size = 20.0;
        node.line_height = 24.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.0, 0.90, 1.0, 1.0);
        node.computed_rect = Rect::new(left + 20.0, top + 44.0, ABOUT_DIALOG_WIDTH - 40.0, 24.0);
    }
    let _ = tree.add_child(card_id, engine_title);

    // 4b. Copyright Subtext
    let copyright_label = tree.create_node();
    if let Some(node) = tree.get_mut(copyright_label) {
        node.set_name("CopyrightLabel");
        node.set_text("Copyright (C) 2026 AethelisDEV / Aeon Engine");
        node.font_size = 11.5;
        node.line_height = 16.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.65, 0.65, 0.70, 1.0);
        node.computed_rect = Rect::new(left + 20.0, top + 70.0, ABOUT_DIALOG_WIDTH - 40.0, 16.0);
    }
    let _ = tree.add_child(card_id, copyright_label);

    // 4c. Separator Line
    let sep_id = tree.create_node();
    if let Some(node) = tree.get_mut(sep_id) {
        node.set_name("AboutSeparator");
        node.computed_rect = Rect::new(left + 24.0, top + 94.0, ABOUT_DIALOG_WIDTH - 48.0, 1.0);
        node.style = Style::new().background(Color::rgba(0.18, 0.20, 0.26, 1.0));
    }
    let _ = tree.add_child(card_id, sep_id);

    // 4d. MPL 2.0 License Notice (Line 1)
    let license_line1 = tree.create_node();
    if let Some(node) = tree.get_mut(license_line1) {
        node.set_name("LicenseLine1");
        node.set_text("This Source Code Form is subject to the terms of the Mozilla Public");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.72, 0.72, 0.78, 1.0);
        node.computed_rect = Rect::new(left + 20.0, top + 104.0, ABOUT_DIALOG_WIDTH - 40.0, 16.0);
    }
    let _ = tree.add_child(card_id, license_line1);

    // 4e. MPL 2.0 License Notice (Interactive Hyperlink Line)
    let license_line2 = tree.create_node();
    if let Some(node) = tree.get_mut(license_line2) {
        node.set_name("LicenseLine2");
        node.set_text("License, v. 2.0 (MPL-2.0). https://mozilla.org/MPL/2.0/");
        node.font_size = 11.0;
        node.line_height = 16.0;
        node.text_align = TextAlign::Center;
        node.text_color = if is_link_hovered {
            Color::rgba(0.35, 0.95, 1.0, 1.0)
        } else {
            Color::rgba(0.0, 0.85, 0.95, 1.0)
        };
        node.computed_rect = link_rect;
    }
    let _ = tree.add_child(card_id, license_line2);

    // 4f. Warranty Disclaimer (Soft Red Warning)
    let warranty_label = tree.create_node();
    if let Some(node) = tree.get_mut(warranty_label) {
        node.set_name("WarrantyLabel");
        node.set_text("This program comes with ABSOLUTELY NO WARRANTY.");
        node.font_size = 11.5;
        node.line_height = 16.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(1.0, 0.40, 0.40, 1.0);
        node.computed_rect = Rect::new(left + 20.0, top + 150.0, ABOUT_DIALOG_WIDTH - 40.0, 18.0);
    }
    let _ = tree.add_child(card_id, warranty_label);

    // 4g. Bottom Close Push Button
    let btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(btn_id) {
        node.set_name("AboutCloseButton");
        node.set_text("Close");
        node.font_size = 12.0;
        node.line_height = 28.0;
        node.text_align = TextAlign::Center;
        node.text_color = if is_bottom_close_hovered {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.85, 0.85, 0.90, 1.0)
        };
        node.computed_rect = bottom_close_rect;
        node.style = Style::new()
            .border_radius(4.0)
            .border(
                1.0,
                if is_bottom_close_hovered {
                    Color::rgba(0.0, 0.85, 0.95, 0.80)
                } else {
                    Color::rgba(0.24, 0.26, 0.34, 1.0)
                },
            )
            .background(if is_bottom_close_hovered {
                Color::rgba(0.18, 0.22, 0.30, 1.0)
            } else {
                Color::rgba(0.12, 0.14, 0.18, 1.0)
            });
    }
    let _ = tree.add_child(card_id, btn_id);

    (
        scrim_id,
        AboutDialogTargets {
            dialog_rect,
            header_close_rect,
            bottom_close_rect,
            link_rect,
        },
    )
}