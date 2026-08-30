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
}