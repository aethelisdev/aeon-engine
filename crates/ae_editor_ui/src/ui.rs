// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor UI Subsystem
//!
//! Root module orchestrating editor panels, docking layouts, Iris UI overlays,
//! themes, modal dialogs, and the core `workbench` runtime.

pub mod iris_bridge;
pub mod panel_layout;
pub mod types;
pub mod workbench;

// Re-exports for  engine consumption
pub use iris_bridge::hierarchy::{HierarchyPanelParams, HierarchyRow};
pub use panel_layout::PanelId;
pub use types::{ConsoleEntry, EditorTelemetrySnapshot, EngineUiAction, UiElementType};
pub use workbench::{EditorUiRenderParams, EngineUi, SceneDialogAction};