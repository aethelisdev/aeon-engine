// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Panel Orchestrator & Viewport Lifecycle
//!
//! Assembles the Scene Hierarchy root container, header toolbar with bound search input,
//! virtualized DFS entity rows, footer status bar, and cascading add menu using pure
//! declarative [`UiScope`] composition.

use super::add_menu::draw_hierarchy_add_menu;
use super::footer::build_hierarchy_footer;
use super::header::build_hierarchy_header;
use super::rows::build_hierarchy_rows;
use super::types::HierarchyPanelState;
use crate::ui::panel_registry::EditorPanelContext;
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

impl HierarchyPanelState {
    /// Entrypoint invoked directly by [`irisui::workbench::ScopedPanelRegistry`].
    pub(crate) fn draw_panel(&mut self, scope: &mut UiScope<'_>, ctx: &mut EditorPanelContext<'_>) {
        build_hierarchy_panel(
            scope,
            ctx.data.world,
            ctx.data.editor_state.history_revision,
            ctx.data.editor_state.primary_selected(),
            ctx.data.is_2d_mode,
            self,
            ctx.action_bus,
        );
    }
}

/// Builds the complete Scene Hierarchy panel tree purely using declarative [`UiScope`].
pub fn build_hierarchy_panel(
    scope: &mut UiScope<'_>,
    world: &ae_ecs::World,
    revision: u64,
    selected_entity: Option<ae_ecs::Entity>,
    is_2d: bool,
    state: &mut HierarchyPanelState,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    build_hierarchy_header(scope, &mut state.search_input, selected_entity, action_bus);
    build_hierarchy_footer(
        build_hierarchy_rows(
            scope,
            world,
            revision,
            state.search_input.text(),
            selected_entity,
            action_bus,
        ),
        selected_entity.is_some(),
        scope,
    );
    draw_hierarchy_add_menu(scope, is_2d, action_bus);
}