// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # High-Performance GPU Canvas & Charting Widgets (`iris-widgets::canvas`)
//!
//! Provides first-class, zero-allocation hardware-accelerated 2D canvas drawing and charting
//! components for game engine tools, timelines, telemetry graphs, and node graph wires.

use iris_core::{Color, Rect, Style};
use iris_wgpu::{DrawCommandList, QuadInstance};

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