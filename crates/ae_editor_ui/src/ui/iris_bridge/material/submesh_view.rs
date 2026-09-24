// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio 3D Submesh View
//!
//! Declarative inspector for active 3D model geometry, submesh slot alpha blending,
//! material parameters, and texture slot bindings using [`UiScope`].
//!

use super::types::{make_submesh_alpha_tag, make_submesh_texture_tag};
use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_FOLDER, ICON_WORLD};
use ae_renderer::render::types::SubmeshAlphaMode;
use irisui::prelude::*;

/// Parameters for rendering the 3D Submesh material inspector view.
pub struct SubmeshViewParams<'a> {
    /// Target entity being inspected.
    pub entity: hecs::Entity,
    /// Reference to the active ECS world.
    pub world: &'a hecs::World,
    /// Model asset storage.
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    /// Texture asset storage.
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
}

/// Builds the 3D Model overview card and submesh material slot cards directly on [`UiScope`].
pub fn build_submesh_view(scope: &mut UiScope<'_>, params: &SubmeshViewParams<'_>) {
    let model_handle = match params.world.get::<&ae_core::ecs::ModelId>(params.entity) {
        Ok(m) => m.0,
        Err(_) => return,
    };

    let model = match params.models.get(model_handle) {
        Some(m) => m,
        None => return,
    };

    let file_name = std::path::Path::new(&model.source_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| model.source_path.clone());

    let tri_count = model.num_indices / 3;
    let stats_text = format!(
        "{} Submesh Slots • {} Vertices • {} Tris",
        model.submeshes.len(),
        model.gpu_vertices.len(),
        tri_count
    );

    // ── 1. Model Overview Card ────────────────────────────────────────────────
    scope.card_custom(
        |header| {
            header.row(|row| {
                row.icon(ICON_CUBE, Color::rgba(0.0, 0.85, 1.0, 0.95), 14.0);
                row.label(
                    format!("3D Model: {}", file_name),
                    11.0,
                    Color::rgba(0.88, 0.90, 0.94, 1.0),
                    TextAlign::Left,
                );
            });
        },
        |card| {
            card.divider(Color::rgba(0.16, 0.17, 0.20, 0.85));
            card.label(
                stats_text,
                10.5,
                Color::rgba(0.35, 0.82, 0.50, 1.0),
                TextAlign::Left,
            );
            card.label(
                "Configure alpha blending and textures per submesh slot below:",
                9.5,
                Color::rgba(0.55, 0.58, 0.64, 1.0),
                TextAlign::Left,
            );
        },
    );

    // ── 2. Submesh Slots List ─────────────────────────────────────────────────
    let alpha_modes = [
        (SubmeshAlphaMode::Opaque, "🟫 Opaque"),
        (SubmeshAlphaMode::Mask, "✂️ Cutout"),
        (SubmeshAlphaMode::Blend, "💧 Blend"),
    ];

    for (idx, submesh) in model.submeshes.iter().enumerate() {
        let card_title = format!("Submesh #{} ({} tris)", idx, submesh.index_count / 3);
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

        let swatch_color = Color::rgba(
            submesh.base_color[0],
            submesh.base_color[1],
            submesh.base_color[2],
            submesh.base_color[3],
        );

        scope.card_custom(
            |header| {
                header.label(
                    card_title,
                    11.0,
                    Color::rgba(0.95, 0.82, 0.35, 1.0),
                    TextAlign::Left,
                );
                header.color_swatch(swatch_color, 20.0, 16.0);
            },
            |card| {
                // Alpha Mode: Dedicated label and responsive flex-grow pills dividing width equally
                card.label(
                    "Alpha Mode:",
                    10.5,
                    Color::rgba(0.65, 0.68, 0.75, 1.0),
                    TextAlign::Left,
                );
                card.row(|row| {
                    for (mode, label) in &alpha_modes {
                        let pill_tag = make_submesh_alpha_tag(idx, *mode);
                        let is_active = submesh.alpha_mode == *mode;
                        row.toggle_pill_flex_tagged(label, is_active, pill_tag);
                    }
                });

                // Texture Assignment: Label, flex-expanding name badge, and content-sized button
                card.row(|row| {
                    row.label(
                        "Texture:",
                        10.5,
                        Color::rgba(0.65, 0.68, 0.75, 1.0),
                        TextAlign::Left,
                    );

                    let badge_style = Style::new()
                        .flex_row()
                        .flex_grow(1.0)
                        .clip_children(true)
                        .align_items(AlignItems::Center)
                        .gap(5.0)
                        .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
                        .background(Color::rgba(0.12, 0.13, 0.16, 0.95))
                        .border(1.0, Color::rgba(0.20, 0.22, 0.27, 0.85))
                        .border_radius(3.0)
                        .height(24.0);

                    row.container(badge_style, |badge| {
                        badge.icon(ICON_WORLD, Color::rgba(0.0, 0.85, 1.0, 0.85), 12.0);
                        badge.label(
                            &current_tex_name,
                            10.0,
                            Color::rgba(0.35, 0.75, 0.98, 1.0),
                            TextAlign::Left,
                        );
                    });

                    let chg_tag = make_submesh_texture_tag(idx);
                    row.button_with_icon_styled_tagged(
                        ICON_FOLDER,
                        Color::rgba(0.95, 0.80, 0.25, 0.95),
                        "Change",
                        None,
                        chg_tag,
                    );
                });
            },
        );
    }
}