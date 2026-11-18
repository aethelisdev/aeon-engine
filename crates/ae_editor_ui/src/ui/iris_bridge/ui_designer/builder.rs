// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Panel Autonomous Builder
//!
//! Autonomous builder coordinating canvas aspect ratios, element positioning,
//! drag handles, hierarchy tree, and widget property cards.

use super::panel::build_ui_designer_panel;
use super::types::{UiDesignerPanelParams, UiDesignerPanelState};
use crate::ui::iris_bridge::panel_views::DeferredPanelNotifier;
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use irisui::prelude::*;

impl UiDesignerPanelState {
    /// Builds the 2D Visual UI Designer panel directly using typed designer state and `UiScope`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        cursor_pos: Point,
        notifier: &mut DeferredPanelNotifier<'_>,
    ) {
        if scope.is_collapsed() {
            self.drag_contexts.clear();
            self.canvas_metrics = Default::default();
            return;
        }

        _ = build_ui_designer_panel(
            scope,
            &UiDesignerPanelParams {
                panel_rect: scope.parent_rect(),
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                cursor_pos,
                state: params.panel_data.ui_designer_state,
            },
            self,
        );
        if self.has_pending_actions() {
            notifier.tag_redraw(crate::ui::panel_layout::PanelId::UiDesigner);
        }
    }
}