// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Aeon UI Designer (AUD) - 2D Visual Canvas, Layout, & HUD Studio
//!
//! Provides a dedicated, resolution-independent WYSIWYG 2D canvas editor for designing
//! in-game HUDs, health bars, interactive buttons, menus, and typography layouts with
//! visual anchor guides, grid snapping, and interactive drag-and-drop.
//!

pub mod anchors;
pub mod canvas;
pub mod palette;
pub mod spawning;
pub mod state;
#[cfg(test)]
mod tests;
pub mod types;

pub use anchors::draw_anchor_pin_and_guide;
pub use canvas::{draw_canvas_area, draw_canvas_grid};
pub use palette::{draw_designer_toolbar, dropdown_item};
pub use spawning::spawn_ui_element;
pub use state::{UiDesignerContext, UiDesignerState};
pub use types::{CanvasAspectRatio, UiDesignerAction, UiDragState, UiElementType};

/// Renders the complete 2D UI Designer panel frame, top toolbar, virtual canvas, and interactive widget overlays.
pub fn draw_ui_designer_panel(ui: &mut egui::Ui, ctx: &mut UiDesignerContext<'_>) {
    // 1. Top Toolbar (Aspect Ratio, Zoom, Grid Snap, Anchor Guides, ➕ Add Element Palette)
    draw_designer_toolbar(ui, ctx);

    ui.separator();

    // 2. Interactive 2D Virtual Canvas Area
    draw_canvas_area(ui, ctx);
}