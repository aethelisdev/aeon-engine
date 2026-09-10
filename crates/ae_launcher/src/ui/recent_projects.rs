// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use irisui::prelude::*;

use crate::icons::{ICON_CUBE, ICON_FOLDER, ICON_SELECT};
use crate::project::ProjectRegistry;
use crate::ui::types::ViewLayoutContext;

/// Builds the Recent Projects tab content view into the UI tree.
/// Iterates over recent projects stored in the registry, constructs project cards,
/// badges (2D vs 3D icon indicator), file paths, and launch action buttons.
pub fn build_recent_projects_view(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: ViewLayoutContext<'_>,
    registry: &ProjectRegistry,
) {
    let start_x = ctx.start_x;
    let content_w = ctx.content_w;
    let cursor_pos = ctx.cursor_pos;

    // Header title
    let header_id = tree.create_node();
    if let Some(node) = tree.get_mut(header_id) {
        node.set_text("Projects");
        node.font_size = 20.0;
        node.text_color = Color::hex("#ffffff");
        node.computed_rect = Rect::new(start_x + 32.0, 24.0, 200.0, 28.0);
    }
    let _ = tree.add_child(parent_id, header_id);

    let subhead_id = tree.create_node();
    if let Some(node) = tree.get_mut(subhead_id) {
        node.set_text("Select a recent project to launch the editor.");
        node.font_size = 11.5;
        node.text_color = Color::hex("#73798c");
        node.computed_rect = Rect::new(start_x + 32.0, 54.0, 400.0, 20.0);
    }
    let _ = tree.add_child(parent_id, subhead_id);

    let mut card_y = 90.0;
    let card_w = content_w - 64.0;
    let card_h = 56.0;

    if registry.recent_projects.is_empty() {
        let empty_id = tree.create_node();
        if let Some(node) = tree.get_mut(empty_id) {
            node.set_text("No recent projects found. Click 'New Project' to create one.");
            node.font_size = 12.0;
            node.text_color = Color::hex("#525866");
            node.computed_rect = Rect::new(start_x + 32.0, card_y + 20.0, 500.0, 24.0);
        }
        let _ = tree.add_child(parent_id, empty_id);
        return;
    }

    for (idx, project) in registry.recent_projects.iter().enumerate() {
        if idx >= 10 {
            break;
        }

        let card_rect = Rect::new(start_x + 32.0, card_y, card_w, card_h);
        let is_hovered = card_rect.contains_point(cursor_pos);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.computed_rect = card_rect;
            let bg = if is_hovered {
                Color::hex("#181c25")
            } else {
                Color::hex("#13161d")
            };
            let border = if is_hovered {
                Color::hex("#283040")
            } else {
                Color::hex("#1e222d")
            };
            node.style = Style::new()
                .background(bg)
                .border_radius(8.0)
                .border(1.0, border);
        }
        let _ = tree.add_child(parent_id, card_id);

        // Project Mode Badge: 2D (🟩 Emerald) vs 3D (🟦 Cyan Cube)
        let is_2d = project.dimension_mode.to_uppercase() == "2D";
        let badge_icon_id = tree.create_node();
        if let Some(node) = tree.get_mut(badge_icon_id) {
            node.computed_rect = Rect::new(start_x + 48.0, card_y + 18.0, 20.0, 20.0);
            if is_2d {
                node.set_texture_uv(ICON_FOLDER);
                node.set_texture_tint(Color::hex("#2ecc71"));
            } else {
                node.set_texture_uv(ICON_CUBE);
                node.set_texture_tint(Color::hex("#00e5ff"));
            }
        }
        let _ = tree.add_child(card_id, badge_icon_id);

        // Project Name
        let name_id = tree.create_node();
        if let Some(node) = tree.get_mut(name_id) {
            node.set_text(&project.name);
            node.font_size = 13.0;
            node.text_color = Color::hex("#ffffff");
            node.computed_rect = Rect::new(start_x + 78.0, card_y + 11.0, 300.0, 18.0);
        }
        let _ = tree.add_child(card_id, name_id);

        // Project Path & Mode Badge Text
        let detail_text = format!(
            "[{}] • {}",
            if is_2d { "2D Sprite" } else { "3D Spatial" },
            project.path
        );
        let path_id = tree.create_node();
        if let Some(node) = tree.get_mut(path_id) {
            node.set_text(&detail_text);
            node.font_size = 10.5;
            node.text_color = Color::hex("#6b7280");
            node.computed_rect = Rect::new(start_x + 78.0, card_y + 30.0, card_w - 200.0, 16.0);
        }
        let _ = tree.add_child(card_id, path_id);

        // Launch Button on right side of card
        let btn_w = 78.0;
        let btn_h = 28.0;
        let btn_rect = Rect::new(
            start_x + 32.0 + card_w - btn_w - 14.0,
            card_y + (card_h - btn_h) * 0.5,
            btn_w,
            btn_h,
        );
        let btn_hovered = btn_rect.contains_point(cursor_pos);

        let launch_btn_id = tree.create_node();
        if let Some(node) = tree.get_mut(launch_btn_id) {
            node.computed_rect = btn_rect;
            node.style = Style::new()
                .background(if btn_hovered {
                    Color::hex("#00c4db")
                } else {
                    Color::hex("#00e5ff")
                })
                .border_radius(5.0);
        }
        let _ = tree.add_child(card_id, launch_btn_id);

        let launch_icon_id = tree.create_node();
        if let Some(node) = tree.get_mut(launch_icon_id) {
            node.computed_rect = Rect::new(
                start_x + 32.0 + card_w - btn_w + 8.0,
                card_y + (card_h - btn_h) * 0.5 + 7.0,
                14.0,
                14.0,
            );
            node.set_texture_uv(ICON_SELECT);
            node.set_texture_tint(Color::hex("#0a0c10"));
        }
        let _ = tree.add_child(card_id, launch_icon_id);

        let launch_lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(launch_lbl_id) {
            node.set_text("Open");
            node.font_size = 11.0;
            node.text_color = Color::hex("#0a0c10");
            node.computed_rect = Rect::new(
                start_x + 32.0 + card_w - btn_w + 26.0,
                card_y + (card_h - btn_h) * 0.5 + 6.0,
                46.0,
                16.0,
            );
        }
        let _ = tree.add_child(card_id, launch_lbl_id);

        card_y += card_h + 10.0;
    }
}