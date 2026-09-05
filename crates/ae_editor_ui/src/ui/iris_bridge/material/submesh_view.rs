// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio 3D Submesh View
//!
//! Renders glTF/GLB 3D model submesh material slots, interactive alpha blending mode
//! selector pills, and hardware texture assignment buttons.
//!

use super::types::MaterialPanelTargets;
use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_FOLDER, ICON_WORLD};
use ae_renderer::render::types::SubmeshAlphaMode;
use irisui::prelude::*;

/// Parameters for rendering the 3D Model submesh material slots view.
pub struct SubmeshViewParams<'a> {
    /// Target entity being inspected.
    pub entity: hecs::Entity,
    /// Reference to the active ECS world.
    pub world: &'a hecs::World,
    /// Model asset storage.
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    /// Texture asset storage.
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    /// Starting Y offset inside the viewport.
    pub start_y: f32,
    /// Current mouse cursor coordinates.
    pub cursor_pos: Point,
}

/// Builds 3D Model overview card and per-submesh material slot cards.
pub fn build_submesh_view(
    tree: &mut UiTree,
    parent_id: WidgetId,
    panel_rect: Rect,
    params: &SubmeshViewParams<'_>,
    targets: &mut MaterialPanelTargets,
) -> f32 {
    let padding = 8.0;
    let card_w = (panel_rect.width - padding * 2.0).max(100.0);
    let mut cur_y = params.start_y;

    let model_handle = match params.world.get::<&ae_core::ecs::ModelId>(params.entity) {
        Ok(m) => m.0,
        Err(_) => return 0.0,
    };

    let model = match params.models.get(model_handle) {
        Some(m) => m,
        None => return 0.0,
    };

    // ── 1. Model Overview Card ────────────────────────────────────────────────
    let card1_h = 76.0;
    let card1_rect = Rect::new(panel_rect.x + padding, cur_y, card_w, card1_h);

    let card1_id = tree.create_node();
    if let Some(node) = tree.get_mut(card1_id) {
        node.set_name("ModelOverviewCard");
        node.computed_rect = card1_rect;
        node.style = Style::new()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.14, 0.15, 0.18, 0.85))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, card1_id);

    // Header: ICON_CUBE + Model Path
    let c1_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(c1_icon_id) {
        node.set_name("ModelCardIcon");
        node.computed_rect = Rect::new(card1_rect.x + 8.0, card1_rect.y + 7.0, 14.0, 14.0);
        node.set_texture_uv(ICON_CUBE);
        node.set_texture_tint(Color::rgba(0.0, 0.85, 1.0, 0.95));
    }
    let _ = tree.add_child(card1_id, c1_icon_id);

    let file_name = std::path::Path::new(&model.source_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| model.source_path.clone());

    let c1_hdr_id = tree.create_node();
    if let Some(node) = tree.get_mut(c1_hdr_id) {
        node.set_name("ModelCardHeader");
        node.set_text(format!("3D Model: {}", file_name));
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
        node.set_name("ModelCardDivider");
        node.computed_rect = Rect::new(card1_rect.x + 8.0, card1_rect.y + 26.0, card_w - 16.0, 1.0);
        node.style = Style::new().background(Color::rgba(0.16, 0.17, 0.20, 0.85));
    }
    let _ = tree.add_child(card1_id, div1_id);

    // Telemetry details
    let tri_count = model.num_indices / 3;
    let stats_text = format!(
        "{} Submesh Slots • {} Vertices • {} Tris",
        model.submeshes.len(),
        model.gpu_vertices.len(),
        tri_count
    );
    let stats_id = tree.create_node();
    if let Some(node) = tree.get_mut(stats_id) {
        node.set_name("ModelStatsText");
        node.set_text(&stats_text);
        node.font_size = 10.5;
        node.line_height = 18.0;
        node.text_color = Color::rgba(0.35, 0.82, 0.50, 1.0);
        node.computed_rect = Rect::new(
            card1_rect.x + 10.0,
            card1_rect.y + 34.0,
            card_w - 20.0,
            18.0,
        );
    }
    let _ = tree.add_child(card1_id, stats_id);

    // Subtitle
    let sub_id = tree.create_node();
    if let Some(node) = tree.get_mut(sub_id) {
        node.set_name("ModelSubText");
        node.set_text("Configure alpha blending and textures per submesh slot below:");
        node.font_size = 9.5;
        node.line_height = 14.0;
        node.text_color = Color::rgba(0.55, 0.58, 0.64, 1.0);
        node.computed_rect = Rect::new(
            card1_rect.x + 10.0,
            card1_rect.y + 54.0,
            card_w - 20.0,
            14.0,
        );
    }
    let _ = tree.add_child(card1_id, sub_id);

    cur_y += card1_h + 8.0;

    // ── 2. Submesh Slots List ─────────────────────────────────────────────────
    for (idx, submesh) in model.submeshes.iter().enumerate() {
        let slot_h = 104.0;
        let slot_rect = Rect::new(panel_rect.x + padding, cur_y, card_w, slot_h);

        let slot_id = tree.create_node();
        if let Some(node) = tree.get_mut(slot_id) {
            node.set_name("SubmeshSlotCard");
            node.computed_rect = slot_rect;
            node.style = Style::new()
                .background(Color::rgba(0.082, 0.086, 0.100, 0.95))
                .border(1.0, Color::rgba(0.15, 0.16, 0.20, 0.85))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, slot_id);

        // Header Row: Submesh #idx (N tris) + Base Color Swatch
        let slot_hdr_id = tree.create_node();
        if let Some(node) = tree.get_mut(slot_hdr_id) {
            node.set_name("SubmeshSlotHeader");
            node.set_text(format!(
                "Submesh #{} ({} tris)",
                idx,
                submesh.index_count / 3
            ));
            node.font_size = 11.0;
            node.line_height = 20.0;
            node.text_color = Color::rgba(0.95, 0.82, 0.35, 1.0); // Warm gold
            node.computed_rect =
                Rect::new(slot_rect.x + 10.0, slot_rect.y + 4.0, card_w - 56.0, 20.0);
        }
        let _ = tree.add_child(slot_id, slot_hdr_id);

        // Base Color Preview Swatch
        let swatch_rect = Rect::new(slot_rect.x + card_w - 30.0, slot_rect.y + 6.0, 20.0, 16.0);
        let swatch_id = tree.create_node();
        if let Some(node) = tree.get_mut(swatch_id) {
            node.set_name("SubmeshColorSwatch");
            node.computed_rect = swatch_rect;
            node.style = Style::new()
                .background(Color::rgba(
                    submesh.base_color[0],
                    submesh.base_color[1],
                    submesh.base_color[2],
                    submesh.base_color[3],
                ))
                .border(1.0, Color::rgba(0.85, 0.88, 0.95, 0.70))
                .border_radius(2.0);
        }
        let _ = tree.add_child(slot_id, swatch_id);

        // Row 1: Alpha Mode Selector Pills
        let alpha_row_y = slot_rect.y + 30.0;
        let lbl_alpha_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_alpha_id) {
            node.set_name("SubmeshAlphaLabel");
            node.set_text("Alpha Mode:");
            node.font_size = 10.5;
            node.line_height = 22.0;
            node.text_color = Color::rgba(0.65, 0.68, 0.75, 1.0);
            node.computed_rect = Rect::new(slot_rect.x + 10.0, alpha_row_y, 75.0, 22.0);
        }
        let _ = tree.add_child(slot_id, lbl_alpha_id);

        let pills_x = slot_rect.x + 90.0;
        let pill_w = (card_w - 98.0) / 3.0;
        let pill_h = 22.0;

        let alpha_modes = [
            (SubmeshAlphaMode::Opaque, "🟫 Opaque"),
            (SubmeshAlphaMode::Mask, "✂️ Cutout"),
            (SubmeshAlphaMode::Blend, "💧 Blend"),
        ];

        for (m_idx, (mode, label)) in alpha_modes.iter().enumerate() {
            let p_rect = Rect::new(
                pills_x + (m_idx as f32) * pill_w,
                alpha_row_y,
                pill_w - 4.0,
                pill_h,
            );
            targets
                .submesh_alpha_buttons
                .push((model_handle, idx, *mode, p_rect));

            let is_active = submesh.alpha_mode == *mode;
            let is_hovered = p_rect.contains_point(params.cursor_pos);

            let (p_bg, p_border, p_txt) = if is_active {
                (
                    Color::rgba(0.0, 0.38, 0.50, 0.95),
                    Color::rgba(0.0, 0.85, 1.0, 0.95),
                    Color::rgba(1.0, 1.0, 1.0, 1.0),
                )
            } else if is_hovered {
                (
                    Color::rgba(0.16, 0.18, 0.22, 0.95),
                    Color::rgba(0.35, 0.40, 0.50, 0.90),
                    Color::rgba(0.85, 0.88, 0.92, 1.0),
                )
            } else {
                (
                    Color::rgba(0.11, 0.12, 0.15, 0.95),
                    Color::rgba(0.18, 0.20, 0.24, 0.85),
                    Color::rgba(0.65, 0.68, 0.74, 1.0),
                )
            };

            let p_id = tree.create_node();
            if let Some(node) = tree.get_mut(p_id) {
                node.set_name("AlphaPillButton");
                node.computed_rect = p_rect;
                node.style = Style::new()
                    .background(p_bg)
                    .border(1.0, p_border)
                    .border_radius(3.0);
            }
            let _ = tree.add_child(slot_id, p_id);

            let p_txt_id = tree.create_node();
            if let Some(node) = tree.get_mut(p_txt_id) {
                node.set_name("AlphaPillText");
                node.set_text(*label);
                node.font_size = 9.5;
                node.line_height = pill_h;
                node.text_color = p_txt;
                node.computed_rect =
                    Rect::new(p_rect.x + 2.0, p_rect.y, p_rect.width - 4.0, pill_h);
            }
            let _ = tree.add_child(p_id, p_txt_id);
        }

        // Row 2: Texture Assignment (Name badge + Change Texture button)
        let tex_row_y = slot_rect.y + 64.0;
        let lbl_tex_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_tex_id) {
            node.set_name("SubmeshTextureLabel");
            node.set_text("Texture:");
            node.font_size = 10.5;
            node.line_height = 24.0;
            node.text_color = Color::rgba(0.65, 0.68, 0.75, 1.0);
            node.computed_rect = Rect::new(slot_rect.x + 10.0, tex_row_y, 55.0, 24.0);
        }
        let _ = tree.add_child(slot_id, lbl_tex_id);

        // Texture Name Badge
        let current_tex_name = submesh
            .texture_index
            .and_then(|t_idx| model.embedded_textures.get(t_idx))
            .and_then(|&t_h| params.textures.get(t_h))
            .map(|t| {
                std::path::Path::new(&t.source_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| t.source_path.clone())
            })
            .unwrap_or_else(|| "Default Embedded Texture".to_string());

        let badge_x = slot_rect.x + 70.0;
        let change_btn_w = 76.0;
        let name_badge_w = (card_w - 78.0 - change_btn_w).max(80.0);

        let nb_id = tree.create_node();
        if let Some(node) = tree.get_mut(nb_id) {
            node.set_name("SubmeshTextureBadge");
            node.computed_rect = Rect::new(badge_x, tex_row_y, name_badge_w, 24.0);
            node.style = Style::new()
                .background(Color::rgba(0.12, 0.13, 0.16, 0.95))
                .border(1.0, Color::rgba(0.20, 0.22, 0.27, 0.85))
                .border_radius(3.0);
        }
        let _ = tree.add_child(slot_id, nb_id);

        // ICON_WORLD in badge
        let w_icon_id = tree.create_node();
        if let Some(node) = tree.get_mut(w_icon_id) {
            node.set_name("BadgeWorldIcon");
            node.computed_rect = Rect::new(badge_x + 5.0, tex_row_y + 6.0, 12.0, 12.0);
            node.set_texture_uv(ICON_WORLD);
            node.set_texture_tint(Color::rgba(0.0, 0.85, 1.0, 0.85));
        }
        let _ = tree.add_child(nb_id, w_icon_id);

        let nb_txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(nb_txt_id) {
            node.set_name("SubmeshTextureName");
            node.set_text(&current_tex_name);
            node.font_size = 10.0;
            node.line_height = 24.0;
            node.text_color = Color::rgba(0.35, 0.75, 0.98, 1.0);
            node.computed_rect = Rect::new(badge_x + 21.0, tex_row_y, name_badge_w - 24.0, 24.0);
        }
        let _ = tree.add_child(nb_id, nb_txt_id);

        // Change Texture Button (uses ICON_FOLDER quad)
        let chg_btn_x = badge_x + name_badge_w + 6.0;
        let chg_rect = Rect::new(chg_btn_x, tex_row_y, change_btn_w, 24.0);
        targets
            .submesh_texture_buttons
            .push((model_handle, idx, chg_rect));

        let is_chg_hovered = chg_rect.contains_point(params.cursor_pos);
        let (chg_bg, chg_border) = if is_chg_hovered {
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
            node.set_name("SubmeshChangeBtn");
            node.computed_rect = chg_rect;
            node.style = Style::new()
                .background(chg_bg)
                .border(1.0, chg_border)
                .border_radius(3.0);
        }
        let _ = tree.add_child(slot_id, chg_id);

        // ICON_FOLDER on change button
        let f_icon_id = tree.create_node();
        if let Some(node) = tree.get_mut(f_icon_id) {
            node.set_name("SubmeshFolderIcon");
            node.computed_rect = Rect::new(chg_rect.x + 5.0, chg_rect.y + 5.0, 14.0, 14.0);
            node.set_texture_uv(ICON_FOLDER);
            node.set_texture_tint(Color::rgba(0.95, 0.80, 0.25, 0.95));
        }
        let _ = tree.add_child(chg_id, f_icon_id);

        let chg_txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(chg_txt_id) {
            node.set_name("SubmeshChangeText");
            node.set_text("Change");
            node.font_size = 10.0;
            node.line_height = 24.0;
            node.text_color = Color::rgba(0.90, 0.92, 0.95, 1.0);
            node.computed_rect =
                Rect::new(chg_rect.x + 22.0, chg_rect.y, change_btn_w - 24.0, 24.0);
        }
        let _ = tree.add_child(chg_id, chg_txt_id);

        cur_y += slot_h + 8.0;
    }

    cur_y - params.start_y
}