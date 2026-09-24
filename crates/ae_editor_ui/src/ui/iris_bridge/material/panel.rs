// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Panel Orchestrator
//!
//! Assembles the root panel container, header bar, scrollable viewport, and
//! vertical scrollbar for specialized submesh or sprite material views purely using [`UiScope`].
//!

use super::empty_state::{build_no_entity_selected, build_no_renderable_geometry};
use super::header::{MATERIAL_HEADER_HEIGHT, build_material_header};
use super::sprite_view::{SpriteViewParams, build_sprite_view};
use super::submesh_view::{SubmeshViewParams, build_submesh_view};
use super::types::{
    MATERIAL_TAG_SCROLLBAR_THUMB, MATERIAL_TAG_SCROLLBAR_TRACK, MATERIAL_TAG_VIEWPORT,
    MaterialPanelParams,
};
use irisui::prelude::*;

/// Builds the complete Material & Surface Studio panel tree in the retained `UiTree`.
///
/// Returns the computed maximum vertical scroll limit in physical pixels.
pub fn build_material_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &MaterialPanelParams<'_>,
) -> f32 {
    let mut scope =
        UiScope::with_tagged_interactions(tree, parent_id, params.events, params.hovered_tag);

    let root_style = Style::new()
        .flex_col()
        .background(Color::rgba(0.065, 0.068, 0.080, 0.98))
        .border(1.0, Color::rgba(0.12, 0.13, 0.16, 0.90))
        .clip_children(true);

    let mut max_scroll = 0.0;

    scope.container(root_style, |panel_scope| {
        // 1. Top Header Bar
        build_material_header(panel_scope, params.entity, params.world);

        // 2. Main Body Content Area
        let vp_h = (params.panel_rect.height - MATERIAL_HEADER_HEIGHT).max(10.0);

        match params.entity {
            None => {
                let center_style = Style::new()
                    .flex_col()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .height(vp_h)
                    .padding_insets(Insets::new(12.0, 12.0, 12.0, 12.0));
                panel_scope.container(center_style, |center_scope| {
                    build_no_entity_selected(center_scope);
                });
            }
            Some(entity) => {
                let model_handle = params
                    .world
                    .get::<&ae_core::ecs::ModelId>(entity)
                    .ok()
                    .map(|m| m.0);
                let has_model = model_handle.is_some();
                let has_sprite = params.world.get::<&ae_core::ecs::SpriteId>(entity).is_ok();

                if has_model {
                    let mut vp_style = Style::new()
                        .clip_children(true)
                        .flex_col()
                        .height(vp_h)
                        .gap(6.0)
                        .padding_insets(Insets::new(6.0, 6.0, 6.0, 6.0));
                    vp_style.scroll_offset_y = params.scroll_y;

                    let vp_id = panel_scope.container_tagged(
                        "MaterialViewport",
                        vp_style,
                        WidgetRole::Default,
                        MATERIAL_TAG_VIEWPORT,
                        |vp_scope| {
                            let submesh_params = SubmeshViewParams {
                                entity,
                                world: params.world,
                                models: params.models,
                                textures: params.textures,
                            };
                            build_submesh_view(vp_scope, &submesh_params);
                        },
                    );

                    let content_h = measure_content_height(panel_scope.tree(), vp_id);
                    max_scroll = (content_h - vp_h).max(0.0);
                } else if has_sprite {
                    let mut vp_style = Style::new()
                        .clip_children(true)
                        .flex_col()
                        .height(vp_h)
                        .gap(6.0)
                        .padding_insets(Insets::new(6.0, 6.0, 6.0, 6.0));
                    vp_style.scroll_offset_y = params.scroll_y;

                    let vp_id = panel_scope.container_tagged(
                        "MaterialViewport",
                        vp_style,
                        WidgetRole::Default,
                        MATERIAL_TAG_VIEWPORT,
                        |vp_scope| {
                            let sprite_params = SpriteViewParams {
                                entity,
                                world: params.world,
                                textures: params.textures,
                            };
                            build_sprite_view(vp_scope, &sprite_params);
                        },
                    );

                    let content_h = measure_content_height(panel_scope.tree(), vp_id);
                    max_scroll = (content_h - vp_h).max(0.0);
                } else {
                    let center_style = Style::new()
                        .flex_col()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center)
                        .height(vp_h)
                        .padding_insets(Insets::new(12.0, 12.0, 12.0, 12.0));
                    panel_scope.container(center_style, |center_scope| {
                        build_no_renderable_geometry(center_scope);
                    });
                }
            }
        }

        // 3. Vertical Scrollbar
        if max_scroll > 0.0 {
            let content_h = vp_h + max_scroll;
            let effective_scroll_y = params.scroll_y.clamp(0.0, max_scroll);
            let vp_rect = Rect::new(
                params.panel_rect.x,
                params.panel_rect.y + MATERIAL_HEADER_HEIGHT,
                params.panel_rect.width,
                vp_h,
            );
            let scroll_style = ScrollAreaStyle::dark_default();
            if let Some(geom) = ScrollBarGeometry::compute_vertical(
                vp_rect,
                content_h,
                effective_scroll_y,
                &scroll_style,
            ) {
                panel_scope.scrollbar_vertical(
                    geom,
                    params.panel_rect,
                    params.is_scrollbar_dragging,
                    Some(params.cursor_pos),
                    MATERIAL_TAG_SCROLLBAR_TRACK,
                    MATERIAL_TAG_SCROLLBAR_THUMB,
                );
            }
        }

        panel_scope.finish_layout(params.panel_rect);
    });

    max_scroll
}