// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Panel Autonomous Builder
//!
//! Autonomous builder coordinating material property inspection, texture maps,
//! shader parameter cards, and scroll offsets.

use super::panel::build_material_panel;
use super::types::{MaterialPanelParams, MaterialPanelState};
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use irisui::prelude::*;

impl MaterialPanelState {
    /// Builds the Material & Surface Studio panel directly using typed material state and `UiScope`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        cursor_pos: Point,
    ) {
        if scope.is_collapsed() {
            self.active_model = None;
            return;
        }

        if self.selected_entity != params.scene.selected_entity {
            self.selected_entity = params.scene.selected_entity;
            scope.set_scroll_offset(super::types::MATERIAL_TAG_VIEWPORT, 0.0);
        }

        build_material_panel(
            scope,
            &MaterialPanelParams {
                panel_rect: scope.parent_rect(),
                entity: params.scene.selected_entity,
                world: params.scene.world,
                textures: params.panel_data.textures,
                models: params.panel_data.models,
                cursor_pos,
                events: &std::mem::take(&mut self.pending_interaction_events),
            },
            &mut self.interactions.actions,
        );
        self.active_model = params
            .scene
            .selected_entity
            .and_then(|e| params.scene.world.get::<&ae_core::ecs::ModelId>(e).ok())
            .map(|m| m.0);
    }
}