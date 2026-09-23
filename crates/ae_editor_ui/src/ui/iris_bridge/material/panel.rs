// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Panel Orchestrator
//!
//! Assembles the root panel container, header bar, and delegates to specialized
//! submesh or sprite material views purely using [`UiScope`].
//!

use super::empty_state::{build_no_entity_selected, build_no_renderable_geometry};
use super::header::{MATERIAL_HEADER_HEIGHT, build_material_header};
use super::sprite_view::{SpriteViewParams, build_sprite_view};
use super::submesh_view::{SubmeshViewParams, build_submesh_view};
use super::types::{MaterialPanelParams, MaterialPanelTargets};
use irisui::prelude::*;

/// Builds the complete Material & Surface Studio panel tree in the retained `UiTree`.
pub fn build_material_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &MaterialPanelParams<'_>,
    targets: &mut MaterialPanelTargets,
) {
    targets.panel_rect = params.panel_rect;

    let mut scope =
        UiScope::with_tagged_interactions(tree, parent_id, params.events, params.hovered_tag);

    let root_style = Style::new()
        .flex_col()
        .background(Color::rgba(0.065, 0.068, 0.080, 0.98))
        .border(1.0, Color::rgba(0.12, 0.13, 0.16, 0.90))
        .clip_children(true);

    scope.container(root_style, |panel_scope| {
        // 1. Top Header Bar
        build_material_header(panel_scope, params.entity, params.world);

        // 2. Main Body Content Area
        match params.entity {
            None => {
                let body_h = (params.panel_rect.height - MATERIAL_HEADER_HEIGHT).max(120.0);
                let center_style = Style::new()
                    .flex_col()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .height(body_h)
                    .padding_insets(Insets::new(12.0, 12.0, 12.0, 12.0));
                panel_scope.container(center_style, |center_scope| {
                    build_no_entity_selected(center_scope);
                });
                targets.content_height = 160.0;
            }
            Some(entity) => {
                let model_handle = params
                    .world
                    .get::<&ae_core::ecs::ModelId>(entity)
                    .ok()
                    .map(|m| m.0);
                targets.active_model = model_handle;
                let has_model = model_handle.is_some();
                let has_sprite = params.world.get::<&ae_core::ecs::SpriteId>(entity).is_ok();

                if has_model {
                    let vp_h = (params.panel_rect.height - MATERIAL_HEADER_HEIGHT).max(10.0);
                    let vp_style = Style::new()
                        .clip_children(true)
                        .flex_col()
                        .height(vp_h)
                        .gap(6.0)
                        .padding_insets(Insets::new(6.0 - params.scroll_y, 6.0, 6.0, 6.0));
                    panel_scope.container(vp_style, |vp_scope| {
                        let submesh_params = SubmeshViewParams {
                            entity,
                            world: params.world,
                            models: params.models,
                            textures: params.textures,
                        };
                        let added_h = build_submesh_view(vp_scope, &submesh_params, targets);
                        targets.content_height = added_h + 16.0;
                    });
                } else if has_sprite {
                    let vp_h = (params.panel_rect.height - MATERIAL_HEADER_HEIGHT).max(10.0);
                    let vp_style = Style::new()
                        .clip_children(true)
                        .flex_col()
                        .height(vp_h)
                        .gap(6.0)
                        .padding_insets(Insets::new(6.0 - params.scroll_y, 6.0, 6.0, 6.0));
                    panel_scope.container(vp_style, |vp_scope| {
                        let sprite_params = SpriteViewParams {
                            entity,
                            world: params.world,
                            textures: params.textures,
                        };
                        let added_h = build_sprite_view(vp_scope, &sprite_params, targets);
                        targets.content_height = added_h + 16.0;
                    });
                } else {
                    let body_h = (params.panel_rect.height - MATERIAL_HEADER_HEIGHT).max(120.0);
                    let center_style = Style::new()
                        .flex_col()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center)
                        .height(body_h)
                        .padding_insets(Insets::new(12.0, 12.0, 12.0, 12.0));
                    panel_scope.container(center_style, |center_scope| {
                        build_no_renderable_geometry(center_scope, targets);
                    });
                    targets.content_height = 200.0;
                }
            }
        }

        panel_scope.finish_layout(params.panel_rect);
    });
}