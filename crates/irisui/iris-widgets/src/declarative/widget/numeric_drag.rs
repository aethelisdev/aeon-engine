// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scalar Numeric Drag Builder & Response (`NumericDrag`, `ScalarResponse`)
//!
//! Provides the fluent [`NumericDrag`] widget builder implementing [`Widget`] and yielding
//! a rich [`ScalarResponse`] with direct in-place mutation, velocity-based scrubbing,
//! boundary clamping, optional keyboard edit state integration, and unit suffixes.

use std::ops::{Deref, RangeInclusive};

use iris_core::{
    AlignItems, Color, JustifyContent, Style, TextAlign, WidgetCursor, WidgetResponse, WidgetRole,
};

use crate::declarative::scope::core::UiScope;
use crate::declarative::widget::Widget;
use crate::numeric_input::NumericInputEditState;

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
        }
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
}

impl<'a> Widget for NumericDrag<'a> {
    type Response = ScalarResponse;

    fn ui(self, scope: &mut UiScope<'_>) -> Self::Response {
        let (visible_label, _) = crate::declarative::types::split_label_id(self.label);
        let tag = self.tag.unwrap_or_else(|| scope.tag_for(self.label));

        // When label is empty, render just the numeric pill directly without an outer row container
        if visible_label.is_empty() {
            let pill_id = scope.tree.create_node();
            let _ = scope.tree.add_child(scope.parent, pill_id);
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

            if clicked {
                let is_already_editing = scope
                    .tree
                    .event_state
                    .active_numeric_edit
                    .as_ref()
                    .is_some_and(|(t, ..)| *t == tag);
                if is_already_editing {
                    if let Some((_, ref buf, ref mut cursor, ref mut is_all_sel)) =
                        scope.tree.event_state.active_numeric_edit
                    {
                        *is_all_sel = false;
                        *cursor = buf.len();
                    }
                    scope.tree.event_state.focused = Some(pill_id);
                    if tag != 0 {
                        scope.tree.event_state.focused_tag = Some(tag);
                    }
                } else {
                    let initial_buf =
                        crate::declarative::types::format_compact_float(*self.value, self.decimals);
                    let len = initial_buf.len();
                    scope.tree.event_state.start_numeric_edit(
                        Some(pill_id),
                        tag,
                        initial_buf,
                        len,
                        true,
                    );
                }
            }

            let mut changed = false;
            let mut lost_focus = false;
            let mut submitted = false;
            let min_val = *self.range.start();
            let max_val = *self.range.end();

            let is_editing_this_pill = scope
                .tree
                .event_state
                .active_numeric_edit
                .as_ref()
                .is_some_and(|(t, ..)| *t == tag);

            if is_editing_this_pill {
                scope.tree.event_state.focused = Some(pill_id);
                if tag != 0 {
                    scope.tree.event_state.focused_tag = Some(tag);
                }
            }

            let is_clicked_outside =
                if let Some(clicked_tag) = scope.tree.event_state.frame_clicked_tag {
                    clicked_tag != tag
                } else if let Some(clicked_id) = scope.tree.event_state.frame_clicked_id {
                    clicked_id != pill_id
                } else {
                    false
                };

            if is_editing_this_pill && is_clicked_outside {
                if let Some((_, ref buf, ..)) = scope.tree.event_state.active_numeric_edit
                    && let Some(parsed) = crate::declarative::types::eval_numeric_expression(buf)
                {
                    let next = parsed.clamp(min_val, max_val);
                    if (next - *self.value).abs() > f32::EPSILON {
                        *self.value = next;
                        changed = true;
                    }
                }
                scope.tree.event_state.active_numeric_edit = None;
                if scope.tree.event_state.focused_tag == Some(tag) {
                    scope.tree.event_state.clear_focus();
                }
                lost_focus = true;
            }

            let is_numeric_focused = scope.is_focused(pill_id)
                || (tag != 0 && scope.tree.event_state.focused_tag == Some(tag))
                || scope
                    .tree
                    .event_state
                    .active_numeric_edit
                    .as_ref()
                    .is_some_and(|(t, ..)| *t == tag);

            if is_numeric_focused
                && let Some((_, ref mut buf, ref mut cursor, ref mut is_all_sel)) =
                    scope.tree.event_state.active_numeric_edit
            {
                let is_ctrl_key = scope.tree.event_state.frame_key_pressed
                    && scope.tree.event_state.frame_key.is_some_and(|k| {
                        matches!(
                            k,
                            iris_core::event::KeyCode::ArrowLeft
                                | iris_core::event::KeyCode::ArrowRight
                                | iris_core::event::KeyCode::Home
                                | iris_core::event::KeyCode::End
                                | iris_core::event::KeyCode::Backspace
                                | iris_core::event::KeyCode::Delete
                                | iris_core::event::KeyCode::Enter
                                | iris_core::event::KeyCode::Escape
                        )
                    });

                if is_ctrl_key {
                    if let Some(key) = scope.tree.event_state.frame_key {
                        match key {
                            iris_core::event::KeyCode::ArrowLeft => {
                                if *is_all_sel {
                                    *is_all_sel = false;
                                    *cursor = 0;
                                } else if *cursor > 0 {
                                    *cursor -= 1;
                                }
                            }
                            iris_core::event::KeyCode::ArrowRight => {
                                if *is_all_sel {
                                    *is_all_sel = false;
                                    *cursor = buf.len();
                                } else if *cursor < buf.len() {
                                    *cursor += 1;
                                }
                            }
                            iris_core::event::KeyCode::Home => {
                                *is_all_sel = false;
                                *cursor = 0;
                            }
                            iris_core::event::KeyCode::End => {
                                *is_all_sel = false;
                                *cursor = buf.len();
                            }
                            iris_core::event::KeyCode::Backspace => {
                                if *is_all_sel {
                                    buf.clear();
                                    *cursor = 0;
                                    *is_all_sel = false;
                                } else if *cursor > 0 && *cursor <= buf.len() {
                                    buf.remove(*cursor - 1);
                                    *cursor -= 1;
                                }
                            }
                            iris_core::event::KeyCode::Delete => {
                                if *is_all_sel {
                                    buf.clear();
                                    *cursor = 0;
                                    *is_all_sel = false;
                                } else if *cursor < buf.len() {
                                    buf.remove(*cursor);
                                }
                            }
                            iris_core::event::KeyCode::Enter => {
                                if let Some(parsed) =
                                    crate::declarative::types::eval_numeric_expression(buf)
                                {
                                    let next = parsed.clamp(min_val, max_val);
                                    *self.value = next;
                                    changed = true;
                                }
                                scope.tree.event_state.active_numeric_edit = None;
                                scope.tree.event_state.clear_focus();
                                lost_focus = true;
                                submitted = true;
                            }
                            iris_core::event::KeyCode::Escape => {
                                scope.tree.event_state.active_numeric_edit = None;
                                scope.tree.event_state.clear_focus();
                            }
                            _ => {}
                        }
                    }
                } else if scope.tree.event_state.frame_key_pressed
                    && let Some(ref text) = scope.tree.event_state.frame_text_input
                {
                    let filtered: String = text
                        .chars()
                        .filter(|c| {
                            c.is_ascii_digit()
                                || *c == '.'
                                || *c == '-'
                                || *c == '+'
                                || *c == '*'
                                || *c == '/'
                                || *c == '('
                                || *c == ')'
                                || *c == ' '
                        })
                        .collect();
                    if !filtered.is_empty() {
                        if *is_all_sel {
                            *buf = filtered;
                            *cursor = buf.len();
                            *is_all_sel = false;
                        } else {
                            let idx = (*cursor).min(buf.len());
                            buf.insert_str(idx, &filtered);
                            *cursor += filtered.len();
                        }
                    }
                }
            }

            let is_currently_editing = is_editing_this_pill
                || scope
                    .tree
                    .event_state
                    .active_numeric_edit
                    .as_ref()
                    .is_some_and(|(t, ..)| *t == tag);

            if !is_currently_editing {
                if let Some(text) = scope.check_interaction_text(pill_id) {
                    let clean = text.trim();
                    if let Some(parsed) = crate::declarative::types::eval_numeric_expression(clean)
                    {
                        let next = parsed.clamp(min_val, max_val);
                        if (next - *self.value).abs() > f32::EPSILON {
                            *self.value = next;
                            changed = true;
                        }
                    }
                } else if let Some(delta) = drag_delta {
                    let next = (*self.value + delta.x * self.speed).clamp(min_val, max_val);
                    if (next - *self.value).abs() > f32::EPSILON {
                        *self.value = next;
                        changed = true;
                    }
                }
            }
            *self.value = self.value.clamp(min_val, max_val);

            let active_in_scope =
                scope.active_text_input.and_then(
                    |(t, text, sel)| {
                        if t == tag { Some((text, sel)) } else { None }
                    },
                );

            let active_numeric = scope
                .tree
                .event_state
                .active_numeric_edit
                .as_ref()
                .filter(|(t, ..)| *t == tag);

            let is_editing =
                self.edit_state.is_some() || active_in_scope.is_some() || active_numeric.is_some();
            let is_all_selected = self.edit_state.as_ref().is_some_and(|s| s.is_all_selected)
                || active_in_scope.as_ref().is_some_and(|(_, sel)| *sel)
                || active_numeric.as_ref().is_some_and(|(.., sel)| *sel);

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

            let display_str = if let Some(s) = self.edit_state {
                let buf = s.buffer;
                let cursor = s.cursor_idx.min(buf.len());
                let (left, right) = buf.split_at(cursor);
                if s.is_all_selected {
                    buf.to_string()
                } else if s.blink_caret {
                    format!("{}|{}", left, right)
                } else {
                    buf.to_string()
                }
            } else if let Some((_, buf, cursor, is_all_sel)) = active_numeric {
                if *is_all_sel {
                    buf.clone()
                } else {
                    let c = (*cursor).min(buf.len());
                    let (left, right) = buf.split_at(c);
                    let blink_caret = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(true, |d| (d.as_millis() / 500).is_multiple_of(2));
                    if blink_caret {
                        format!("{}|{}", left, right)
                    } else {
                        buf.clone()
                    }
                }
            } else if let Some((active_text, _)) = active_in_scope {
                active_text.to_string()
            } else if let Some(txt) = self.custom_text {
                txt.to_string()
            } else {
                crate::declarative::types::format_compact_float(*self.value, self.decimals)
            };

            let text_len = display_str.len();
            if let Some(node) = scope.tree.get_mut(pill_id) {
                node.set_name("NumericDrag");
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
                let w = self.box_width.unwrap_or(48.0);
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
            if is_drag_released {
                resp = resp.with_drag_released(true);
            }
            if lost_focus {
                resp.flags = resp
                    .flags
                    .union(iris_core::response::ResponseFlags::LOST_FOCUS);
            }
            if submitted {
                resp.flags = resp
                    .flags
                    .union(iris_core::response::ResponseFlags::SUBMITTED);
            }
            return ScalarResponse {
                response: resp,
                value: *self.value,
            };
        }

        // When label is not empty, build standard property row with label + pill (+ optional unit)
        let label_w = self.label_width.unwrap_or(55.0);
        let box_w = self.box_width.unwrap_or(48.0);
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
        if let Some(node) = row.tree.get_mut(pill_id) {
            node.set_name(self.name.unwrap_or("NumericDrag"));
            node.tag = tag;
            node.interactive = true;
            node.role = WidgetRole::NumericInput;
            node.cursor = Some(WidgetCursor::Default);
        }

        let (clicked, hovered, drag_delta) = row.check_interaction(pill_id);
        let is_pressed = row.is_pressed(pill_id);
        let is_dragging = drag_delta.is_some()
            || (row.tree.event_state.is_pointer_drag_active
                && (row.tree.event_state.pointer_capture == Some(pill_id)
                    || (tag != 0 && row.tree.event_state.pointer_capture_tag == Some(tag))
                    || (tag != 0
                        && row
                            .tree
                            .event_state
                            .pointer_capture
                            .and_then(|cid| row.tree.get(cid))
                            .is_some_and(|n| n.tag == tag))));

        if is_dragging && tag != 0 && row.tree.event_state.pointer_capture_tag == Some(tag) {
            row.tree.event_state.pointer_capture = Some(pill_id);
            row.tree.event_state.pressed = Some(pill_id);
        }

        if clicked {
            let is_already_editing = row
                .tree
                .event_state
                .active_numeric_edit
                .as_ref()
                .is_some_and(|(t, ..)| *t == tag);
            if is_already_editing {
                if let Some((_, ref buf, ref mut cursor, ref mut is_all_sel)) =
                    row.tree.event_state.active_numeric_edit
                {
                    *is_all_sel = false;
                    *cursor = buf.len();
                }
                row.tree.event_state.focused = Some(pill_id);
                if tag != 0 {
                    row.tree.event_state.focused_tag = Some(tag);
                }
            } else {
                let initial_buf =
                    crate::declarative::types::format_compact_float(*self.value, self.decimals);
                let len = initial_buf.len();
                row.tree
                    .event_state
                    .start_numeric_edit(Some(pill_id), tag, initial_buf, len, true);
            }
        }

        let mut changed = false;
        let mut lost_focus = false;
        let mut submitted = false;
        let min_val = *self.range.start();
        let max_val = *self.range.end();

        let is_editing_this_pill = row
            .tree
            .event_state
            .active_numeric_edit
            .as_ref()
            .is_some_and(|(t, ..)| *t == tag);

        if is_editing_this_pill {
            row.tree.event_state.focused = Some(pill_id);
            if tag != 0 {
                row.tree.event_state.focused_tag = Some(tag);
            }
        }

        let is_clicked_outside = if let Some(clicked_tag) = row.tree.event_state.frame_clicked_tag {
            clicked_tag != tag
        } else if let Some(clicked_id) = row.tree.event_state.frame_clicked_id {
            clicked_id != pill_id
        } else {
            false
        };

        if is_editing_this_pill && is_clicked_outside {
            if let Some((_, ref buf, ..)) = row.tree.event_state.active_numeric_edit
                && let Some(parsed) = crate::declarative::types::eval_numeric_expression(buf)
            {
                let next = parsed.clamp(min_val, max_val);
                if (next - *self.value).abs() > f32::EPSILON {
                    *self.value = next;
                    changed = true;
                }
            }
            row.tree.event_state.active_numeric_edit = None;
            if row.tree.event_state.focused_tag == Some(tag) {
                row.tree.event_state.clear_focus();
            }
            lost_focus = true;
        }

        let is_numeric_focused = row.is_focused(pill_id)
            || (tag != 0 && row.tree.event_state.focused_tag == Some(tag))
            || row
                .tree
                .event_state
                .active_numeric_edit
                .as_ref()
                .is_some_and(|(t, ..)| *t == tag);

        if is_numeric_focused
            && let Some((_, ref mut buf, ref mut cursor, ref mut is_all_sel)) =
                row.tree.event_state.active_numeric_edit
        {
            let is_ctrl_key = row.tree.event_state.frame_key_pressed
                && row.tree.event_state.frame_key.is_some_and(|k| {
                    matches!(
                        k,
                        iris_core::event::KeyCode::ArrowLeft
                            | iris_core::event::KeyCode::ArrowRight
                            | iris_core::event::KeyCode::Home
                            | iris_core::event::KeyCode::End
                            | iris_core::event::KeyCode::Backspace
                            | iris_core::event::KeyCode::Delete
                            | iris_core::event::KeyCode::Enter
                            | iris_core::event::KeyCode::Escape
                    )
                });

            if is_ctrl_key {
                if let Some(key) = row.tree.event_state.frame_key {
                    match key {
                        iris_core::event::KeyCode::ArrowLeft => {
                            if *is_all_sel {
                                *is_all_sel = false;
                                *cursor = 0;
                            } else if *cursor > 0 {
                                *cursor -= 1;
                            }
                        }
                        iris_core::event::KeyCode::ArrowRight => {
                            if *is_all_sel {
                                *is_all_sel = false;
                                *cursor = buf.len();
                            } else if *cursor < buf.len() {
                                *cursor += 1;
                            }
                        }
                        iris_core::event::KeyCode::Home => {
                            *is_all_sel = false;
                            *cursor = 0;
                        }
                        iris_core::event::KeyCode::End => {
                            *is_all_sel = false;
                            *cursor = buf.len();
                        }
                        iris_core::event::KeyCode::Backspace => {
                            if *is_all_sel {
                                buf.clear();
                                *cursor = 0;
                                *is_all_sel = false;
                            } else if *cursor > 0 && *cursor <= buf.len() {
                                buf.remove(*cursor - 1);
                                *cursor -= 1;
                            }
                        }
                        iris_core::event::KeyCode::Delete => {
                            if *is_all_sel {
                                buf.clear();
                                *cursor = 0;
                                *is_all_sel = false;
                            } else if *cursor < buf.len() {
                                buf.remove(*cursor);
                            }
                        }
                        iris_core::event::KeyCode::Enter => {
                            if let Some(parsed) =
                                crate::declarative::types::eval_numeric_expression(buf)
                            {
                                let next = parsed.clamp(min_val, max_val);
                                *self.value = next;
                                changed = true;
                            }
                            row.tree.event_state.active_numeric_edit = None;
                            row.tree.event_state.clear_focus();
                            lost_focus = true;
                            submitted = true;
                        }
                        iris_core::event::KeyCode::Escape => {
                            row.tree.event_state.active_numeric_edit = None;
                            row.tree.event_state.clear_focus();
                        }
                        _ => {}
                    }
                }
            } else if row.tree.event_state.frame_key_pressed
                && let Some(ref text) = row.tree.event_state.frame_text_input
            {
                let filtered: String = text
                    .chars()
                    .filter(|c| {
                        c.is_ascii_digit()
                            || *c == '.'
                            || *c == '-'
                            || *c == '+'
                            || *c == '*'
                            || *c == '/'
                            || *c == '('
                            || *c == ')'
                            || *c == ' '
                    })
                    .collect();
                if !filtered.is_empty() {
                    if *is_all_sel {
                        *buf = filtered;
                        *cursor = buf.len();
                        *is_all_sel = false;
                    } else {
                        let idx = (*cursor).min(buf.len());
                        buf.insert_str(idx, &filtered);
                        *cursor += filtered.len();
                    }
                }
            }
        }

        let is_currently_editing = is_editing_this_pill
            || row
                .tree
                .event_state
                .active_numeric_edit
                .as_ref()
                .is_some_and(|(t, ..)| *t == tag);

        if !is_currently_editing {
            if let Some(text) = row.check_interaction_text(pill_id) {
                let clean = text.trim();
                if let Some(parsed) = crate::declarative::types::eval_numeric_expression(clean) {
                    let next = parsed.clamp(min_val, max_val);
                    if (next - *self.value).abs() > f32::EPSILON {
                        *self.value = next;
                        changed = true;
                    }
                }
            } else if let Some(delta) = drag_delta {
                let next = (*self.value + delta.x * self.speed).clamp(min_val, max_val);
                if (next - *self.value).abs() > f32::EPSILON {
                    *self.value = next;
                    changed = true;
                }
            }
        }
        *self.value = self.value.clamp(min_val, max_val);

        let active_in_scope =
            row.active_text_input.and_then(
                |(t, text, sel)| {
                    if t == tag { Some((text, sel)) } else { None }
                },
            );

        let active_numeric = row
            .tree
            .event_state
            .active_numeric_edit
            .as_ref()
            .filter(|(t, ..)| *t == tag);

        let is_editing =
            self.edit_state.is_some() || active_in_scope.is_some() || active_numeric.is_some();
        let is_all_selected = self.edit_state.as_ref().is_some_and(|s| s.is_all_selected)
            || active_in_scope.as_ref().is_some_and(|(_, sel)| *sel)
            || active_numeric.as_ref().is_some_and(|(.., sel)| *sel);

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

        let box_w = self.box_width.unwrap_or(48.0);
        let pill_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .width(box_w)
            .height(22.0)
            .background(bg)
            .border(1.0, border_col)
            .border_radius(4.0);

        let display_str = if let Some(s) = self.edit_state {
            let buf = s.buffer;
            let cursor = s.cursor_idx.min(buf.len());
            let (left, right) = buf.split_at(cursor);
            if s.is_all_selected {
                buf.to_string()
            } else if s.blink_caret {
                format!("{}|{}", left, right)
            } else {
                buf.to_string()
            }
        } else if let Some((_, buf, cursor, is_all_sel)) = active_numeric {
            if *is_all_sel {
                buf.clone()
            } else {
                let c = (*cursor).min(buf.len());
                let (left, right) = buf.split_at(c);
                let blink_caret = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(true, |d| (d.as_millis() / 500).is_multiple_of(2));
                if blink_caret {
                    format!("{}|{}", left, right)
                } else {
                    buf.clone()
                }
            }
        } else if let Some((active_text, _)) = active_in_scope {
            active_text.to_string()
        } else if let Some(txt) = self.custom_text {
            txt.to_string()
        } else {
            crate::declarative::types::format_compact_float(*self.value, self.decimals)
        };

        let text_len = display_str.len();
        if let Some(node) = row.tree.get_mut(pill_id) {
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
            node.set_style(pill_style);
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
            let mut pill_scope = row.child_scope(pill_id);
            pill_scope.empty_box_passive_named("Numeric_Sel", sel_style);
        }

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

        let is_drag_released = (row.tree.event_state.frame_drag_released_tag == Some(tag)
            && tag != 0)
            || row.tree.event_state.frame_drag_released_id == Some(pill_id);

        let mut resp = WidgetResponse::new(pill_id, clicked, hovered, changed);
        if is_pressed {
            resp = resp.with_pressed(true);
        }
        if is_dragging {
            resp = resp.with_dragged(true);
        }
        if is_drag_released {
            resp = resp.with_drag_released(true);
        }
        if lost_focus {
            resp.flags = resp
                .flags
                .union(iris_core::response::ResponseFlags::LOST_FOCUS);
        }
        if submitted {
            resp.flags = resp
                .flags
                .union(iris_core::response::ResponseFlags::SUBMITTED);
        }

        ScalarResponse {
            response: resp,
            value: *self.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::UiTree;

    #[test]
    fn test_numeric_drag_builder_and_response() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        let mut val = 42.5;
        let resp = scope.add(
            NumericDrag::new("Speed", &mut val)
                .speed(0.2)
                .range(0.0..=100.0)
                .decimals(1)
                .unit("m/s")
                .label_width(70.0)
                .box_width(50.0),
        );

        assert!(!resp.changed());
        assert!(!resp.clicked());
        assert_eq!(resp.value(), 42.5);
        assert_eq!(val, 42.5);
    }

    #[test]
    fn test_numeric_drag_clamping() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        let mut val = 150.0;
        let _ = scope.add(NumericDrag::new("Clamped", &mut val).range(0.0..=100.0));
        assert_eq!(val, 100.0);

        let mut under = -50.0;
        let _ = scope.add(NumericDrag::new("Under", &mut under).range(0.0..=100.0));
        assert_eq!(under, 0.0);
    }

    #[test]
    fn test_numeric_drag_enter_confirmation_and_submitted_response() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");

        let tag = 0x998877;
        let mut val = 10.0;

        // Simulate active numeric editing session on tag
        tree.event_state.active_numeric_edit = Some((tag, "25.5".to_string(), 4, false));
        tree.event_state.focused_tag = Some(tag);
        tree.event_state.frame_key = Some(iris_core::event::KeyCode::Enter);
        tree.event_state.frame_key_pressed = true;

        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("", &mut val).tag(tag).range(0.0..=100.0));

        assert_eq!(val, 25.5);
        assert!(resp.changed());
        assert!(resp.submitted());
        assert!(resp.committed());
        assert!(tree.event_state.active_numeric_edit.is_none());
    }
}