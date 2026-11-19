// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio 2D Sprite View
//!
//! Declarative inspector for active texture parameters, dimensions, mipmap telemetry,
//! sampler modes, and hardware texture actions using [`UiScope`].

use crate::ui::iris_bridge::icons::{ICON_FOLDER, ICON_PLUS};
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;
use std::fmt::Write;

/// Parameters for rendering the 2D Sprite material inspector view.
pub struct SpriteViewParams<'a> {
    /// Target entity being inspected.
    pub entity: hecs::Entity,
    /// Reference to the active ECS world.
    pub world: &'a hecs::World,
    /// Texture asset storage.
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
}

#[inline]
fn resolve_sprite_texture<'a>(
    params: &'a SpriteViewParams<'a>,
) -> Option<&'a ae_renderer::render::TextureAsset> {
    params
        .world
        .get::<&ae_core::ecs::SpriteId>(params.entity)
        .ok()
        .and_then(|s| params.textures.get(s.0))
}

#[inline]
fn asset_filename(source_path: &str) -> &str {
    std::path::Path::new(source_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(source_path)
}

#[inline]
fn mip_count(width: u32, height: u32) -> u32 {
    if width.max(height) > 0 {
        width.max(height).ilog2() + 1
    } else {
        1
    }
}

/// Builds the 2D Sprite material, texture inspector, and sampler setting cards directly on [`UiScope`].
pub fn build_sprite_view(
    scope: &mut UiScope<'_>,
    params: &SpriteViewParams<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope.card("Active Texture Asset", None, None, |card| {
        card.text_colored_fmt::<128, _>(Color::rgba(0.35, 0.75, 0.98, 1.0), |buf| {
            _ = write!(
                buf,
                "File: {}",
                resolve_sprite_texture(params).map_or("Embedded Texture", |asset| asset_filename(
                    &asset.source_path
                ))
            );
        });
        card.text_colored_fmt::<96, _>(Color::rgba(0.30, 0.82, 0.45, 1.0), |buf| {
            if let Some(asset) = resolve_sprite_texture(params) {
                _ = write!(
                    buf,
                    "{} x {} px • sRGB • Mips: {}",
                    asset.width,
                    asset.height,
                    mip_count(asset.width, asset.height)
                );
            } else {
                _ = buf.write_str("Standard sRGB");
            }
        });
        card.row(|row| {
            row.add(
                Button::new("Change Texture")
                    .icon(ICON_FOLDER, 16.0)
                    .fill(Color::rgba(0.95, 0.80, 0.25, 0.25))
                    .text_color(Color::rgba(0.95, 0.80, 0.25, 0.95)),
            )
            .on_click(|| {
                action_bus.emit(EngineUiAction::PickAndAssignEntityTexture(params.entity));
            });

            row.add(Button::new("🗑 Remove")).on_click(|| {
                action_bus.emit(EngineUiAction::RemoveTextureFromEntity(params.entity));
            });
        });
    });

    scope.card("Surface Tiling & Sampler Settings", None, None, |card| {
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

    scope.card("Object Base Color", None, None, |card| {
        card.when_ok(
            params.world.get::<&ae_core::ecs::Color>(params.entity),
            |c, color_ref| {
                c.row(|row| {
                    row.label("Color Swatch:");
                    row.color_swatch(
                        Color::rgba(color_ref.r, color_ref.g, color_ref.b, color_ref.a),
                        36.0,
                        20.0,
                    );
                    row.text_colored_fmt::<16, _>(Color::rgba(0.85, 0.88, 0.92, 1.0), |buf| {
                        _ = write!(
                            buf,
                            "#{:02X}{:02X}{:02X}",
                            (color_ref.r * 255.0).clamp(0.0, 255.0) as u8,
                            (color_ref.g * 255.0).clamp(0.0, 255.0) as u8,
                            (color_ref.b * 255.0).clamp(0.0, 255.0) as u8,
                        );
                    });
                });
            },
        );
        card.when(
            params
                .world
                .get::<&ae_core::ecs::Color>(params.entity)
                .is_err(),
            |c| {
                c.button_with_icon_styled_tagged(
                    ICON_PLUS,
                    Color::rgba(0.0, 0.85, 1.0, 0.95),
                    "Add Color Tint",
                    None,
                    c.tag_for("Add Color Tint"),
                )
                .on_click(|| {
                    action_bus.emit(EngineUiAction::AddComponent(params.entity, "Color"));
                });
            },
        );
    });
}