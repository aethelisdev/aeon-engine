// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio 3D Submesh View
//!
//! Declarative inspector for active 3D model geometry, submesh slot alpha blending,
//! material parameters, and texture slot bindings using [`UiScope`].

use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_FOLDER, ICON_WORLD};
use crate::ui::types::EngineUiAction;
use ae_renderer::asset::{AssetHandle, AssetStorage};
use ae_renderer::render::types::{ModelSubmesh, SubmeshAlphaMode};
use ae_renderer::render::{ModelAsset, TextureAsset};
use irisui::prelude::*;
use irisui::workbench::ActionBus;
use std::fmt::Write;

const ALPHA_MODES: [(SubmeshAlphaMode, &str); 3] = [
    (SubmeshAlphaMode::Opaque, "🟫 Opaque"),
    (SubmeshAlphaMode::Mask, "✂️ Cutout"),
    (SubmeshAlphaMode::Blend, "💧 Blend"),
];

#[inline]
fn asset_filename(source_path: &str) -> &str {
    std::path::Path::new(source_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(source_path)
}

#[inline]
fn submesh_texture_name<'a>(
    model: &'a ae_renderer::render::ModelAsset,
    textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    texture_index: Option<usize>,
) -> &'a str {
    texture_index
        .and_then(|t_idx| model.embedded_textures.get(t_idx))
        .and_then(|&t_h| textures.get(t_h))
        .map_or("Default Embedded Texture", |t| {
            asset_filename(&t.source_path)
        })
}

struct SubmeshSlotContext<'a> {
    model_handle: AssetHandle,
    model: &'a ModelAsset,
    textures: &'a AssetStorage<TextureAsset>,
    idx: usize,
    submesh: &'a ModelSubmesh,
}

/// Builds the 3D Model overview card and submesh material slot cards directly on [`UiScope`].
pub fn build_submesh_view(
    scope: &mut UiScope<'_>,
    entity: hecs::Entity,
    world: &hecs::World,
    models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope.when_some(
        world
            .get::<&ae_core::ecs::ModelId>(entity)
            .ok()
            .and_then(|m| models.get(m.0).map(|model| (m.0, model))),
        |view, (model_handle, model)| {
            render_model_overview(view, model);

            for (idx, submesh) in model.submeshes.iter().enumerate() {
                render_submesh_slot(
                    view,
                    &SubmeshSlotContext {
                        model_handle,
                        model,
                        textures,
                        idx,
                        submesh,
                    },
                    action_bus,
                );
            }
        },
    );
}

fn render_model_overview(scope: &mut UiScope<'_>, model: &ModelAsset) {
    scope.card_custom(
        |header| render_model_overview_header(header, model),
        |card| render_model_overview_body(card, model),
    );
}

fn render_model_overview_header(header: &mut UiScope<'_>, model: &ModelAsset) {
    header.row(|row| {
        row.icon(ICON_CUBE, Color::rgba(0.0, 0.85, 1.0, 0.95), 14.0);
        row.text_colored_fmt::<128, _>(Color::rgba(0.88, 0.90, 0.94, 1.0), |buf| {
            _ = write!(buf, "3D Model: {}", asset_filename(&model.source_path));
        });
    });
}

fn render_model_overview_body(card: &mut UiScope<'_>, model: &ModelAsset) {
    card.divider(Color::rgba(0.16, 0.17, 0.20, 0.85));
    card.text_colored_fmt::<96, _>(Color::rgba(0.35, 0.82, 0.50, 1.0), |buf| {
        _ = write!(
            buf,
            "{} Submesh Slots • {} Vertices • {} Tris",
            model.submeshes.len(),
            model.gpu_vertices.len(),
            model.num_indices / 3
        );
    });
    card.colored_label(
        Color::rgba(0.55, 0.58, 0.64, 1.0),
        "Configure alpha blending and textures per submesh slot below:",
    );
}

fn render_submesh_slot(
    scope: &mut UiScope<'_>,
    ctx: &SubmeshSlotContext<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope.with_id(ctx.idx, |sub_scope| {
        sub_scope.card_custom(
            |header| render_submesh_header(header, ctx),
            |card| render_submesh_body(card, ctx, action_bus),
        );
    });
}

fn render_submesh_header(header: &mut UiScope<'_>, ctx: &SubmeshSlotContext<'_>) {
    header.text_colored_fmt::<64, _>(Color::rgba(0.95, 0.82, 0.35, 1.0), |buf| {
        _ = write!(
            buf,
            "Submesh #{} ({} tris)",
            ctx.idx,
            ctx.submesh.index_count / 3
        );
    });
    header.color_swatch(
        Color::rgba(
            ctx.submesh.base_color[0],
            ctx.submesh.base_color[1],
            ctx.submesh.base_color[2],
            ctx.submesh.base_color[3],
        ),
        20.0,
        16.0,
    );
}

fn render_submesh_body(
    card: &mut UiScope<'_>,
    ctx: &SubmeshSlotContext<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    card.label("Alpha Mode:");
    card.row(|row| render_alpha_mode_buttons(row, ctx, action_bus));
    card.row(|row| render_submesh_texture_row(row, ctx, action_bus));
}

fn render_alpha_mode_buttons(
    row: &mut UiScope<'_>,
    ctx: &SubmeshSlotContext<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    for (mode, label) in &ALPHA_MODES {
        row.add(
            Button::new(*label)
                .small()
                .active(ctx.submesh.alpha_mode == *mode)
                .flex(true),
        )
        .on_click(|| {
            action_bus.emit(EngineUiAction::SetModelSubmeshAlphaMode(
                ctx.model_handle,
                ctx.idx,
                *mode,
            ));
        });
    }
}

fn render_submesh_texture_row(
    row: &mut UiScope<'_>,
    ctx: &SubmeshSlotContext<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    row.label("Texture:");
    row.icon_label_badge(
        ICON_WORLD,
        Color::rgba(0.0, 0.85, 1.0, 0.85),
        submesh_texture_name(ctx.model, ctx.textures, ctx.submesh.texture_index),
        Color::rgba(0.35, 0.75, 0.98, 1.0),
    );
    row.button_with_icon_styled_tagged(
        ICON_FOLDER,
        Color::rgba(0.95, 0.80, 0.25, 0.95),
        "Change",
        None,
        row.tag_for("Change"),
    )
    .on_click(|| {
        action_bus.emit(EngineUiAction::PickAndSetSubmeshTexture(
            ctx.model_handle,
            ctx.idx,
        ));
    });
}