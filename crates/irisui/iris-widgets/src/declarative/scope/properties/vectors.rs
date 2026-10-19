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

/// Target sub-element inside a 3-channel vector property row (`property_vec3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyVec3Target {
    /// Numeric input editor for axis index (0 for X, 1 for Y, 2 for Z).
    Axis(usize),
    /// Dedicated reset button returning vector components to defaults.
    Reset,
}

/// Derives the semantic tag for an individual axis input in a 3-channel vector row.
#[inline]
pub const fn property_vec3_axis_tag(base_tag: u64, axis_idx: usize) -> u64 {
    base_tag.wrapping_add((axis_idx as u64) + 1)
}

/// Derives the semantic tag for the reset button in a 3-channel vector row.
#[inline]
pub const fn property_vec3_reset_tag(base_tag: u64) -> u64 {
    base_tag.wrapping_add(4)
}

/// Resolves an interactive tag against a base vector row tag in $O(1)$ arithmetic without heap allocations.
///
/// Returns `Some(PropertyVec3Target::Axis(0..=2))` for axis inputs,
/// `Some(PropertyVec3Target::Reset)` for the reset button, or `None` if unrelated.
#[inline]
pub const fn resolve_property_vec3_tag(tag: u64, base_tag: u64) -> Option<PropertyVec3Target> {
    match tag.wrapping_sub(base_tag) {
        1 => Some(PropertyVec3Target::Axis(0)),
        2 => Some(PropertyVec3Target::Axis(1)),
        3 => Some(PropertyVec3Target::Axis(2)),
        4 => Some(PropertyVec3Target::Reset),
        _ => None,
    }
}

/// Resolves an interactive tag against a vector row property label and scope seed.
#[inline]
pub fn resolve_property_vec3_for_seed(
    tag: u64,
    label: &str,
    seed: u64,
) -> Option<PropertyVec3Target> {
    let base_tag = crate::declarative::types::hash_label_with_seed(seed, label);
    resolve_property_vec3_tag(tag, base_tag)
}

/// Resolves an interactive tag against a vector row property label across candidate hierarchical seeds.
///
/// Enables robust tag resolution in compound UI panels where scopes may inherit either root seed
/// or named parent container seeds.
#[inline]
pub fn resolve_property_vec3_for_seeds(
    tag: u64,
    label: &str,
    seeds: &[u64],
) -> Option<PropertyVec3Target> {
    for &seed in seeds {
        if let Some(target) = resolve_property_vec3_for_seed(tag, label, seed) {
            return Some(target);
        }
    }
    None
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
    /// Index of the axis that changed during this frame (0 for X, 1 for Y, 2 for Z).
    pub changed_axis: Option<usize>,
    /// Index of the axis that was committed during this frame (via Enter, blur, or drag release).
    pub committed_axis: Option<usize>,
    /// Whether the reset button was clicked during this frame.
    pub reset_clicked: bool,
}

impl PropertyVec3Response {
    /// Returns `true` if any vector axis value was committed during this frame.
    #[inline]
    #[must_use]
    pub const fn committed(&self) -> bool {
        self.committed_axis.is_some()
    }
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
        let mut changed_axis_out = None;
        let mut committed_axis_out = None;
        let mut reset_clicked_out = false;
        let mut slot_executed = false;

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
                let axis_tag = property_vec3_axis_tag(base_tag, idx);
                let edit_state = options.edit_states[idx];
                let active_numeric_info = slot
                    .tree
                    .event_state
                    .active_numeric_edit
                    .as_ref()
                    .filter(|(t, ..)| *t == axis_tag)
                    .map(|(_, buf, cursor, is_all_sel)| (buf.clone(), *cursor, *is_all_sel));
                let is_editing = edit_state.is_some() || active_numeric_info.is_some();

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
                } else if let Some((ref buf, cursor, is_all_sel)) = active_numeric_info {
                    if is_all_sel {
                        format!("{}{}", prefix, buf)
                    } else {
                        let c = cursor.min(buf.len());
                        let (left, right) = buf.split_at(c);
                        let blink_caret = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map_or(true, |d| (d.as_millis() / 500).is_multiple_of(2));
                        if blink_caret {
                            format!("{}{}|{}", prefix, left, right)
                        } else {
                            format!("{}{}{}", prefix, left, right)
                        }
                    }
                } else {
                    format!(
                        "{}{}",
                        prefix,
                        crate::declarative::types::format_compact_float(
                            values[idx],
                            options.decimals
                        )
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

                let axis_box_id = slot.tree.create_node();
                let is_dragging = slot.tree.event_state.is_pointer_drag_active
                    && (slot.tree.event_state.pointer_capture == Some(axis_box_id)
                        || (axis_tag != 0
                            && slot.tree.event_state.pointer_capture_tag == Some(axis_tag)));
                if let Some(node) = slot.tree.get_mut(axis_box_id) {
                    node.set_name(box_name);
                    node.set_style(pill_style);
                    node.interactive = true;
                    node.role = WidgetRole::NumericInput;
                    node.tag = axis_tag;
                    node.cursor = Some(if is_editing {
                        WidgetCursor::Text
                    } else if is_dragging {
                        WidgetCursor::EwResize
                    } else {
                        WidgetCursor::Default
                    });
                }
                let _ = slot.tree.add_child(slot.parent, axis_box_id);

                {
                    let mut pill_scope = slot.child_scope(axis_box_id);
                    let active_sel_buf = edit_state
                        .filter(|s| s.is_all_selected)
                        .map(|s| s.buffer)
                        .or_else(|| {
                            active_numeric_info
                                .as_ref()
                                .filter(|(_, _, sel)| *sel)
                                .map(|(b, ..)| b.as_str())
                        });

                    if let Some(buf_str) = active_sel_buf {
                        let approx_char_w = 6.2;
                        let val_w = (buf_str.len() as f32 * approx_char_w).max(12.0);
                        let prefix_w = prefix.len() as f32 * approx_char_w;
                        let tot_w = (prefix.len() + buf_str.len()) as f32 * approx_char_w;
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
                }

                axes_out[idx] = axis_box_id;

                let (clicked, hovered, drag_delta) = slot.check_interaction(axis_box_id);

                if clicked && options.edit_states[idx].is_none() {
                    let is_already_editing = slot
                        .tree
                        .event_state
                        .active_numeric_edit
                        .as_ref()
                        .is_some_and(|(t, ..)| *t == axis_tag);
                    if is_already_editing {
                        if let Some((_, ref buf, ref mut cursor, ref mut is_all_sel)) =
                            slot.tree.event_state.active_numeric_edit
                        {
                            *is_all_sel = false;
                            *cursor = buf.len();
                        }
                    } else {
                        let initial_buf = crate::declarative::types::format_compact_float(
                            values[idx],
                            options.decimals,
                        );
                        let len = initial_buf.len();
                        slot.tree.event_state.active_numeric_edit =
                            Some((axis_tag, initial_buf, len, true));
                        slot.tree.event_state.focused = Some(axis_box_id);
                        slot.tree.event_state.focused_tag = Some(axis_tag);
                    }
                }

                let is_editing_this_axis = slot
                    .tree
                    .event_state
                    .active_numeric_edit
                    .as_ref()
                    .is_some_and(|(t, ..)| *t == axis_tag);

                let is_clicked_outside =
                    if let Some(clicked_tag) = slot.tree.event_state.frame_clicked_tag {
                        clicked_tag != axis_tag
                    } else if let Some(clicked_id) = slot.tree.event_state.frame_clicked_id {
                        clicked_id != axis_box_id
                    } else {
                        false
                    };

                if is_editing_this_axis && is_clicked_outside {
                    if let Some((_, ref buf, ..)) = slot.tree.event_state.active_numeric_edit
                        && let Some(num) = crate::declarative::types::eval_numeric_expression(buf)
                        && (num - values[idx]).abs() > f32::EPSILON
                    {
                        values[idx] = num;
                        any_changed = true;
                        changed_axis_out = Some(idx);
                    }
                    committed_axis_out = Some(idx);
                    slot.tree.event_state.active_numeric_edit = None;
                    if slot.tree.event_state.focused_tag == Some(axis_tag) {
                        slot.tree.event_state.clear_focus();
                    }
                }

                let is_axis_focused = slot.is_focused(axis_box_id)
                    || slot.tree.event_state.focused_tag == Some(axis_tag)
                    || slot
                        .tree
                        .event_state
                        .active_numeric_edit
                        .as_ref()
                        .is_some_and(|(t, ..)| *t == axis_tag);

                if is_axis_focused
                    && let Some((_, ref mut buf, ref mut cursor, ref mut is_all_sel)) =
                        slot.tree.event_state.active_numeric_edit
                {
                    if slot.tree.event_state.frame_key_pressed
                        && let Some(ref text) = slot.tree.event_state.frame_text_input
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
                                let idx_c = (*cursor).min(buf.len());
                                buf.insert_str(idx_c, &filtered);
                                *cursor += filtered.len();
                            }
                        }
                    } else if let Some(key) = slot.tree.event_state.frame_key
                        && slot.tree.event_state.frame_key_pressed
                    {
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
                                if let Some(num) =
                                    crate::declarative::types::eval_numeric_expression(buf)
                                {
                                    values[idx] = num;
                                    any_changed = true;
                                    changed_axis_out = Some(idx);
                                }
                                committed_axis_out = Some(idx);
                                slot.tree.event_state.active_numeric_edit = None;
                                slot.tree.event_state.clear_focus();
                            }
                            iris_core::event::KeyCode::Escape => {
                                slot.tree.event_state.active_numeric_edit = None;
                                slot.tree.event_state.clear_focus();
                            }
                            _ => {}
                        }
                    }
                }

                let is_currently_editing = is_editing_this_axis
                    || slot
                        .tree
                        .event_state
                        .active_numeric_edit
                        .as_ref()
                        .is_some_and(|(t, ..)| *t == axis_tag);

                if !is_currently_editing {
                    if let Some(delta) = drag_delta {
                        let next = values[idx] + delta.x * options.speed;
                        if (next - values[idx]).abs() > f32::EPSILON {
                            values[idx] = next;
                            any_changed = true;
                            changed_axis_out = Some(idx);
                        }
                    }
                    if let Some(text) = slot.check_interaction_text(axis_box_id) {
                        let clean = text.trim();
                        if let Some(num) = crate::declarative::types::eval_numeric_expression(clean)
                        {
                            values[idx] = num;
                            any_changed = true;
                            changed_axis_out = Some(idx);
                        }
                    }
                }

                let is_drag_released = (slot.tree.event_state.frame_drag_released_tag
                    == Some(axis_tag)
                    && axis_tag != 0)
                    || slot.tree.event_state.frame_drag_released_id == Some(axis_box_id);
                if is_drag_released {
                    committed_axis_out = Some(idx);
                }

                any_clicked |= clicked;
                any_hovered |= hovered;
            }

            // Optional Reset Button
            if let Some(reset_val) = options.reset_value {
                let reset_tag = property_vec3_reset_tag(base_tag);
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
                let reset_btn_id = slot.tree.create_node();
                if let Some(node) = slot.tree.get_mut(reset_btn_id) {
                    node.set_name(reset_name);
                    node.set_style(reset_style);
                    node.interactive = true;
                    node.role = WidgetRole::Button;
                    node.tag = reset_tag;
                    node.cursor = Some(WidgetCursor::Pointer);
                }
                let _ = slot.tree.add_child(slot.parent, reset_btn_id);

                let mut btn_scope = slot.child_scope(reset_btn_id);
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

                reset_btn_out = Some(reset_btn_id);

                let (reset_clicked, reset_hovered, _) = slot.check_interaction(reset_btn_id);
                if reset_clicked {
                    *values = reset_val;
                    any_changed = true;
                    reset_clicked_out = true;
                }
                any_clicked |= reset_clicked;
                any_hovered |= reset_hovered;
            }

            let mut wr = WidgetResponse::new(slot.parent, any_clicked, any_hovered, any_changed);
            if committed_axis_out.is_some() {
                wr.flags = wr
                    .flags
                    .union(iris_core::response::ResponseFlags::SUBMITTED);
            }
            widget_response = wr;
            slot_executed = true;
        });

        if !slot_executed {
            widget_response = self.response(row_id);
        }

        PropertyVec3Response {
            response: widget_response,
            axes: axes_out,
            reset_btn: reset_btn_out,
            changed_axis: changed_axis_out,
            committed_axis: committed_axis_out,
            reset_clicked: reset_clicked_out,
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
        let mut slot_executed = false;

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

                let axis_box_id = slot.tree.create_node();
                if let Some(node) = slot.tree.get_mut(axis_box_id) {
                    node.set_name(box_name);
                    node.set_style(box_style);
                    node.role = WidgetRole::NumericInput;
                    node.tag = axis_tag;
                    node.interactive = true;
                    node.cursor = Some(WidgetCursor::EwResize);
                }
                let _ = slot.tree.add_child(slot.parent, axis_box_id);

                let mut pill_scope = slot.child_scope(axis_box_id);
                pill_scope.label_styled_passive(
                    txt_name,
                    text_val,
                    10.5,
                    col,
                    TextAlign::Center,
                    Style::new().width(72.0).height(16.0),
                );

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
            slot_executed = true;
        });

        if !slot_executed {
            response = self.response(row_id);
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
        let mut slot_executed = false;

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

                let axis_box_id = slot.tree.create_node();
                if let Some(node) = slot.tree.get_mut(axis_box_id) {
                    node.set_name(box_name);
                    node.set_style(box_style);
                    node.role = WidgetRole::NumericInput;
                    node.tag = axis_tag;
                    node.interactive = true;
                    node.cursor = Some(WidgetCursor::EwResize);
                }
                let _ = slot.tree.add_child(slot.parent, axis_box_id);

                let mut pill_scope = slot.child_scope(axis_box_id);
                pill_scope.label_styled_passive(
                    txt_name,
                    text_val,
                    10.5,
                    col,
                    TextAlign::Center,
                    Style::new().width(50.0).height(16.0),
                );

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
            slot_executed = true;
        });

        if !slot_executed {
            response = self.response(row_id);
        }
        response
    }
}