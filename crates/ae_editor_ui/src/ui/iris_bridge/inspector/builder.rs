// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector Panel Autonomous Builder
//!
//! Autonomous builder coordinating entity inspection, component reflection cards,
//! live numeric math expression sessions, and rename inputs.

use super::panel::build_inspector_panel;
use super::types::{InspectorPanelParams, InspectorPanelState, InspectorSessionRefs};
use crate::ui::iris_bridge::panel_views::DeferredPanelNotifier;
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use irisui::prelude::*;

impl InspectorPanelState {
    /// Builds the Scene Inspector panel directly using typed inspector state and `UiScope`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        cursor_pos: Point,
        pending_actions: &mut Vec<crate::ui::EngineUiAction>,
        notifier: &mut DeferredPanelNotifier<'_>,
    ) {
        if scope.is_collapsed() {
            self.inspected_entity = None;
            self.inspected_entity_name.clear();
            return;
        }

        self.commit_pending_edits_on_selection_change(params.scene.selected_entity);

        if scope.popups_dismissed() {
            self.active_submenu = None;
        }

        build_inspector_panel(
            scope,
            &InspectorPanelParams {
                panel_rect: scope.parent_rect(),
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                inspector_euler: params.panel_data.inspector_euler,
                inspector_color_hex: params.panel_data.inspector_color_hex,
                saved_swatches: params.panel_data.saved_swatches,
                cursor_pos,
                active_submenu: self.active_submenu,
                active_rename_buffer: self
                    .rename_input
                    .as_ref()
                    .filter(|(ent, _)| Some(*ent) == params.scene.selected_entity)
                    .map(|(_, input)| input.text()),
                is_rename_all_selected: self.rename_is_all_selected
                    && self
                        .rename_input
                        .as_ref()
                        .is_some_and(|(ent, _)| Some(*ent) == params.scene.selected_entity),
                inspector_hsv: self.hsv,
            },
            &mut InspectorSessionRefs {
                inspected_name: &mut self.inspected_entity_name,
                saved_swatches: &mut self.saved_swatches,
                active_submenu: &mut self.active_submenu,
            },
        )
        .into_iter()
        .for_each(|ui_action| {
            pending_actions.push(ui_action);
            notifier.tag_redraw(crate::ui::panel_layout::PanelId::Inspector);
        });

        self.sync_after_build(
            params.scene.selected_entity,
            *params.panel_data.inspector_euler,
            params.scene.world,
        );
    }
}