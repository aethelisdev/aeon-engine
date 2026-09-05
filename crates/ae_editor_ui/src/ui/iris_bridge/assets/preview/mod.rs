// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Interactive Quick Asset Preview Modal Builder ( Style)
//!
//! Renders a floating, hardware-accelerated GPU SDF modal window providing rich,
//! interactive preview inspections: 3D mesh orbit wireframe viewports with mouse drag
//! rotation and zoom, texture specifications, WGSL shader diagnostics, scene summaries,
//! and direct spawn/load operations.
//!

pub(crate) mod details;
pub(crate) mod model;

use super::cards::resolve_category_color;
use super::types::{AssetPreviewModalTargets, AssetsPanelParams, AssetsPanelTargets};
use crate::ui::panels::assets::types::{AssetBrowserState, AssetCategory};
use irisui::prelude::*;

/// Width of the quick preview modal card in logical pixels.
pub const PREVIEW_MODAL_WIDTH: f32 = 620.0;

/// Height of the quick preview modal card in logical pixels.
pub const PREVIEW_MODAL_HEIGHT: f32 = 470.0;

/// Builds the interactive quick asset preview modal into the `UiTree` if currently open.
pub fn build_asset_preview_modal(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &AssetsPanelParams<'_>,
    targets: &mut AssetsPanelTargets,
) {
    targets.preview_modal = None;

    let Some(modal) = params.active_preview_modal else {
        return;
    };

    let (screen_w, screen_h) = params.screen_size;

    // 1. Semi-transparent Backdrop Overlay across the full editor window
    let backdrop_rect = Rect::new(0.0, 0.0, screen_w, screen_h);
    let backdrop_id = tree.create_node();
    if let Some(node) = tree.get_mut(backdrop_id) {
        node.set_name("PreviewModalBackdrop");
        node.computed_rect = backdrop_rect;
        node.style = Style::new().background(Color::rgba(0.0, 0.0, 0.0, 0.65));
    }
    let _ = tree.add_child(parent_id, backdrop_id);

    // 2. Centered Modal Window Card on the whole editor window
    let modal_w = PREVIEW_MODAL_WIDTH.min(screen_w - 40.0).max(360.0);
    let modal_h = PREVIEW_MODAL_HEIGHT.min(screen_h - 40.0).max(320.0);
    let modal_x = (screen_w - modal_w) * 0.5;
    let modal_y = (screen_h - modal_h) * 0.5;
    let dialog_rect = Rect::new(modal_x, modal_y, modal_w, modal_h);

    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("PreviewModalCard");
        node.computed_rect = dialog_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.09, 0.12, 0.99))
            .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.75))
            .border_radius(8.0)
            .box_shadow(0.0, 8.0, 28.0, Color::rgba(0.0, 0.0, 0.0, 0.90))
            .clip_children(true);
    }
    let _ = tree.add_child(backdrop_id, card_id);

    // 3. Header Bar (Height: 34 px)
    let header_h = 34.0;
    let header_rect = Rect::new(modal_x + 1.0, modal_y + 1.0, modal_w - 2.0, header_h - 1.0);
    let header_id = tree.create_node();
    if let Some(node) = tree.get_mut(header_id) {
        node.set_name("PreviewModalHeader");
        node.computed_rect = header_rect;
        node.style = Style::new()
            .background(Color::rgba(0.05, 0.06, 0.08, 0.98))
            .corner_radii(CornerRadii::new(7.0, 7.0, 0.0, 0.0));
    }
    let _ = tree.add_child(card_id, header_id);

    // Header Bottom Separator Divider
    let divider_rect = Rect::new(modal_x + 1.0, modal_y + header_h, modal_w - 2.0, 1.0);
    let divider_id = tree.create_node();
    if let Some(node) = tree.get_mut(divider_id) {
        node.set_name("PreviewHeaderDivider");
        node.computed_rect = divider_rect;
        node.style = Style::new().background(Color::rgba(0.18, 0.22, 0.30, 0.70));
    }
    let _ = tree.add_child(card_id, divider_id);

    let mut cur_hx = modal_x + 12.0;

    // Category Badge
    let cat_col = resolve_category_color(modal.item.category);
    let cat_badge_w = 46.0;
    let cat_badge_rect = Rect::new(cur_hx, modal_y + 8.0, cat_badge_w, 18.0);
    let cat_id = tree.create_node();
    if let Some(node) = tree.get_mut(cat_id) {
        node.set_name("PreviewCatBadge");
        node.set_text(modal.item.category.badge());
        node.font_size = 9.5;
        node.line_height = 18.0;
        node.text_align = TextAlign::Center;
        node.text_color = cat_col;
        node.computed_rect = cat_badge_rect;
        node.style = Style::new()
            .background(Color::rgba(cat_col.r, cat_col.g, cat_col.b, 0.18))
            .border_radius(3.0);
    }
    let _ = tree.add_child(header_id, cat_id);
    cur_hx += cat_badge_w + 10.0;

    // Asset Name
    let name_w = (modal_w - 280.0).max(80.0);
    let name_rect = Rect::new(cur_hx, modal_y, name_w, header_h);
    let name_id = tree.create_node();
    if let Some(node) = tree.get_mut(name_id) {
        node.set_name("PreviewAssetName");
        node.set_text(&modal.item.name);
        node.font_size = 12.5;
        node.line_height = header_h;
        node.text_color = Color::WHITE;
        node.computed_rect = name_rect;
    }
    let _ = tree.add_child(header_id, name_id);
    cur_hx += name_w + 10.0;

    // File Size Label
    let size_text = AssetBrowserState::format_file_size(modal.item.file_size_bytes);
    let size_rect = Rect::new(cur_hx, modal_y, 70.0, header_h);
    let size_id = tree.create_node();
    if let Some(node) = tree.get_mut(size_id) {
        node.set_name("PreviewSizeLabel");
        node.set_text(&size_text);
        node.font_size = 10.5;
        node.line_height = header_h;
        node.text_color = Color::rgba(0.65, 0.70, 0.80, 1.0);
        node.computed_rect = size_rect;
    }
    let _ = tree.add_child(header_id, size_id);

    // Close "✖" Button (Top Right)
    let close_btn_rect = Rect::new(dialog_rect.right() - 28.0, modal_y + 6.0, 22.0, 22.0);
    let is_close_hovered = close_btn_rect.contains_point(params.cursor_pos);
    let close_id = tree.create_node();
    if let Some(node) = tree.get_mut(close_id) {
        node.set_name("PreviewCloseButton");
        node.set_text("✖");
        node.font_size = 11.0;
        node.line_height = 22.0;
        node.text_align = TextAlign::Center;
        node.text_color = if is_close_hovered {
            Color::WHITE
        } else {
            Color::rgba(0.65, 0.70, 0.80, 1.0)
        };
        node.computed_rect = close_btn_rect;
        node.style = Style::new()
            .background(if is_close_hovered {
                Color::rgba(0.85, 0.20, 0.20, 0.90)
            } else {
                Color::TRANSPARENT
            })
            .border_radius(4.0);
    }
    let _ = tree.add_child(header_id, close_id);

    // 4. Content Body
    let body_y = modal_y + header_h + 8.0;
    let body_w = modal_w - 24.0;
    let body_x = modal_x + 12.0;

    let mut orbit_canvas_rect = None;
    let mut action_btn_rect = None;

    match modal.item.category {
        AssetCategory::Models3D => {
            let (orb_rect, act_rect) = model::render_model_preview_content(
                tree,
                card_id,
                body_x,
                body_y,
                body_w,
                modal,
                params.cursor_pos,
            );
            orbit_canvas_rect = Some(orb_rect);
            action_btn_rect = Some(act_rect);
        }
        AssetCategory::Textures2D => {
            let act_rect = details::render_texture_preview_content(
                tree,
                card_id,
                body_x,
                body_y,
                body_w,
                modal,
                params.cursor_pos,
            );
            action_btn_rect = Some(act_rect);
        }
        AssetCategory::Shaders => {
            details::render_shader_preview_content(tree, card_id, body_x, body_y, body_w, modal);
        }
        AssetCategory::Scenes => {
            let act_rect = details::render_scene_preview_content(
                tree,
                card_id,
                body_x,
                body_y,
                body_w,
                modal,
                params.cursor_pos,
            );
            action_btn_rect = Some(act_rect);
        }
        AssetCategory::Audio => {
            let act_rect = details::render_audio_preview_content(
                tree,
                card_id,
                body_x,
                body_y,
                body_w,
                modal,
                params.cursor_pos,
            );
            action_btn_rect = Some(act_rect);
        }
        AssetCategory::Materials | AssetCategory::All => {
            details::render_generic_preview_content(tree, card_id, body_x, body_y, body_w, modal);
        }
    }

    // 5. Footer Bar (Height: 34 px)
    let footer_h = 32.0;
    let footer_y = dialog_rect.bottom() - footer_h - 4.0;

    let reveal_rect = Rect::new(body_x, footer_y, 140.0, 26.0);
    let is_rev_hovered = reveal_rect.contains_point(params.cursor_pos);
    let rev_id = tree.create_node();
    if let Some(node) = tree.get_mut(rev_id) {
        node.set_name("PreviewRevealBtn");
        node.set_text("Reveal in Explorer");
        node.font_size = 11.0;
        node.line_height = 26.0;
        node.text_align = TextAlign::Center;
        node.text_color = if is_rev_hovered {
            Color::WHITE
        } else {
            Color::rgba(0.75, 0.80, 0.90, 1.0)
        };
        node.computed_rect = reveal_rect;
        node.style = Style::new()
            .background(if is_rev_hovered {
                Color::rgba(0.20, 0.24, 0.32, 1.0)
            } else {
                Color::rgba(0.12, 0.14, 0.18, 0.80)
            })
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.22, 0.25, 0.32, 0.60));
    }
    let _ = tree.add_child(card_id, rev_id);

    // Escape Key Hint Label (Right aligned)
    let esc_rect = Rect::new(dialog_rect.right() - 150.0, footer_y, 138.0, 26.0);
    let esc_id = tree.create_node();
    if let Some(node) = tree.get_mut(esc_id) {
        node.set_name("PreviewEscHint");
        node.set_text("Press Esc to Close");
        node.font_size = 10.5;
        node.line_height = 26.0;
        node.text_align = TextAlign::Right;
        node.text_color = Color::rgba(0.50, 0.54, 0.64, 1.0);
        node.computed_rect = esc_rect;
    }
    let _ = tree.add_child(card_id, esc_id);

    targets.preview_modal = Some(AssetPreviewModalTargets {
        dialog_rect,
        close_btn_rect,
        orbit_canvas_rect,
        action_btn_rect,
        reveal_btn_rect: reveal_rect,
        item: modal.item.clone(),
    });
}