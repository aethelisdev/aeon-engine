// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Panel Orchestrator
//!
//! Assembles the root panel container, header bar, hardware scissor clipping, and delegates
//! to specialized submesh or sprite material views based on active ECS entity state.
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

    // 1. Root Panel Container with Hardware Scissor Clipping
    let root_id = tree.create_node();
    if let Some(node) = tree.get_mut(root_id) {
        node.set_name("MaterialPanelRoot");
        node.computed_rect = params.panel_rect;
        node.style = Style::new()
            .background(Color::rgba(0.065, 0.068, 0.080, 0.98))
            .border(1.0, Color::rgba(0.12, 0.13, 0.16, 0.90))
            .clip_children(true);
    }
    let _ = tree.add_child(parent_id, root_id);

    // 2. Top Header Bar
    build_material_header(
        tree,
        root_id,
        params.panel_rect,
        params.entity,
        params.world,
    );

    // 3. Main Body Content Area
    match params.entity {
        None => {
            build_no_entity_selected(tree, root_id, params.panel_rect);
            targets.content_height = 140.0;
        }
        Some(entity) => {
            let has_model = params.world.get::<&ae_core::ecs::ModelId>(entity).is_ok();
            let has_sprite = params.world.get::<&ae_core::ecs::SpriteId>(entity).is_ok();

            if has_model {
                let vp_h = (params.panel_rect.height - MATERIAL_HEADER_HEIGHT).max(10.0);
                let vp_rect = Rect::new(
                    params.panel_rect.x,
                    params.panel_rect.y + MATERIAL_HEADER_HEIGHT,
                    params.panel_rect.width,
                    vp_h,
                );

                let vp_id = tree.create_node();
                if let Some(node) = tree.get_mut(vp_id) {
                    node.set_name("MaterialSubmeshViewport");
                    node.computed_rect = vp_rect;
                    node.style = Style::new().clip_children(true);
                }
                let _ = tree.add_child(root_id, vp_id);

                let start_y = vp_rect.y - params.scroll_y + 8.0;
                let submesh_params = SubmeshViewParams {
                    entity,
                    world: params.world,
                    models: params.models,
                    textures: params.textures,
                    start_y,
                    cursor_pos: params.cursor_pos,
                };
                let added_h =
                    build_submesh_view(tree, vp_id, params.panel_rect, &submesh_params, targets);
                targets.content_height = added_h + 16.0;
            } else if has_sprite {
                let vp_h = (params.panel_rect.height - MATERIAL_HEADER_HEIGHT).max(10.0);
                let vp_rect = Rect::new(
                    params.panel_rect.x,
                    params.panel_rect.y + MATERIAL_HEADER_HEIGHT,
                    params.panel_rect.width,
                    vp_h,
                );

                let vp_id = tree.create_node();
                if let Some(node) = tree.get_mut(vp_id) {
                    node.set_name("MaterialSpriteViewport");
                    node.computed_rect = vp_rect;
                    node.style = Style::new().clip_children(true);
                }
                let _ = tree.add_child(root_id, vp_id);

                let start_y = vp_rect.y - params.scroll_y + 8.0;
                let sprite_params = SpriteViewParams {
                    entity,
                    world: params.world,
                    textures: params.textures,
                    start_y,
                    cursor_pos: params.cursor_pos,
                };
                let added_h =
                    build_sprite_view(tree, vp_id, params.panel_rect, &sprite_params, targets);
                targets.content_height = added_h + 16.0;
            } else {
                build_no_renderable_geometry(
                    tree,
                    root_id,
                    params.panel_rect,
                    targets,
                    params.cursor_pos,
                );
                targets.content_height = 170.0;
            }
        }
    }
}