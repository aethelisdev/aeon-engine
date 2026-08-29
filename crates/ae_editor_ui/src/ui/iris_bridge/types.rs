// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Type definitions, actions, and event response structures for the Iris UI editor bridge.

use crate::ui::EngineUiAction;
use crate::ui::panel_layout::PanelId;

/// Top menu bar categories for active open dropdown menus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveMenu {
    /// File operations (New, Load, Save, Save As, Exit).
    File,
    /// Edit actions (Undo, Redo, Preferences).
    Edit,
    /// View layout and tool panel visibility toggles.
    View,
    /// Tool windows and workspace resets.
    Window,
    /// Documentation, engine information, and shortcuts.
    Help,
}

/// Action payload dispatched from clicking a dropdown menu item.
#[derive(Debug, Clone)]
pub enum DropdownAction {
    /// Dispatches an event-bus UI action.
    UiAction(EngineUiAction),
    /// Toggles tool panel visibility.
    TogglePanel(PanelId),
    /// Resets docking layout to default preset.
    ResetLayout,
    /// Opens preferences modal dialog.
    OpenPreferences,
    /// Opens about engine modal dialog.
    OpenAbout,
}

/// Event handling response payload returned from `IrisEditorOverlay::handle_event`.
#[derive(Debug, Default, Clone)]
pub struct IrisOverlayEventResult {
    /// Whether the event was intercepted and consumed by the Iris UI overlay.
    pub consumed: bool,
    /// UI action payload to enqueue.
    pub ui_action: Option<EngineUiAction>,
    /// Panel toggle request.
    pub toggle_panel: Option<PanelId>,
    /// Reset layout request.
    pub reset_layout: bool,
    /// Open preferences dialog request.
    pub open_preferences: bool,
    /// Open about dialog request.
    pub open_about: bool,
}