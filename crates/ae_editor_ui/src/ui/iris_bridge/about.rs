// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # About Aeon Engine Modal Dialog
//!
//! Renders the premium GPU SDF modal dialogue containing version details,
//! copyright, MPL 2.0 licensing terms, and interactive close buttons directly via
//! declarative [`UiScope`].
//!

use irisui::prelude::*;

/// Width of the modal dialogue card in physical pixels.
pub const ABOUT_DIALOG_WIDTH: f32 = 480.0;
/// Height of the modal dialogue card in physical pixels.
pub const ABOUT_DIALOG_HEIGHT: f32 = 250.0;

/// Opens the provided web URL in the user's default web browser.
///
/// Dispatches OS-specific shell commands to launch external web links without blocking
/// or mutating editor UI state.
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
/// Assembles the complete modal hierarchy purely through declarative [`UiScope`]
/// primitives without raw node instantiation, comprising:
/// 1. A full-screen backdrop blocker ([`UiScope::modal_scrim`]) tagged with [`MODAL_TAG_SCRIM`].
/// 2. An elevated modal card container ([`UiScope::modal_card`]) centered dynamically on the screen.
/// 3. A title header row with left-aligned branding label and a right-aligned [`UiScope::modal_close_button`].
/// 4. A centered body column displaying engine typography, copyright notice, separator, clickable MPL-2.0 license hyperlink, and warranty disclaimer.
/// 5. An action footer containing the primary [`UiScope::modal_confirm_button`].
///
/// Returns the root allocated [`WidgetId`] (the backdrop scrim blocker).
pub fn build_about_dialog(
    tree: &mut UiTree,
    screen_width: f32,
    screen_height: f32,
    cursor_pos: Point,
    events: &[(WidgetId, InteractionEvent)],
    hovered_id: Option<WidgetId>,
) -> WidgetId {
    let parent = tree.root().unwrap_or_default();
    let mut scope = UiScope::with_interactions(tree, parent, events, hovered_id);

    scope.modal_scrim(Color::rgba(0.0, 0.0, 0.0, 0.55), |scrim| {
        scrim.modal_card(ABOUT_DIALOG_WIDTH, ABOUT_DIALOG_HEIGHT, |card| {
            // 1. Header Bar: Title and '✖' close button
            card.container(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::SpaceBetween),
                |header| {
                    header.label(
                        "ℹ  About Aeon Engine",
                        12.5,
                        Color::rgba(0.88, 0.88, 0.92, 1.0),
                        TextAlign::Left,
                    );
                    header.modal_close_button();
                },
            );

            // 2. Body Column: Engine identity, copyright, license link, and warranty
            card.column(|col| {
                col.label(
                    "Aeon Engine",
                    20.0,
                    Color::rgba(0.0, 0.90, 1.0, 1.0),
                    TextAlign::Center,
                );
                col.label(
                    "Copyright (C) 2026 AethelisDEV / Aeon Engine",
                    11.5,
                    Color::rgba(0.65, 0.65, 0.70, 1.0),
                    TextAlign::Center,
                );
                col.separator();
                col.label(
                    "This Source Code Form is subject to the terms of the Mozilla Public",
                    11.0,
                    Color::rgba(0.72, 0.72, 0.78, 1.0),
                    TextAlign::Center,
                );
                if col
                    .link(
                        "License, v. 2.0 (MPL-2.0). https://mozilla.org/MPL/2.0/",
                        11.0,
                    )
                    .clicked()
                {
                    open_url("https://mozilla.org/MPL/2.0/");
                }
                col.label(
                    "This program comes with ABSOLUTELY NO WARRANTY.",
                    11.5,
                    Color::rgba(1.0, 0.40, 0.40, 1.0),
                    TextAlign::Center,
                );
            });

            // 3. Footer Bar: Confirmation close button
            card.container(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::FlexEnd),
                |footer| {
                    footer.modal_confirm_button("Close", 100.0);
                },
            );
        });

        scrim
            .finish_layout_with_hover(Rect::new(0.0, 0.0, screen_width, screen_height), cursor_pos);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_about_dialog_declarative_structure_and_layout() {
        let mut tree = UiTree::new();
        let _root = tree.create_root().expect("Root node should be created");

        let screen_w = 1920.0;
        let screen_h = 1080.0;

        let scrim_id = build_about_dialog(
            &mut tree,
            screen_w,
            screen_h,
            Point::new(0.0, 0.0),
            &[],
            None,
        );

        let scrim_node = tree.get(scrim_id).expect("Scrim node must exist");
        assert_eq!(scrim_node.tag, MODAL_TAG_SCRIM);
        assert_eq!(scrim_node.role, WidgetRole::ModalWindow);
        assert_eq!(scrim_node.layer, UiLayer::Modal);
        assert_eq!(
            scrim_node.computed_rect,
            Rect::new(0.0, 0.0, screen_w, screen_h)
        );

        // Card container is first child of scrim
        let card_id = scrim_node.children[0];
        let card_node = tree.get(card_id).expect("Card node must exist");
        assert_eq!(card_node.role, WidgetRole::ModalWindow);
        assert_eq!(card_node.layer, UiLayer::Modal);
        assert_eq!(card_node.computed_rect.width, ABOUT_DIALOG_WIDTH);
        assert_eq!(card_node.computed_rect.height, ABOUT_DIALOG_HEIGHT);
        assert_eq!(
            card_node.computed_rect.x,
            (screen_w - ABOUT_DIALOG_WIDTH) * 0.5
        );
        assert_eq!(
            card_node.computed_rect.y,
            (screen_h - ABOUT_DIALOG_HEIGHT) * 0.5
        );

        // Verify required action tags exist in card subtree
        let mut has_close_btn = false;
        let mut has_confirm_btn = false;
        let mut has_title = false;
        let mut has_license_link = false;

        tree.traverse_depth_first(card_id, &mut |_id, node| {
            if node.tag == MODAL_TAG_CLOSE {
                has_close_btn = true;
            }
            if node.tag == MODAL_TAG_CONFIRM {
                has_confirm_btn = true;
            }
            if let Some(text) = &node.text {
                if text.contains("About Aeon Engine") {
                    has_title = true;
                }
                if text.contains("MPL-2.0") {
                    has_license_link = true;
                }
            }
        });

        assert!(
            has_close_btn,
            "Close button with MODAL_TAG_CLOSE must exist"
        );
        assert!(
            has_confirm_btn,
            "Confirm button with MODAL_TAG_CONFIRM must exist"
        );
        assert!(has_title, "Title label must exist");
        assert!(has_license_link, "MPL-2.0 license link must exist");
    }
}