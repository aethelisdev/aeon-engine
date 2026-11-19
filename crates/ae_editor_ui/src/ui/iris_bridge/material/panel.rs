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
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

#[derive(Clone, Copy)]
enum MaterialSurfaceKind {
    NoSelection,
    Model(hecs::Entity),
    Sprite(hecs::Entity),
    UnsupportedGeometry,
}

impl MaterialSurfaceKind {
    fn resolve(world: &hecs::World, entity: Option<hecs::Entity>) -> Self {
        match entity {
            None => Self::NoSelection,
            Some(entity) if world.get::<&ae_core::ecs::ModelId>(entity).is_ok() => {
                Self::Model(entity)
            }
            Some(entity) if world.get::<&ae_core::ecs::SpriteId>(entity).is_ok() => {
                Self::Sprite(entity)
            }
            Some(_) => Self::UnsupportedGeometry,
        }
    }
}

/// Builds the complete Material & Surface Studio panel tree in the retained `UiTree`.
pub fn build_material_panel(
    scope: &mut UiScope<'_>,
    params: &MaterialPanelParams<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope.panel("MaterialPanelRoot").show(|panel_scope| {
        build_material_header(panel_scope, params.entity, params.world);
        build_material_body(panel_scope, params, action_bus);
    });
}

fn build_material_body(
    scope: &mut UiScope<'_>,
    params: &MaterialPanelParams<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    match MaterialSurfaceKind::resolve(params.world, params.entity) {
        MaterialSurfaceKind::NoSelection => {
            scope.centered_stretch(12.0, |center_scope| {
                build_no_entity_selected(center_scope);
            });
        }
        MaterialSurfaceKind::UnsupportedGeometry => {
            scope.centered_stretch(12.0, |center_scope| {
                build_no_renderable_geometry(center_scope, params.entity, action_bus);
            });
        }
        MaterialSurfaceKind::Model(entity) => {
            render_material_viewport(scope, |vp_scope| {
                build_submesh_view(
                    vp_scope,
                    &SubmeshViewParams {
                        entity,
                        world: params.world,
                        models: params.models,
                        textures: params.textures,
                    },
                    action_bus,
                );
            });
        }
        MaterialSurfaceKind::Sprite(entity) => {
            render_material_viewport(scope, |vp_scope| {
                build_sprite_view(
                    vp_scope,
                    &SpriteViewParams {
                        entity,
                        world: params.world,
                        textures: params.textures,
                    },
                    action_bus,
                );
            });
        }
    }
}

fn render_material_viewport(scope: &mut UiScope<'_>, f: impl FnOnce(&mut UiScope<'_>)) {
    scope.scroll_area(
        "MaterialViewport",
        MATERIAL_TAG_VIEWPORT,
        Some((MATERIAL_TAG_SCROLLBAR_TRACK, MATERIAL_TAG_SCROLLBAR_THUMB)),
        f,
    );
}