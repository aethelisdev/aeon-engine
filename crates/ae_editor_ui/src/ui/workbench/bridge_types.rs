// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Type definitions, actions, and event response structures for the Iris UI editor bridge.

use crate::ui::EngineUiAction;
pub use crate::ui::iris_bridge::assets::{AssetsPanelState, AssetsState};
pub use crate::ui::iris_bridge::console::{ConsolePanelState, ConsoleState};
pub use crate::ui::iris_bridge::hierarchy::{HierarchyPanelState, HierarchyState};
pub use crate::ui::iris_bridge::inspector::{InspectorPanelState, InspectorState};
pub use crate::ui::iris_bridge::material::{MaterialPanelState, MaterialState};
use crate::ui::iris_bridge::preferences::PreferencesDialogState;
pub use crate::ui::iris_bridge::stats::{StatsPanelState, StatsState};
pub use crate::ui::iris_bridge::timeline::{TimelinePanelState, TimelineState};
pub use crate::ui::iris_bridge::ui_designer::UiDesignerPanelState;
pub use crate::ui::iris_bridge::viewport_hud::ViewportHudState;

/// Central coordinator governing editor overlay panel states, modals, focus, and actions.
///
/// Rendering, layout solving, retained tree ownership, and GPU text/quad buffers reside on
/// `irisui::workbench::Workbench` and `irisui::IrisSurfaceHost`.
pub struct IrisEditorOverlay {
    /// Creation instant used for smooth sub-second continuous UI animations.
    pub start_time: std::time::Instant,

    // --- Modular Subsystem States ---
    /// Preferences modal dialog interactive state.
    pub preferences: PreferencesDialogState,
    /// 3D Viewport HUD controls and dropdown state.
    pub viewport_hud: ViewportHudState,
    /// Performance Stats & Telemetry profiler panel state.
    pub stats: StatsPanelState,
    /// Scene Hierarchy tree panel state.
    pub hierarchy: HierarchyPanelState,
    /// Developer Console and log filter panel state.
    pub console: ConsolePanelState,
    /// Content / Asset Browser panel state.
    pub assets: AssetsPanelState,
    /// Animation Timeline Studio panel state.
    pub timeline: TimelinePanelState,
    /// Material & Surface Studio panel state.
    pub material: MaterialPanelState,
    /// 2D Visual UI Designer panel state.
    pub ui_designer: UiDesignerPanelState,
    /// Scene Inspector component editor panel state.
    pub inspector: InspectorPanelState,
    /// Centralized queue of pending engine UI actions emitted by panels and overlays.
    pub pending_actions: Vec<EngineUiAction>,
    /// Declarative state for the `File -> Export Project...` modal dialog.
    pub export_modal: crate::ui::types::ExportDialogState,
}

impl Default for IrisEditorOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl IrisEditorOverlay {
    /// Returns `true` if there are pending UI actions queued in the central action queue.
    #[inline]
    #[must_use]
    pub fn has_pending_actions(&self) -> bool {
        !self.pending_actions.is_empty()
    }

    /// Initializes a new [`IrisEditorOverlay`] with default editor panel and dialog states.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            preferences: PreferencesDialogState::default(),
            viewport_hud: ViewportHudState::default(),
            stats: StatsPanelState::default(),
            hierarchy: HierarchyPanelState::default(),
            console: ConsolePanelState::default(),
            assets: AssetsPanelState::default(),
            timeline: TimelinePanelState::default(),
            material: MaterialPanelState::default(),
            ui_designer: UiDesignerPanelState::default(),
            inspector: InspectorPanelState::default(),
            pending_actions: Vec::new(),
            export_modal: crate::ui::types::ExportDialogState::default(),
        }
    }
}

pub use super::bridge_params::*;