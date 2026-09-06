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
        std::mem::take(&mut self.viewport_hud_actions)
    }

    /// Consumes and returns all queued Stats & Profiler panel actions.
    pub fn take_stats_actions(&mut self) -> Vec<StatsPanelAction> {
        std::mem::take(&mut self.stats_actions)
    }

    /// Consumes and returns all queued Scene Hierarchy panel actions.
    pub fn take_hierarchy_actions(&mut self) -> Vec<HierarchyAction> {
        std::mem::take(&mut self.hierarchy_actions)
    }

    /// Consumes and returns all queued Scene Inspector panel actions.
    pub fn take_inspector_actions(&mut self) -> Vec<super::inspector::InspectorAction> {
        std::mem::take(&mut self.inspector_actions)
    }

    /// Consumes and returns all queued Content / Asset Browser panel actions.
    pub fn take_assets_actions(&mut self) -> Vec<super::assets::AssetsPanelAction> {
        std::mem::take(&mut self.assets_actions)
    }

    /// Consumes and returns all queued Animation Timeline Studio panel actions.
    pub fn take_timeline_actions(&mut self) -> Vec<super::timeline::TimelineAction> {
        std::mem::take(&mut self.timeline_actions)
    }

    /// Consumes and returns all queued Material & Surface Studio panel actions.
    pub fn take_material_actions(&mut self) -> Vec<super::material::MaterialAction> {
        std::mem::take(&mut self.material_actions)
    }

    /// Consumes and returns all queued 2D Visual UI Designer panel actions.
    pub fn take_ui_designer_actions(&mut self) -> Vec<super::ui_designer::UiDesignerAction> {
        std::mem::take(&mut self.ui_designer_actions)
    }
}