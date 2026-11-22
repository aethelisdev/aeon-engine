// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Configuration for `PanZoomViewport`
//!
//! Defines [`PanZoomSpec`] encapsulating viewport semantic tags, virtual reference bounds,
//! two-way bound zoom/pan state, grid appearance, and letterbox styling.

use irisui_core::{Color, Size};

/// Configuration specification for emitting a [`PanZoomViewport`](super::viewport::PanZoomViewportResponse) via [`UiScope::pan_zoom_viewport`](crate::declarative::scope::UiScope::pan_zoom_viewport).
///
/// Binds `zoom` and `pan_offset` mutably so scroll-wheel zooming and middle-mouse panning
/// update state autonomously before child virtual geometry is evaluated.
pub struct PanZoomSpec<'a> {
    /// Diagnostic node name assigned to the outer letterbox container.
    pub name: &'static str,
    /// Diagnostic node name assigned to the inner scaled virtual canvas container.
    pub canvas_name: &'static str,
    /// Semantic 64-bit tag for the outer letterbox viewport container (handles wheel & middle pan).
    pub tag: u64,
    /// Semantic 64-bit tag for the inner virtual canvas surface.
    pub canvas_tag: u64,
    /// Virtual coordinate space dimensions `[width, height]` fitted inside the letterbox.
    pub virtual_size: Size,
    /// Minimum letterbox margin surrounding the fitted virtual surface at `zoom == 1.0`.
    pub margin: f32,
    /// Two-way bound viewport zoom multiplier (e.g. `1.0` = 100%).
    pub zoom: &'a mut f32,
    /// Two-way bound `[x, y]` screen-space pan offset in logical pixels.
    pub pan_offset: &'a mut [f32; 2],
    /// Minimum allowed zoom factor for wheel zoom clamping.
    pub min_zoom: f32,
    /// Maximum allowed zoom factor for wheel zoom clamping.
    pub max_zoom: f32,
    /// Whether the background alignment grid is rendered inside the virtual surface.
    pub show_grid: bool,
    /// Virtual pixel spacing between minor alignment grid lines (default `64.0`).
    pub grid_step_virtual: f32,
    /// Interval of grid lines rendered with major grid color (default every `4` lines).
    pub grid_major_interval: usize,
    /// Stroke color for minor background grid lines.
    pub grid_minor_color: Color,
    /// Stroke color for major background grid lines.
    pub grid_major_color: Color,
    /// Whether to display the bottom-left virtual resolution badge inside the canvas.
    pub show_dimensions_badge: bool,
    /// Outer letterbox viewport background color.
    pub letterbox_color: Color,
    /// Inner virtual canvas surface background fill color.
    pub canvas_background: Color,
    /// Border outline color of the virtual canvas surface.
    pub canvas_border_color: Color,
    /// Border thickness in logical pixels around the virtual canvas surface.
    pub canvas_border_width: f32,
}

impl<'a> PanZoomSpec<'a> {
    /// Creates a new [`PanZoomSpec`] with dark editor styling and safe defaults.
    #[must_use]
    pub fn new(
        tag: u64,
        canvas_tag: u64,
        virtual_size: Size,
        zoom: &'a mut f32,
        pan_offset: &'a mut [f32; 2],
    ) -> Self {
        Self {
            name: "PanZoomViewport",
            canvas_name: "PanZoomCanvasSurface",
            tag,
            canvas_tag,
            virtual_size,
            margin: 32.0,
            zoom,
            pan_offset,
            min_zoom: 0.2,
            max_zoom: 5.0,
            show_grid: true,
            grid_step_virtual: 64.0,
            grid_major_interval: 4,
            grid_minor_color: Color::rgba(1.0, 1.0, 1.0, 0.035),
            grid_major_color: Color::rgba(1.0, 1.0, 1.0, 0.080),
            show_dimensions_badge: true,
            letterbox_color: Color::rgba(0.055, 0.063, 0.086, 1.0),
            canvas_background: Color::rgba(0.086, 0.102, 0.137, 1.0),
            canvas_border_color: Color::rgba(0.0, 0.706, 0.941, 0.85),
            canvas_border_width: 1.5,
        }
    }

    /// Overrides the debug names assigned to the outer letterbox and inner canvas nodes.
    #[inline]
    #[must_use]
    pub const fn with_names(mut self, name: &'static str, canvas_name: &'static str) -> Self {
        self.name = name;
        self.canvas_name = canvas_name;
        self
    }

    /// Sets the letterbox padding margin in logical pixels around the virtual canvas at `zoom = 1.0`.
    #[inline]
    #[must_use]
    pub const fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    /// Configures `[min_zoom, max_zoom]` clamp limits for mouse wheel zoom interaction.
    #[inline]
    #[must_use]
    pub const fn zoom_bounds(mut self, min_zoom: f32, max_zoom: f32) -> Self {
        self.min_zoom = min_zoom;
        self.max_zoom = max_zoom;
        self
    }

    /// Controls visibility and virtual spacing of the background grid canvas.
    #[inline]
    #[must_use]
    pub const fn grid(mut self, show: bool, step_virtual: f32) -> Self {
        self.show_grid = show;
        self.grid_step_virtual = step_virtual;
        self
    }

    /// Controls whether the virtual resolution badge (`"W × H"`) is displayed inside the canvas.
    #[inline]
    #[must_use]
    pub const fn show_dimensions_badge(mut self, show: bool) -> Self {
        self.show_dimensions_badge = show;
        self
    }
}