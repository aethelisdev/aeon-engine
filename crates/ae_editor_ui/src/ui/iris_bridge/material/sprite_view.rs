// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio 2D Sprite View
//!
//! Declarative inspector for active texture parameters, dimensions, mipmap telemetry,
//! sampler modes, and hardware texture actions using [`UiScope`].

use super::types::{
    MATERIAL_TAG_ADD_COLOR, MATERIAL_TAG_SPRITE_CHANGE, MATERIAL_TAG_SPRITE_REMOVE,
};
use crate::ui::iris_bridge::icons::{ICON_FOLDER, ICON_PLUS};
use irisui::prelude::*;

/// Parameters for rendering the 2D Sprite material inspector view.
pub struct SpriteViewParams<'a> {
    /// Target entity being inspected.
    pub entity: hecs::Entity,
    /// Reference to the active ECS world.
    pub world: &'a hecs::World,
    /// Texture asset storage.
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
}

/// Builds the 2D Sprite material, texture inspector, and sampler setting cards directly on [`UiScope`].
pub fn build_sprite_view(
    scope: &mut UiScope<'_>,
    params: &SpriteViewParams<'_>,
    actions: &mut Vec<super::types::MaterialAction>,
) {
    let sprite_handle = params
        .world
        .get::<&ae_core::ecs::SpriteId>(params.entity)
        .map(|s| s.0)
        .ok();

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

    // ── 1. Active Texture Asset Card ──────────────────────────────────────────
    scope.card("Active Texture Asset", |card| {
        card.colored_label(
            Color::rgba(0.35, 0.75, 0.98, 1.0),
            format!("File: {}", file_name),
        );
        card.colored_label(Color::rgba(0.30, 0.82, 0.45, 1.0), info_text);
        card.row(|row| {
            if row
                .add(
                    Button::new("Change Texture")
                        .icon(ICON_FOLDER, 16.0)
                        .fill(Color::rgba(0.95, 0.80, 0.25, 0.25))
                        .text_color(Color::rgba(0.95, 0.80, 0.25, 0.95))
                        .tag(MATERIAL_TAG_SPRITE_CHANGE),
                )
                .clicked()
            {
                actions.push(super::types::MaterialAction::PickAndAssignEntityTexture(
                    params.entity,
                ));
            }
            if row
                .add(Button::new("🗑 Remove").tag(MATERIAL_TAG_SPRITE_REMOVE))
                .clicked()
            {
                actions.push(super::types::MaterialAction::RemoveTextureFromEntity(
                    params.entity,
                ));
            }
        });
    });

    // ── 2. Surface Tiling & Sampler Settings Card ──────────────────────────────
    scope.card("Surface Tiling & Sampler Settings", |card| {
        card.row(|row| {
            row.key_value_badge("Wrap U:", "Repeat", Color::rgba(0.35, 0.85, 0.50, 1.0));
            row.key_value_badge("Wrap V:", "Repeat", Color::rgba(0.35, 0.85, 0.50, 1.0));
        });
        card.key_value_badge(
            "Hardware Filtering:",
            "16x Anisotropic",
            Color::rgba(0.98, 0.78, 0.25, 1.0),
        );
    });

    // ── 3. Object Tint / Color Swatch Card ─────────────────────────────────────
    scope.card("Object Base Color", |card| {
        if let Ok(color_ref) = params.world.get::<&ae_core::ecs::Color>(params.entity) {
            let hex_str = format!(
                "#{:02X}{:02X}{:02X}",
                (color_ref.r * 255.0).clamp(0.0, 255.0) as u8,
                (color_ref.g * 255.0).clamp(0.0, 255.0) as u8,
                (color_ref.b * 255.0).clamp(0.0, 255.0) as u8,
            );
            card.row(|row| {
                row.label("Color Swatch:");
                row.color_swatch(
                    Color::rgba(color_ref.r, color_ref.g, color_ref.b, color_ref.a),
                    36.0,
                    20.0,
                );
                row.colored_label(Color::rgba(0.85, 0.88, 0.92, 1.0), hex_str);
            });
        } else if card
            .button_with_icon_styled_tagged(
                ICON_PLUS,
                Color::rgba(0.0, 0.85, 1.0, 0.95),
                "Add Color Tint",
                None,
                MATERIAL_TAG_ADD_COLOR,
            )
            .clicked()
        {
            actions.push(super::types::MaterialAction::AddColorComponent(
                params.entity,
            ));
        }
    });
}