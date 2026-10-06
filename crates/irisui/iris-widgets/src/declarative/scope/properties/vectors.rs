// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Multi-Channel Vector & Color Editors
//!
//! Provides color-coded axis inputs for 3D coordinates, 2D offsets/tilings, and RGB multipliers on [`UiScope`].
//!

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetId, WidgetRole,
};

/// Configuration options for 3-channel vector property rows.
#[derive(Debug, Clone, Copy)]
pub struct PropertyVec3Options<'a> {
    /// Drag sensitivity multiplier applied per horizontal delta pixel.
    pub speed: f32,
    /// Floating point decimal precision displayed in the axis boxes.
    pub decimals: usize,
    /// Optional default `[x, y, z]` value for the reset button `🔄`.
    pub reset_value: Option<[f32; 3]>,
    /// Optional active text editing states for the X, Y, and Z axis boxes.
    pub edit_states: [Option<crate::numeric_input::NumericInputEditState<'a>>; 3],
    /// Fixed column width in pixels for the left-aligned property label.
    pub label_width: f32,
}

impl<'a> Default for PropertyVec3Options<'a> {
    fn default() -> Self {
        Self {
            speed: 0.1,
            decimals: 2,
            reset_value: None,
            edit_states: [None, None, None],
            label_width: 52.0,
        }
    }
}

impl<'a> PropertyVec3Options<'a> {
    /// Creates a new vector options descriptor with specified drag speed, default 2 decimals, and standard 52.0px label width.
    pub const fn new(speed: f32) -> Self {
        Self {
            speed,
            decimals: 2,
            reset_value: None,
            edit_states: [None, None, None],
            label_width: 52.0,
        }
    }

    /// Sets the displayed decimal precision.
    pub const fn with_decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    /// Enables an axis reset button that restores the vector to `reset_value` upon click.
    pub const fn with_reset(mut self, reset_value: [f32; 3]) -> Self {
        self.reset_value = Some(reset_value);
        self
    }

    /// Informs the property row of active text editing states for each axis.
    pub const fn with_edit_states(
        mut self,
        edit_states: [Option<crate::numeric_input::NumericInputEditState<'a>>; 3],
    ) -> Self {
        self.edit_states = edit_states;
        self
    }

    /// Sets the fixed column width in pixels for the left-aligned property label.
    pub const fn with_label_width(mut self, label_width: f32) -> Self {
        self.label_width = label_width;
        self
    }
}

/// Detailed response returned from a 3-channel vector property row.
///
/// Encapsulates change/interaction status as well as allocated widget identifiers
/// for color-coded axis input boxes and optional reset buttons.
#[derive(Debug, Clone, Copy, Default)]
pub struct PropertyVec3Response {
    /// Generic interaction state (clicked, hovered, changed).
    pub response: WidgetResponse,
    /// Allocated widget identifiers for the X, Y, and Z axis numeric inputs.
    pub axes: [WidgetId; 3],
    /// Allocated widget identifier for the axis reset button, if enabled.
    pub reset_btn: Option<WidgetId>,
}

impl std::ops::Deref for PropertyVec3Response {
    type Target = WidgetResponse;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl<'a> UiScope<'a> {
    /// Emits a 3D coordinate vector property row with distinct color-coded X, Y, Z axis editors.
    ///
    /// Modifies `values` (`[x, y, z]`) in place based on continuous pointer drag deltas.
    ///
    /// # Arguments
    /// * `label` - Property description displayed on the left (supports `##` separator).
    /// * `values` - Mutable reference to the 3-element float array.
    /// * `speed` - Drag sensitivity multiplier applied per horizontal delta pixel.
    pub fn property_vec3(
        &mut self,
        label: &str,
        values: &mut [f32; 3],
        speed: f32,
    ) -> WidgetResponse {
        self.property_vec3_with_options(label, values, PropertyVec3Options::new(speed))
            .response
    }

    /// Emits a 3D coordinate vector property row with distinct color-coded X, Y, Z axis editors and optional reset button.
    ///
    /// Modifies `values` (`[x, y, z]`) in place based on continuous pointer drag deltas, numeric typing, or reset button clicks.
    ///
    /// # Arguments
    /// * `label` - Property description displayed on the left (supports `##` separator).
    /// * `values` - Mutable reference to the 3-element float array.
    /// * `options` - Custom configuration options (speed, decimals, reset_value, edit_states).
    pub fn property_vec3_with_options(
        &mut self,
        label: &str,
        values: &mut [f32; 3],
        options: PropertyVec3Options<'_>,
    ) -> PropertyVec3Response {
        let (visible_label, _) = crate::declarative::types::split_label_id(label);
        let axis_names = ["X", "Y", "Z"];

        let base_tag = self.tag_for(label);
        let mut widget_response = WidgetResponse::default();
        let mut axes_out = [WidgetId::default(); 3];
        let mut reset_btn_out = None;

        let row_id = self.property_row_styled(label, Some(options.label_width), |slot| {
            let axes_meta = [
                ("X: ", Color::rgba(0.90, 0.30, 0.30, 1.0), 0),
                ("Y: ", Color::rgba(0.30, 0.85, 0.40, 1.0), 1),
                ("Z: ", Color::rgba(0.30, 0.55, 0.95, 1.0), 2),
            ];

            let mut any_changed = false;
            let mut any_clicked = false;
            let mut any_hovered = false;

            for (prefix, _col, idx) in axes_meta {
                let axis_tag = base_tag.wrapping_add((idx as u64) + 1);
                let edit_state = options.edit_states[idx];
                let is_editing = edit_state.is_some();

                let (bg_col, border_col) = if is_editing {
                    (
                        Color::rgba(0.118, 0.125, 0.145, 1.0),
                        Color::rgba(0.0, 0.80, 1.00, 0.95),
                    )
                } else {
                    (
                        Color::rgba(0.125, 0.133, 0.153, 0.98),
                        Color::rgba(0.180, 0.192, 0.227, 0.85),
                    )
                };

                let txt_col = if is_editing {
                    Color::WHITE
                } else {
                    Color::rgba(0.886, 0.894, 0.918, 1.0)
                };

                let display_str = if let Some(s) = edit_state {
                    let buf = s.buffer;
                    let cursor = s.cursor_idx.min(buf.len());
                    let (left, right) = buf.split_at(cursor);
                    if s.is_all_selected {
                        format!("{}{}", prefix, buf)
                    } else if s.blink_caret {
                        format!("{}{}|{}", prefix, left, right)
                    } else {
                        format!("{}{}{}", prefix, left, right)
                    }
                } else {
                    format!(
                        "{}{:.precision$}",
                        prefix,
                        values[idx],
                        precision = options.decimals
                    )
                };

                let axis_name = axis_names[idx];
                let box_name = format!("NumBox_{}_{}", visible_label, axis_name);
                let sel_name = format!("NumSel_{}_{}", visible_label, axis_name);
                let txt_name = format!("NumText_{}_{}", visible_label, axis_name);

                let pill_style = Style::new()
                    .width(54.0)
                    .height(20.0)
                    .background(bg_col)
                    .border(1.0, border_col)
                    .border_radius(5.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center);

                let axis_box_id = slot.container_named(box_name, pill_style, |pill_scope| {
                    if let Some(s) = edit_state.filter(|s| s.is_all_selected) {
                        let approx_char_w = 6.2;
                        let val_w = (s.buffer.len() as f32 * approx_char_w).max(12.0);
                        let prefix_w = prefix.len() as f32 * approx_char_w;
                        let tot_w = (prefix.len() + s.buffer.len()) as f32 * approx_char_w;
                        let cx = 54.0 * 0.5;
                        let text_start_x = cx - tot_w * 0.5;
                        let val_start_x = text_start_x + prefix_w;
                        let sel_x = (val_start_x - 2.0).clamp(2.0, 54.0 - 6.0);
                        let sel_w = (val_w + 4.0).min(54.0 - 2.0 - sel_x);

                        let sel_style = Style::new()
                            .position_absolute()
                            .left(sel_x)
                            .top(2.5)
                            .width(sel_w)
                            .height(15.0)
                            .background(Color::rgba(0.14, 0.46, 0.88, 0.95))
                            .border_radius(3.0);
                        pill_scope.empty_box_passive_named(sel_name, sel_style);
                    }

                    pill_scope.label_styled_passive(
                        txt_name,
                        display_str,
                        10.5,
                        txt_col,
                        TextAlign::Center,
                        Style::new().width(54.0).height(20.0),
                    );
                });

                if let Some(node) = slot.tree.get_mut(axis_box_id) {
                    node.interactive = true;
                    node.role = WidgetRole::NumericInput;
                    node.tag = axis_tag;
                    node.cursor = Some(if is_editing {
                        WidgetCursor::Text
                    } else {
                        WidgetCursor::EwResize
                    });
                }

                axes_out[idx] = axis_box_id;

                let (clicked, hovered, drag_delta) = slot.check_interaction(axis_box_id);
                if let Some(delta) = drag_delta {
                    let next = values[idx] + delta.x * options.speed;
                    if (next - values[idx]).abs() > f32::EPSILON {
                        values[idx] = next;
                        any_changed = true;
                    }
                }
                if let Some(text) = slot.check_interaction_text(axis_box_id) {
                    let clean = text.trim();
                    if let Ok(num) = clean.parse::<f32>() {
                        values[idx] = num;
                        any_changed = true;
                    }
                }
                any_clicked |= clicked;
                any_hovered |= hovered;
            }

            // Optional Reset Button
            if let Some(reset_val) = options.reset_value {
                let reset_tag = base_tag.wrapping_add(4);
                let is_reset_hovered = slot.hovered_tag == Some(reset_tag);

                let reset_style = Style::new()
                    .width(18.0)
                    .height(18.0)
                    .background(if is_reset_hovered {
                        Color::rgba(0.157, 0.169, 0.200, 1.0)
                    } else {
                        Color::rgba(0.125, 0.133, 0.153, 0.98)
                    })
                    .border(
                        1.0,
                        if is_reset_hovered {
                            Color::rgba(0.235, 0.247, 0.286, 0.95)
                        } else {
                            Color::rgba(0.180, 0.192, 0.227, 0.85)
                        },
                    )
                    .border_radius(5.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center);

                let reset_name = format!("ResetBtn_{}", visible_label);
                let reset_btn_id = slot.container_named(reset_name, reset_style, |btn_scope| {
                    btn_scope.label_styled_passive(
                        "ResetBtnText",
                        "🔄",
                        10.0,
                        if is_reset_hovered {
                            Color::WHITE
                        } else {
                            Color::rgba(0.70, 0.73, 0.80, 0.90)
                        },
                        TextAlign::Center,
                        Style::new().width(18.0).height(18.0),
                    );
                });

                if let Some(node) = slot.tree.get_mut(reset_btn_id) {
                    node.interactive = true;
                    node.role = WidgetRole::Button;
                    node.tag = reset_tag;
                    node.cursor = Some(WidgetCursor::Pointer);
                }

                reset_btn_out = Some(reset_btn_id);

                let (reset_clicked, reset_hovered, _) = slot.check_interaction(reset_btn_id);
                if reset_clicked {
                    *values = reset_val;
                    any_changed = true;
                }
                any_clicked |= reset_clicked;
                any_hovered |= reset_hovered;
            }

            widget_response =
                WidgetResponse::new(slot.parent, any_clicked, any_hovered, any_changed);
        });

        if widget_response.id == WidgetId::default() {
            widget_response.id = row_id;
        }

        PropertyVec3Response {
            response: widget_response,
            axes: axes_out,
            reset_btn: reset_btn_out,
        }
    }

    /// Emits an inline horizontal 2D vector editor (e.g. UV Tiling or 2D Offset) with labeled axes.
    ///
    /// # Arguments
    /// * `label` - Property description displayed on the left (supports `##` separator).
    /// * `values` - Mutable reference to the 2-element float array.
    /// * `speed` - Drag sensitivity multiplier applied per horizontal delta pixel.
    pub fn property_vec2(
        &mut self,
        label: &str,
        values: &mut [f32; 2],
        speed: f32,
    ) -> WidgetResponse {
        let (visible_label, _) = crate::declarative::types::split_label_id(label);
        let base_tag = self.tag_for(label);
        let mut response = WidgetResponse::default();

        let row_id = self.property_row_styled(label, Some(52.0), |slot| {
            let axes = [
                ("U:", Color::rgba(0.35, 0.75, 0.98, 1.0), 0),
                ("V:", Color::rgba(0.95, 0.80, 0.30, 1.0), 1),
            ];

            let mut any_changed = false;
            let mut any_clicked = false;
            let mut any_hovered = false;

            for (prefix, col, idx) in axes {
                let axis_tag = base_tag.wrapping_add((idx as u64) + 1);
                let axis_name = if idx == 0 { "U" } else { "V" };
                let box_name = format!("PropertyVec2_{}_{}", visible_label, axis_name);
                let txt_name = format!("PropertyVec2Text_{}_{}", visible_label, axis_name);

                let box_style = Style::new()
                    .width(80.0)
                    .height(20.0)
                    .background(Color::rgba(0.10, 0.11, 0.14, 0.95))
                    .border(1.0, Color::rgba(0.20, 0.22, 0.28, 0.80))
                    .border_radius(3.0)
                    .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center);

                let text_val = format!("{} {:.2}", prefix, values[idx]);

                let axis_box_id = slot.container_tagged(
                    box_name,
                    box_style,
                    WidgetRole::NumericInput,
                    axis_tag,
                    |pill_scope| {
                        pill_scope.label_styled_passive(
                            txt_name,
                            text_val,
                            10.5,
                            col,
                            TextAlign::Center,
                            Style::new().width(72.0).height(16.0),
                        );
                    },
                );

                if let Some(node) = slot.tree.get_mut(axis_box_id) {
                    node.cursor = Some(WidgetCursor::EwResize);
                }

                let (clicked, hovered, drag_delta) = slot.check_interaction(axis_box_id);
                if let Some(delta) = drag_delta {
                    let next = values[idx] + delta.x * speed;
                    if (next - values[idx]).abs() > f32::EPSILON {
                        values[idx] = next;
                        any_changed = true;
                    }
                }
                any_clicked |= clicked;
                any_hovered |= hovered;
            }

            response = WidgetResponse::new(slot.parent, any_clicked, any_hovered, any_changed);
        });

        if response.id == WidgetId::default() {
            response.id = row_id;
        }
        response
    }

    /// Emits an inline horizontal RGB color channel multiplier editor with R, G, B color coding.
    ///
    /// # Arguments
    /// * `label` - Property description displayed on the left (supports `##` separator).
    /// * `rgb` - Mutable reference to the 3-element float array `[r, g, b]`.
    /// * `speed` - Drag sensitivity multiplier applied per horizontal delta pixel.
    pub fn property_rgb(&mut self, label: &str, rgb: &mut [f32; 3], speed: f32) -> WidgetResponse {
        let (visible_label, _) = crate::declarative::types::split_label_id(label);
        let base_tag = self.tag_for(label);
        let mut response = WidgetResponse::default();

        let row_id = self.property_row_styled(label, Some(52.0), |slot| {
            let axes = [
                ("R:", Color::rgba(0.95, 0.35, 0.35, 1.0), 0),
                ("G:", Color::rgba(0.35, 0.90, 0.45, 1.0), 1),
                ("B:", Color::rgba(0.35, 0.60, 0.98, 1.0), 2),
            ];

            let mut any_changed = false;
            let mut any_clicked = false;
            let mut any_hovered = false;

            for (prefix, col, idx) in axes {
                let axis_tag = base_tag.wrapping_add((idx as u64) + 1);
                let axis_char = match idx {
                    0 => "R",
                    1 => "G",
                    _ => "B",
                };
                let box_name = format!("PropertyRgb_{}_{}", visible_label, axis_char);
                let txt_name = format!("PropertyRgbText_{}_{}", visible_label, axis_char);

                let box_style = Style::new()
                    .width(56.0)
                    .height(20.0)
                    .background(Color::rgba(0.10, 0.11, 0.14, 0.95))
                    .border(1.0, Color::rgba(0.20, 0.22, 0.28, 0.80))
                    .border_radius(3.0)
                    .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center);

                let text_val = format!("{} {:.2}", prefix, rgb[idx]);

                let axis_box_id = slot.container_tagged(
                    box_name,
                    box_style,
                    WidgetRole::NumericInput,
                    axis_tag,
                    |pill_scope| {
                        pill_scope.label_styled_passive(
                            txt_name,
                            text_val,
                            10.5,
                            col,
                            TextAlign::Center,
                            Style::new().width(50.0).height(16.0),
                        );
                    },
                );

                if let Some(node) = slot.tree.get_mut(axis_box_id) {
                    node.cursor = Some(WidgetCursor::EwResize);
                }

                let (clicked, hovered, drag_delta) = slot.check_interaction(axis_box_id);
                if let Some(delta) = drag_delta {
                    let next = (rgb[idx] + delta.x * speed).clamp(0.0, 1.0);
                    if (next - rgb[idx]).abs() > f32::EPSILON {
                        rgb[idx] = next;
                        any_changed = true;
                    }
                }
                any_clicked |= clicked;
                any_hovered |= hovered;
            }

            response = WidgetResponse::new(slot.parent, any_clicked, any_hovered, any_changed);
        });

        if response.id == WidgetId::default() {
            response.id = row_id;
        }
        response
    }
}