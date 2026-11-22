// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor UI Subsystem
//!
//! Root module orchestrating editor panels, docking layouts, Iris UI overlays,
//! themes, modal dialogs, and the [`EngineUi`] consumer facade.

pub mod editor_ui;
pub mod icons;
pub mod iris_bridge;
pub mod panel_layout;
pub mod panel_registry;
pub mod types;

// Re-exports for  engine consumption
pub use editor_ui::EngineUi;
pub use iris_bridge::hierarchy::HierarchyRow;
pub use panel_layout::PanelId;
pub use types::{
    ConsoleEntry, EditorTelemetrySnapshot, EditorUiRenderParams, EngineUiAction, UiElementType,
};