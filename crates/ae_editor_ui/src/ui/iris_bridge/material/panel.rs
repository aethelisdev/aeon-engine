// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Panel Orchestrator
//!
//! Assembles the root panel container, header bar, scrollable viewport, and
//! vertical scrollbar for specialized submesh or sprite material views purely using [`UiScope`].

use super::empty_state::{build_no_entity_selected, build_no_renderable_geometry};
use super::header::build_material_header;
use super::sprite_view::build_sprite_view;
use super::submesh_view::build_submesh_view;
use super::types::{
    MATERIAL_TAG_SCROLLBAR_THUMB, MATERIAL_TAG_SCROLLBAR_TRACK, MATERIAL_TAG_VIEWPORT,
    MaterialPanelState,
};
use crate::ui::panel_registry::EditorPanelContext;
use crate::ui::types::EngineUiAction;
use ae_renderer::asset::AssetStorage;
use ae_renderer::render::{ModelAsset, TextureAsset};
use irisui::prelude::*;
use irisui::workbench::ActionBus;

impl MaterialPanelState {
    /// Draws the Material & Surface Studio panel directly through Iris UI workbench context.
    pub(crate) fn draw_panel(&mut self, scope: &mut UiScope<'_>, ctx: &mut EditorPanelContext<'_>) {
        self.selected_entity = ctx.data.editor_state.primary_selected();
        self.active_model = self
            .selected_entity
            .and_then(|e| ctx.data.world.get_copied::<ae_plugin_api::ModelId>(e).ok())
            .map(|m| m.0);

        build_material_panel(
            scope,
            self.selected_entity,
            ctx.data.world,
            ctx.data.textures,
            ctx.data.models,
            ctx.action_bus,
        );
    }
}

#[derive(Clone, Copy)]
enum MaterialSurfaceKind {
    NoSelection,
    Model(ae_ecs::Entity),
    Sprite(ae_ecs::Entity),
    UnsupportedGeometry,
}

impl MaterialSurfaceKind {
    fn resolve(world: &ae_ecs::World, entity: Option<ae_ecs::Entity>) -> Self {
        match entity {
            None => Self::NoSelection,
            Some(entity) if world.has::<ae_plugin_api::ModelId>(entity) => Self::Model(entity),
            Some(entity) if world.has::<ae_plugin_api::SpriteId>(entity) => Self::Sprite(entity),
            Some(_) => Self::UnsupportedGeometry,
        }
    }
}

/// Builds the complete Material & Surface Studio panel tree in the retained `UiTree`.
pub fn build_material_panel(
    scope: &mut UiScope<'_>,
    entity: Option<ae_ecs::Entity>,
    world: &ae_ecs::World,
    textures: &AssetStorage<TextureAsset>,
    models: &AssetStorage<ModelAsset>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    build_material_header(scope, entity, world);
    build_material_body(scope, entity, world, textures, models, action_bus);
}

fn build_material_body(
    scope: &mut UiScope<'_>,
    entity: Option<ae_ecs::Entity>,
    world: &ae_ecs::World,
    textures: &AssetStorage<TextureAsset>,
    models: &AssetStorage<ModelAsset>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    match MaterialSurfaceKind::resolve(world, entity) {
        MaterialSurfaceKind::NoSelection => {
            scope.centered_stretch(12.0, |center_scope| {
                build_no_entity_selected(center_scope);
            });
        }
        MaterialSurfaceKind::UnsupportedGeometry => {
            scope.centered_stretch(12.0, |center_scope| {
                build_no_renderable_geometry(center_scope, entity, action_bus);
            });
        }
        MaterialSurfaceKind::Model(entity) => {
            render_material_viewport(scope, entity, |vp_scope| {
                build_submesh_view(vp_scope, entity, world, models, textures, action_bus);
            });
        }
        MaterialSurfaceKind::Sprite(entity) => {
            render_material_viewport(scope, entity, |vp_scope| {
                build_sprite_view(vp_scope, entity, world, textures, action_bus);
            });
        }
    }
}

fn render_material_viewport(
    scope: &mut UiScope<'_>,
    entity: ae_ecs::Entity,
    f: impl FnOnce(&mut UiScope<'_>),
) {
    ScrollAreaView::new("MaterialViewport")
        .tag(MATERIAL_TAG_VIEWPORT)
        .scrollbar_tags(MATERIAL_TAG_SCROLLBAR_TRACK, MATERIAL_TAG_SCROLLBAR_THUMB)
        .scroll_key(entity.to_bits().get())
        .show_scrolled(scope, f);
}