// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Persistent Editor State and Session Context for the UI Designer.
//!

use crate::types::{CanvasAspectRatio, UiDesignerAction, UiDragState};

/// Persistent editor state for the UI Designer canvas panel.
#[derive(Debug, Clone)]
pub struct UiDesignerState {
    /// Active target resolution aspect ratio preset.
    pub aspect_ratio: CanvasAspectRatio,
    /// Viewport zoom scale factor (default = 1.0).
    pub zoom: f32,
    /// 2D pan offset in pixels.
    pub pan_offset: [f32; 2],
    /// Grid snapping interval in pixels (None = free placement).
    pub snap_grid: Option<f32>,
    /// Whether visual anchor pins and distance guidelines are visible.
    pub show_anchor_guides: bool,
    /// Whether the background pixel grid is drawn on the virtual canvas.
    pub show_grid: bool,
    /// Active element dragging state.
    pub drag_state: Option<UiDragState>,
}

impl Default for UiDesignerState {
    fn default() -> Self {
        Self {
            aspect_ratio: CanvasAspectRatio::Ratio16x9,
            zoom: 1.0,
            pan_offset: [0.0, 0.0],
            snap_grid: Some(8.0),
            show_anchor_guides: true,
            show_grid: true,
            drag_state: None,
        }
    }
}

/// Context parameters passed into the UI Designer panel renderer.
pub struct UiDesignerContext<'a> {
    /// Read-only reference to the active ECS world.
    pub world: &'a hecs::World,
    /// Currently selected entity in the editor hierarchy/scene.
    pub selected_entity: Option<hecs::Entity>,
    /// Output buffer of UI Designer actions emitted during the frame.
    pub actions: &'a mut Vec<UiDesignerAction>,
    /// Persistent UI Designer state.
    pub state: &'a mut UiDesignerState,
}