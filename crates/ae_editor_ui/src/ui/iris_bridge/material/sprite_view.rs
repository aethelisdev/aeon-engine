// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio 2D Sprite View
//!
//! Renders active texture parameters, dimensions, mipmap telemetry, sampler modes,
//! and hardware texture array icon buttons for 2D sprites.
//!

use super::types::MaterialPanelTargets;
use crate::ui::iris_bridge::icons::{ICON_FOLDER, ICON_PLUS, ICON_WORLD};
use irisui::prelude::*;

/// Parameters for rendering the 2D Sprite material inspector view.
pub struct SpriteViewParams<'a> {
    /// Target entity being inspected.
    pub entity: hecs::Entity,
    /// Reference to the active ECS world.
    pub world: &'a hecs::World,
    /// Texture asset storage.
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    /// Starting Y offset inside the viewport.
    pub start_y: f32,
    /// Current mouse cursor coordinates.
    pub cursor_pos: Point,
}

/// Builds the 2D Sprite material, texture inspector, and sampler setting cards.
pub fn build_sprite_view(
    tree: &mut UiTree,
    parent_id: WidgetId,
    panel_rect: Rect,
    params: &SpriteViewParams<'_>,
    targets: &mut MaterialPanelTargets,
) -> f32 {
    let padding = 8.0;
    let card_w = (panel_rect.width - padding * 2.0).max(100.0);
    let mut cur_y = params.start_y;

    let sprite_handle = params
        .world
        .get::<&ae_core::ecs::SpriteId>(params.entity)
        .map(|s| s.0)
        .ok();

    // ── 1. Active Texture Asset Card ──────────────────────────────────────────
    let card1_h = 136.0;
    let card1_rect = Rect::new(panel_rect.x + padding, cur_y, card_w, card1_h);

    let card1_id = tree.create_node();
    if let Some(node) = tree.get_mut(card1_id) {
        node.set_name("SpriteTextureCard");
        node.computed_rect = card1_rect;
        node.style = Style::new()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.14, 0.15, 0.18, 0.85))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, card1_id);

    // Card 1 Header: ICON_WORLD + "Active Texture Asset"
    let c1_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(c1_icon_id) {
        node.set_name("SpriteCardIcon");
        node.computed_rect = Rect::new(card1_rect.x + 8.0, card1_rect.y + 7.0, 14.0, 14.0);
        node.set_texture_uv(ICON_WORLD);
        node.set_texture_tint(Color::rgba(0.0, 0.85, 1.0, 0.95));
    }
    let _ = tree.add_child(card1_id, c1_icon_id);

    let c1_hdr_id = tree.create_node();
    if let Some(node) = tree.get_mut(c1_hdr_id) {
        node.set_name("SpriteCardHeader");
        node.set_text("Active Texture Asset");
        node.font_size = 11.0;
        node.line_height = 20.0;
        node.text_color = Color::rgba(0.88, 0.90, 0.94, 1.0);
        node.computed_rect =
            Rect::new(card1_rect.x + 26.0, card1_rect.y + 4.0, card_w - 34.0, 20.0);
    }
    let _ = tree.add_child(card1_id, c1_hdr_id);

    // Divider
    let div1_id = tree.create_node();
    if let Some(node) = tree.get_mut(div1_id) {
        node.set_name("SpriteCardDivider");
        node.computed_rect = Rect::new(card1_rect.x + 8.0, card1_rect.y + 26.0, card_w - 16.0, 1.0);
        node.style = Style::new().background(Color::rgba(0.16, 0.17, 0.20, 0.85));
    }
    let _ = tree.add_child(card1_id, div1_id);

    // Texture details or fallback
    let mut file_name = "Embedded Texture".to_string();
    let mut info_text = "Standard sRGB".to_string();

    if let Some(handle) = sprite_handle
        && let Some(asset) = params.textures.get(handle)
    {
        file_name = std::path::Path::new(&asset.source_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| asset.source_path.clone());

        let max_dim = asset.width.max(asset.height);
        let mip_levels = if max_dim > 0 { max_dim.ilog2() + 1 } else { 1 };
        info_text = format!(
            "{} x {} px • sRGB • Mips: {}",
            asset.width, asset.height, mip_levels
        );
    }

    // Row 1: File Name
    let fn_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(fn_lbl_id) {
        node.set_name("SpriteFileName");
        node.set_text(format!("File: {}", file_name));
        node.font_size = 11.0;
        node.line_height = 18.0;
        node.text_color = Color::rgba(0.35, 0.75, 0.98, 1.0);
        node.computed_rect = Rect::new(
            card1_rect.x + 10.0,
            card1_rect.y + 32.0,
            card_w - 20.0,
            18.0,
        );
    }
    let _ = tree.add_child(card1_id, fn_lbl_id);

    // Row 2: Resolution & Mipmaps
    let info_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(info_lbl_id) {
        node.set_name("SpriteInfoText");
        node.set_text(&info_text);
        node.font_size = 10.5;
        node.line_height = 18.0;
        node.text_color = Color::rgba(0.30, 0.82, 0.45, 1.0);
        node.computed_rect = Rect::new(
            card1_rect.x + 10.0,
            card1_rect.y + 54.0,
            card_w - 20.0,
            18.0,
        );
    }
    let _ = tree.add_child(card1_id, info_lbl_id);

    // Row 3: Action Buttons (Change Texture + Remove Texture)
    let btn_y = card1_rect.y + 88.0;
    let btn_h = 24.0;
    let change_btn_w = (card_w - 26.0) * 0.65;
    let remove_btn_w = (card_w - 26.0) * 0.35;

    // Change Texture Button (uses ICON_FOLDER quad)
    let change_rect = Rect::new(card1_rect.x + 10.0, btn_y, change_btn_w, btn_h);
    targets.btn_change_texture = Some(change_rect);

    let is_change_hovered = change_rect.contains_point(params.cursor_pos);
    let (c_bg, c_border) = if is_change_hovered {
        (
            Color::rgba(0.0, 0.35, 0.48, 0.95),
            Color::rgba(0.0, 0.85, 1.0, 0.95),
        )
    } else {
        (
            Color::rgba(0.12, 0.14, 0.17, 0.95),
            Color::rgba(0.20, 0.23, 0.28, 0.90),
        )
    };

    let chg_id = tree.create_node();
    if let Some(node) = tree.get_mut(chg_id) {
        node.set_name("SpriteChangeBtn");
        node.computed_rect = change_rect;
        node.style = Style::new()
            .background(c_bg)
            .border(1.0, c_border)
            .border_radius(4.0);
    }
    let _ = tree.add_child(card1_id, chg_id);

    // ICON_FOLDER on change button
    let f_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(f_icon_id) {
        node.set_name("ChangeFolderIcon");
        node.computed_rect = Rect::new(change_rect.x + 6.0, change_rect.y + 5.0, 14.0, 14.0);
        node.set_texture_uv(ICON_FOLDER);
        node.set_texture_tint(Color::rgba(0.95, 0.80, 0.25, 0.95));
    }
    let _ = tree.add_child(chg_id, f_icon_id);

    let chg_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(chg_txt_id) {
        node.set_name("ChangeBtnText");
        node.set_text("Change Texture");
        node.font_size = 10.5;
        node.line_height = btn_h;
        node.text_color = Color::rgba(0.90, 0.92, 0.95, 1.0);
        node.computed_rect = Rect::new(
            change_rect.x + 24.0,
            change_rect.y,
            change_btn_w - 26.0,
            btn_h,
        );
    }
    let _ = tree.add_child(chg_id, chg_txt_id);

    // Remove Texture Button
    let remove_rect = Rect::new(
        change_rect.x + change_btn_w + 6.0,
        btn_y,
        remove_btn_w,
        btn_h,
    );
    targets.btn_remove_texture = Some(remove_rect);

    let is_rem_hovered = remove_rect.contains_point(params.cursor_pos);
    let (r_bg, r_border) = if is_rem_hovered {
        (
            Color::rgba(0.50, 0.12, 0.12, 0.95),
            Color::rgba(0.95, 0.35, 0.35, 0.95),
        )
    } else {
        (
            Color::rgba(0.14, 0.11, 0.12, 0.90),
            Color::rgba(0.24, 0.18, 0.20, 0.85),
        )
    };

    let rem_id = tree.create_node();
    if let Some(node) = tree.get_mut(rem_id) {
        node.set_name("SpriteRemoveBtn");
        node.computed_rect = remove_rect;
        node.style = Style::new()
            .background(r_bg)
            .border(1.0, r_border)
            .border_radius(4.0);
    }
    let _ = tree.add_child(card1_id, rem_id);

    let rem_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(rem_txt_id) {
        node.set_name("RemoveBtnText");
        node.set_text("🗑 Remove");
        node.font_size = 10.5;
        node.line_height = btn_h;
        node.text_color = Color::rgba(0.95, 0.55, 0.55, 1.0);
        node.computed_rect = Rect::new(
            remove_rect.x + 4.0,
            remove_rect.y,
            remove_btn_w - 8.0,
            btn_h,
        );
    }
    let _ = tree.add_child(rem_id, rem_txt_id);

    cur_y += card1_h + 8.0;

    // ── 2. Surface Tiling & Sampler Settings Card ──────────────────────────────
    let card2_h = 104.0;
    let card2_rect = Rect::new(panel_rect.x + padding, cur_y, card_w, card2_h);

    let card2_id = tree.create_node();
    if let Some(node) = tree.get_mut(card2_id) {
        node.set_name("SamplerSettingsCard");
        node.computed_rect = card2_rect;
        node.style = Style::new()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.14, 0.15, 0.18, 0.85))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, card2_id);

    // Card 2 Header
    let c2_hdr_id = tree.create_node();
    if let Some(node) = tree.get_mut(c2_hdr_id) {
        node.set_name("SamplerCardHeader");
        node.set_text("Surface Tiling & Sampler Settings");
        node.font_size = 11.0;
        node.line_height = 20.0;
        node.text_color = Color::rgba(0.88, 0.90, 0.94, 1.0);
        node.computed_rect =
            Rect::new(card2_rect.x + 10.0, card2_rect.y + 4.0, card_w - 20.0, 20.0);
    }
    let _ = tree.add_child(card2_id, c2_hdr_id);

    let div2_id = tree.create_node();
    if let Some(node) = tree.get_mut(div2_id) {
        node.set_name("SamplerCardDivider");
        node.computed_rect = Rect::new(card2_rect.x + 8.0, card2_rect.y + 26.0, card_w - 16.0, 1.0);
        node.style = Style::new().background(Color::rgba(0.16, 0.17, 0.20, 0.85));
    }
    let _ = tree.add_child(card2_id, div2_id);

    // Tiling badges row: Wrap U: Repeat, Wrap V: Repeat
    let r1_y = card2_rect.y + 34.0;
    let badge_w = (card_w - 24.0) * 0.48;

    // Wrap U Badge
    build_telemetry_pill(
        tree,
        card2_id,
        Rect::new(card2_rect.x + 8.0, r1_y, badge_w, 22.0),
        "Wrap U:",
        "Repeat",
        Color::rgba(0.35, 0.85, 0.50, 1.0),
    );
    // Wrap V Badge
    build_telemetry_pill(
        tree,
        card2_id,
        Rect::new(card2_rect.x + 14.0 + badge_w, r1_y, badge_w, 22.0),
        "Wrap V:",
        "Repeat",
        Color::rgba(0.35, 0.85, 0.50, 1.0),
    );

    // Anisotropy Badge
    let r2_y = card2_rect.y + 64.0;
    build_telemetry_pill(
        tree,
        card2_id,
        Rect::new(card2_rect.x + 8.0, r2_y, card_w - 16.0, 22.0),
        "Hardware Filtering:",
        "16x Anisotropic",
        Color::rgba(0.98, 0.78, 0.25, 1.0),
    );

    cur_y += card2_h + 8.0;

    // ── 3. Object Tint / Color Swatch Card ─────────────────────────────────────
    let card3_h = 44.0;
    let card3_rect = Rect::new(panel_rect.x + padding, cur_y, card_w, card3_h);

    let card3_id = tree.create_node();
    if let Some(node) = tree.get_mut(card3_id) {
        node.set_name("ColorTintCard");
        node.computed_rect = card3_rect;
        node.style = Style::new()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.14, 0.15, 0.18, 0.85))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, card3_id);

    if let Ok(color_ref) = params.world.get::<&ae_core::ecs::Color>(params.entity) {
        // Color label
        let tint_lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(tint_lbl_id) {
            node.set_name("TintLabel");
            node.set_text("Object Base Color:");
            node.font_size = 11.0;
            node.line_height = card3_h;
            node.text_color = Color::rgba(0.70, 0.72, 0.78, 1.0);
            node.computed_rect = Rect::new(card3_rect.x + 10.0, card3_rect.y, 110.0, card3_h);
        }
        let _ = tree.add_child(card3_id, tint_lbl_id);

        // Color swatch box
        let swatch_rect = Rect::new(card3_rect.x + 124.0, card3_rect.y + 11.0, 36.0, 22.0);
        let swatch_id = tree.create_node();
        if let Some(node) = tree.get_mut(swatch_id) {
            node.set_name("TintSwatchBox");
            node.computed_rect = swatch_rect;
            node.style = Style::new()
                .background(Color::rgba(
                    color_ref.r,
                    color_ref.g,
                    color_ref.b,
                    color_ref.a,
                ))
                .border(1.0, Color::rgba(0.85, 0.88, 0.95, 0.85))
                .border_radius(3.0);
        }
        let _ = tree.add_child(card3_id, swatch_id);

        // HEX readout
        let hex_str = format!(
            "#{:02X}{:02X}{:02X}",
            (color_ref.r * 255.0).clamp(0.0, 255.0) as u8,
            (color_ref.g * 255.0).clamp(0.0, 255.0) as u8,
            (color_ref.b * 255.0).clamp(0.0, 255.0) as u8,
        );
        let hex_lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(hex_lbl_id) {
            node.set_name("TintHexLabel");
            node.set_text(&hex_str);
            node.font_size = 11.0;
            node.line_height = card3_h;
            node.text_color = Color::rgba(0.85, 0.88, 0.92, 1.0);
            node.computed_rect = Rect::new(card3_rect.x + 168.0, card3_rect.y, 80.0, card3_h);
        }
        let _ = tree.add_child(card3_id, hex_lbl_id);
    } else {
        // Add Color Button
        let add_c_rect = Rect::new(card3_rect.x + 10.0, card3_rect.y + 9.0, card_w - 20.0, 26.0);
        targets.btn_add_color = Some(add_c_rect);

        let is_ac_hovered = add_c_rect.contains_point(params.cursor_pos);
        let (ac_bg, ac_border) = if is_ac_hovered {
            (
                Color::rgba(0.0, 0.35, 0.48, 0.95),
                Color::rgba(0.0, 0.85, 1.0, 0.95),
            )
        } else {
            (
                Color::rgba(0.12, 0.14, 0.17, 0.95),
                Color::rgba(0.20, 0.23, 0.28, 0.90),
            )
        };

        let add_c_id = tree.create_node();
        if let Some(node) = tree.get_mut(add_c_id) {
            node.set_name("AddColorTintBtn");
            node.computed_rect = add_c_rect;
            node.style = Style::new()
                .background(ac_bg)
                .border(1.0, ac_border)
                .border_radius(4.0);
        }
        let _ = tree.add_child(card3_id, add_c_id);

        let p_icon_id = tree.create_node();
        if let Some(node) = tree.get_mut(p_icon_id) {
            node.set_name("AddColorPlusIcon");
            node.computed_rect = Rect::new(add_c_rect.x + 8.0, add_c_rect.y + 7.0, 12.0, 12.0);
            node.set_texture_uv(ICON_PLUS);
            node.set_texture_tint(Color::rgba(0.0, 0.85, 1.0, 0.95));
        }
        let _ = tree.add_child(add_c_id, p_icon_id);

        let ac_txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(ac_txt_id) {
            node.set_name("AddColorTintText");
            node.set_text("Add Color Tint");
            node.font_size = 11.0;
            node.line_height = 26.0;
            node.text_color = Color::rgba(0.92, 0.94, 0.98, 1.0);
            node.computed_rect = Rect::new(
                add_c_rect.x + 24.0,
                add_c_rect.y,
                add_c_rect.width - 28.0,
                26.0,
            );
        }
        let _ = tree.add_child(add_c_id, ac_txt_id);
    }

    cur_y += card3_h + 8.0;

    cur_y - params.start_y
}

fn build_telemetry_pill(
    tree: &mut UiTree,
    parent_id: WidgetId,
    rect: Rect,
    label: &str,
    value: &str,
    val_color: Color,
) {
    let pill_id = tree.create_node();
    if let Some(node) = tree.get_mut(pill_id) {
        node.set_name("TelemetryPill");
        node.computed_rect = rect;
        node.style = Style::new()
            .background(Color::rgba(0.12, 0.13, 0.16, 0.95))
            .border(1.0, Color::rgba(0.18, 0.20, 0.24, 0.85))
            .border_radius(4.0);
    }
    let _ = tree.add_child(parent_id, pill_id);

    let lbl_w = rect.width * 0.5;
    let l_id = tree.create_node();
    if let Some(node) = tree.get_mut(l_id) {
        node.set_name("PillLabel");
        node.set_text(label);
        node.font_size = 10.0;
        node.line_height = rect.height;
        node.text_color = Color::rgba(0.60, 0.63, 0.70, 1.0);
        node.computed_rect = Rect::new(rect.x + 6.0, rect.y, lbl_w, rect.height);
    }
    let _ = tree.add_child(pill_id, l_id);

    let v_id = tree.create_node();
    if let Some(node) = tree.get_mut(v_id) {
        node.set_name("PillValue");
        node.set_text(value);
        node.font_size = 10.0;
        node.line_height = rect.height;
        node.text_color = val_color;
        node.computed_rect = Rect::new(
            rect.x + lbl_w + 6.0,
            rect.y,
            rect.width - lbl_w - 12.0,
            rect.height,
        );
    }
    let _ = tree.add_child(pill_id, v_id);
}