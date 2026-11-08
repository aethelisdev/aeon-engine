// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scalar Numeric Drag Builder & Response (`NumericDrag`, `ScalarResponse`)
//!
//! Provides the fluent [`NumericDrag`] widget builder implementing [`Widget`] and yielding
//! a rich [`ScalarResponse`] with direct in-place mutation, velocity-based scrubbing,
//! boundary clamping, optional keyboard edit state integration, and unit suffixes.

use std::ops::{Deref, RangeInclusive};

use iris_core::{
    AlignItems, Color, JustifyContent, Style, TextAlign, WidgetCursor, WidgetId, WidgetResponse,
    WidgetRole,
};

use crate::controls::numeric_drag::DEFAULT_PIXELS_PER_STEP;
pub use crate::controls::numeric_drag::DragCurve;
use crate::declarative::scope::core::UiScope;
use crate::declarative::widget::Widget;
use crate::numeric_input::NumericInputEditState;

/// Canonical default column width in logical pixels allocated for property labels.
///
/// Ensures all property rows across inspector cards align to a standard horizontal column boundary.
pub const DEFAULT_PROPERTY_LABEL_WIDTH: f32 = 105.0;

/// Comprehensive interaction response returned by evaluating a [`NumericDrag`] widget.
///
/// Implements [`Deref<Target = WidgetResponse>`] for transparent access to generic
/// interaction flags (`clicked()`, `hovered()`, `changed()`, `dragged()`) while providing
/// direct access to the mutated floating-point scalar value.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScalarResponse {
    /// Underlying generic widget response for the interactive numeric pill.
    pub response: WidgetResponse,
    /// Mutated scalar floating-point value after evaluation.
    pub value: f32,
    /// Pre-edit or baseline scalar value captured prior to the modification.
    pub old_value: f32,
}

impl Deref for ScalarResponse {
    type Target = WidgetResponse;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl ScalarResponse {
    /// Returns the evaluated floating-point scalar value.
    #[inline]
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.value
    }

    /// Returns the baseline or pre-edit floating-point scalar value before this interaction.
    #[inline]
    #[must_use]
    pub const fn old_value(&self) -> f32 {
        self.old_value
    }

    /// Returns `true` if the numeric value was confirmed or submitted via the Enter key during this frame.
    #[inline]
    #[must_use]
    pub const fn submitted(&self) -> bool {
        self.response
            .flags
            .contains(iris_core::response::ResponseFlags::SUBMITTED)
    }

    /// Returns `true` if the numeric value was committed during this frame
    /// (either by releasing pointer drag, pressing Enter to submit, or finalizing keyboard edit via blur).
    #[inline]
    #[must_use]
    pub const fn committed(&self) -> bool {
        self.response
            .flags
            .contains(iris_core::response::ResponseFlags::DRAG_RELEASED)
            || self
                .response
                .flags
                .contains(iris_core::response::ResponseFlags::SUBMITTED)
            || (self
                .response
                .flags
                .contains(iris_core::response::ResponseFlags::CHANGED)
                && self
                    .response
                    .flags
                    .contains(iris_core::response::ResponseFlags::LOST_FOCUS))
    }

    /// Invokes `f` with the newly committed scalar value if committed during this frame.
    #[inline]
    pub fn on_commit(self, f: impl FnOnce(f32)) -> Self {
        if self.committed() {
            f(self.value);
        }
        self
    }

    /// Invokes `f(old_val, new_val)` with the pre-edit and committed scalar values if committed
    /// with a non-trivial delta during this frame.
    #[inline]
    pub fn on_commit_change(self, f: impl FnOnce(f32, f32)) -> Self {
        if self.committed() && (self.value - self.old_value).abs() > f32::EPSILON {
            f(self.old_value, self.value);
        }
        self
    }

    /// Evaluates and returns `Some(f(old_val, new_val))` if committed with a non-trivial delta
    /// during this frame, or `None` otherwise.
    #[inline]
    pub fn map_commit<T>(self, f: impl FnOnce(f32, f32) -> T) -> Option<T> {
        (self.committed() && (self.value - self.old_value).abs() > f32::EPSILON)
            .then(|| f(self.old_value, self.value))
    }
}

/// Declarative floating-point property scrubber builder implementing [`Widget`].
///
/// Modifies `value` in place based on continuous pointer drag deltas, optional keyboard
/// inline editing, and boundary constraints.
///
/// # Example
/// ```ignore
/// if scope.add(NumericDrag::new("Volume", &mut volume).range(0.0..=1.0)).changed() {
///     // Handle volume changed
/// }
/// ```
pub struct NumericDrag<'a> {
    label: &'a str,
    value: &'a mut f32,
    speed: f32,
    range: RangeInclusive<f32>,
    decimals: usize,
    unit: Option<&'a str>,
    label_width: Option<f32>,
    box_width: Option<f32>,
    edit_state: Option<NumericInputEditState<'a>>,
    custom_text: Option<&'a str>,
    tag: Option<u64>,
    name: Option<&'a str>,
    curve: DragCurve,
    pixels_per_step: f32,
}

impl<'a> NumericDrag<'a> {
    /// Creates a new [`NumericDrag`] builder for the given label and mutable float reference.
    ///
    /// Defaults to `speed: 0.1`, `range: -10000.0..=10000.0`, `decimals: 2`,
    /// `label_width: None` (defaults to 55.0 when rendered), and `box_width: None` (defaults to 48.0).
    #[inline]
    pub fn new(label: &'a str, value: &'a mut f32) -> Self {
        Self {
            label,
            value,
            speed: 0.1,
            range: -10000.0..=10000.0,
            decimals: 2,
            unit: None,
            label_width: None,
            box_width: None,
            edit_state: None,
            custom_text: None,
            tag: None,
            name: None,
            curve: DragCurve::Linear,
            pixels_per_step: DEFAULT_PIXELS_PER_STEP as f32,
        }
    }

    /// Sets the non-linear response curve for pointer drag scrubbing.
    #[inline]
    #[must_use]
    pub fn curve(mut self, curve: DragCurve) -> Self {
        self.curve = curve;
        self
    }

    /// Sets the reference pixel displacement per step unit for non-linear curve evaluation.
    #[inline]
    #[must_use]
    pub fn pixels_per_step(mut self, pixels: f32) -> Self {
        self.pixels_per_step = pixels;
        self
    }

    /// Sets an explicit debug node name for the interactive drag pill in the UI tree.
    #[inline]
    #[must_use]
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the continuous drag sensitivity multiplier applied per horizontal delta pixel.
    #[inline]
    #[must_use]
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Constrains the scalar value within an inclusive range `[min, max]`.
    #[inline]
    #[must_use]
    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    /// Sets the minimum allowable value for the scalar.
    #[inline]
    #[must_use]
    pub fn min(mut self, min: f32) -> Self {
        self.range = min..=*self.range.end();
        self
    }

    /// Sets the maximum allowable value for the scalar.
    #[inline]
    #[must_use]
    pub fn max(mut self, max: f32) -> Self {
        self.range = *self.range.start()..=max;
        self
    }

    /// Sets the number of decimal places displayed for non-editing values.
    #[inline]
    #[must_use]
    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    /// Attaches an optional physical unit suffix string (e.g. `"m"`, `"s"`, `"Hz"`).
    #[inline]
    #[must_use]
    pub fn unit(mut self, unit: &'a str) -> Self {
        self.unit = Some(unit);
        self
    }

    /// Sets the column width in pixels for the left-aligned property label.
    #[inline]
    #[must_use]
    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = Some(width);
        self
    }

    /// Sets the fixed width in pixels for the draggable numeric pill box.
    #[inline]
    #[must_use]
    pub fn box_width(mut self, width: f32) -> Self {
        self.box_width = Some(width);
        self
    }

    /// Attaches an active inline text editing buffer and caret selection state.
    #[inline]
    #[must_use]
    pub fn edit_state(mut self, edit_state: Option<NumericInputEditState<'a>>) -> Self {
        self.edit_state = edit_state;
        self
    }

    /// Sets an explicit display string overriding the formatted numeric representation.
    #[inline]
    #[must_use]
    pub fn custom_text(mut self, custom_text: &'a str) -> Self {
        self.custom_text = Some(custom_text);
        self
    }

    /// Sets an explicit persistent 64-bit tag for the interactive numeric pill node.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Internal helper that renders and processes pointer scrubbing interactions for a numeric pill node.
    ///
    /// # Parameters
    /// - `scope`: UI tree scope hosting the pill node.
    /// - `pill_id`: Allocated node identifier for the interactive pill.
    /// - `tag`: Persistent 64-bit tag for interaction tracking and pointer capture.
    /// - `min_val`: Lower inclusive scalar constraint bound.
    /// - `max_val`: Upper inclusive scalar constraint bound.
    /// - `box_w`: Effective width of the pill input box in logical pixels.
    fn render_pill(
        &mut self,
        scope: &mut UiScope<'_>,
        pill_id: WidgetId,
        tag: u64,
        min_val: f32,
        max_val: f32,
        box_w: f32,
    ) -> (WidgetResponse, f32) {
        let initial_val = *self.value;
        if let Some(node) = scope.tree.get_mut(pill_id) {
            node.set_name(self.name.unwrap_or("NumericDrag"));
            node.tag = tag;
            node.interactive = true;
            node.role = WidgetRole::NumericInput;
            node.cursor = Some(WidgetCursor::Default);
        }

        let (clicked, hovered, drag_delta) = scope.check_interaction(pill_id);
        let is_pressed = scope.is_pressed(pill_id);
        let is_dragging = drag_delta.is_some()
            || (scope.tree.event_state.is_pointer_drag_active
                && (scope.tree.event_state.pointer_capture == Some(pill_id)
                    || (tag != 0 && scope.tree.event_state.pointer_capture_tag == Some(tag))
                    || (tag != 0
                        && scope
                            .tree
                            .event_state
                            .pointer_capture
                            .and_then(|cid| scope.tree.get(cid))
                            .is_some_and(|n| n.tag == tag))));

        if is_dragging && tag != 0 && scope.tree.event_state.pointer_capture_tag == Some(tag) {
            scope.tree.event_state.pointer_capture = Some(pill_id);
            scope.tree.event_state.pressed = Some(pill_id);
        }

        let edit_res = if self.edit_state.is_none() {
            let config =
                crate::declarative::scope::properties::numeric_input::NumericEditConfig::new(
                    min_val,
                    max_val,
                    self.decimals,
                );
            Some(
                crate::declarative::scope::properties::numeric_input::handle_numeric_edit_interaction(
                    scope,
                    pill_id,
                    tag,
                    self.value,
                    config,
                    clicked,
                ),
            )
        } else {
            None
        };

        let mut changed = false;
        let mut submitted = false;

        if self.edit_state.is_some()
            && let Some(restored) = scope.tree.event_state.numeric_undo.take_pending_scalar(tag)
        {
            let next = restored.clamp(min_val, max_val);
            if (next - *self.value).abs() > f32::EPSILON {
                *self.value = next;
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

        let is_editing = edit_res
            .as_ref()
            .map_or(self.edit_state.is_some(), |r| r.is_editing);

        if !is_editing {
            if let Some(text) = scope.check_interaction_text(pill_id) {
                let clean = text.trim();
                if let Some(parsed) = crate::declarative::types::eval_numeric_expression(clean) {
                    let next = parsed.clamp(min_val, max_val);
                    if (next - *self.value).abs() > f32::EPSILON {
                        scope
                            .tree
                            .event_state
                            .numeric_undo
                            .push_scalar(tag, *self.value, next);
                        *self.value = next;
                        changed = true;
                    }
                }
            } else if let Some(delta) = drag_delta {
                scope
                    .tree
                    .event_state
                    .numeric_undo
                    .begin_drag(tag, *self.value);
                let delta_val = self.curve.compute_delta_f32(
                    delta.x,
                    self.speed,
                    self.pixels_per_step,
                    &scope.tree.event_state,
                );
                let next = (*self.value + delta_val).clamp(min_val, max_val);
                if (next - *self.value).abs() > f32::EPSILON {
                    *self.value = next;
                    changed = true;
                }
            }
        }
        *self.value = self.value.clamp(min_val, max_val);

        let (display_str, is_all_selected) = if is_editing {
            if let Some(ref res) = edit_res {
                let is_all_sel = res
                    .selection
                    .is_some_and(|(s, e)| s == 0 && e == res.raw_text.len());
                (res.display_text.clone(), is_all_sel)
            } else if let Some(s) = self.edit_state {
                let buf = s.buffer;
                let cursor = s.cursor_idx.min(buf.len());
                let (left, right) = buf.split_at(cursor);
                let disp = if s.is_all_selected {
                    buf.to_owned()
                } else if s.blink_caret {
                    format!("{}|{}", left, right)
                } else {
                    buf.to_owned()
                };
                (disp, s.is_all_selected)
            } else {
                (String::new(), false)
            }
        } else if let Some(txt) = self.custom_text {
            (txt.to_owned(), false)
        } else {
            (
                crate::declarative::types::format_compact_float(*self.value, self.decimals),
                false,
            )
        };

        let (bg, border_col) = if is_editing {
            (
                Color::rgba(0.180, 0.190, 0.220, 1.0),
                Color::rgba(0.85, 0.88, 0.98, 0.95),
            )
        } else if is_dragging {
            (
                Color::rgba(0.20, 0.28, 0.38, 0.98),
                Color::rgba(0.35, 0.75, 0.95, 0.90),
            )
        } else if hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
            )
        };

        let text_len = display_str.len();
        if let Some(node) = scope.tree.get_mut(pill_id) {
            node.set_name(self.name.unwrap_or("NumericDrag"));
            node.interactive = true;
            node.role = WidgetRole::NumericInput;
            node.tag = tag;
            node.cursor = Some(if is_dragging {
                WidgetCursor::EwResize
            } else if is_editing {
                WidgetCursor::Text
            } else {
                WidgetCursor::Default
            });
            node.set_text(display_str);
            node.font_size = 10.5;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if is_editing || hovered {
                Color::WHITE
            } else {
                Color::rgba(0.85, 0.88, 0.94, 1.0)
            };

            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .height(22.0)
                .background(bg)
                .border(1.0, border_col)
                .border_radius(4.0);
            if let Some(w) = self.box_width {
                style.width = Some(w);
            }
            node.set_style(style);
        }

        if is_all_selected {
            let approx_char_w = 6.2;
            let val_w = (text_len as f32 * approx_char_w).max(12.0);
            let w = box_w;
            let sel_x = ((w - val_w) * 0.5 - 2.0).clamp(2.0, w - 6.0);
            let sel_w = (val_w + 4.0).min(w - 2.0 - sel_x);
            let sel_style = Style::new()
                .position_absolute()
                .left(sel_x)
                .top(3.0)
                .width(sel_w)
                .height(16.0)
                .background(Color::rgba(0.14, 0.46, 0.88, 0.95))
                .border_radius(3.0);
            let mut pill_scope = scope.child_scope(pill_id);
            pill_scope.empty_box_passive_named("Numeric_Sel", sel_style);
        }

        let is_drag_released = (scope.tree.event_state.frame_drag_released_tag == Some(tag)
            && tag != 0)
            || scope.tree.event_state.frame_drag_released_id == Some(pill_id);

        let mut resp = WidgetResponse::new(pill_id, clicked, hovered, changed);
        if is_pressed {
            resp = resp.with_pressed(true);
        }
        if is_dragging {
            resp = resp.with_dragged(true);
        }
        let old_val = if is_drag_released {
            let (_, start_opt) = scope
                .tree
                .event_state
                .numeric_undo
                .commit_drag_with_start(tag, *self.value);
            resp = resp.with_drag_released(true);
            if !is_editing {
                scope.tree.event_state.focused = Some(pill_id);
                if tag != 0 {
                    scope.tree.event_state.focused_tag = Some(tag);
                }
            }
            start_opt.unwrap_or(initial_val)
        } else {
            scope
                .tree
                .event_state
                .numeric_undo
                .active_drag_start(tag)
                .unwrap_or(initial_val)
        };
        if submitted {
            resp.flags = resp
                .flags
                .union(iris_core::response::ResponseFlags::LOST_FOCUS)
                .union(iris_core::response::ResponseFlags::SUBMITTED);
        }

        (resp, old_val)
    }
}

impl<'a> Widget for NumericDrag<'a> {
    type Response = ScalarResponse;

    fn ui(mut self, scope: &mut UiScope<'_>) -> Self::Response {
        let (visible_label, _) = crate::declarative::types::split_label_id(self.label);
        let tag = self.tag.unwrap_or_else(|| scope.tag_for(self.label));
        let min_val = *self.range.start();
        let max_val = *self.range.end();
        let box_w = self.box_width.unwrap_or(48.0);

        // When label is empty, render just the numeric pill directly without an outer row container
        if visible_label.is_empty() {
            let pill_id = scope.tree.create_node();
            let _ = scope.tree.add_child(scope.parent, pill_id);
            let (resp, old_val) = self.render_pill(scope, pill_id, tag, min_val, max_val, box_w);
            return ScalarResponse {
                response: resp,
                value: *self.value,
                old_value: old_val,
            };
        }

        // When label is not empty, build standard property row with label + pill (+ optional unit)
        let label_w = self.label_width.unwrap_or(DEFAULT_PROPERTY_LABEL_WIDTH);
        let gap_w = 4.0;
        let mut row_w = label_w + gap_w + box_w;
        if self.unit.is_some() {
            row_w += gap_w + 35.0;
        }

        let row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .width(row_w)
            .height(22.0)
            .gap(gap_w);

        let row_id = scope.tree.create_node();
        if let Some(node) = scope.tree.get_mut(row_id) {
            node.set_name("NumericRow");
            node.set_style(row_style);
        }
        let _ = scope.tree.add_child(scope.parent, row_id);

        let mut row = scope.child_scope(row_id);

        // 1. Left Label
        let label_style = Style::new().width(label_w).height(22.0).flex_shrink(0.0);
        row.label_styled_passive(
            "NumLbl",
            visible_label,
            11.0,
            row.theme().text_muted,
            TextAlign::Left,
            label_style,
        );

        // 2. Pill Node
        let pill_id = row.tree.create_node();
        let _ = row.tree.add_child(row.parent, pill_id);
        let (resp, old_val) = self.render_pill(&mut row, pill_id, tag, min_val, max_val, box_w);

        // 3. Optional Unit Suffix
        if let Some(unit_str) = self.unit {
            row.label_styled_passive(
                "NumUnit",
                unit_str,
                11.0,
                row.theme().text_muted,
                TextAlign::Left,
                Style::new().width(35.0).height(22.0),
            );
        }

        ScalarResponse {
            response: resp,
            value: *self.value,
            old_value: old_val,
        }
    }
}

#[cfg(test)]
mod tests;