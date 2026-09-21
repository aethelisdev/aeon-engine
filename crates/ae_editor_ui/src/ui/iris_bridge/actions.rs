// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Action extraction and queue draining helpers for Iris UI editor overlays.

use super::hierarchy::HierarchyAction;
use super::stats::StatsPanelAction;
use super::types::IrisEditorOverlay;
use super::viewport_hud::ViewportHudAction;

impl IrisEditorOverlay {
    /// Consumes and returns all queued Viewport HUD actions.
    pub fn take_viewport_hud_actions(&mut self) -> Vec<ViewportHudAction> {
        self.viewport_hud.take_actions()
    }

    /// Consumes and returns all queued Stats & Profiler panel actions.
    pub fn take_stats_actions(&mut self) -> Vec<StatsPanelAction> {
        self.stats.take_actions()
    }

    /// Consumes and returns all queued Scene Hierarchy panel actions.
    pub fn take_hierarchy_actions(&mut self) -> Vec<HierarchyAction> {
        self.hierarchy.take_actions()
    }

    /// Consumes and returns all queued Scene Inspector panel actions.
    pub fn take_inspector_actions(&mut self) -> Vec<super::inspector::InspectorAction> {
        self.inspector.take_actions()
    }

    /// Consumes and returns all queued Content / Asset Browser panel actions.
    pub fn take_assets_actions(&mut self) -> Vec<super::assets::AssetsPanelAction> {
        self.assets.take_actions()
    }

    /// Consumes and returns all queued Animation Timeline Studio panel actions.
    pub fn take_timeline_actions(&mut self) -> Vec<super::timeline::TimelineAction> {
        self.timeline.take_actions()
    }

    /// Consumes and returns all queued Material & Surface Studio panel actions.
    pub fn take_material_actions(&mut self) -> Vec<super::material::MaterialAction> {
        self.material.take_actions()
    }

    /// Consumes and returns all queued 2D Visual UI Designer panel actions.
    pub fn take_ui_designer_actions(&mut self) -> Vec<super::ui_designer::UiDesignerAction> {
        self.ui_designer.take_actions()
    }
}