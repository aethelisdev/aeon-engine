// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # High-Performance GPU Canvas & Charting Widgets (`iris-widgets::canvas`)
//!
//! Provides first-class, zero-allocation hardware-accelerated 2D canvas drawing and charting
//! components for game engine tools, timelines, telemetry graphs, and node graph wires.

use iris_core::canvas::CanvasPrimitive;
use iris_core::{Color, Point, Rect, Style, UiTree, WidgetId};
use iris_wgpu::{DrawCommandList, QuadInstance};

/// High-frequency immediate-mode canvas drawing context.
///
/// Designed for per-frame dynamic visual components (oscilloscope graphs, profiler telemetry curves,
/// audio waveforms, timeline scrubbers) where allocating persistent retained nodes in the `UiTree`
/// would cause unnecessary layout recalculations and memory thrashing.
///
/// Primitives appended through [`ImmediateCanvas`] bypass Taffy layout passes entirely and are
/// translated directly to hardware draw quads by the renderer compiler.
pub struct ImmediateCanvas<'a> {
    tree: &'a mut UiTree,
    node_id: WidgetId,
    bounds: Rect,
}

impl<'a> ImmediateCanvas<'a> {
    /// Creates a new immediate canvas context bounded to the specified screen rectangle.
    ///
    /// # Arguments
    /// * `tree` - Mutable reference to the parent UI tree storing volatile commands.
    /// * `node_id` - Retained host widget node identifier holding the canvas layout box.
    /// * `bounds` - Resolved screen-space bounding rectangle of the canvas container.
    #[inline]
    pub fn new(tree: &'a mut UiTree, node_id: WidgetId, bounds: Rect) -> Self {
        Self {
            tree,
            node_id,
            bounds,
        }
    }

    /// Returns the resolved screen-space bounding rectangle of the canvas container.
    #[inline]
    #[must_use]
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Returns the resolved pixel width of the canvas container.
    #[inline]
    #[must_use]
    pub fn width(&self) -> f32 {
        self.bounds.width
    }

    /// Returns the resolved pixel height of the canvas container.
    #[inline]
    #[must_use]
    pub fn height(&self) -> f32 {
        self.bounds.height
    }

    /// Appends a solid or rounded filled rectangle to the immediate canvas command buffer.
    ///
    /// # Arguments
    /// * `rect` - Rectangle destination in absolute screen pixels.
    /// * `color` - Solid fill color.
    /// * `corner_radius` - Corner radius in pixels for rounded corners.
    #[inline]
    pub fn fill_rect(&mut self, rect: Rect, color: Color, corner_radius: f32) {
        self.tree.add_canvas_primitive(
            self.node_id,
            CanvasPrimitive::Rect {
                rect,
                color,
                corner_radius,
            },
        );
    }

    /// Appends a hollow stroked border outline to the immediate canvas command buffer.
    ///
    /// # Arguments
    /// * `rect` - Rectangle destination in absolute screen pixels.
    /// * `color` - Stroke color.
    /// * `border_width` - Border thickness in pixels.
    /// * `corner_radius` - Corner radius in pixels for rounded corners.
    #[inline]
    pub fn stroke_rect(&mut self, rect: Rect, color: Color, border_width: f32, corner_radius: f32) {
        self.tree.add_canvas_primitive(
            self.node_id,
            CanvasPrimitive::Border {
                rect,
                border_width,
                color,
                corner_radius,
            },
        );
    }

    /// Appends a straight line segment between two points to the immediate canvas command buffer.
    ///
    /// # Arguments
    /// * `start` - Start coordinate in screen pixels.
    /// * `end` - End coordinate in screen pixels.
    /// * `color` - Line stroke color.
    /// * `thickness` - Line stroke thickness in pixels.
    #[inline]
    pub fn draw_line(&mut self, start: Point, end: Point, color: Color, thickness: f32) {
        self.tree.add_canvas_primitive(
            self.node_id,
            CanvasPrimitive::Line {
                start,
                end,
                thickness,
                color,
            },
        );
    }

    /// Appends connected polyline segments across an array of coordinate points.
    ///
    /// # Arguments
    /// * `points` - Slice of ordered 2D vertices in screen coordinates.
    /// * `color` - Line stroke color.
    /// * `thickness` - Line stroke thickness in pixels.
    pub fn draw_polyline(&mut self, points: &[Point], color: Color, thickness: f32) {
        if points.len() < 2 {
            return;
        }
        for window in points.windows(2) {
            self.draw_line(window[0], window[1], color, thickness);
        }
    }

    /// Appends a volatile dynamic chart curve sampled across discrete data points.
    ///
    /// # Arguments
    /// * `samples` - Sampling closure retrieving value for index.
    /// * `sample_count` - Total number of available samples.
    /// * `max_scale` - Vertical value scale ceiling.
    /// * `target_steps` - Maximum number of interpolation segments.
    /// * `color_fn` - Dynamic color evaluation closure per sample value.
    pub fn draw_chart_polyline<F>(
        &mut self,
        samples: impl Fn(usize) -> Option<f32>,
        sample_count: usize,
        max_scale: f32,
        target_steps: usize,
        color_fn: F,
    ) where
        F: Fn(f32) -> Color,
    {
        if sample_count < 2 || target_steps < 2 || self.bounds.width <= 0.0 || max_scale <= 0.001 {
            return;
        }

        let steps = target_steps.min(120);
        let step_stride = (sample_count as f32) / (steps as f32);
        let step_x = self.bounds.width / (steps - 1) as f32;

        let mut prev_pt: Option<(Point, f32)> = None;

        for step_idx in 0..steps {
            let sample_idx = ((step_idx as f32 * step_stride) as usize).min(sample_count - 1);
            let val = samples(sample_idx).unwrap_or(0.0).clamp(0.0, max_scale);

            let pt_x = self.bounds.x + (step_idx as f32) * step_x;
            let pt_y = self.bounds.y + self.bounds.height - (val / max_scale) * self.bounds.height;
            let current_pt = Point::new(pt_x, pt_y);

            if let Some((prev_pt_val, prev_val)) = prev_pt {
                let segment_color = color_fn(val.max(prev_val));
                self.draw_line(prev_pt_val, current_pt, segment_color, 1.5);
            }

            prev_pt = Some((current_pt, val));
        }
    }
}

/// Zero-allocation, direct GPU drawing utility for hardware-accelerated time-series and oscilloscope charts.
pub struct ChartDrawer;

impl ChartDrawer {
    /// Renders a series of historical data points directly into the `DrawCommandList` as instanced SDF quads.
    ///
    /// # Arguments
    /// * `command_list` - Hardware draw command buffer.
    /// * `canvas_rect` - Bounding rectangle of the canvas container.
    /// * `samples` - Chronological iterator over data samples.
    /// * `sample_count` - Number of samples in the buffer.
    /// * `max_scale` - Maximum vertical scale value.
    /// * `color_fn` - Color calculation closure based on sample value.
    pub fn draw_polyline<F>(
        command_list: &mut DrawCommandList,
        canvas_rect: Rect,
        samples: impl Fn(usize) -> Option<f32>,
        sample_count: usize,
        max_scale: f32,
        target_steps: usize,
        color_fn: F,
    ) where
        F: Fn(f32) -> Color,
    {
        if sample_count < 2 || target_steps < 2 || canvas_rect.width <= 0.0 || max_scale <= 0.001 {
            return;
        }

        let steps = target_steps.min(120);
        let step_stride = (sample_count as f32) / (steps as f32);
        let step_x = canvas_rect.width / (steps - 1) as f32;

        let mut prev_pt: Option<(f32, f32, f32)> = None;

        for step_idx in 0..steps {
            let sample_idx = ((step_idx as f32 * step_stride) as usize).min(sample_count - 1);
            let val = samples(sample_idx).unwrap_or(0.0).clamp(0.0, max_scale);

            let pt_x = canvas_rect.x + (step_idx as f32) * step_x;
            let pt_y = canvas_rect.y + canvas_rect.height - (val / max_scale) * canvas_rect.height;

            if let Some((prev_x, prev_y, prev_val)) = prev_pt {
                let segment_color = color_fn(val.max(prev_val));
                let seg_min_x = prev_x.min(pt_x);
                let seg_min_y = prev_y.min(pt_y);
                let seg_w = (pt_x - prev_x).abs().max(1.5);
                let seg_h = (pt_y - prev_y).abs().max(1.5);

                command_list.push_quad(QuadInstance::from_style(
                    Rect::new(seg_min_x, seg_min_y, seg_w, seg_h),
                    &Style::new().background(segment_color).border_radius(0.75),
                    Some(canvas_rect),
                ));
            }

            prev_pt = Some((pt_x, pt_y, val));
        }
    }
}