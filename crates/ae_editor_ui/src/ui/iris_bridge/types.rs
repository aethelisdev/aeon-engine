// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Type definitions, actions, and event response structures for the Iris UI editor bridge.

pub use super::assets::{AssetsPanelState, AssetsState};
pub use super::console::{ConsolePanelState, ConsoleState};
pub use super::hierarchy::{HierarchyPanelState, HierarchyState};
pub use super::inspector::{InspectorPanelState, InspectorState};
pub use super::material::{MaterialPanelState, MaterialState};
use super::modals::ModalsOverlayState;
use super::preferences::PreferencesDialogState;
pub use super::stats::{StatsPanelState, StatsState};
pub use super::timeline::{TimelinePanelState, TimelineState};
pub use super::ui_designer::UiDesignerPanelState;
pub use super::viewport_hud::ViewportHudState;
use crate::ui::EngineUiAction;
use irisui::prelude::*;
use irisui::widgets::controls::{FocusState, FocusTarget};

/// Standardized interaction and hit-testing container for Iris UI editor panels.
///
/// Encapsulates the recurring panel-level fields (transient hit-test targets,
/// scroll offset, search filter query, focus state, and dispatched action queue)
/// to eliminate boilerplate bloat across panel definitions.
#[derive(Debug, Clone)]
pub struct PanelInteractionState<TTargets, TAction> {
    /// Cached interaction and hit-testing targets from the latest layout/render pass.
    pub targets: Option<TTargets>,
    /// Encapsulated interactive text input editing state with selection, caret navigation and clipboard.
    pub search_input: irisui::prelude::TextInputState,
    /// Queue of dispatched actions waiting to be consumed by the editor workbench.
    pub actions: Vec<TAction>,
}

impl<TTargets, TAction> Default for PanelInteractionState<TTargets, TAction> {
    fn default() -> Self {
        Self {
            targets: None,
            search_input: irisui::prelude::TextInputState::default(),
            actions: Vec::new(),
        }
    }
}

impl<TTargets, TAction> PanelInteractionState<TTargets, TAction> {
    /// Consumes and returns all pending dispatched actions.
    pub fn take_actions(&mut self) -> Vec<TAction> {
        std::mem::take(&mut self.actions)
    }

    /// Appends a dispatched action to the pending queue.
    #[inline]
    pub fn push_action(&mut self, action: TAction) {
        self.actions.push(action);
    }

    /// Resets transient per-frame interaction targets.
    pub fn clear_targets(&mut self) {
        self.targets = None;
    }

    /// Returns `true` if there are pending actions queued in this panel interaction state.
    ///
    /// Enables O(1) reactive dirty invalidation without cloning or draining the action queue.
    #[inline]
    #[must_use]
    pub fn has_pending_actions(&self) -> bool {
        !self.actions.is_empty()
    }
}

/// Keyboard modifier tracking, active theme preset, and stats refresh timing for the editor overlay.
///
/// Retained layout, dock chrome, and dirty invalidation state are owned directly by
/// `irisui::workbench::Workbench`.
#[derive(Debug, Default, Clone)]
pub struct IrisChromeState {
    /// Whether Shift modifier key is currently held down.
    pub shift_held: bool,
    /// Whether Control modifier key is currently held down.
    pub ctrl_held: bool,
    /// Currently active workspace theme preset (Obsidian Deep or Cold Slate).
    pub theme_preset: super::theme::ThemePreset,
    /// Last instant when the performance stats telemetry text was refreshed in the UI tree.
    pub last_stats_refresh: Option<std::time::Instant>,
    /// Last theme preset synchronized onto `Workbench` to skip per-frame theme/dock-style rebuilding.
    pub last_synced_workbench_theme: Option<super::theme::ThemePreset>,
}

/// Snapshot of overlay-owned lifecycle signals forwarded into `irisui::workbench::lifecycle::LifecycleInputs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayLifecycleSignals {
    /// Whether a top menu bar dropdown is currently active.
    pub is_menu_active: bool,
    /// Whether any modal dialog or preview card is currently active.
    pub is_modal_active: bool,
    /// Whether any anchored pop-up node or hierarchy context menu is currently active.
    pub is_popup_active: bool,
    /// Packed bitfield capturing preferences visibility/tab and hierarchy pop-up state.
    pub overlay_revision: u64,
}

/// Central coordinator governing editor overlay panel states, modals, focus, and actions.
///
/// Rendering, layout solving, retained tree ownership, and GPU text/quad buffers reside on
/// `irisui::workbench::Workbench` and `irisui::IrisSurfaceHost`.
pub struct IrisEditorOverlay {
    /// Creation instant used for smooth sub-second continuous UI animations.
    pub start_time: std::time::Instant,

    // --- Modular Subsystem States ---
    /// Keyboard modifier, active theme preset, and stats refresh timing state.
    pub chrome: IrisChromeState,
    /// Modal dialogs and asset loading splash overlays.
    pub modals: ModalsOverlayState,
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
    /// Central focus manager tracking keyboard input focus across widgets and panels.
    pub focus_manager: FocusManager,
    /// Autonomous focus state machine providing SSOT for panel inputs and outside-click clearing.
    pub focus_state: FocusState<FocusTarget>,
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
        !self.pending_actions.is_empty() || !self.assets.actions.is_empty()
    }

    /// Collects read-only overlay lifecycle signals for [`irisui::workbench::lifecycle::WorkbenchLifecycle`].
    #[must_use]
    pub fn lifecycle_signals(
        &self,
        tree: &UiTree,
        show_preferences: bool,
    ) -> OverlayLifecycleSignals {
        let is_menu_active = tree.active_menu_bar_tag.is_some();
        let is_modal_active = tree.is_any_modal_open()
            || self.assets.dialog_host.is_active()
            || self.assets.preview_modal.is_some()
            || show_preferences;
        let is_popup_active = tree.has_active_popup_nodes()
            || self.hierarchy.is_add_menu_open
            || self.hierarchy.active_context_menu.is_some();
        let overlay_revision = u64::from(show_preferences)
            | (u64::from(self.preferences.tab) << 1)
            | (u64::from(self.hierarchy.is_add_menu_open) << 9)
            | (u64::from(self.hierarchy.active_context_menu.is_some()) << 10);
        OverlayLifecycleSignals {
            is_menu_active,
            is_modal_active,
            is_popup_active,
            overlay_revision,
        }
    }

    /// Assigns keyboard input focus to the specified 64-bit semantic tag directly on the core focus manager.
    ///
    /// Synchronizes both the core [`FocusManager`] and the widget [`FocusState`].
    #[inline]
    pub fn set_focus_tag(&mut self, tag: u64) {
        self.focus_manager.set_focus_tag(tag);
        self.focus_state.sync_from_core(&self.focus_manager);
    }

    /// Automatically synchronizes keyboard focus with the active text editing state of `tree`.
    ///
    /// Drops focus from both the core [`FocusManager`] and widget [`FocusState`] when text editing concludes.
    /// Returns `true` if active focus was cleared during synchronization.
    #[inline]
    pub fn auto_sync_focus(&mut self, tree: &UiTree) -> bool {
        self.focus_state
            .auto_sync_with_focus_manager(&mut self.focus_manager, tree)
    }

    /// Initializes a new [`IrisEditorOverlay`] with default editor panel and dialog states.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            chrome: IrisChromeState::default(),
            modals: ModalsOverlayState::default(),
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
            focus_manager: FocusManager::new(),
            focus_state: FocusState::new(),
            pending_actions: Vec::new(),
            export_modal: crate::ui::types::ExportDialogState::default(),
        }
    }
}

pub use super::params::*;