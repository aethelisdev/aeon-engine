// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Detailed Multi-Column Table List View.
//!
//! Renders structured table rows with category badges, canonical vector icons,
//! size metrics, memory residency tags, and direct action triggers.
//!

use super::cards::resolve_category_color;
use super::types::{AssetRowTarget, AssetsPanelParams, AssetsPanelTargets};
use crate::ui::iris_bridge::icons::{
    ICON_AUDIO, ICON_CUBE, ICON_FOLDER, ICON_SPHERE, ICON_WIREFRAME, ICON_WORLD,
};
use crate::ui::panels::assets::types::AssetCategory;
use irisui::prelude::*;

/// Height of an individual table row in pixels.
pub const LIST_ROW_HEIGHT: f32 = 28.0;

/// Height of the table header row in pixels.
pub const LIST_HEADER_HEIGHT: f32 = 26.0;

/// Constructs the multi-column table rows into the Iris `UiTree`.
pub fn build_asset_list_table(
    tree: &mut UiTree,
    parent_id: WidgetId,
    vp_rect: Rect,
    params: &AssetsPanelParams<'_>,
    targets: &mut AssetsPanelTargets,
) {
    let total_w = vp_rect.width - 16.0;
    let col_name_w = (total_w * 0.40).max(140.0);
    let col_cat_w = 64.0;
    let col_size_w = 80.0;
    let col_status_w = 72.0;
    let col_act_w = (total_w - col_name_w - col_cat_w - col_size_w - col_status_w).max(80.0);

    // 1. Table Header Row
    let hdr_y = vp_rect.y + 4.0;
    let hdr_rect = Rect::new(vp_rect.x + 8.0, hdr_y, total_w, LIST_HEADER_HEIGHT);
    let hdr_id = tree.create_node();
    if let Some(node) = tree.get_mut(hdr_id) {
        node.set_name("ListTableHeader");
        node.computed_rect = hdr_rect;
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.95))
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.60));
    }
    let _ = tree.add_child(parent_id, hdr_id);

    let mut cur_col_x = hdr_rect.x + 8.0;

    // Header Column Labels
    build_header_label(tree, hdr_id, "Name", cur_col_x, hdr_y, col_name_w - 8.0);
    cur_col_x += col_name_w;
    build_header_label(tree, hdr_id, "Category", cur_col_x, hdr_y, col_cat_w);
    cur_col_x += col_cat_w;
    build_header_label(tree, hdr_id, "Size / Metric", cur_col_x, hdr_y, col_size_w);
    cur_col_x += col_size_w;
    build_header_label(tree, hdr_id, "Status", cur_col_x, hdr_y, col_status_w);
    cur_col_x += col_status_w;
    build_header_label(tree, hdr_id, "Actions", cur_col_x, hdr_y, col_act_w);

    // 2. Table Data Rows
    let mut cur_y = vp_rect.y + LIST_HEADER_HEIGHT + 8.0 - params.scroll_y;

    for (row_idx, item) in params.filtered_items.iter().enumerate() {
        let row_y = cur_y;
        cur_y += LIST_ROW_HEIGHT + 2.0;

        // Viewport Scissor Cull: skip row if outside visible viewport
        if row_y + LIST_ROW_HEIGHT < vp_rect.y || row_y > vp_rect.bottom() {
            continue;
        }

        let row_rect = Rect::new(vp_rect.x + 8.0, row_y, total_w, LIST_ROW_HEIGHT);
        let is_selected = params.selected_asset == Some(&item.path);
        let is_hovered = row_rect.contains_point(params.cursor_pos);

        let row_id = tree.create_node();
        if let Some(node) = tree.get_mut(row_id) {
            node.set_name("ListTableRow");
            node.computed_rect = row_rect;
            let bg_color = if is_selected {
                Color::rgba(0.10, 0.16, 0.24, 0.95)
            } else if is_hovered {
                Color::rgba(0.12, 0.14, 0.18, 0.85)
            } else if row_idx.is_multiple_of(2) {
                Color::rgba(0.06, 0.07, 0.09, 0.60)
            } else {
                Color::rgba(0.08, 0.09, 0.11, 0.60)
            };
            let border_color = if is_selected {
                Color::rgba(0.0, 0.90, 1.0, 0.90)
            } else {
                Color::TRANSPARENT
            };
            node.style = Style::new()
                .background(bg_color)
                .border_radius(4.0)
                .border(1.0, border_color);
        }
        let _ = tree.add_child(parent_id, row_id);

        let mut row_col_x = row_rect.x + 8.0;

        // 1. Column Name (Icon + Text)
        let icon_size = 14.0;
        let icon_y = row_y + (LIST_ROW_HEIGHT - icon_size) * 0.5;
        let icon_rect = Rect::new(row_col_x, icon_y, icon_size, icon_size);
        let icon_id = tree.create_node();
        if let Some(node) = tree.get_mut(icon_id) {
            node.set_name("ListRowIcon");
            node.computed_rect = icon_rect;
            let (uv_coords, tint) = resolve_list_icon(item.category);
            node.set_texture_uv(uv_coords);
            node.set_texture_tint(tint);
        }
        let _ = tree.add_child(row_id, icon_id);

        let name_x = row_col_x + icon_size + 6.0;
        let name_w = col_name_w - icon_size - 14.0;
        let name_rect = Rect::new(name_x, row_y, name_w, LIST_ROW_HEIGHT);
        let name_id = tree.create_node();
        if let Some(node) = tree.get_mut(name_id) {
            node.set_name("ListRowName");
            node.set_text(&item.name);
            node.font_size = 11.5;
            node.line_height = LIST_ROW_HEIGHT;
            node.text_color = if is_selected {
                Color::rgba(0.0, 0.95, 1.0, 1.0)
            } else if is_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.85, 0.88, 0.94, 1.0)
            };
            node.computed_rect = name_rect;
        }
        let _ = tree.add_child(row_id, name_id);
        row_col_x += col_name_w;

        // 2. Column Category Badge
        let cat_color = resolve_category_color(item.category);
        let cat_rect = Rect::new(row_col_x, row_y + 4.0, 42.0, LIST_ROW_HEIGHT - 8.0);
        let cat_id = tree.create_node();
        if let Some(node) = tree.get_mut(cat_id) {
            node.set_name("ListRowCategory");
            node.set_text(item.category.badge());
            node.font_size = 9.0;
            node.line_height = LIST_ROW_HEIGHT - 8.0;
            node.text_align = TextAlign::Center;
            node.text_color = cat_color;
            node.computed_rect = cat_rect;
            node.style = Style::new()
                .background(Color::rgba(cat_color.r, cat_color.g, cat_color.b, 0.16))
                .border_radius(3.0);
        }
        let _ = tree.add_child(row_id, cat_id);
        row_col_x += col_cat_w;

        // 3. Column Size / Metric
        let size_rect = Rect::new(row_col_x, row_y, col_size_w - 4.0, LIST_ROW_HEIGHT);
        let size_id = tree.create_node();
        if let Some(node) = tree.get_mut(size_id) {
            node.set_name("ListRowSize");
            node.set_text(&item.metadata_badge);
            node.font_size = 10.5;
            node.line_height = LIST_ROW_HEIGHT;
            node.text_color = Color::rgba(0.60, 0.64, 0.74, 1.0);
            node.computed_rect = size_rect;
        }
        let _ = tree.add_child(row_id, size_id);
        row_col_x += col_size_w;

        // 4. Column Status
        let status_rect = Rect::new(row_col_x, row_y + 5.0, 48.0, LIST_ROW_HEIGHT - 10.0);
        let status_id = tree.create_node();
        if let Some(node) = tree.get_mut(status_id) {
            node.set_name("ListRowStatus");
            node.set_text(if item.is_loaded_in_memory {
                "VRAM"
            } else {
                "Disk"
            });
            node.font_size = 9.0;
            node.line_height = LIST_ROW_HEIGHT - 10.0;
            node.text_align = TextAlign::Center;
            node.text_color = if item.is_loaded_in_memory {
                Color::rgba(0.0, 0.90, 1.0, 1.0)
            } else {
                Color::rgba(0.50, 0.54, 0.62, 1.0)
            };
            node.computed_rect = status_rect;
            node.style = Style::new()
                .background(if item.is_loaded_in_memory {
                    Color::rgba(0.0, 0.40, 0.50, 0.25)
                } else {
                    Color::rgba(0.14, 0.16, 0.20, 0.40)
                })
                .border_radius(3.0);
        }
        let _ = tree.add_child(row_id, status_id);
        row_col_x += col_status_w;

        // 5. Column Actions ("Spawn", "Inspect")
        let btn_h = 18.0;
        let btn_y = row_y + (LIST_ROW_HEIGHT - btn_h) * 0.5;

        // Spawn action button
        let spawn_rect = Rect::new(row_col_x, btn_y, 44.0, btn_h);
        let is_spawn_hovered = spawn_rect.contains_point(params.cursor_pos);
        let spawn_id = tree.create_node();
        if let Some(node) = tree.get_mut(spawn_id) {
            node.set_name("ListRowSpawnBtn");
            node.set_text("Spawn");
            node.font_size = 9.5;
            node.line_height = btn_h;
            node.text_align = TextAlign::Center;
            node.text_color = if is_spawn_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.70, 0.75, 0.85, 1.0)
            };
            node.computed_rect = spawn_rect;
            node.style = Style::new()
                .background(if is_spawn_hovered {
                    Color::rgba(0.18, 0.24, 0.35, 1.0)
                } else {
                    Color::rgba(0.12, 0.14, 0.18, 0.80)
                })
                .border_radius(3.0)
                .border(
                    1.0,
                    if is_spawn_hovered {
                        Color::rgba(0.0, 0.85, 1.0, 0.80)
                    } else {
                        Color::rgba(0.20, 0.23, 0.30, 0.50)
                    },
                );
        }
        let _ = tree.add_child(row_id, spawn_id);

        // Inspect action button
        let inspect_rect = Rect::new(row_col_x + 48.0, btn_y, 44.0, btn_h);
        let is_inspect_hovered = inspect_rect.contains_point(params.cursor_pos);
        let inspect_id = tree.create_node();
        if let Some(node) = tree.get_mut(inspect_id) {
            node.set_name("ListRowInspectBtn");
            node.set_text("Inspect");
            node.font_size = 9.5;
            node.line_height = btn_h;
            node.text_align = TextAlign::Center;
            node.text_color = if is_inspect_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.70, 0.75, 0.85, 1.0)
            };
            node.computed_rect = inspect_rect;
            node.style = Style::new()
                .background(if is_inspect_hovered {
                    Color::rgba(0.18, 0.24, 0.35, 1.0)
                } else {
                    Color::rgba(0.12, 0.14, 0.18, 0.80)
                })
                .border_radius(3.0)
                .border(
                    1.0,
                    if is_inspect_hovered {
                        Color::rgba(0.35, 0.42, 0.55, 0.80)
                    } else {
                        Color::rgba(0.20, 0.23, 0.30, 0.50)
                    },
                );
        }
        let _ = tree.add_child(row_id, inspect_id);

        // Register Target
        targets.list_rows.push(AssetRowTarget {
            rect: row_rect,
            spawn_btn_rect: Some(spawn_rect),
            inspect_btn_rect: Some(inspect_rect),
            path: item.path.clone(),
            category: item.category,
            item: item.clone(),
        });
    }
}

/// Helper function to build a table header column label.
fn build_header_label(
    tree: &mut UiTree,
    parent_id: WidgetId,
    label: &'static str,
    x: f32,
    y: f32,
    w: f32,
) {
    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name("HeaderColumnLabel");
        node.set_text(label);
        node.font_size = 11.0;
        node.line_height = LIST_HEADER_HEIGHT;
        node.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
        node.computed_rect = Rect::new(x, y, w, LIST_HEADER_HEIGHT);
    }
    let _ = tree.add_child(parent_id, lbl_id);
}

/// Resolves the canonical vector icon texture UV coordinates and color tint for a list row.
fn resolve_list_icon(category: AssetCategory) -> ([f32; 4], Color) {
    match category {
        AssetCategory::Models3D => (ICON_CUBE, Color::rgba(0.0, 0.90, 1.0, 1.0)),
        AssetCategory::Textures2D => (ICON_FOLDER, Color::rgba(0.39, 0.86, 0.47, 1.0)),
        AssetCategory::Shaders => (ICON_WIREFRAME, Color::rgba(1.0, 0.75, 0.24, 1.0)),
        AssetCategory::Scenes => (ICON_WORLD, Color::rgba(0.31, 0.63, 1.0, 1.0)),
        AssetCategory::Materials => (ICON_SPHERE, Color::rgba(0.86, 0.39, 0.86, 1.0)),
        AssetCategory::Audio => (ICON_AUDIO, Color::rgba(1.0, 0.47, 0.39, 1.0)),
        AssetCategory::All => (ICON_FOLDER, Color::WHITE),
    }
}