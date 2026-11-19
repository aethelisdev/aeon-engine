// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Workbench Panel Registry & Direct Typed Panel Bindings
//!
//! Binds all 10 built-in editor panels directly to their own typed state renderers
//! (`ViewportHudState`, `HierarchyPanelState`, `InspectorPanelState`, etc.) through a single
//! data-driven [`ScopedWorkbenchPanel`] descriptor anchored to [`PanelId`] metadata (SSOT).

use crate::ui::iris_bridge::types::{IrisEditorOverlay, OverlayUpdateParams};
use crate::ui::panel_layout::PanelId;
use irisui::dock::{DeferredNotifier, TabPlacement};
use irisui::prelude::{Point, UiScope};
use irisui::workbench::{ScopedPanelContext, ScopedPanelRegistry, ScopedWorkbenchPanel};

/// Execution context passed to editor workbench panels during rendering.
pub struct EditorPanelContext<'a> {
    /// Mutable reference to active editor overlay panel, focus, and action state.
    pub overlay: &'a mut IrisEditorOverlay,
    /// Deferred invalidation queue applied to [`Workbench::notifier`] after the frame pass.
    pub notifier: &'a mut DeferredNotifier<PanelId>,
    /// Mutable reference to the per-frame overlay parameters.
    pub params: &'a mut OverlayUpdateParams<'a>,
    /// Caret visibility sampled from the persistent [`irisui::text::TextSystem`].
    pub blink_caret: bool,
    /// Logical cursor position sampled before panel dispatch.
    pub cursor_pos: Point,
}

/// Lifetime scope marker projecting to [`EditorPanelContext`] for persistent panel registration.
#[derive(Debug, Clone, Copy, Default)]
pub struct EditorPanelScope;

impl ScopedPanelContext for EditorPanelScope {
    type Context<'frame> = EditorPanelContext<'frame>;
}

type EditorPanelDrawerFn = for<'frame> fn(&mut UiScope<'_>, &mut EditorPanelContext<'frame>);

struct EditorWorkbenchPanel {
    id: PanelId,
    drawer: EditorPanelDrawerFn,
}

impl ScopedWorkbenchPanel<PanelId, EditorPanelScope> for EditorWorkbenchPanel {
    fn id(&self) -> PanelId {
        self.id
    }

    fn title(&self) -> &str {
        self.id.title()
    }

    fn icon(&self) -> Option<&'static str> {
        self.id.atlas_icon().is_none().then(|| self.id.icon())
    }

    fn atlas_icon(&self) -> Option<[f32; 4]> {
        self.id.atlas_icon()
    }

    fn is_domain_bound(&self) -> bool {
        matches!(
            self.id,
            PanelId::Hierarchy
                | PanelId::Inspector
                | PanelId::MaterialEditor
                | PanelId::AnimationTimeline
        )
    }

    fn closeable(&self) -> bool {
        !matches!(self.id, PanelId::Viewport)
    }

    fn clear_background(&self) -> bool {
        matches!(self.id, PanelId::Viewport)
    }

    fn placement(&self) -> Option<TabPlacement<PanelId>> {
        Some(self.id.placement())
    }

    fn render<'frame>(&mut self, scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'frame>) {
        (self.drawer)(scope, state);
    }
}

fn draw_viewport_panel<'frame>(scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'frame>) {
    state.overlay.viewport_hud.build_view(
        scope,
        if scope.parent_rect().width > 20.0 {
            scope.parent_rect()
        } else {
            state.params.viewport.viewport_rect
        },
        state.params,
        state.cursor_pos,
    );
}

fn draw_hierarchy_panel<'frame>(scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'frame>) {
    state
        .overlay
        .hierarchy
        .build_view(scope, state.params, (state.cursor_pos, state.blink_caret));
}

fn draw_inspector_panel<'frame>(scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'frame>) {
    state.overlay.inspector.build_view(
        scope,
        state.params,
        state.cursor_pos,
        &mut state.overlay.pending_actions,
        state.notifier,
    );
}

fn draw_console_panel<'frame>(scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'frame>) {
    state.overlay.console.build_view(
        scope,
        state.params.panel_data.console_entries,
        state.notifier,
        (state.cursor_pos, state.blink_caret),
    );
}

fn draw_assets_panel<'frame>(scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'frame>) {
    state.overlay.assets.build_view(
        scope,
        state.params,
        state.notifier,
        (state.cursor_pos, state.blink_caret),
    );
}

fn draw_stats_panel<'frame>(scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'frame>) {
    state
        .overlay
        .stats
        .build_view(scope, state.params, state.cursor_pos);
}

fn draw_material_panel<'frame>(scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'frame>) {
    state
        .overlay
        .material
        .build_view(scope, state.params, state.cursor_pos);
}

fn draw_timeline_panel<'frame>(scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'frame>) {
    state
        .overlay
        .timeline
        .build_view(scope, state.params, state.cursor_pos);
}

fn draw_ui_designer_panel<'frame>(scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'frame>) {
    state
        .overlay
        .ui_designer
        .build_view(scope, state.params, state.cursor_pos, state.notifier);
}

fn draw_preferences_panel<'frame>(scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'frame>) {
    state.overlay.preferences.build_view(
        scope,
        state.params,
        (state.cursor_pos, state.blink_caret),
    );
}

const EDITOR_PANEL_DRAWERS: &[(PanelId, EditorPanelDrawerFn)] = &[
    (PanelId::Viewport, draw_viewport_panel),
    (PanelId::Hierarchy, draw_hierarchy_panel),
    (PanelId::Inspector, draw_inspector_panel),
    (PanelId::Console, draw_console_panel),
    (PanelId::Assets, draw_assets_panel),
    (PanelId::Stats, draw_stats_panel),
    (PanelId::MaterialEditor, draw_material_panel),
    (PanelId::AnimationTimeline, draw_timeline_panel),
    (PanelId::UiDesigner, draw_ui_designer_panel),
    (PanelId::Preferences, draw_preferences_panel),
];

/// Constructs and populates a persistent [`ScopedPanelRegistry`] with all built-in editor workbench panels.
#[must_use]
pub fn create_editor_panel_registry() -> ScopedPanelRegistry<PanelId, EditorPanelScope> {
    EDITOR_PANEL_DRAWERS
        .iter()
        .fold(ScopedPanelRegistry::new(), |mut registry, &(id, drawer)| {
            registry.register(EditorWorkbenchPanel { id, drawer });
            registry
        })
}

#[cfg(test)]
#[path = "panels_tests.rs"]
mod tests;