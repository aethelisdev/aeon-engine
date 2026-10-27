// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hardware-Accelerated Declarative Color Picker Widget (`iris-widgets::declarative::widget::color_picker`)
//!
//! Provides a declarative 2D HSV color picker widget composing a continuous
//! saturation-value spectrum box, vertical rainbow hue bar, interactive indicators,
//! and immediate [`ColorPickerResponse`] state tracking.

use std::ops::Deref;

use iris_core::geometry::Rect;
use iris_core::response::WidgetResponse;
use iris_core::{AlignItems, Color, Style, WidgetCursor, WidgetRole};

use crate::color_picker::{HsvColorPickerState, hsv_to_rgb};
use crate::declarative::scope::UiScope;
use crate::declarative::types::combine_seeds;
use crate::declarative::widget::Widget;

/// Response payload emitted by the declarative [`ColorPicker`] widget.
///
/// Implements [`Deref<Target = WidgetResponse>`] for  access to generic
/// interaction flags (`clicked`, `hovered`, `dragged`) while providing domain-specific
/// color state changes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPickerResponse {
    /// Generic widget response tracking interaction bounds and flags.
    pub response: WidgetResponse,
    /// `true` if the underlying HSV channels were mutated during this frame.
    pub changed: bool,
    /// `true` if an active dragging gesture on the SV box or Hue bar was released this frame.
    pub drag_released: bool,
    /// Snapshot of the active HSV color picker state.
    pub state: HsvColorPickerState,
    /// Computed concrete [`Color`] value corresponding to active HSV state.
    pub color: Color,
}

impl Deref for ColorPickerResponse {
    type Target = WidgetResponse;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl ColorPickerResponse {
    /// Returns `true` if the color channels changed during this interaction pass.
    #[inline]
    pub const fn changed(&self) -> bool {
        self.changed
    }

    /// Returns `true` if the drag gesture ended during this frame (commit trigger).
    #[inline]
    pub const fn drag_released(&self) -> bool {
        self.drag_released
    }

    /// Returns the active RGB [`Color`].
    #[inline]
    pub const fn color(&self) -> Color {
        self.color
    }

    /// Returns the active [`HsvColorPickerState`].
    #[inline]
    pub const fn state(&self) -> HsvColorPickerState {
        self.state
    }
}

/// Declarative builder for an interactive 2D HSV Color Picker control.
///
/// Composes an interactive 2D Saturation-Value spectrum matrix and a vertical Rainbow
/// Hue spectrum bar with real-time pointer capture and dragging feedback.
pub struct ColorPicker<'a> {
    state: &'a mut HsvColorPickerState,
    sv_size: (f32, f32),
    hue_width: f32,
    gap: f32,
    tag: Option<u64>,
    sv_tag: Option<u64>,
    hue_tag: Option<u64>,
    name: Option<&'static str>,
}

impl<'a> ColorPicker<'a> {
    /// Constructs a new [`ColorPicker`] bound to a mutable [`HsvColorPickerState`].
    #[must_use]
    pub fn new(state: &'a mut HsvColorPickerState) -> Self {
        Self {
            state,
            sv_size: (160.0, 130.0),
            hue_width: 18.0,
            gap: 8.0,
            tag: None,
            sv_tag: None,
            hue_tag: None,
            name: None,
        }
    }

    /// Configures the logical dimensions `(width, height)` of the 2D SV box.
    #[must_use]
    pub fn sv_size(mut self, width: f32, height: f32) -> Self {
        self.sv_size = (width.max(20.0), height.max(20.0));
        self
    }

    /// Configures the width of the vertical Hue spectrum bar.
    #[must_use]
    pub fn hue_width(mut self, width: f32) -> Self {
        self.hue_width = width.max(8.0);
        self
    }

    /// Configures the layout gap between the 2D SV box and the vertical Hue bar.
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    /// Assigns an explicit 64-bit semantic tag for the root container node.
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Assigns explicit 64-bit semantic tags for the 2D SV box and vertical Hue bar.
    #[must_use]
    pub fn tags(mut self, sv_tag: u64, hue_tag: u64) -> Self {
        self.sv_tag = Some(sv_tag);
        self.hue_tag = Some(hue_tag);
        self
    }

    /// Assigns a debugging name to the root flex container node.
    #[must_use]
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }
}

impl<'a> Widget for ColorPicker<'a> {
    type Response = ColorPickerResponse;

    fn ui(self, scope: &mut UiScope<'_>) -> ColorPickerResponse {
        let (sv_w, sv_h) = self.sv_size;
        let hue_w = self.hue_width;
        let base_tag = self.tag.unwrap_or_else(|| scope.tag_for("ColorPicker"));
        let sv_tag = self.sv_tag.unwrap_or_else(|| combine_seeds(base_tag, 1));
        let hue_tag = self.hue_tag.unwrap_or_else(|| combine_seeds(base_tag, 2));

        // 1. Root Flex Row Container
        let row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(self.gap)
            .height(sv_h);

        let row_id = scope.tree.create_node();
        let _ = scope.tree.add_child(scope.parent, row_id);
        if let Some(node) = scope.tree.get_mut(row_id) {
            node.set_name(self.name.unwrap_or("ColorPickerRow"));
            node.set_style(row_style);
            node.tag = base_tag;
        }

        let mut row_scope = scope.child_scope(row_id);

        // 2. 2D Saturation-Value Box
        let sv_id = row_scope.tree.create_node();
        let _ = row_scope.tree.add_child(row_scope.parent, sv_id);
        let sv_style = Style::new()
            .width(sv_w)
            .height(sv_h)
            .border(1.0, Color::rgba(0.25, 0.28, 0.35, 0.90))
            .border_radius(4.0)
            .clip_children(true);

        if let Some(node) = row_scope.tree.get_mut(sv_id) {
            node.set_name("ColorPickerSvBox");
            node.tag = sv_tag;
            node.interactive = true;
            node.role = WidgetRole::ColorPickerArea;
            node.cursor = Some(WidgetCursor::Crosshair);
            node.set_style(sv_style);
        }

        // 3. Vertical Hue Spectrum Bar
        let hue_id = row_scope.tree.create_node();
        let _ = row_scope.tree.add_child(row_scope.parent, hue_id);
        let hue_style = Style::new()
            .width(hue_w)
            .height(sv_h)
            .border(1.0, Color::rgba(0.25, 0.28, 0.35, 0.90))
            .border_radius(4.0)
            .clip_children(true);

        if let Some(node) = row_scope.tree.get_mut(hue_id) {
            node.set_name("ColorPickerHueBar");
            node.tag = hue_tag;
            node.interactive = true;
            node.role = WidgetRole::ColorPickerArea;
            node.cursor = Some(WidgetCursor::Crosshair);
            node.set_style(hue_style);
        }

        // 4. Interaction Processing
        let sv_resp = row_scope.response(sv_id);
        let hue_resp = row_scope.response(hue_id);

        let is_sv_active = sv_resp.clicked()
            || sv_resp.dragged()
            || row_scope.is_pressed(sv_id)
            || row_scope.tree.event_state.pointer_capture == Some(sv_id)
            || row_scope.tree.event_state.pointer_capture_tag == Some(sv_tag);

        let is_hue_active = hue_resp.clicked()
            || hue_resp.dragged()
            || row_scope.is_pressed(hue_id)
            || row_scope.tree.event_state.pointer_capture == Some(hue_id)
            || row_scope.tree.event_state.pointer_capture_tag == Some(hue_tag);

        let mut changed = false;

        if is_sv_active {
            if let Some(point) = sv_resp
                .pointer_pos
                .or(row_scope.tree.event_state.last_pointer_pos)
            {
                let sv_rect = row_scope
                    .tree
                    .tag_rect(sv_tag)
                    .or_else(|| {
                        row_scope
                            .tree
                            .get(sv_id)
                            .map(|n| n.computed_rect)
                            .filter(|r| r.width > 0.0 && r.height > 0.0)
                    })
                    .unwrap_or_else(|| {
                        let origin = row_scope.parent_screen_origin();
                        Rect::new(origin.x, origin.y, sv_w, sv_h)
                    });
                let old_s = self.state.saturation;
                let old_v = self.state.value;
                self.state.update_from_sv_point(point, sv_rect);
                if (old_s - self.state.saturation).abs() > 1e-4
                    || (old_v - self.state.value).abs() > 1e-4
                {
                    changed = true;
                }
            }
        } else if is_hue_active
            && let Some(point) = hue_resp
                .pointer_pos
                .or(row_scope.tree.event_state.last_pointer_pos)
        {
            let hue_rect = row_scope
                .tree
                .tag_rect(hue_tag)
                .or_else(|| {
                    row_scope
                        .tree
                        .get(hue_id)
                        .map(|n| n.computed_rect)
                        .filter(|r| r.width > 0.0 && r.height > 0.0)
                })
                .unwrap_or_else(|| {
                    let origin = row_scope.parent_screen_origin();
                    Rect::new(origin.x + sv_w + self.gap, origin.y, hue_w, sv_h)
                });
            let old_h = self.state.hue;
            self.state.update_from_hue_point(point, hue_rect);
            if (old_h - self.state.hue).abs() > 1e-3 {
                changed = true;
            }
        }

        let drag_released = sv_resp.drag_released()
            || hue_resp.drag_released()
            || row_scope.tree.event_state.frame_drag_released_id == Some(sv_id)
            || row_scope.tree.event_state.frame_drag_released_id == Some(hue_id)
            || row_scope.tree.event_state.frame_drag_released_tag == Some(sv_tag)
            || row_scope.tree.event_state.frame_drag_released_tag == Some(hue_tag);

        // 5. Render SV Spectrum Cells and Target Indicator
        let mut sv_scope = row_scope.child_scope(sv_id);
        let grid_nx = 16;
        let grid_ny = 13;
        let cell_w = sv_w / (grid_nx as f32);
        let cell_h = sv_h / (grid_ny as f32);
        let active_hue = self.state.hue;

        for j in 0..grid_ny {
            for i in 0..grid_nx {
                let s = (i as f32 + 0.5) / (grid_nx as f32);
                let v = 1.0 - (j as f32 + 0.5) / (grid_ny as f32);
                let col = hsv_to_rgb(active_hue, s, v);

                let cell_style = Style::new()
                    .position_absolute()
                    .left((i as f32) * cell_w)
                    .top((j as f32) * cell_h)
                    .width(cell_w + 0.2)
                    .height(cell_h + 0.2)
                    .background(col);

                sv_scope.empty_box_passive(cell_style);
            }
        }

        // Saturation-Value Ring Indicator
        let ring_size = 10.0;
        let ring_x = (self.state.saturation * sv_w - ring_size * 0.5).clamp(0.0, sv_w - ring_size);
        let ring_y =
            ((1.0 - self.state.value) * sv_h - ring_size * 0.5).clamp(0.0, sv_h - ring_size);
        let ring_style = Style::new()
            .position_absolute()
            .left(ring_x)
            .top(ring_y)
            .width(ring_size)
            .height(ring_size)
            .border(2.0, Color::WHITE)
            .border_radius(5.0)
            .box_shadow(0.0, 1.0, 3.0, Color::rgba(0.0, 0.0, 0.0, 0.90));

        sv_scope.empty_box_passive(ring_style);

        // 6. Render Hue Spectrum Strips and Line Indicator
        let mut hue_scope = row_scope.child_scope(hue_id);
        let hue_steps = 26;
        let step_h = sv_h / (hue_steps as f32);

        for step in 0..hue_steps {
            let h_val = (step as f32 + 0.5) / (hue_steps as f32) * 360.0;
            let col = hsv_to_rgb(h_val, 1.0, 1.0);

            let strip_style = Style::new()
                .position_absolute()
                .left(0.0)
                .top((step as f32) * step_h)
                .width(hue_w)
                .height(step_h + 0.2)
                .background(col);

            hue_scope.empty_box_passive(strip_style);
        }

        let ind_h = 4.0;
        let ind_y = (self.state.hue / 360.0 * sv_h - ind_h * 0.5).clamp(0.0, sv_h - ind_h);
        let ind_style = Style::new()
            .position_absolute()
            .left(0.0)
            .top(ind_y)
            .width(hue_w)
            .height(ind_h)
            .background(Color::WHITE)
            .border(1.0, Color::BLACK)
            .border_radius(1.0);

        hue_scope.empty_box_passive(ind_style);

        let row_resp = scope.response(row_id);
        let final_state = *self.state;
        let final_color = self.state.to_color();

        ColorPickerResponse {
            response: row_resp,
            changed,
            drag_released,
            state: final_state,
            color: final_color,
        }
    }
}