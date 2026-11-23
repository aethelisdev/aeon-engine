// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Panel Root Builder
//!
//! Assembles the complete UI Designer panel hierarchy in the Iris UI tree,
//! including the 34px elevated toolbar, virtual pan/zoom canvas, and anchored dropdown
//! popups purely using declarative [`UiScope`] containers and 64-bit semantic tags.

use super::canvas::build_designer_canvas;
use super::popups::build_add_element_popup;
use super::toolbar::build_designer_toolbar;
use super::types::{UiDesignerPanelState, UiDesignerState};
use crate::ui::panel_layout::PanelId;
use crate::ui::panel_registry::EditorPanelContext;
use crate::ui::types::EngineUiAction;
use irisui::dock::DeferredNotifier;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

impl UiDesignerPanelState {
    /// Entrypoint invoked directly by [`irisui::workbench::ScopedPanelRegistry`].
    pub(crate) fn draw_panel(&mut self, scope: &mut UiScope<'_>, ctx: &mut EditorPanelContext<'_>) {
        self.last_transform = build_ui_designer_panel(
            scope,
            ctx.data.world,
            ctx.data.editor_state.primary_selected(),
            ctx.data.ui_designer_state,
            self,
            ctx.action_bus,
            ctx.notifier,
        );
    }
}

/// Assembles the complete 2D Visual UI Designer panel tree in native Iris UI.
pub fn build_ui_designer_panel(
    scope: &mut UiScope<'_>,
    world: &ae_ecs::World,
    selected_entity: Option<ae_ecs::Entity>,
    state: &mut UiDesignerState,
    designer: &mut UiDesignerPanelState,
    action_bus: &mut ActionBus<EngineUiAction>,
    notifier: &mut DeferredNotifier<PanelId>,
) -> PanZoomTransform {
    build_designer_toolbar(scope, state, notifier);
    designer.last_transform =
        build_designer_canvas(scope, world, selected_entity, state, action_bus, notifier);

    build_ui_designer_overlays(scope, action_bus, notifier);
    designer.last_transform
}

/// Builds top-level floating popups (`Add Element` palette) on `UiLayer::Popup` via [`UiScope`].
pub fn build_ui_designer_overlays(
    scope: &mut UiScope<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
    notifier: &mut DeferredNotifier<PanelId>,
) {
    build_add_element_popup(scope, action_bus, notifier);
}