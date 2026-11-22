// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Bridge (`iris_bridge::ui_designer`)
//!
//! Renders the interactive virtual canvas HUD & UI Designer studio using
//! `irisui_widgets::PanZoomViewport` and direct `ActionBus<EngineUiAction>` dispatch.

pub mod canvas;
pub mod panel;
pub mod popups;
#[cfg(test)]
mod tests;
pub mod toolbar;
pub mod types;

pub use panel::build_ui_designer_panel;
pub use types::{
    CanvasAspectRatio, UI_DESIGNER_TAG_ADD_ELEMENT_BTN, UI_DESIGNER_TAG_ANCHORS_BTN,
    UI_DESIGNER_TAG_ASPECT_BTN, UI_DESIGNER_TAG_CANVAS_BOARD, UI_DESIGNER_TAG_GRID_BTN,
    UI_DESIGNER_TAG_LETTERBOX, UI_DESIGNER_TAG_PANEL_ROOT, UI_DESIGNER_TAG_SNAP_BTN,
    UI_DESIGNER_TAG_TOOLBAR, UI_DESIGNER_TAG_ZOOM_IN, UI_DESIGNER_TAG_ZOOM_OUT,
    UI_DESIGNER_TAG_ZOOM_RESET, UiDesignerPanelState, UiDesignerState, UiDragState, UiElementType,
    encode_element_tag,
};