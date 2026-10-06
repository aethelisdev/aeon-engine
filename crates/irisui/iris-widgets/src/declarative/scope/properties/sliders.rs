// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Continuous Property Sliders & Scalar Adjusters
//!
//! Provides proportional draggable sliders, scalar inputs, and fine-tuning number boxes on [`UiScope`].
//!

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::{WidgetResponse, split_label_id};
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetId, WidgetRole,
};

/// Configuration options for custom property sliders.
#[derive(Debug, Clone, Copy)]
pub struct PropertySliderOptions<'a> {
    /// Minimum allowable scalar value.
    pub min: f32,
    /// Maximum allowable scalar value.
    pub max: f32,
    /// Drag sensitivity multiplier applied per horizontal delta pixel.
    pub speed: f32,
    /// Optional format string specifier (e.g. `"{:.0} Hz"`, `"{:.0}"`, `"{:.2}"`).
    pub format: Option<&'a str>,
    /// Optional explanatory subtitle displayed below the slider row.
    pub subtitle: Option<&'a str>,
    /// Optional in-progress inline text editing buffer.
    pub inline_edit: Option<&'a str>,
}

impl<'a> PropertySliderOptions<'a> {
    /// Creates a new slider options descriptor with default formatting.
    pub fn new(min: f32, max: f32, speed: f32) -> Self {
        Self {
            min,
            max,
            speed,
            format: None,
            subtitle: None,
            inline_edit: None,
        }
    }

    /// Sets a custom formatting specifier (e.g. `"{:.0} Hz"`).
    pub fn with_format(mut self, format: &'a str) -> Self {
        self.format = Some(format);
        self
    }

    /// Sets an explanatory subtitle displayed beneath the control row.
    pub fn with_subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    /// Sets an active inline editing string buffer to be displayed in the number box.
    pub fn with_inline_edit(mut self, inline_edit: Option<&'a str>) -> Self {
        self.inline_edit = inline_edit;
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a rich two-way bound property slider row configured with custom options.
    ///
    /// Renders an authentic visual slider track with proportional progress fill, an interactive
    /// numeric input pill box (supporting click-to-type inline edits), and optional subtitle text.
    ///
    /// # Arguments
    /// * `label` - Property description displayed on the left (supports `##` separator).
    /// * `value` - Mutable reference to the floating-point value.
    /// * `options` - Slider configuration options (min, max, speed, format, subtitle, inline edit).
    pub fn property_slider_with_options(
        &mut self,
        label: &str,
        value: &mut f32,
        options: PropertySliderOptions<'_>,
    ) -> WidgetResponse {
        let (visible_label, _) = split_label_id(label);
        let tag = self.tag_for(label);
        let num_box_tag = crate::declarative::types::hash_label_with_seed(tag, "##num_box");

        let norm = if options.max > options.min {
            ((*value - options.min) / (options.max - options.min)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let active_in_scope = self.active_text_input.and_then(|(t, text, sel)| {
            if t == num_box_tag {
                Some((text, sel))
            } else {
                None
            }
        });

        let is_editing = options.inline_edit.is_some() || active_in_scope.is_some();
        let is_all_selected = active_in_scope.is_some_and(|(_, sel)| sel);

        let display_str = if let Some(edit) = options.inline_edit {
            edit.to_string()
        } else if let Some((active_text, _)) = active_in_scope {
            active_text.to_string()
        } else if let Some(fmt) = options.format {
            if fmt.contains("Hz") {
                format!("{:.0} Hz", *value)
            } else if fmt.contains('m') {
                format!("{:.0} m", *value)
            } else if fmt.contains('°') || fmt.contains("deg") {
                if fmt.contains(".0") {
                    format!("{:.0}°", *value)
                } else {
                    format!("{:.1}°", *value)
                }
            } else if fmt == "{:.0}" {
                format!("{:.0}", *value)
            } else if fmt == "{:.1}" {
                format!("{:.1}", *value)
            } else {
                format!("{:.2}", *value)
            }
        } else {
            format!("{:.2}", *value)
        };

        let track_w = 240.0;
        let fill_w = (norm * (track_w - 4.0)).clamp(0.0, track_w - 4.0);

        let mut response = WidgetResponse::default();

        let row_content = |row: &mut UiScope<'_>| {
            // 1. Left Label
            let label_id = row.tree.create_node();
            if let Some(node) = row.tree.get_mut(label_id) {
                node.set_name("SliderLabel");
                node.set_text(visible_label);
                node.font_size = 11.5;
                node.line_height = 20.0;
                node.text_align = TextAlign::Left;
                node.text_color = Color::rgba(0.78, 0.82, 0.90, 1.0);
                node.set_style(Style::new().width(170.0).flex_shrink(0.0));
            }
            let _ = row.tree.add_child(row.parent, label_id);

            // 2. Center Slider Track Container
            let track_id = row.tree.create_node();
            if let Some(node) = row.tree.get_mut(track_id) {
                node.set_name("SliderTrack");
                node.interactive = true;
                node.role = WidgetRole::NumericInput;
                node.tag = tag;
                node.cursor = Some(WidgetCursor::EwResize);
                node.set_style(
                    Style::new()
                        .width(track_w)
                        .height(18.0)
                        .background(Color::rgba(0.11, 0.12, 0.16, 0.85))
                        .border(1.0, Color::rgba(0.22, 0.25, 0.34, 0.70))
                        .border_radius(3.0)
                        .align_items(AlignItems::Center)
                        .padding_insets(Insets::new(2.0, 2.0, 2.0, 2.0))
                        .margin_insets(Insets::new(0.0, 10.0, 0.0, 0.0)),
                );
            }
            let _ = row.tree.add_child(row.parent, track_id);

            // Proportional Progress Fill inside Track
            let fill_id = row.tree.create_node();
            if let Some(node) = row.tree.get_mut(fill_id) {
                node.set_name("SliderFill");
                node.interactive = false;
                node.set_style(
                    Style::new()
                        .width(fill_w)
                        .height(12.0)
                        .background(Color::rgba(0.0, 0.65, 0.85, 0.75))
                        .border_radius(2.0),
                );
            }
            let _ = row.tree.add_child(track_id, fill_id);

            // 3. Right Numeric Input Pill Box
            let box_bg = if is_all_selected {
                Color::rgba(0.0, 0.40, 0.70, 0.85)
            } else if is_editing {
                Color::rgba(0.06, 0.10, 0.16, 0.95)
            } else {
                Color::rgba(0.12, 0.13, 0.17, 0.85)
            };
            let border_color = if is_all_selected {
                Color::rgba(0.0, 0.85, 1.0, 1.0)
            } else if is_editing {
                Color::rgba(0.0, 0.75, 1.0, 0.90)
            } else {
                Color::rgba(0.22, 0.25, 0.34, 0.70)
            };

            let num_box_id = row.tree.create_node();
            if let Some(node) = row.tree.get_mut(num_box_id) {
                node.set_name("SliderNumberBox");
                node.interactive = true;
                node.role = WidgetRole::NumericInput;
                node.tag = num_box_tag;
                node.cursor = Some(WidgetCursor::Text);
                node.set_style(
                    Style::new()
                        .width(68.0)
                        .height(22.0)
                        .background(box_bg)
                        .border(1.0, border_color)
                        .border_radius(3.0)
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center),
                );
            }
            let _ = row.tree.add_child(row.parent, num_box_id);

            // Text inside Number Box
            let text_id = row.tree.create_node();
            if let Some(node) = row.tree.get_mut(text_id) {
                node.set_name("SliderNumberText");
                node.interactive = false;
                node.set_text(display_str);
                node.font_size = 11.0;
                node.line_height = 14.0;
                node.text_align = TextAlign::Center;
                node.text_color = if is_all_selected {
                    Color::rgba(1.0, 1.0, 1.0, 1.0)
                } else if is_editing {
                    Color::rgba(0.0, 0.90, 1.0, 1.0)
                } else {
                    Color::rgba(0.90, 0.92, 0.96, 1.0)
                };
            }
            let _ = row.tree.add_child(num_box_id, text_id);

            // Interaction resolution
            let (track_clicked, track_hovered, track_drag) = row.check_interaction(track_id);
            let (box_clicked, box_hovered, _) = row.check_interaction(num_box_id);
            let text_input = row
                .check_interaction_text(num_box_id)
                .or_else(|| row.check_interaction_text(track_id));

            let mut changed = false;

            if let Some(input_text) = text_input {
                let clean_text = input_text
                    .trim()
                    .trim_end_matches(['°', ' ', 'd', 'e', 'g', 'H', 'z', 'm']);
                if let Ok(parsed) = clean_text.parse::<f32>() {
                    let next = parsed.clamp(options.min, options.max);
                    if (next - *value).abs() > f32::EPSILON {
                        *value = next;
                        changed = true;
                    }
                }
            } else if let Some(delta) = track_drag {
                let next = (*value + delta.x * options.speed).clamp(options.min, options.max);
                if (next - *value).abs() > f32::EPSILON {
                    *value = next;
                    changed = true;
                }
            }

            let clicked = track_clicked || box_clicked;
            let hovered = track_hovered || box_hovered;
            let active_id = if box_clicked { num_box_id } else { track_id };

            response = WidgetResponse::new(active_id, clicked, hovered, changed);
        };

        if let Some(subtitle) = options.subtitle {
            self.container_named(
                "SliderWithSubContainer",
                Style::new()
                    .flex_col()
                    .gap(2.0)
                    .margin_insets(Insets::new(0.0, 0.0, 6.0, 0.0)),
                |col| {
                    col.container_named(
                        "SliderRow",
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .height(28.0),
                        row_content,
                    );

                    let sub_id = col.tree.create_node();
                    if let Some(node) = col.tree.get_mut(sub_id) {
                        node.set_name("SliderSubtitle");
                        node.set_text(subtitle);
                        node.font_size = 10.5;
                        node.line_height = 14.0;
                        node.text_align = TextAlign::Left;
                        node.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
                        node.set_style(
                            Style::new()
                                .padding_insets(Insets::new(0.0, 2.0, 0.0, 2.0))
                                .flex_shrink(0.0),
                        );
                    }
                    let _ = col.tree.add_child(col.parent, sub_id);
                },
            );
        } else {
            self.container_named(
                "SliderRow",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .height(28.0)
                    .margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0)),
                row_content,
            );
        }

        response
    }

    /// Emits an immediate two-way bound property slider row.
    ///
    /// Updates `*value` continuously when clicked or dragged across horizontal deltas,
    /// clamping the result within `[min, max]`.
    ///
    /// # Arguments
    /// * `label` - Property description displayed on the left (supports `##` separator).
    /// * `value` - Mutable reference to the floating-point value.
    /// * `min` - Minimum allowable scalar value.
    /// * `max` - Maximum allowable scalar value.
    /// * `speed` - Drag sensitivity multiplier applied per horizontal delta pixel.
    pub fn property_slider(
        &mut self,
        label: &str,
        value: &mut f32,
        min: f32,
        max: f32,
        speed: f32,
    ) -> WidgetResponse {
        self.property_slider_with_options(label, value, PropertySliderOptions::new(min, max, speed))
    }

    /// Emits a compact draggable float scalar input row.
    ///
    /// Modifies `value` in place based on continuous pointer drag deltas.
    ///
    /// # Arguments
    /// * `label` - Property description displayed on the left (supports `##` separator).
    /// * `value` - Mutable reference to the floating-point value.
    /// * `speed` - Drag sensitivity multiplier applied per horizontal delta pixel.
    /// * `min` - Minimum allowable scalar value.
    /// * `max` - Maximum allowable scalar value.
    pub fn property_drag_float(
        &mut self,
        label: &str,
        value: &mut f32,
        speed: f32,
        min: f32,
        max: f32,
    ) -> WidgetResponse {
        let tag = self.tag_for(label);
        let mut response = WidgetResponse::default();

        let row_id = self.property_row(label, |slot| {
            let drag_id = slot.tree.create_node();
            if let Some(node) = slot.tree.get_mut(drag_id) {
                node.set_name("PropertyDragFloat");
                node.interactive = true;
                node.role = WidgetRole::NumericInput;
                node.tag = tag;
                node.cursor = Some(WidgetCursor::EwResize);
                node.set_text(format!("{:.2}", *value));
                node.font_size = 11.0;
                node.line_height = 14.0;
                node.text_align = TextAlign::Center;
                node.text_color = Color::rgba(0.85, 0.88, 0.94, 1.0);
                node.set_style(
                    Style::new()
                        .width(62.0)
                        .height(20.0)
                        .background(Color::rgba(0.12, 0.13, 0.16, 0.95))
                        .border(1.0, Color::rgba(0.24, 0.26, 0.32, 0.80))
                        .border_radius(3.0)
                        .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center),
                );
            }
            let _ = slot.tree.add_child(slot.parent, drag_id);

            let (clicked, hovered, drag_delta) = slot.check_interaction(drag_id);
            let mut changed = false;
            if let Some(delta) = drag_delta {
                let next = (*value + delta.x * speed).clamp(min, max);
                if (next - *value).abs() > f32::EPSILON {
                    *value = next;
                    changed = true;
                }
            }

            response = WidgetResponse::new(drag_id, clicked, hovered, changed);
        });

        if response.id == WidgetId::default() {
            response.id = row_id;
        }
        response
    }
}