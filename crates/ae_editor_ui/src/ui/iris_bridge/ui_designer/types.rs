// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Types & Semantic Tags
//!
//! Defines deterministic `UiTag` constants, panel view parameters, and persistent
//! state wrapper for the native Iris UI `PanZoomViewport`-backed designer panel.

pub use ae_uidesign::{CanvasAspectRatio, UiDesignerState, UiDragState, UiElementType};
use irisui::prelude::*;

/// Tag for the UI Designer root panel container.
pub const UI_DESIGNER_TAG_PANEL_ROOT: u64 = UiTag::from_str("ui_designer.panel_root").raw();
/// Tag for the letterbox outer pan/zoom viewport container.
pub const UI_DESIGNER_TAG_LETTERBOX: u64 = UiTag::from_str("ui_designer.letterbox").raw();
/// Tag for the scaled virtual canvas board.
pub const UI_DESIGNER_TAG_CANVAS_BOARD: u64 = UiTag::from_str("ui_designer.canvas_board").raw();
/// Tag for the 34px top toolbar container.
pub const UI_DESIGNER_TAG_TOOLBAR: u64 = UiTag::from_str("ui_designer.toolbar").raw();

/// Tag for Aspect Ratio dropdown trigger button.
pub const UI_DESIGNER_TAG_ASPECT_BTN: u64 = UiTag::from_str("ui_designer.toolbar.aspect").raw();
/// Tag for Zoom Out button.
pub const UI_DESIGNER_TAG_ZOOM_OUT: u64 = UiTag::from_str("ui_designer.toolbar.zoom_out").raw();
/// Tag for Zoom Reset button.
pub const UI_DESIGNER_TAG_ZOOM_RESET: u64 = UiTag::from_str("ui_designer.toolbar.zoom_reset").raw();
/// Tag for Zoom In button.
pub const UI_DESIGNER_TAG_ZOOM_IN: u64 = UiTag::from_str("ui_designer.toolbar.zoom_in").raw();
/// Tag for Grid Snap cycle button.
pub const UI_DESIGNER_TAG_SNAP_BTN: u64 = UiTag::from_str("ui_designer.toolbar.snap").raw();
/// Tag for Anchor Guide lines toggle button.
pub const UI_DESIGNER_TAG_ANCHORS_BTN: u64 = UiTag::from_str("ui_designer.toolbar.anchors").raw();
/// Tag for visual Grid toggle button.
pub const UI_DESIGNER_TAG_GRID_BTN: u64 = UiTag::from_str("ui_designer.toolbar.grid").raw();
/// Tag for Add Element palette button.
pub const UI_DESIGNER_TAG_ADD_ELEMENT_BTN: u64 =
    UiTag::from_str("ui_designer.toolbar.add_element").raw();

const UI_DESIGNER_TAG_ASPECT_ITEM_BASE: UiTag = UiTag::from_str("ui_designer.aspect_item");
const UI_DESIGNER_TAG_ADD_ITEM_BASE: UiTag = UiTag::from_str("ui_designer.add_item");
const UI_DESIGNER_TAG_ELEMENT_BASE: UiTag = UiTag::from_str("ui_designer.elem");

/// Constructs a semantic tag for an aspect ratio preset item.
#[inline]
#[must_use]
pub const fn make_aspect_item_tag(idx: usize) -> u64 {
    UI_DESIGNER_TAG_ASPECT_ITEM_BASE.indexed(idx as u64).raw()
}

/// Constructs a semantic tag for an add-element palette item.
#[inline]
#[must_use]
pub const fn make_add_item_tag(idx: usize) -> u64 {
    UI_DESIGNER_TAG_ADD_ITEM_BASE.indexed(idx as u64).raw()
}

/// Encodes an ECS entity handle into a stable 64-bit semantic element tag.
#[inline]
#[must_use]
pub const fn encode_element_tag(entity: hecs::Entity) -> u64 {
    UI_DESIGNER_TAG_ELEMENT_BASE
        .indexed(entity.to_bits().get())
        .raw()
}

/// Persistent panel cache storing the most recently resolved pan/zoom transformation.
#[derive(Debug, Default, Clone)]
pub struct UiDesignerPanelState {
    /// Latest geometric transform computed by [`PanZoomViewport`](irisui::prelude::PanZoomTransform).
    pub last_transform: PanZoomTransform,
}