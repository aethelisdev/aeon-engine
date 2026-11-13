// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Editor Workbench Panels
//!
//! Provides concrete implementations of [`irisui::workbench::WorkbenchPanel`] for all
//! built-in tool panels of the Aeon Engine editor interface, decoupling individual
//! tool views from monolithic parameters and enabling granular $O(M)$ dock invalidation.

use crate::ui::iris_bridge::types::{IrisEditorOverlay, OverlayUpdateParams};
use crate::ui::panel_layout::PanelId;
use irisui::prelude::UiScope;
use irisui::workbench::{WorkbenchPanel, WorkbenchPanelRegistry};

/// Execution context passed to autonomous editor workbench panels during rendering.
pub struct EditorPanelContext<'a> {
    /// Mutable reference to the parent Iris editor overlay subsystem.
    pub overlay: &'a mut IrisEditorOverlay,
    /// Immutable reference to the per-frame overlay parameters.
    pub params: &'a OverlayUpdateParams<'a>,
}

/// Autonomous 3D Viewport workbench panel.
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewportWorkbenchPanel;

impl<'a> WorkbenchPanel<PanelId, EditorPanelContext<'a>> for ViewportWorkbenchPanel {
    fn id(&self) -> PanelId {
        PanelId::Viewport
    }

    fn title(&self) -> &str {
        "Viewport"
    }

    fn icon(&self) -> Option<&'static str> {
        Some("🎥")
    }

    fn closeable(&self) -> bool {
        false
    }

    fn clear_background(&self) -> bool {
        true
    }

    fn render(&mut self, scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'a>) {
        let vp_rect = if scope.parent_rect().width > 20.0 {
            scope.parent_rect()
        } else {
            state.params.viewport.viewport_rect
        };
        if vp_rect.width > 20.0 && vp_rect.height > 20.0 {
            state
                .overlay
                .build_viewport_content(scope, vp_rect, state.params);
        }
    }
}

/// Autonomous Scene Hierarchy outliner workbench panel.
#[derive(Debug, Clone, Copy, Default)]
pub struct HierarchyWorkbenchPanel;

impl<'a> WorkbenchPanel<PanelId, EditorPanelContext<'a>> for HierarchyWorkbenchPanel {
    fn id(&self) -> PanelId {
        PanelId::Hierarchy
    }

    fn title(&self) -> &str {
        "Hierarchy"
    }

    fn icon(&self) -> Option<&'static str> {
        Some("🏗️")
    }

    fn is_domain_bound(&self) -> bool {
        true
    }

    fn closeable(&self) -> bool {
        true
    }

    fn render(&mut self, scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'a>) {
        state
            .overlay
            .build_hierarchy_panel_if_active(scope, state.params);
    }
}

/// Autonomous Entity Inspector workbench panel.
#[derive(Debug, Clone, Copy, Default)]
pub struct InspectorWorkbenchPanel;

impl<'a> WorkbenchPanel<PanelId, EditorPanelContext<'a>> for InspectorWorkbenchPanel {
    fn id(&self) -> PanelId {
        PanelId::Inspector
    }

    fn title(&self) -> &str {
        "Inspector"
    }

    fn icon(&self) -> Option<&'static str> {
        Some("⚙️")
    }

    fn is_domain_bound(&self) -> bool {
        true
    }

    fn closeable(&self) -> bool {
        true
    }

    fn render(&mut self, scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'a>) {
        state
            .overlay
            .build_inspector_panel_if_active(scope, state.params);
    }
}

/// Autonomous Developer Log & Diagnostics Console workbench panel.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConsoleWorkbenchPanel;

impl<'a> WorkbenchPanel<PanelId, EditorPanelContext<'a>> for ConsoleWorkbenchPanel {
    fn id(&self) -> PanelId {
        PanelId::Console
    }

    fn title(&self) -> &str {
        "Console"
    }

    fn icon(&self) -> Option<&'static str> {
        Some("📜")
    }

    fn closeable(&self) -> bool {
        true
    }

    fn render(&mut self, scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'a>) {
        state
            .overlay
            .build_console_panel_if_active(scope, state.params);
    }
}

/// Autonomous Content & Asset Browser workbench panel.
#[derive(Debug, Clone, Copy, Default)]
pub struct AssetsWorkbenchPanel;

impl<'a> WorkbenchPanel<PanelId, EditorPanelContext<'a>> for AssetsWorkbenchPanel {
    fn id(&self) -> PanelId {
        PanelId::Assets
    }

    fn title(&self) -> &str {
        "Assets"
    }

    fn atlas_icon(&self) -> Option<[f32; 4]> {
        Some(crate::ui::iris_bridge::icons::ICON_FOLDER)
    }

    fn closeable(&self) -> bool {
        true
    }

    fn render(&mut self, scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'a>) {
        state
            .overlay
            .build_assets_panel_if_active(scope, state.params);
    }
}

/// Autonomous CPU & GPU Profiler Telemetry Stats workbench panel.
#[derive(Debug, Clone, Copy, Default)]
pub struct StatsWorkbenchPanel;

impl<'a> WorkbenchPanel<PanelId, EditorPanelContext<'a>> for StatsWorkbenchPanel {
    fn id(&self) -> PanelId {
        PanelId::Stats
    }

    fn title(&self) -> &str {
        "Stats"
    }

    fn icon(&self) -> Option<&'static str> {
        Some("📊")
    }

    fn closeable(&self) -> bool {
        true
    }

    fn render(&mut self, scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'a>) {
        state
            .overlay
            .build_stats_panel_if_active(scope, state.params);
    }
}

/// Autonomous Material & Texture Studio workbench panel.
#[derive(Debug, Clone, Copy, Default)]
pub struct MaterialEditorWorkbenchPanel;

impl<'a> WorkbenchPanel<PanelId, EditorPanelContext<'a>> for MaterialEditorWorkbenchPanel {
    fn id(&self) -> PanelId {
        PanelId::MaterialEditor
    }

    fn title(&self) -> &str {
        "Material Editor"
    }

    fn icon(&self) -> Option<&'static str> {
        Some("🌐")
    }

    fn closeable(&self) -> bool {
        true
    }

    fn render(&mut self, scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'a>) {
        state
            .overlay
            .build_material_panel_if_active(scope, state.params);
    }
}

/// Autonomous Skeletal Animation Timeline Studio workbench panel.
#[derive(Debug, Clone, Copy, Default)]
pub struct TimelineWorkbenchPanel;

impl<'a> WorkbenchPanel<PanelId, EditorPanelContext<'a>> for TimelineWorkbenchPanel {
    fn id(&self) -> PanelId {
        PanelId::AnimationTimeline
    }

    fn title(&self) -> &str {
        "Timeline"
    }

    fn icon(&self) -> Option<&'static str> {
        Some("🎬")
    }

    fn closeable(&self) -> bool {
        true
    }

    fn render(&mut self, scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'a>) {
        state
            .overlay
            .build_timeline_panel_if_active(scope, state.params);
    }
}

/// Autonomous 2D UI Canvas Designer workbench panel.
#[derive(Debug, Clone, Copy, Default)]
pub struct UiDesignerWorkbenchPanel;

impl<'a> WorkbenchPanel<PanelId, EditorPanelContext<'a>> for UiDesignerWorkbenchPanel {
    fn id(&self) -> PanelId {
        PanelId::UiDesigner
    }

    fn title(&self) -> &str {
        "UI Designer"
    }

    fn icon(&self) -> Option<&'static str> {
        Some("📐")
    }

    fn closeable(&self) -> bool {
        true
    }

    fn render(&mut self, scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'a>) {
        state
            .overlay
            .build_ui_designer_panel_if_active(scope, state.params);
    }
}

/// Autonomous Global Engine & Editor Preferences workbench panel.
#[derive(Debug, Clone, Copy, Default)]
pub struct PreferencesWorkbenchPanel;

impl<'a> WorkbenchPanel<PanelId, EditorPanelContext<'a>> for PreferencesWorkbenchPanel {
    fn id(&self) -> PanelId {
        PanelId::Preferences
    }

    fn title(&self) -> &str {
        "Preferences"
    }

    fn icon(&self) -> Option<&'static str> {
        Some("⚙️")
    }

    fn closeable(&self) -> bool {
        true
    }

    fn render(&mut self, scope: &mut UiScope<'_>, state: &mut EditorPanelContext<'a>) {
        state
            .overlay
            .build_preferences_panel_if_active(scope, state.params);
    }
}

/// Constructs and populates a [`WorkbenchPanelRegistry`] with all built-in editor workbench panels.
#[must_use]
pub fn create_editor_panel_registry<'a>() -> WorkbenchPanelRegistry<PanelId, EditorPanelContext<'a>>
{
    let mut registry = WorkbenchPanelRegistry::new();
    registry.register(ViewportWorkbenchPanel);
    registry.register(HierarchyWorkbenchPanel);
    registry.register(InspectorWorkbenchPanel);
    registry.register(ConsoleWorkbenchPanel);
    registry.register(AssetsWorkbenchPanel);
    registry.register(StatsWorkbenchPanel);
    registry.register(MaterialEditorWorkbenchPanel);
    registry.register(TimelineWorkbenchPanel);
    registry.register(UiDesignerWorkbenchPanel);
    registry.register(PreferencesWorkbenchPanel);
    registry
}

#[cfg(test)]
#[path = "panels_tests.rs"]
mod tests;