// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Pan & Zoom Viewport Coordinate Math
//!
//! Computes aspect-fitted letterbox metrics, virtual-to-local layout coordinates,
//! and screen-to-virtual pointer projection with zero-division protection.

use irisui_core::{Point, Rect, Size};

/// Snaps `value` to the nearest multiple of `interval` when snapping is active and positive.
#[inline]
#[must_use]
pub fn snap_scalar(value: f32, interval: Option<f32>) -> f32 {
    match interval {
        Some(step) if step > 0.001 && step.is_finite() => (value / step).round() * step,
        _ => value,
    }
}

/// Computed geometric transformation between screen pixels, local canvas pixels, and virtual space.
///
/// All coordinate conversions guard against zero-sized viewports or degenerate virtual bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanZoomTransform {
    /// Resolved screen-space bounding rectangle of the outer letterbox viewport container.
    pub viewport_rect: Rect,
    /// Resolved screen-space bounding rectangle of the scaled & panned virtual canvas surface.
    pub canvas_rect: Rect,
    /// Local bounding rectangle of the virtual canvas relative to `viewport_rect` origin.
    pub canvas_local_rect: Rect,
    /// Reference virtual canvas dimensions `(width, height)`.
    pub virtual_size: Size,
    /// Combined pixel scale factor (`fit_scale * zoom`) mapping 1 virtual unit to screen pixels.
    pub base_scale: f32,
    /// Aspect-fit scale factor at `zoom == 1.0`.
    pub fit_scale: f32,
    /// Active clamped zoom multiplier.
    pub zoom: f32,
}

impl Default for PanZoomTransform {
    fn default() -> Self {
        Self {
            viewport_rect: Rect::ZERO,
            canvas_rect: Rect::ZERO,
            canvas_local_rect: Rect::ZERO,
            virtual_size: Size::new(1920.0, 1080.0),
            base_scale: 1.0,
            fit_scale: 1.0,
            zoom: 1.0,
        }
    }
}

impl PanZoomTransform {
    /// Computes aspect-fitted canvas placement and scale factors within `viewport_rect`.
    ///
    /// Guards against zero/negative bounds, non-finite zoom multipliers, and division by zero.
    #[must_use]
    pub fn compute(
        viewport_rect: Rect,
        virtual_size: Size,
        margin: f32,
        zoom: f32,
        pan_offset: [f32; 2],
    ) -> Self {
        let avail_w = viewport_rect.width.max(100.0);
        let avail_h = viewport_rect.height.max(100.0);
        let safe_margin = if margin.is_finite() {
            margin.max(0.0)
        } else {
            0.0
        };

        let max_w = (avail_w - safe_margin * 2.0).max(100.0);
        let max_h = (avail_h - safe_margin * 2.0).max(100.0);

        let vw = if virtual_size.width.is_finite() {
            virtual_size.width.max(1.0)
        } else {
            1920.0
        };
        let vh = if virtual_size.height.is_finite() {
            virtual_size.height.max(1.0)
        } else {
            1080.0
        };

        let fit_scale = (max_w / vw).min(max_h / vh).max(0.001);
        let safe_zoom = if zoom.is_finite() {
            zoom.max(0.01)
        } else {
            1.0
        };
        let base_scale = (fit_scale * safe_zoom).max(0.001);

        let canvas_w = vw * base_scale;
        let canvas_h = vh * base_scale;
        let pan_x = if pan_offset[0].is_finite() {
            pan_offset[0]
        } else {
            0.0
        };
        let pan_y = if pan_offset[1].is_finite() {
            pan_offset[1]
        } else {
            0.0
        };

        let local_x = (avail_w - canvas_w) * 0.5 + pan_x;
        let local_y = (avail_h - canvas_h) * 0.5 + pan_y;

        let canvas_local_rect = Rect::new(local_x, local_y, canvas_w, canvas_h);
        let canvas_rect = Rect::new(
            viewport_rect.x + local_x,
            viewport_rect.y + local_y,
            canvas_w,
            canvas_h,
        );

        Self {
            viewport_rect: Rect::new(viewport_rect.x, viewport_rect.y, avail_w, avail_h),
            canvas_rect,
            canvas_local_rect,
            virtual_size: Size::new(vw, vh),
            base_scale,
            fit_scale,
            zoom: safe_zoom,
        }
    }

    /// Converts a point in virtual canvas coordinates into local pixel offsets inside the canvas node.
    #[inline]
    #[must_use]
    pub fn virtual_to_local(&self, virtual_pt: Point) -> Point {
        if self.virtual_size.width <= 0.0 || self.virtual_size.height <= 0.0 {
            return Point::ZERO;
        }
        Point::new(
            (virtual_pt.x / self.virtual_size.width) * self.canvas_rect.width,
            (virtual_pt.y / self.virtual_size.height) * self.canvas_rect.height,
        )
    }

    /// Converts a virtual bounding box `[min_x, min_y, max_x, max_y]` into a local pixel [`Rect`].
    #[inline]
    #[must_use]
    pub fn virtual_bounds_to_local(&self, bounds: [f32; 4]) -> Rect {
        let min_p = self.virtual_to_local(Point::new(bounds[0], bounds[1]));
        let max_p = self.virtual_to_local(Point::new(bounds[2], bounds[3]));
        Rect::new(
            min_p.x,
            min_p.y,
            (max_p.x - min_p.x).max(0.0),
            (max_p.y - min_p.y).max(0.0),
        )
    }

    /// Converts a virtual canvas point into absolute screen coordinates.
    #[inline]
    #[must_use]
    pub fn virtual_to_screen(&self, virtual_pt: Point) -> Point {
        let local = self.virtual_to_local(virtual_pt);
        Point::new(self.canvas_rect.x + local.x, self.canvas_rect.y + local.y)
    }

    /// Converts an absolute screen coordinate into virtual canvas coordinates.
    ///
    /// Returns [`Point::ZERO`] if the canvas surface currently has zero width or height.
    #[inline]
    #[must_use]
    pub fn screen_to_virtual(&self, screen_pt: Point) -> Point {
        if self.canvas_rect.width <= 0.0 || self.canvas_rect.height <= 0.0 {
            return Point::ZERO;
        }
        let rel_x = (screen_pt.x - self.canvas_rect.x) / self.canvas_rect.width;
        let rel_y = (screen_pt.y - self.canvas_rect.y) / self.canvas_rect.height;
        Point::new(
            rel_x * self.virtual_size.width,
            rel_y * self.virtual_size.height,
        )
    }

    /// Converts a screen-space displacement delta into virtual canvas units.
    #[inline]
    #[must_use]
    pub fn screen_delta_to_virtual(&self, delta: Point) -> Point {
        if self.base_scale <= 0.0001 {
            return Point::ZERO;
        }
        Point::new(delta.x / self.base_scale, delta.y / self.base_scale)
    }

    /// Returns whether `screen_pt` lies within the bounds of the virtual canvas surface.
    #[inline]
    #[must_use]
    pub fn contains_screen_point(&self, screen_pt: Point) -> bool {
        self.canvas_rect.contains_point(screen_pt)
    }
}