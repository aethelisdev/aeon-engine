// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Continuous Property Sliders & Scalar Adjusters
//!
//! Provides proportional draggable sliders, scalar inputs, and fine-tuning number boxes on [`UiScope`].

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::{WidgetResponse, split_label_id};
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetRole,
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

        let active_in_scope = self.active_text_input.and_then(|(t, text, sel)| {
            if t == num_box_tag {
                Some((text, sel))
            } else {
                None
            }
        });

        let track_w = 240.0;

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
                node.tag = tag;
            }
            let _ = row.tree.add_child(row.parent, track_id);

            // 3. Right Numeric Input Pill Box
            let num_box_id = row.tree.create_node();
            if let Some(node) = row.tree.get_mut(num_box_id) {
                node.tag = num_box_tag;
            }
            let _ = row.tree.add_child(row.parent, num_box_id);

            // Interaction resolution
            let (track_clicked, track_hovered, track_drag) = row.check_interaction(track_id);
            let (box_clicked, box_hovered, _) = row.check_interaction(num_box_id);

            let edit_res = if options.inline_edit.is_none() {
                let config =
                    super::numeric_input::NumericEditConfig::new(options.min, options.max, 2);
                Some(super::numeric_input::handle_numeric_edit_interaction(
                    row,
                    num_box_id,
                    num_box_tag,
                    value,
                    config,
                    box_clicked,
                ))
            } else {
                None
            };

            let mut changed = false;
            let mut submitted = false;
            if let Some(restored) = row.tree.event_state.numeric_undo.take_pending_scalar(tag) {
                let next = restored.clamp(options.min, options.max);
                if (next - *value).abs() > f32::EPSILON {
                    *value = next;
                    changed = true;
                }
                submitted = true;
            }
            if let Some(ref res) = edit_res {
                if res.value_changed {
                    changed = true;
                }
                if res.submitted {
                    submitted = true;
                }
            }

            let is_editing_now = edit_res
                .as_ref()
                .map_or(options.inline_edit.is_some(), |r| r.is_editing);
            let is_all_sel_now = edit_res.as_ref().is_some_and(|r| {
                r.selection
                    .is_some_and(|(s, e)| s == 0 && e == r.raw_text.len())
            }) || active_in_scope.is_some_and(|(_, sel)| sel);

            let is_track_captured = row.tree.event_state.pointer_capture == Some(track_id)
                || (tag != 0 && row.tree.event_state.pointer_capture_tag == Some(tag));
            let is_track_pressed = is_track_captured
                && row.tree.event_state.pressed_button == Some(iris_core::MouseButton::Left);

            if !is_editing_now {
                let text_input = row
                    .check_interaction_text(num_box_id)
                    .or_else(|| row.check_interaction_text(track_id));

                if let Some(input_text) = text_input {
                    let clean_text = input_text
                        .trim()
                        .trim_end_matches(['°', ' ', 'd', 'e', 'g', 'H', 'z', 'm']);
                    if let Some(parsed) =
                        crate::declarative::types::eval_numeric_expression(clean_text)
                    {
                        let next = parsed.clamp(options.min, options.max);
                        if (next - *value).abs() > f32::EPSILON {
                            row.tree.event_state.numeric_undo.push_scalar(
                                num_box_tag,
                                *value,
                                next,
                            );
                            *value = next;
                            changed = true;
                        }
                    }
                } else if is_track_pressed || track_clicked {
                    row.tree.event_state.numeric_undo.begin_drag(tag, *value);
                    let track_rect = if tag != 0
                        && let Some(r) = row.tree.tag_rect(tag)
                        && r.width > 10.0
                    {
                        r
                    } else if let Some(r) = row.tree.event_state.frame_clicked_rect
                        && r.width > 10.0
                    {
                        r
                    } else {
                        row.tree
                            .get(track_id)
                            .map(|n| n.computed_rect)
                            .unwrap_or(iris_core::geometry::Rect::ZERO)
                    };
                    if track_rect.width > 10.0 {
                        let click_pos = row
                            .tree
                            .event_state
                            .last_pointer_pos
                            .or(row.tree.event_state.press_origin)
                            .unwrap_or(iris_core::geometry::Point::new(track_rect.x, track_rect.y));
                        let norm = ((click_pos.x - (track_rect.x + 2.0))
                            / (track_rect.width - 4.0))
                            .clamp(0.0, 1.0);
                        let next = options.min + norm * (options.max - options.min);
                        if (next - *value).abs() > f32::EPSILON {
                            if track_clicked && !is_track_pressed {
                                row.tree.event_state.numeric_undo.commit_drag(tag, next);
                            }
                            *value = next;
                            changed = true;
                        }
                    } else if let Some(delta) = track_drag {
                        let next =
                            (*value + delta.x * options.speed).clamp(options.min, options.max);
                        if (next - *value).abs() > f32::EPSILON {
                            *value = next;
                            changed = true;
                        }
                    }
                } else if let Some(delta) = track_drag {
                    row.tree.event_state.numeric_undo.begin_drag(tag, *value);
                    let next = (*value + delta.x * options.speed).clamp(options.min, options.max);
                    if (next - *value).abs() > f32::EPSILON {
                        *value = next;
                        changed = true;
                    }
                }
            }

            let norm_now = ((*value - options.min) / (options.max - options.min)).clamp(0.0, 1.0);
            let current_fill_w = (norm_now * (track_w - 4.0)).clamp(0.0, track_w - 4.0);

            let display_str_now = if let Some(edit) = options.inline_edit {
                edit.to_owned()
            } else if let Some(ref res) = edit_res {
                if res.is_editing {
                    res.display_text.clone()
                } else if let Some((active_text, _)) = active_in_scope {
                    active_text.to_owned()
                } else if let Some(fmt) = options.format {
                    if fmt.contains("Hz") {
                        format!("{:.0} Hz", *value)
                    } else if fmt.contains('m') {
                        format!("{:.0} m", *value)
                    } else if fmt.contains("dB") {
                        format!("{:.1} dB", *value)
                    } else if fmt.contains('%') {
                        format!("{:.0}%", *value * 100.0)
                    } else if fmt.contains("deg") || fmt.contains('°') {
                        format!("{:.1}°", *value)
                    } else {
                        crate::declarative::types::format_compact_float(*value, 2)
                    }
                } else {
                    crate::declarative::types::format_compact_float(*value, 2)
                }
            } else {
                crate::declarative::types::format_compact_float(*value, 2)
            };

            let is_track_dragging = track_drag.is_some()
                || (row.tree.event_state.is_pointer_drag_active
                    && (row.tree.event_state.pointer_capture == Some(track_id)
                        || (tag != 0 && row.tree.event_state.pointer_capture_tag == Some(tag))));

            if let Some(node) = row.tree.get_mut(track_id) {
                node.set_name("SliderTrack");
                node.interactive = true;
                node.role = WidgetRole::NumericInput;
                node.tag = tag;
                node.cursor = Some(if is_track_dragging {
                    WidgetCursor::EwResize
                } else {
                    WidgetCursor::Default
                });
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

            // Proportional Progress Fill inside Track
            let fill_id = row.tree.create_node();
            if let Some(node) = row.tree.get_mut(fill_id) {
                node.set_name("SliderFill");
                node.interactive = false;
                node.set_style(
                    Style::new()
                        .width(current_fill_w)
                        .height(12.0)
                        .background(Color::rgba(0.0, 0.65, 0.85, 0.75))
                        .border_radius(2.0),
                );
            }
            let _ = row.tree.add_child(track_id, fill_id);

            let box_bg = if is_all_sel_now {
                Color::rgba(0.0, 0.40, 0.70, 0.85)
            } else if is_editing_now {
                Color::rgba(0.06, 0.10, 0.16, 0.95)
            } else {
                Color::rgba(0.12, 0.13, 0.17, 0.85)
            };
            let border_color = if is_all_sel_now {
                Color::rgba(0.0, 0.85, 1.0, 1.0)
            } else if is_editing_now {
                Color::rgba(0.0, 0.75, 1.0, 0.90)
            } else {
                Color::rgba(0.22, 0.25, 0.34, 0.70)
            };

            if let Some(node) = row.tree.get_mut(num_box_id) {
                node.set_name("SliderNumberBox");
                node.interactive = true;
                node.role = WidgetRole::NumericInput;
                node.tag = num_box_tag;
                node.cursor = Some(if is_editing_now {
                    WidgetCursor::Text
                } else {
                    WidgetCursor::Default
                });
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

            // Text inside Number Box
            let text_id = row.tree.create_node();
            if let Some(node) = row.tree.get_mut(text_id) {
                node.set_name("SliderNumberText");
                node.interactive = false;
                node.set_text(display_str_now);
                node.font_size = 11.0;
                node.line_height = 14.0;
                node.text_align = TextAlign::Center;
                node.text_color = if is_all_sel_now {
                    Color::rgba(1.0, 1.0, 1.0, 1.0)
                } else if is_editing_now {
                    Color::rgba(0.0, 0.90, 1.0, 1.0)
                } else {
                    Color::rgba(0.90, 0.92, 0.96, 1.0)
                };
            }
            let _ = row.tree.add_child(num_box_id, text_id);

            let clicked = track_clicked || box_clicked;
            let hovered = track_hovered || box_hovered;
            let active_id = if box_clicked { num_box_id } else { track_id };

            let is_drag_released = (row.tree.event_state.frame_drag_released_tag == Some(tag)
                && tag != 0)
                || row.tree.event_state.frame_drag_released_id == Some(track_id)
                || row.tree.event_state.frame_drag_released_id == Some(num_box_id);

            let mut resp = WidgetResponse::new(active_id, clicked, hovered, changed);
            if is_drag_released {
                row.tree.event_state.numeric_undo.commit_drag(tag, *value);
                resp = resp.with_drag_released(true);
            }
            if submitted {
                resp.flags = resp
                    .flags
                    .union(iris_core::response::ResponseFlags::LOST_FOCUS)
                    .union(iris_core::response::ResponseFlags::SUBMITTED);
            }
            response = resp;
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
        let mut slot_executed = false;

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
            if let Some(restored) = slot.tree.event_state.numeric_undo.take_pending_scalar(tag) {
                let next = restored.clamp(min, max);
                if (next - *value).abs() > f32::EPSILON {
                    *value = next;
                    changed = true;
                }
            }
            if let Some(delta) = drag_delta {
                slot.tree.event_state.numeric_undo.begin_drag(tag, *value);
                let next = (*value + delta.x * speed).clamp(min, max);
                if (next - *value).abs() > f32::EPSILON {
                    *value = next;
                    changed = true;
                }
            }
            let is_drag_released = (slot.tree.event_state.frame_drag_released_tag == Some(tag)
                && tag != 0)
                || slot.tree.event_state.frame_drag_released_id == Some(drag_id);
            if is_drag_released {
                slot.tree.event_state.numeric_undo.commit_drag(tag, *value);
            }

            response = WidgetResponse::new(drag_id, clicked, hovered, changed);
            slot_executed = true;
        });

        if !slot_executed {
            response = self.response(row_id);
        }
        response
    }

    /// Emits a declarative floating-point property scrubber row using canonical label alignment.
    ///
    /// Modifies `value` in-place, clamping strictly within `range` and applying `speed` velocity
    /// multiplier per pointer drag displacement pixel. Defaults to [`crate::declarative::widget::DEFAULT_PROPERTY_LABEL_WIDTH`].
    ///
    /// # Arguments
    /// * `label` - Left column title caption displayed on the property row.
    /// * `value` - Mutable reference to the floating-point scalar value modified in-place upon interaction.
    /// * `range` - Inclusive numeric boundary constraints governing valid scalar values.
    /// * `speed` - Drag sensitivity multiplier applied per pointer displacement pixel.
    pub fn prop_f32(
        &mut self,
        label: &str,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        speed: f32,
    ) -> crate::declarative::widget::ScalarResponse {
        self.prop_f32_styled(label, value, range, speed, None, None)
    }

    /// Emits a declarative floating-point property scrubber row with explicit label width and input box width.
    ///
    /// Evaluates continuous horizontal drag scrubbing with custom velocity, boundary bounds, and explicit sizing,
    /// yielding a [`crate::declarative::widget::ScalarResponse`] with direct access to `.changed()` and `.committed()` signals.
    ///
    /// # Arguments
    /// * `label` - Left column title caption displayed on the property row.
    /// * `value` - Mutable reference to the floating-point scalar value modified in-place upon interaction.
    /// * `range` - Inclusive numeric boundary constraints governing valid scalar values.
    /// * `speed` - Drag sensitivity multiplier applied per pointer displacement pixel.
    /// * `label_width` - Optional column width in pixels for the left-aligned property label.
    /// * `box_width` - Optional fixed width in pixels for the draggable numeric pill box.
    pub fn prop_f32_styled(
        &mut self,
        label: &str,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        speed: f32,
        label_width: Option<f32>,
        box_width: Option<f32>,
    ) -> crate::declarative::widget::ScalarResponse {
        let tag = self.tag_for(label);
        let mut builder = crate::declarative::widget::NumericDrag::new(label, value)
            .range(range)
            .speed(speed)
            .tag(tag);

        if let Some(lw) = label_width {
            builder = builder.label_width(lw);
        }
        if let Some(bw) = box_width {
            builder = builder.box_width(bw);
        }

        self.add(builder)
    }

    /// Emits a declarative integer property scrubber row modifying `value` in-place.
    ///
    /// Evaluates continuous horizontal drag scrubbing with discrete integer increments (`decimals: 0`),
    /// strictly clamping within `range`. Defaults to [`crate::declarative::widget::DEFAULT_PROPERTY_LABEL_WIDTH`].
    ///
    /// # Arguments
    /// * `label` - Left column title caption displayed on the property row.
    /// * `value` - Mutable reference to the 32-bit signed integer value modified in-place upon interaction.
    /// * `range` - Inclusive integer boundary constraints governing valid scalar values.
    pub fn prop_i32(
        &mut self,
        label: &str,
        value: &mut i32,
        range: std::ops::RangeInclusive<i32>,
    ) -> crate::declarative::widget::ScalarResponse {
        self.prop_i32_styled(label, value, range, None, None)
    }

    /// Emits a declarative integer property scrubber row with explicit label width and input box width.
    ///
    /// Evaluates continuous horizontal drag scrubbing with discrete integer increments (`decimals: 0`),
    /// strictly clamping within `range` and applying explicit layout widths.
    ///
    /// # Arguments
    /// * `label` - Left column title caption displayed on the property row.
    /// * `value` - Mutable reference to the 32-bit signed integer value modified in-place upon interaction.
    /// * `range` - Inclusive integer boundary constraints governing valid scalar values.
    /// * `label_width` - Optional column width in pixels for the left-aligned property label.
    /// * `box_width` - Optional fixed width in pixels for the draggable numeric pill box.
    pub fn prop_i32_styled(
        &mut self,
        label: &str,
        value: &mut i32,
        range: std::ops::RangeInclusive<i32>,
        label_width: Option<f32>,
        box_width: Option<f32>,
    ) -> crate::declarative::widget::ScalarResponse {
        let tag = self.tag_for(label);
        let mut f_val = *value as f32;
        let range_f = (*range.start() as f32)..=(*range.end() as f32);

        let mut builder = crate::declarative::widget::NumericDrag::new(label, &mut f_val)
            .range(range_f)
            .speed(1.0)
            .decimals(0)
            .tag(tag);

        if let Some(lw) = label_width {
            builder = builder.label_width(lw);
        }
        if let Some(bw) = box_width {
            builder = builder.box_width(bw);
        }

        let resp = self.add(builder);
        if resp.changed() || resp.committed() {
            *value = f_val.round() as i32;
        }

        resp
    }
}