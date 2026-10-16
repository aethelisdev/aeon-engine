// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Interactive Quick Asset Preview Modal Builder
//!
//! Renders a floating, hardware-accelerated GPU SDF modal window providing rich,
//! interactive preview inspections: 3D mesh orbit wireframe viewports with mouse drag
//! rotation and zoom, texture specifications, WGSL shader diagnostics, scene summaries,
//! and direct spawn/load operations.

use super::details;
use super::model;
use crate::assets::types::{AssetBrowserState, AssetCategory};
use crate::ui::iris_bridge::assets::cards::resolve_category_color;
use crate::ui::iris_bridge::assets::types::AssetsPanelParams;
use irisui::prelude::*;

/// Width of the quick preview modal card in logical pixels.
pub const PREVIEW_MODAL_WIDTH: f32 = 620.0;

/// Height of the quick preview modal card in logical pixels.
pub const PREVIEW_MODAL_HEIGHT: f32 = 470.0;

/// Builds the interactive quick asset preview modal into the `UiTree` if currently open.
///
/// Constructed using pure declarative [`UiScope`] architecture, presenting category badges,
/// formatted file metadata, modern close glyphs ('✕') with soft danger hover tinting,
/// interactive inspection bodies, and elevated action controls.
pub fn build_asset_preview_modal(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &AssetsPanelParams<'_>,
) {
    let Some(modal) = params.active_preview_modal else {
        return;
    };

    let (modal_w, modal_h) = compute_modal_dimensions(params.screen_size);
    let cat_col = resolve_category_color(modal.item.category);

    tree.set_cursor_position(params.cursor_pos);
    let mut scope = UiScope::new(tree, parent_id);
    scope.modal_scrim_sized(
        Rect::new(0.0, 0.0, params.screen_size.0, params.screen_size.1),
        Color::rgba(0.0, 0.0, 0.0, 0.65),
        |scrim| {
            scrim.modal_card(modal_w, modal_h, |card| {
                // 1. Unified Header Section (Height: 36px = 34px row + 1px divider + 1px spacing)
                card.container(Style::new().flex_col().height(36.0), |header_group| {
                    header_group.container(
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::SpaceBetween)
                            .height(34.0)
                            .padding_insets(Insets::new(0.0, 4.0, 0.0, 4.0)),
                        |header| {
                            // Left group: Category badge and asset name
                            header.container(
                                Style::new()
                                    .flex_row()
                                    .align_items(AlignItems::Center)
                                    .gap(8.0)
                                    .width(380.0),
                                |left| {
                                    left.container(
                                        Style::new()
                                            .width(52.0)
                                            .height(18.0)
                                            .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
                                            .background(Color::rgba(
                                                cat_col.r, cat_col.g, cat_col.b, 0.18,
                                            ))
                                            .border_radius(3.0)
                                            .align_items(AlignItems::Center)
                                            .justify_content(JustifyContent::Center),
                                        |badge| {
                                            badge.colored_label(
                                                cat_col,
                                                modal.item.category.badge(),
                                            );
                                        },
                                    );
                                    left.label(&modal.item.name);
                                },
                            );

                            // Right group: File size label and modern '✕' close button
                            header.container(
                                Style::new()
                                    .flex_row()
                                    .align_items(AlignItems::Center)
                                    .justify_content(JustifyContent::FlexEnd)
                                    .gap(10.0)
                                    .width(130.0),
                                |right| {
                                    right.colored_label(
                                        Color::rgba(0.65, 0.70, 0.80, 1.0),
                                        AssetBrowserState::format_file_size(
                                            modal.item.file_size_bytes,
                                        ),
                                    );
                                    right.modal_close_button();
                                },
                            );
                        },
                    );

                    // Header Bottom Separator Divider (flush beneath header row)
                    header_group.divider(Color::rgba(0.15, 0.16, 0.19, 0.80));
                });

                // 2. Dedicated Content Body Container (Height: 350px) - completely fills the inspection area
                card.container(
                    Style::new().flex_col().height(350.0).gap(8.0),
                    |body| match modal.item.category {
                        AssetCategory::Models3D => model::render_model_preview_content(body, modal),
                        AssetCategory::Textures2D => {
                            details::render_texture_preview_content(body, modal);
                        }
                        AssetCategory::Shaders => {
                            details::render_shader_preview_content(body, modal);
                        }
                        AssetCategory::Scenes => {
                            details::render_scene_preview_content(body, modal);
                        }
                        AssetCategory::Audio => {
                            details::render_audio_preview_content(body, modal);
                        }
                        AssetCategory::Materials | AssetCategory::All => {
                            details::render_generic_preview_content(body, modal);
                        }
                    },
                );

                // 3. Footer Bar: Reveal in Explorer and Escape Key Hint
                card.container(
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::SpaceBetween)
                        .height(32.0)
                        .padding_insets(Insets::new(0.0, 4.0, 0.0, 4.0)),
                    |footer| {
                        footer.add(
                            Button::new("Reveal in Explorer").width(140.0).tag(
                                crate::ui::iris_bridge::assets::types::ASSET_PREVIEW_TAG_REVEAL,
                            ),
                        );

                        footer.colored_label(
                            Color::rgba(0.50, 0.54, 0.64, 1.0),
                            "Press Esc to Close",
                        );
                    },
                );
            });
        },
    );
}

/// Computes the clamped modal dimensions based on available screen viewport extents.
#[inline]
fn compute_modal_dimensions((screen_w, screen_h): (f32, f32)) -> (f32, f32) {
    (
        PREVIEW_MODAL_WIDTH.min(screen_w - 40.0).max(360.0),
        PREVIEW_MODAL_HEIGHT.min(screen_h - 40.0).max(320.0),
    )
}