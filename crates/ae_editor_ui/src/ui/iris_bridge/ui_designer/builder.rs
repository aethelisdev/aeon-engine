// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Panel Autonomous Builder
//!
//! Autonomous builder coordinating canvas aspect ratios, element positioning,
//! drag handles, hierarchy tree, and widget property cards.
//!

use super::panel::build_ui_designer_panel;
use super::types::UiDesignerPanelParams;
use crate::ui::iris_bridge::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds the 2D Visual UI Designer panel if active in either docked or floating mode.
    ///
    /// Manages the full lifecycle of the UI Designer:
    /// 1. Resolves physical panel bounds directly from the unified [`UiScope`] docking hierarchy.
    /// 2. Assembles visual canvas preview with aspect ratio constraints in unified coordinates.
    /// 3. Manages element selection, bounding box transform gizmos, and property panels.
    /// 4. Signals notifier redraw when designer actions or drag events occur.
    pub(crate) fn build_ui_designer_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        let panel_rect = scope.parent_rect();
        if panel_rect.width <= 20.0 || panel_rect.height <= 20.0 {
            self.ui_designer.drag_contexts.clear();
            self.ui_designer.canvas_metrics = Default::default();
            return;
        }

        let designer_params = UiDesignerPanelParams {
            panel_rect,
            world: params.scene.world,
            selected_entity: params.scene.selected_entity,
            cursor_pos: self.cursor_pos(),
            state: params.panel_data.ui_designer_state,
            is_aspect_dropdown_open: self.ui_designer.is_aspect_open,
            is_add_menu_open: self.ui_designer.is_add_menu_open,
        };

        let _metrics = build_ui_designer_panel(scope, &designer_params, &mut self.ui_designer);
        if self.ui_designer.has_pending_actions() {
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
        }
    }
}