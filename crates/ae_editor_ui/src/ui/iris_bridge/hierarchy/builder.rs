// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Panel Autonomous Builder
//!
//! Synchronizes search input buffers with `UiTree`, updates the irisui cached [`TreeModel`]
//! through the swappable [`SceneHierarchySource`] adapter, and builds the declarative panel view.

use super::panel::build_hierarchy_panel;
use super::rows::SceneHierarchySource;
use super::types::{
    HIERARCHY_TAG_SEARCH_CLEAR, HIERARCHY_TAG_SEARCH_INPUT, HierarchyPanelParams,
    HierarchyPanelState,
};
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

impl HierarchyPanelState {
    /// Builds the Scene Hierarchy panel using cached [`TreeModel`] sync and [`ActionBus`].
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        blink_caret: bool,
        action_bus: &mut ActionBus<EngineUiAction>,
    ) {
        if scope.is_collapsed() {
            return;
        }

        scope.auto_sync_text_input(
            HIERARCHY_TAG_SEARCH_INPUT.raw(),
            Some(HIERARCHY_TAG_SEARCH_CLEAR.raw()),
            &mut self.search_input,
        );

        self.model.sync(
            &SceneHierarchySource(params.scene.world),
            &self.tree_state,
            params.context.history_revision,
            params.scene.active_entities_count,
            self.search_input.text(),
        );

        build_hierarchy_panel(
            scope,
            &HierarchyPanelParams {
                panel_rect: scope.parent_rect(),
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                search_query: self.search_input.text(),
                is_editing: params.context.is_editing,
                is_2d: params.context.is_2d_mode,
                blink_caret,
                cursor_byte_idx: self.search_input.cursor_byte_idx,
            },
            &self.model,
            &mut self.tree_state,
            action_bus,
        );
    }
}