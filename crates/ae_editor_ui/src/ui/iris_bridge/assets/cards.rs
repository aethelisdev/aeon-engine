// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Interactive Card Grid View.
//!
//! Renders responsive wrapping grid cards with category badges, memory status indicators,
//! canonical vector icons, truncated names, and file size metadata.
//!

use super::types::{AssetCardTarget, AssetsPanelParams, AssetsPanelTargets};
use crate::ui::iris_bridge::icons::{
    ICON_AUDIO, ICON_CAMERA, ICON_CUBE, ICON_FOLDER, ICON_LIGHT, ICON_SPHERE, ICON_WORLD,
};
use crate::ui::panels::assets::types::AssetCategory;
use irisui::prelude::*;

/// Standard width of an asset grid card in logical pixels.
pub const CARD_WIDTH: f32 = 115.0;

/// Standard height of an asset grid card in logical pixels.
pub const CARD_HEIGHT: f32 = 125.0;

/// Horizontal and vertical spacing between adjacent grid cards.
pub const CARD_SPACING: f32 = 10.0;

/// Constructs the responsive grid cards into the Iris `UiTree`.
pub fn build_asset_grid_cards(
    tree: &mut UiTree,
    parent_id: WidgetId,
    vp_rect: Rect,
    params: &AssetsPanelParams<'_>,
    targets: &mut AssetsPanelTargets,
) {
    if params.filtered_items.is_empty() {
        build_empty_assets_notice(tree, parent_id, vp_rect, params.search_query);
        return;
    }

    let items = params.filtered_items;
    let avail_w = (vp_rect.width - 20.0).max(CARD_WIDTH);
    let cols = ((avail_w + CARD_SPACING) / (CARD_WIDTH + CARD_SPACING))
        .floor()
        .max(1.0) as usize;

    let start_x = vp_rect.x + 10.0;
    let mut cur_y = vp_rect.y + 10.0 - params.scroll_y;

    for chunk in items.chunks(cols) {
        let row_y = cur_y;
        cur_y += CARD_HEIGHT + CARD_SPACING;

        // Viewport Scissor Cull: skip rows completely outside visible screen
        if row_y + CARD_HEIGHT < vp_rect.y || row_y > vp_rect.bottom() {
            continue;
        }

        for (col_idx, item) in chunk.iter().enumerate() {
            let card_x = start_x + (col_idx as f32 * (CARD_WIDTH + CARD_SPACING));
            let card_rect = Rect::new(card_x, row_y, CARD_WIDTH, CARD_HEIGHT);
            let is_selected = params.selected_asset == Some(&item.path);
            let is_hovered = card_rect.contains_point(params.cursor_pos);

            let cat_color = resolve_category_color(item.category);

            // 1. Card Outer Container
            let card_id = tree.create_node();
            if let Some(node) = tree.get_mut(card_id) {
                node.set_name("AssetCard");
                node.computed_rect = card_rect;
                let bg_color = if is_selected {
                    Color::rgba(0.10, 0.13, 0.18, 0.98)
                } else if is_hovered {
                    Color::rgba(0.11, 0.12, 0.16, 0.95)
                } else {
                    Color::rgba(0.07, 0.08, 0.10, 0.95)
                };
                let border_color = if is_selected {
                    Color::rgba(0.0, 0.90, 1.0, 1.0) // Aeon Cyan selected
                } else if is_hovered {
                    cat_color
                } else {
                    Color::rgba(0.16, 0.18, 0.23, 0.70)
                };
                let border_width = if is_selected { 1.5 } else { 1.0 };
                node.style = Style::new()
                    .background(bg_color)
                    .border_radius(6.0)
                    .border(border_width, border_color)
                    .clip_children(true);
            }
            let _ = tree.add_child(parent_id, card_id);

            // 2. Category Pill Badge (Top Left)
            let badge_rect = Rect::new(card_x + 6.0, row_y + 6.0, 38.0, 16.0);
            let badge_id = tree.create_node();
            if let Some(node) = tree.get_mut(badge_id) {
                node.set_name("CategoryBadge");
                node.set_text(item.category.badge());
                node.font_size = 9.0;
                node.line_height = 16.0;
                node.text_align = TextAlign::Center;
                node.text_color = cat_color;
                node.computed_rect = badge_rect;
                node.style = Style::new()
                    .background(Color::rgba(cat_color.r, cat_color.g, cat_color.b, 0.16))
                    .border_radius(3.0);
            }
            let _ = tree.add_child(card_id, badge_id);

            // 3. VRAM Resident Indicator Dot (Top Right)
            if item.is_loaded_in_memory {
                let dot_rect = Rect::new(card_rect.right() - 14.0, row_y + 8.0, 7.0, 7.0);
                let dot_id = tree.create_node();
                if let Some(node) = tree.get_mut(dot_id) {
                    node.set_name("VramResidentDot");
                    node.computed_rect = dot_rect;
                    node.style = Style::new()
                        .background(Color::rgba(0.0, 0.90, 1.0, 1.0))
                        .border_radius(3.5);
                }
                let _ = tree.add_child(card_id, dot_id);
            }

            // 4. Center Thumbnail / Vector Icon Preview Box (54x54 px)
            let box_size = 54.0;
            let box_x = card_x + (CARD_WIDTH - box_size) * 0.5;
            let box_y = row_y + 26.0;
            let box_rect = Rect::new(box_x, box_y, box_size, box_size);

            let preview_box_id = tree.create_node();
            if let Some(node) = tree.get_mut(preview_box_id) {
                node.set_name("ThumbnailBox");
                node.computed_rect = box_rect;
                node.style = Style::new()
                    .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
                    .border_radius(4.0)
                    .border(1.0, Color::rgba(0.16, 0.18, 0.24, 0.60));
            }
            let _ = tree.add_child(card_id, preview_box_id);

            if let Some(&layer) = params.thumbnail_layers.get(&item.path) {
                // Real rendered thumbnail quad
                let thumb_id = tree.create_node();
                if let Some(node) = tree.get_mut(thumb_id) {
                    node.set_name("CardRealThumbnail");
                    node.computed_rect = box_rect;
                    node.set_texture_uv([0.0, 0.0, 1.0, layer as f32]);
                    node.set_texture_tint(Color::WHITE);
                    node.style = Style::new().border_radius(4.0);
                }
                let _ = tree.add_child(preview_box_id, thumb_id);
            } else {
                // Category Vector Icon quad centered in preview box
                let (uv_coords, tint_color) = resolve_category_icon(item.category);
                let icon_dim = 28.0;
                let icon_rect = Rect::new(
                    box_x + (box_size - icon_dim) * 0.5,
                    box_y + (box_size - icon_dim) * 0.5,
                    icon_dim,
                    icon_dim,
                );
                let icon_id = tree.create_node();
                if let Some(node) = tree.get_mut(icon_id) {
                    node.set_name("CardVectorIcon");
                    node.computed_rect = icon_rect;
                    node.set_texture_uv(uv_coords);
                    node.set_texture_tint(tint_color);
                }
                let _ = tree.add_child(preview_box_id, icon_id);
            }

            // 5. Truncated Asset Name Label
            let display_name = if item.name.len() > 14 {
                format!("{}...", &item.name[..11])
            } else {
                item.name.clone()
            };
            let name_rect = Rect::new(card_x + 4.0, row_y + 84.0, CARD_WIDTH - 8.0, 16.0);
            let name_id = tree.create_node();
            if let Some(node) = tree.get_mut(name_id) {
                node.set_name("CardAssetName");
                node.set_text(&display_name);
                node.font_size = 11.0;
                node.line_height = 16.0;
                node.text_align = TextAlign::Center;
                node.text_color = if is_selected {
                    Color::WHITE
                } else if is_hovered {
                    Color::rgba(0.92, 0.94, 0.98, 1.0)
                } else {
                    Color::rgba(0.80, 0.83, 0.90, 1.0)
                };
                node.computed_rect = name_rect;
            }
            let _ = tree.add_child(card_id, name_id);

            // 6. Metadata Badge / Size Label (Bottom)
            let meta_rect = Rect::new(card_x + 4.0, row_y + 102.0, CARD_WIDTH - 8.0, 14.0);
            let meta_id = tree.create_node();
            if let Some(node) = tree.get_mut(meta_id) {
                node.set_name("CardMetadata");
                node.set_text(&item.metadata_badge);
                node.font_size = 9.5;
                node.line_height = 14.0;
                node.text_align = TextAlign::Center;
                node.text_color = Color::rgba(0.50, 0.54, 0.64, 1.0);
                node.computed_rect = meta_rect;
            }
            let _ = tree.add_child(card_id, meta_id);

            // Register Hit Target
            targets.grid_cards.push(AssetCardTarget {
                rect: card_rect,
                path: item.path.clone(),
                category: item.category,
                item: item.clone(),
            });
        }
    }
}

/// Constructs the empty state notice when no assets are found.
fn build_empty_assets_notice(tree: &mut UiTree, parent_id: WidgetId, vp_rect: Rect, query: &str) {
    let notice_w = 420.0;
    let notice_h = 130.0;
    let notice_x = vp_rect.x + (vp_rect.width - notice_w) * 0.5;
    let notice_y = vp_rect.y + (vp_rect.height - notice_h) * 0.4;
    let notice_rect = Rect::new(notice_x, notice_y, notice_w, notice_h);

    let box_id = tree.create_node();
    if let Some(node) = tree.get_mut(box_id) {
        node.set_name("EmptyAssetsBox");
        node.computed_rect = notice_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.09, 0.12, 0.95))
            .border_radius(6.0)
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.70));
    }
    let _ = tree.add_child(parent_id, box_id);

    // Large Vector Folder Logo (`ICON_FOLDER`, Layer 6)
    let logo_size = 32.0;
    let logo_rect = Rect::new(
        notice_x + (notice_w - logo_size) * 0.5,
        notice_y + 14.0,
        logo_size,
        logo_size,
    );
    let logo_id = tree.create_node();
    if let Some(node) = tree.get_mut(logo_id) {
        node.set_name("EmptyNoticeFolderLogo");
        node.computed_rect = logo_rect;
        node.set_texture_uv(ICON_FOLDER);
        node.set_texture_tint(Color::rgba(0.95, 0.76, 0.28, 0.85));
    }
    let _ = tree.add_child(box_id, logo_id);

    // Title text
    let title_text = if query.is_empty() {
        "No Assets Found in Active Directory"
    } else {
        "No Assets Matching Search Query"
    };
    let title_rect = Rect::new(notice_x + 10.0, notice_y + 52.0, notice_w - 20.0, 20.0);
    let title_id = tree.create_node();
    if let Some(node) = tree.get_mut(title_id) {
        node.set_name("EmptyNoticeTitle");
        node.set_text(title_text);
        node.font_size = 13.0;
        node.line_height = 20.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::WHITE;
        node.computed_rect = title_rect;
    }
    let _ = tree.add_child(box_id, title_id);

    // Subtitle text
    let sub_rect = Rect::new(notice_x + 10.0, notice_y + 74.0, notice_w - 20.0, 36.0);
    let sub_id = tree.create_node();
    if let Some(node) = tree.get_mut(sub_id) {
        node.set_name("EmptyNoticeSub");
        node.set_text(
            "Place 3D models (.gltf, .glb, .fbx), textures (.png), shaders (.wgsl), or scenes (.aee) into this folder.",
        );
        node.font_size = 11.0;
        node.line_height = 18.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.58, 0.62, 0.72, 1.0);
        node.computed_rect = sub_rect;
    }
    let _ = tree.add_child(box_id, sub_id);
}

/// Resolves the canonical RGBA badge color for an asset category.
pub fn resolve_category_color(category: AssetCategory) -> Color {
    match category {
        AssetCategory::Models3D => Color::rgba(0.0, 0.90, 1.0, 1.0), // Aeon Cyan
        AssetCategory::Textures2D => Color::rgba(0.39, 0.86, 0.47, 1.0), // Emerald Green
        AssetCategory::Shaders => Color::rgba(1.0, 0.75, 0.24, 1.0), // Amber / Yellow
        AssetCategory::Materials => Color::rgba(0.86, 0.39, 0.86, 1.0), // Magenta
        AssetCategory::Scenes => Color::rgba(0.31, 0.63, 1.0, 1.0),  // Sky Blue
        AssetCategory::Audio => Color::rgba(1.0, 0.47, 0.39, 1.0),   // Coral
        AssetCategory::All => Color::rgba(0.70, 0.72, 0.78, 1.0),
    }
}

/// Resolves the canonical vector icon texture UV coordinates and color tint.
fn resolve_category_icon(category: AssetCategory) -> ([f32; 4], Color) {
    match category {
        AssetCategory::Models3D => (ICON_CUBE, Color::rgba(0.0, 0.90, 1.0, 1.0)),
        AssetCategory::Textures2D => (ICON_WORLD, Color::rgba(0.39, 0.86, 0.47, 1.0)),
        AssetCategory::Shaders => (ICON_LIGHT, Color::rgba(1.0, 0.75, 0.24, 1.0)),
        AssetCategory::Scenes => (ICON_CAMERA, Color::rgba(0.31, 0.63, 1.0, 1.0)),
        AssetCategory::Materials => (ICON_SPHERE, Color::rgba(0.86, 0.39, 0.86, 1.0)),
        AssetCategory::Audio => (ICON_AUDIO, Color::rgba(1.0, 0.47, 0.39, 1.0)),
        AssetCategory::All => (ICON_FOLDER, Color::WHITE),
    }
}