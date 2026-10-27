// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Panel Orchestrator
//!
//! Assembles the root panel container, header bar, scrollable viewport, and
//! vertical scrollbar for specialized submesh or sprite material views purely using [`UiScope`].

use super::empty_state::{build_no_entity_selected, build_no_renderable_geometry};
use super::header::build_material_header;
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
    scope: &mut UiScope<'_>,
    params: &MaterialPanelParams<'_>,
    actions: &mut Vec<super::types::MaterialAction>,
) {
    let panel_bg = scope.theme().panel_bg;
    scope
        .panel("MaterialPanelRoot")
        .background(panel_bg)
        .show(|panel_scope| {
            // 1. Top Header Bar
            build_material_header(panel_scope, params.entity, params.world);

            // 2. Main Body Content Area
            match params.entity {
                None => {
                    let center_style = Style::new()
                        .flex_col()
                        .flex_grow(1.0)
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center)
                        .padding_insets(Insets::new(12.0, 12.0, 12.0, 12.0));
                    panel_scope.container(center_style, |center_scope| {
                        build_no_entity_selected(center_scope, params.panel_rect.width);
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
                        panel_scope.scroll_area(
                            "MaterialViewport",
                            MATERIAL_TAG_VIEWPORT,
                            Some((MATERIAL_TAG_SCROLLBAR_TRACK, MATERIAL_TAG_SCROLLBAR_THUMB)),
                            |vp_scope| {
                                let submesh_params = SubmeshViewParams {
                                    entity,
                                    world: params.world,
                                    models: params.models,
                                    textures: params.textures,
                                };
                                build_submesh_view(vp_scope, &submesh_params, actions);
                            },
                        );
                    } else if has_sprite {
                        panel_scope.scroll_area(
                            "MaterialViewport",
                            MATERIAL_TAG_VIEWPORT,
                            Some((MATERIAL_TAG_SCROLLBAR_TRACK, MATERIAL_TAG_SCROLLBAR_THUMB)),
                            |vp_scope| {
                                let sprite_params = SpriteViewParams {
                                    entity,
                                    world: params.world,
                                    textures: params.textures,
                                };
                                build_sprite_view(vp_scope, &sprite_params, actions);
                            },
                        );
                    } else {
                        let center_style = Style::new()
                            .flex_col()
                            .flex_grow(1.0)
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::Center)
                            .padding_insets(Insets::new(12.0, 12.0, 12.0, 12.0));
                        panel_scope.container(center_style, |center_scope| {
                            build_no_renderable_geometry(
                                center_scope,
                                params.panel_rect.width,
                                params.entity,
                                actions,
                            );
                        });
                    }
                }
            }
        });
}