// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # High-Performance GPU Canvas & Charting Widgets (`iris-widgets::canvas`)
//!
//! Provides first-class, zero-allocation hardware-accelerated 2D canvas drawing and charting
//! components for game engine tools, timelines, telemetry graphs, and node graph wires.

use iris_core::{Color, Rect, Style, TextAlign, UiTree, WidgetId};
use iris_wgpu::{DrawCommandList, QuadInstance};

/// Visual threshold milestone line configuration for time-series and profiler charts.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartThreshold {
    /// Metric value threshold level (e.g. 16.67 ms, 33.33 ms).
    pub value: f32,
    /// Milestone label text displayed on the right edge.
    pub label: &'static str,
    /// Color and opacity of the threshold line and label text.
    pub color: Color,
}

impl ChartThreshold {
    /// Creates a new chart milestone threshold line descriptor.
    pub const fn new(value: f32, label: &'static str, color: Color) -> Self {
        Self {
            value,
            label,
            color,
        }
    }
}

/// Visual styling configuration for charting widgets.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartStyle {
    /// Background fill color of the canvas container.
    pub background_color: Color,
    /// Border stroke color.
    pub border_color: Color,
    /// Border thickness in physical pixels.
    pub border_width: f32,
    /// Corner rounding radius in physical pixels.
    pub corner_radius: f32,
    /// Polyline segment thickness in pixels.
    pub line_thickness: f32,
    /// Maximum vertical scale limit.
    pub max_scale_value: f32,
}

impl Default for ChartStyle {
    fn default() -> Self {
        Self {
            background_color: Color::rgba(0.05, 0.05, 0.07, 1.0),
            border_color: Color::rgba(0.12, 0.13, 0.17, 1.0),
            border_width: 1.0,
            corner_radius: 4.0,
            line_thickness: 1.5,
            max_scale_value: 36.0,
        }
    }
}

/// Builder for constructing retained GPU canvas containers in the `UiTree`.
pub struct CanvasBuilder<'a> {
    tree: &'a mut UiTree,
    parent_id: Option<WidgetId>,
    name: String,
    rect: Rect,
    style: ChartStyle,
    thresholds: Vec<ChartThreshold>,
}

impl<'a> CanvasBuilder<'a> {
    /// Creates a new canvas container builder.
    pub fn new(tree: &'a mut UiTree) -> Self {
        Self {
            tree,
            parent_id: None,
            name: "GpuCanvas".into(),
            rect: Rect::ZERO,
            style: ChartStyle::default(),
            thresholds: Vec::new(),
        }
    }

    /// Assigns the parent widget handle.
    pub fn parent(mut self, parent: WidgetId) -> Self {
        self.parent_id = Some(parent);
        self
    }

    /// Sets the debug node name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the layout bounds of the canvas.
    pub fn rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    /// Sets the visual chart style.
    pub fn style(mut self, style: ChartStyle) -> Self {
        self.style = style;
        self
    }

    /// Adds a milestone threshold line to the canvas layout.
    pub fn add_threshold(mut self, threshold: ChartThreshold) -> Self {
        self.thresholds.push(threshold);
        self
    }

    /// Finalizes and instantiates the canvas container and its milestone lines in the `UiTree`.
    pub fn build(self) -> WidgetId {
        let canvas_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(canvas_id) {
            node.set_name(self.name);
            node.computed_rect = self.rect;
            node.style = Style::new()
                .background(self.style.background_color)
                .border(self.style.border_width, self.style.border_color)
                .border_radius(self.style.corner_radius);
        }

        if let Some(parent) = self.parent_id {
            let _ = self.tree.add_child(parent, canvas_id);
        }

        let max_val = self.style.max_scale_value.max(0.001);

        for threshold in &self.thresholds {
            let ly =
                self.rect.y + self.rect.height - (threshold.value / max_val) * self.rect.height;

            let line_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(line_id) {
                node.set_name("ChartThresholdLine");
                node.computed_rect = Rect::new(self.rect.x, ly, self.rect.width, 1.0);
                node.style = Style::new().background(threshold.color);
            }
            let _ = self.tree.add_child(canvas_id, line_id);

            let lbl_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(lbl_id) {
                node.set_name("ChartThresholdLabel");
                node.set_text(threshold.label);
                node.font_size = 8.5;
                node.line_height = 10.0;
                node.text_align = TextAlign::Right;
                node.text_color = threshold.color;
                node.computed_rect = Rect::new(self.rect.x, ly - 11.0, self.rect.width - 6.0, 10.0);
            }
            let _ = self.tree.add_child(canvas_id, lbl_id);
        }

        canvas_id
    }
}

/// Zero-allocation, direct GPU drawing utility for hardware-accelerated time-series and oscilloscope charts.
pub struct ChartDrawer;

impl ChartDrawer {
    /// Renders a series of historical data points directly into the `DrawCommandList` as instanced SDF quads.
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