// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Iris UI Hybrid Bridge for the Aeon Engine Editor.
//!
//! Manages the retained-mode `UiTree`, `IrisRenderer`, typography, interactive hover/click
//! event routing, and `MenuBarBuilder`/`DropdownMenuBuilder` rendering directly on top of the editor frame.

pub mod assets;
pub mod console;
pub mod export_modal;
pub mod hierarchy;
pub use crate::ui::icons;
pub mod inspector;
pub mod material;
pub mod menubar;
pub mod preferences;
pub mod stats;
pub mod status_bar;
pub mod theme;
pub mod timeline;
pub mod ui_designer;
pub mod viewport_hud;
pub mod viewport_texture;

#[cfg(test)]
mod tests_update;

pub use viewport_texture::VIEWPORT_TEXTURE_ID;

pub use assets::{AssetClickTracker, AssetsPanelAction, build_assets_panel};
pub use console::{ConsoleFilterLevel, ConsolePanelState, build_console_panel};
pub use hierarchy::{AddSubmenuId, HierarchyRow, build_hierarchy_panel};
pub use icons::*;
pub use material::build_material_panel;
pub use preferences::{PreferencesDialogState, build_preferences_view};
pub use stats::{StatsPanelState, build_stats_panel};
pub use theme::*;
pub use timeline::build_timeline_panel;
pub use ui_designer::{
    CanvasAspectRatio, UiDesignerState, UiDragState, UiElementType, build_ui_designer_panel,
};
pub use viewport_hud::{ViewportHudState, build_viewport_hud_scope};