// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Panel Orchestrator & Viewport Lifecycle
//!
//! Assembles the Scene Hierarchy root container, header toolbar, virtualized DFS
//! entity rows, footer status bar, and cascading add menu using pure declarative [`UiScope`].

use super::add_menu::draw_hierarchy_add_menu;
use super::footer::build_hierarchy_footer;
use super::header::build_hierarchy_header;
use super::rows::build_hierarchy_rows;
use super::types::{HIERARCHY_TAG_PANEL_ROOT, HierarchyPanelParams};
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

/// Builds the complete Scene Hierarchy panel tree purely using declarative [`UiScope`].
pub fn build_hierarchy_panel(
    scope: &mut UiScope<'_>,
    params: &HierarchyPanelParams<'_>,
    model: &TreeModel<hecs::Entity>,
    tree_state: &mut TreeState<hecs::Entity>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope
        .panel("HierarchyPanelRoot")
        .tagged(HIERARCHY_TAG_PANEL_ROOT.raw())
        .bounds(params.panel_rect)
        .background_themed(|t| t.panel_bg)
        .show(|panel_scope| {
            build_hierarchy_header(panel_scope, params, action_bus);
            build_hierarchy_rows(panel_scope, model, tree_state, params, action_bus);
            build_hierarchy_footer(panel_scope, model.total_count(), params);
            draw_hierarchy_add_menu(panel_scope, params, action_bus);
        });
}