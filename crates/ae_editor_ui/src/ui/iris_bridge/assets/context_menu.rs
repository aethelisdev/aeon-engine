// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Browser Right-Click Context Menu Builder
//!
//! Renders floating context menus for asset cards and folder tree nodes,
//! supporting quick inspection, spawning, renaming, deletion, path copying,
//! and OS explorer reveal operations.
//!

use super::types::{
    AssetsContextMenuTarget, AssetsContextMenuTargets, AssetsPanelParams, AssetsPanelTargets,
};
use crate::ui::panels::assets::types::AssetCategory;
use irisui::prelude::*;

/// Height of an individual context menu item row in logical pixels.
pub const CONTEXT_ITEM_HEIGHT: f32 = 26.0;

/// Width of the floating context menu popup card in logical pixels.
pub const CONTEXT_MENU_WIDTH: f32 = 190.0;

/// Builds the floating right-click context menu into the `UiTree` if currently open.
pub fn build_assets_context_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &AssetsPanelParams<'_>,
    targets: &mut AssetsPanelTargets,
) {
    let (target, click_pos) = match params.active_context_menu {
        Some(ctx_menu) => ctx_menu,
        None => {
            targets.context_menu = None;
            return;
        }
    };

    // Determine menu height based on target type
    let is_folder_root = match target {
        AssetsContextMenuTarget::Folder(path) => {
            path == std::path::Path::new("assets") || path.as_os_str().is_empty()
        }
        AssetsContextMenuTarget::Asset(_) => false,
    };

    let item_count = match target {
        AssetsContextMenuTarget::Asset(_) => 6, // Inspect, Spawn, Rename, Delete, Copy Path, Reveal
        AssetsContextMenuTarget::Folder(_) => {
            if is_folder_root {
                2 // New Subfolder, Reveal
            } else {
                4 // New Subfolder, Rename, Delete, Reveal
            }
        }
    };

    let menu_h = 32.0 + (item_count as f32) * CONTEXT_ITEM_HEIGHT + 8.0;
    let menu_w = CONTEXT_MENU_WIDTH;

    // Clamp menu to stay within panel boundary
    let mut menu_x = click_pos.x;
    let mut menu_y = click_pos.y;

    if menu_x + menu_w > params.panel_rect.right() - 8.0 {
        menu_x = (params.panel_rect.right() - menu_w - 8.0).max(params.panel_rect.x);
    }
    if menu_y + menu_h > params.panel_rect.bottom() - 8.0 {
        menu_y = (params.panel_rect.bottom() - menu_h - 8.0).max(params.panel_rect.y);
    }
    // Final clamp against overall window screen bounds
    menu_x = menu_x.min(params.screen_size.0 - menu_w - 8.0).max(8.0);
    menu_y = menu_y.min(params.screen_size.1 - menu_h - 8.0).max(8.0);

    let card_rect = Rect::new(menu_x, menu_y, menu_w, menu_h);

    // 1. Floating Menu Background Card Node
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("AssetsContextMenuCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.07, 0.08, 0.11, 0.98))
            .border_radius(6.0)
            .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.70))
            .box_shadow(0.0, 4.0, 14.0, Color::rgba(0.0, 0.0, 0.0, 0.85));
    }
    let _ = tree.add_child(parent_id, card_id);

    // 2. Header Bar: Subject Identifier
    let header_rect = Rect::new(menu_x + 8.0, menu_y + 6.0, menu_w - 16.0, 18.0);
    let header_id = tree.create_node();
    if let Some(node) = tree.get_mut(header_id) {
        node.set_name("ContextMenuHeader");
        let title = match target {
            AssetsContextMenuTarget::Asset(item) => {
                let name = &item.name;
                if name.len() > 18 {
                    format!("{}...", &name[..15])
                } else {
                    name.clone()
                }
            }
            AssetsContextMenuTarget::Folder(path) => {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("assets");
                format!("📁 {}", name)
            }
        };
        node.set_text(&title);
        node.font_size = 11.0;
        node.line_height = 18.0;
        node.text_align = TextAlign::Left;
        node.text_color = Color::rgba(0.0, 0.90, 1.0, 0.95);
        node.computed_rect = header_rect;
    }
    let _ = tree.add_child(card_id, header_id);

    // 3. Render Context Menu Items
    let mut cur_y = menu_y + 28.0;
    let mut inspect_rect = None;
    let mut spawn_rect = None;
    let mut new_folder_rect = None;
    let mut rename_rect = None;
    let mut delete_rect = None;
    let mut copy_path_rect = None;

    let reveal_rect = match target {
        AssetsContextMenuTarget::Asset(item) => {
            // Item 1: Quick Inspect (Space)
            let rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, CONTEXT_ITEM_HEIGHT);
            let r = render_context_item(
                tree,
                card_id,
                rect,
                "Quick Inspect (Space)",
                false,
                params.cursor_pos,
            );
            inspect_rect = Some(r);
            cur_y += CONTEXT_ITEM_HEIGHT;

            // Item 2: Primary Spawn Action
            let spawn_label = match item.category {
                AssetCategory::Models3D => "Spawn into Scene",
                AssetCategory::Textures2D => "Spawn as Sprite",
                AssetCategory::Scenes => "Load Scene",
                _ => "Select Asset",
            };
            let rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, CONTEXT_ITEM_HEIGHT);
            let r = render_context_item(tree, card_id, rect, spawn_label, false, params.cursor_pos);
            spawn_rect = Some(r);
            cur_y += CONTEXT_ITEM_HEIGHT;

            // Item 3: Rename (F2)
            let rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, CONTEXT_ITEM_HEIGHT);
            let r =
                render_context_item(tree, card_id, rect, "Rename (F2)", false, params.cursor_pos);
            rename_rect = Some(r);
            cur_y += CONTEXT_ITEM_HEIGHT;

            // Item 4: Delete Asset
            let rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, CONTEXT_ITEM_HEIGHT);
            let r =
                render_context_item(tree, card_id, rect, "Delete Asset", true, params.cursor_pos);
            delete_rect = Some(r);
            cur_y += CONTEXT_ITEM_HEIGHT;

            // Item 5: Copy File Path
            let rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, CONTEXT_ITEM_HEIGHT);
            let r = render_context_item(
                tree,
                card_id,
                rect,
                "Copy File Path",
                false,
                params.cursor_pos,
            );
            copy_path_rect = Some(r);
            cur_y += CONTEXT_ITEM_HEIGHT;

            // Item 6: Reveal in Explorer
            let rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, CONTEXT_ITEM_HEIGHT);
            let r = render_context_item(
                tree,
                card_id,
                rect,
                "Reveal in Explorer",
                false,
                params.cursor_pos,
            );
            Some(r)
        }
        AssetsContextMenuTarget::Folder(_) => {
            // Item 1: New Subfolder
            let rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, CONTEXT_ITEM_HEIGHT);
            let r = render_context_item(
                tree,
                card_id,
                rect,
                "New Subfolder",
                false,
                params.cursor_pos,
            );
            new_folder_rect = Some(r);
            cur_y += CONTEXT_ITEM_HEIGHT;

            if !is_folder_root {
                // Item 2: Rename Folder
                let rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, CONTEXT_ITEM_HEIGHT);
                let r = render_context_item(
                    tree,
                    card_id,
                    rect,
                    "Rename Folder",
                    false,
                    params.cursor_pos,
                );
                rename_rect = Some(r);
                cur_y += CONTEXT_ITEM_HEIGHT;

                // Item 3: Delete Folder
                let rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, CONTEXT_ITEM_HEIGHT);
                let r = render_context_item(
                    tree,
                    card_id,
                    rect,
                    "Delete Folder",
                    true,
                    params.cursor_pos,
                );
                delete_rect = Some(r);
                cur_y += CONTEXT_ITEM_HEIGHT;
            }

            // Item 4: Reveal in Explorer
            let rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, CONTEXT_ITEM_HEIGHT);
            let r = render_context_item(
                tree,
                card_id,
                rect,
                "Reveal in Explorer",
                false,
                params.cursor_pos,
            );
            Some(r)
        }
    };

    targets.context_menu = Some(AssetsContextMenuTargets {
        card_rect,
        inspect_rect,
        spawn_rect,
        new_folder_rect,
        rename_rect,
        delete_rect,
        copy_path_rect,
        reveal_rect,
        target: target.clone(),
    });
}

/// Helper for rendering an individual hoverable context menu item row.
fn render_context_item(
    tree: &mut UiTree,
    parent_id: WidgetId,
    item_rect: Rect,
    label: &str,
    is_destructive: bool,
    cursor_pos: Point,
) -> Rect {
    let is_hovered = item_rect.contains_point(cursor_pos);

    let (bg, text_col) = if is_hovered {
        if is_destructive {
            (
                Color::rgba(0.42, 0.10, 0.10, 0.90),
                Color::rgba(1.0, 0.50, 0.50, 1.0),
            )
        } else {
            (
                Color::rgba(0.12, 0.18, 0.28, 0.95),
                Color::rgba(0.20, 0.90, 1.0, 1.0),
            )
        }
    } else if is_destructive {
        (Color::TRANSPARENT, Color::rgba(0.95, 0.40, 0.40, 0.90))
    } else {
        (Color::TRANSPARENT, Color::rgba(0.82, 0.86, 0.94, 0.90))
    };

    let item_id = tree.create_node();
    if let Some(node) = tree.get_mut(item_id) {
        node.set_name("ContextMenuItem");
        node.set_text(label);
        node.font_size = 11.5;
        node.line_height = CONTEXT_ITEM_HEIGHT;
        node.text_align = TextAlign::Left;
        node.text_color = text_col;
        node.computed_rect = item_rect;
        node.style = Style::new().background(bg).border_radius(3.0);
    }
    let _ = tree.add_child(parent_id, item_id);

    item_rect
}