// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector Panel Autonomous Builder
//!
//! Autonomous builder coordinating entity inspection, component reflection cards,
//! live numeric math expression sessions, and rename inputs.

use super::panel::build_inspector_panel;
use super::types::{ActiveNumberInputState, InspectorPanelParams};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds the Scene Inspector panel if active in either docked or floating mode.
    ///
    /// Evaluates currently selected entity, extracts active numeric drag/text input sessions,
    /// and constructs the component property cards, transform math controls, and empty state cards
    /// directly within the unified [`UiScope`] docking coordinate system.
    pub(crate) fn build_inspector_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        let panel_rect = scope.parent_rect();
        if panel_rect.width <= 20.0 || panel_rect.height <= 20.0 {
            self.inspector.inspected_entity = None;
            self.inspector.inspected_entity_name.clear();
            return;
        }

        let num_input_ref = self
            .inspector
            .active_number_input
            .as_ref()
            .filter(|session| Some(session.entity) == params.scene.selected_entity)
            .map(|session| ActiveNumberInputState {
                id: session.id,
                buffer: session.buffer.as_str(),
                cursor_idx: session.cursor_idx,
                is_all_selected: session.is_all_selected,
            });
        let text_input_ref = self
            .inspector
            .active_text_input
            .as_ref()
            .filter(|(ent, _, _)| Some(*ent) == params.scene.selected_entity)
            .map(|(_, id, s)| (*id, s.as_str()));
        let rename_buf_ref = self
            .inspector
            .rename_input
            .as_ref()
            .filter(|(ent, _)| Some(*ent) == params.scene.selected_entity)
            .map(|(_, input)| input.text());
        let hex_buf_ref = self
            .inspector
            .hex_buffer
            .as_ref()
            .filter(|(ent, _)| Some(*ent) == params.scene.selected_entity)
            .map(|(_, s)| s.as_str());

        let insp_params = InspectorPanelParams {
            panel_rect,
            world: params.scene.world,
            selected_entity: params.scene.selected_entity,
            inspector_euler: params.panel_data.inspector_euler,
            inspector_color_hex: params.panel_data.inspector_color_hex,
            saved_swatches: params.panel_data.saved_swatches,
            cursor_pos: self.cursor_pos(),
            active_dropdown: self.inspector.active_dropdown,
            active_submenu: self.inspector.active_submenu,
            is_add_menu_open: self.inspector.is_add_menu_open,
            is_color_picker_open: self.inspector.is_color_picker_open,
            active_number_input: num_input_ref,
            active_text_input: text_input_ref,
            active_rename_buffer: rename_buf_ref,
            is_rename_all_selected: self.inspector.rename_is_all_selected
                && rename_buf_ref.is_some(),
            active_hex_buffer: hex_buf_ref,
            inspector_hsv: self.inspector.hsv,
            blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
        };

        build_inspector_panel(
            scope,
            &insp_params,
            &mut self.inspector.interactions.actions,
        );
        self.inspector.inspected_entity = params.scene.selected_entity;
        self.inspector.inspector_euler = *params.panel_data.inspector_euler;
        if let Some(entity) = params.scene.selected_entity {
            if let Ok(name) = params.scene.world.get::<&ae_core::ecs::Name>(entity) {
                self.inspector.inspected_entity_name.clear();
                self.inspector
                    .inspected_entity_name
                    .push_str(name.0.as_str());
            } else {
                self.inspector.inspected_entity_name.clear();
            }
        } else {
            self.inspector.inspected_entity_name.clear();
        }
    }
}