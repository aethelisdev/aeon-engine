// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer & HUD Studio Native Iris Bridge
//!
//! Provides the complete 100% native Iris UI GPU SDF implementation of the
//! 2D in-game UI Designer and interactive WYSIWYG canvas studio.
//!

pub mod anchors;
pub mod canvas;
pub mod events;
pub mod panel;
pub mod popups;
#[cfg(test)]
mod tests;
pub mod toolbar;
pub mod types;

pub use events::{
    UiDesignerClickResult, handle_ui_designer_click, handle_ui_designer_drag,
    handle_ui_designer_scroll,
};
pub use panel::build_ui_designer_panel;
pub use toolbar::UI_DESIGNER_TOOLBAR_HEIGHT;
pub use types::{
    CanvasAspectRatio, UiDesignerAction, UiDesignerPanelParams, UiDesignerPanelTargets,
    UiDesignerState, UiDragState, UiElementType,
};