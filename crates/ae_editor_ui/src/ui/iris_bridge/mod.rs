// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Iris UI Hybrid Bridge for the Aeon Engine Editor.
//!
//! Manages the retained-mode `UiTree`, `IrisRenderer`, typography, interactive hover/click
//! event routing, and `MenuBarBuilder`/`DropdownMenuBuilder` rendering directly on top of the editor frame.

pub mod about;
pub mod actions;
pub mod assets;
pub mod console;
pub mod events;
pub mod hierarchy;
pub mod icons;
pub mod inspector;
pub mod menubar;
pub mod modals;
pub mod preferences;
pub mod render;
pub mod stats;
pub mod status_bar;
pub mod types;
pub mod update;
pub mod viewport_hud;

pub use about::{AboutDialogTargets, build_about_dialog};
pub use assets::{
    AssetClickTracker, AssetsPanelAction, AssetsPanelParams, AssetsPanelTargets,
    build_assets_panel, handle_assets_panel_event,
};
pub use console::{
    ConsoleAction, ConsoleFilterLevel, ConsolePanelParams, ConsolePanelTargets, build_console_panel,
};
pub use hierarchy::{
    AddSubmenuId, HierarchyAction, HierarchyPanelParams, HierarchyPanelTargets, HierarchyRow,
    build_hierarchy_panel, handle_hierarchy_click, handle_hierarchy_hover,
};
pub use icons::*;
pub use modals::*;
pub use preferences::{
    PreferencesAction, PreferencesDropdownId, PreferencesParams, PreferencesSliderId,
    PreferencesTargets, PreferencesToggleId, build_preferences_dialog,
};
pub use stats::{
    StatsPanelAction, StatsPanelNodes, StatsPanelParams, StatsPanelTargets, build_stats_panel,
};
pub use types::{
    ActiveMenu, DropdownAction, IrisEditorOverlay, IrisOverlayEventResult, OverlayUpdateParams,
};
pub use viewport_hud::{
    ViewportHudAction, ViewportHudDropdownId, ViewportHudParams, ViewportHudTargets,
    build_viewport_hud,
};