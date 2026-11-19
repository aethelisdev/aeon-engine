// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector Panel Autonomous Builder
//!
//! Autonomous builder coordinating entity inspection, component reflection cards,
//! live numeric math expression sessions, and rename inputs.

use super::panel::build_inspector_panel;
use super::types::{InspectorPanelParams, InspectorPanelState};
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use crate::ui::panel_layout::PanelId;
use irisui::prelude::*;

impl InspectorPanelState {
    /// Builds the Scene Inspector panel directly using typed inspector state and `UiScope`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        cursor_pos: Point,
        pending_actions: &mut Vec<crate::ui::EngineUiAction>,
        notifier: &mut DeferredNotifier<PanelId>,
    ) {
        if scope.is_collapsed() {
            self.inspected_entity_name.clear();
            return;
        }

        build_inspector_panel(
            scope,
            &InspectorPanelParams {
                panel_rect: scope.parent_rect(),
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                cursor_pos,
            },
            &mut self.inspected_entity_name,
        )
        .into_iter()
        .for_each(|ui_action| {
            pending_actions.push(ui_action);
            notifier.tag_redraw(crate::ui::panel_layout::PanelId::Inspector);
        });

        self.sync_entity_name(params.scene.selected_entity, params.scene.world);
    }
}