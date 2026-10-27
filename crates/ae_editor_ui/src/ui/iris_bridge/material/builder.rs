// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Panel Autonomous Builder
//!
//! Autonomous builder coordinating material property inspection, texture maps,
//! shader parameter cards, and scroll offsets.

use super::panel::build_material_panel;
use super::types::MaterialPanelParams;
use crate::ui::iris_bridge::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds the Material & Surface Studio panel if active in either docked or floating mode.
    ///
    /// Manages the full lifecycle of the Material Editor:
    /// 1. Resolves physical panel bounds directly from the unified [`UiScope`] docking hierarchy.
    /// 2. Resolves model assets and textures attached to the currently selected entity.
    /// 3. Drains pending material click, scroll, and drag interaction events.
    /// 4. Assembles PBR material properties, texture assignment cards, and empty selection views in unified coordinates.
    pub(crate) fn build_material_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        let panel_rect = scope.parent_rect();
        if panel_rect.width <= 20.0 || panel_rect.height <= 20.0 {
            self.material.active_model = None;
            return;
        }

        let events = std::mem::take(&mut self.material.pending_interaction_events);

        let material_params = MaterialPanelParams {
            panel_rect,
            entity: params.scene.selected_entity,
            world: params.scene.world,
            textures: params.panel_data.textures,
            models: params.panel_data.models,
            cursor_pos: self.cursor_pos(),
            events: &events,
        };

        build_material_panel(
            scope,
            &material_params,
            &mut self.material.interactions.actions,
        );
        self.material.active_model = params
            .scene
            .selected_entity
            .and_then(|e| params.scene.world.get::<&ae_core::ecs::ModelId>(e).ok())
            .map(|m| m.0);
    }
}