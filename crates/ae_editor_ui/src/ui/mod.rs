// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor UI Subsystem
//!
//! Root module orchestrating editor panels, docking layouts, Iris UI overlays,
//! themes, modal dialogs, and the core `workbench` runtime.

pub mod iris_bridge;
pub mod panel_layout;
pub mod panels;
pub mod workbench;

pub(crate) mod docking;
pub mod menubar;
pub(crate) mod style;
pub mod types;
pub(crate) mod viewport_hud;

// Re-exports for  engine consumption
pub use iris_bridge::IrisEditorOverlay;
pub use iris_bridge::hierarchy::{HierarchyAction, HierarchyPanelParams, HierarchyRow};
pub use menubar::*;
pub use panel_layout::{PanelId, PanelLayoutState};
pub use types::{ConsoleEntry, EngineUiAction, UiElementType};
pub use workbench::{EditorUiRenderParams, EngineUi, SceneDialogAction};