// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Aeon UI Designer (AUD) - 2D Visual Canvas, Layout, & HUD Studio
//!
//! Provides a dedicated, resolution-independent WYSIWYG 2D canvas editor for designing
//! in-game HUDs, health bars, interactive buttons, menus, and typography layouts with
//! visual anchor guides, grid snapping, and interactive drag-and-drop.
//!

pub mod spawning;
pub mod state;
#[cfg(test)]
mod tests;
pub mod types;

pub use spawning::spawn_ui_element;
pub use state::{UiDesignerContext, UiDesignerState};
pub use types::{CanvasAspectRatio, UiDesignerAction, UiDragState, UiElementType};