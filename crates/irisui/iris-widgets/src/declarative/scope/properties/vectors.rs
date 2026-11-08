// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Multi-Channel Vector & Color Editors
//!
//! Provides color-coded axis inputs for 3D coordinates, 2D offsets/tilings, and RGB multipliers on [`UiScope`].

use crate::controls::numeric_drag::{DEFAULT_PIXELS_PER_STEP, DragCurve};
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
    /// Response curve for pointer drag scrubbing.
    pub curve: DragCurve,
    /// Reference pixel displacement per step unit for non-linear curve evaluation.
    pub pixels_per_step: f32,
}

impl<'a> Default for PropertyVec3Options<'a> {
    fn default() -> Self {
        Self {
            speed: 0.1,
            decimals: 2,
            reset_value: None,
            edit_states: [None, None, None],
            label_width: 52.0,
            curve: DragCurve::Linear,
            pixels_per_step: DEFAULT_PIXELS_PER_STEP as f32,
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
            curve: DragCurve::Linear,
            pixels_per_step: DEFAULT_PIXELS_PER_STEP as f32,
        }
    }

    /// Sets the response curve for pointer drag scrubbing.
    #[must_use]
    pub const fn with_curve(mut self, curve: DragCurve) -> Self {
        self.curve = curve;
        self
    }

    /// Sets the reference pixel displacement per step unit for non-linear curve evaluation.
    #[must_use]
    pub const fn with_pixels_per_step(mut self, pixels: f32) -> Self {
        self.pixels_per_step = pixels;
        self
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

/// Derives the semantic tag for an individual axis input in a 2-channel vector row.
#[inline]
pub const fn property_vec2_axis_tag(base_tag: u64, axis_idx: usize) -> u64 {
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
    /// The pre-edit or baseline 3-channel vector values prior to this interaction.
    pub old_values: [f32; 3],
    /// The updated 3-channel vector values after evaluating interaction for this frame.
    pub new_values: [f32; 3],
}

impl PropertyVec3Response {
    /// Returns the baseline or pre-edit 3-channel vector values before this interaction.
    #[inline]
    #[must_use]
    pub const fn old_values(&self) -> [f32; 3] {
        self.old_values
    }

    /// Returns the updated 3-channel vector values after this interaction.
    #[inline]
    #[must_use]
    pub const fn new_values(&self) -> [f32; 3] {
        self.new_values
    }

    /// Returns `true` if any vector axis value was committed during this frame.
    #[inline]
    #[must_use]
    pub const fn committed(&self) -> bool {
        self.committed_axis.is_some()
    }

    /// Returns `true` if any vector axis value was modified or the reset button was clicked during this frame.
    #[inline]
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.changed_axis.is_some() || self.reset_clicked || self.response.changed()
    }

    /// Invokes `f` with the updated 3-channel vector values if any axis changed or the reset button was clicked.
    #[inline]
    pub fn on_change(self, f: impl FnOnce([f32; 3])) -> Self {
        if self.changed() {
            f(self.new_values);
        }
        self
    }

    /// Invokes `f` with the newly committed 3-channel vector values if any axis was committed during this frame.
    #[inline]
    pub fn on_commit(self, f: impl FnOnce([f32; 3])) -> Self {
        if self.committed() {
            f(self.new_values);
        }
        self
    }

    /// Invokes `f(old_vals, new_vals)` if any axis was committed with a non-trivial change.
    #[inline]
    pub fn on_commit_change(self, f: impl FnOnce([f32; 3], [f32; 3])) -> Self {
        if self.committed() && (self.new_values != self.old_values) {
            f(self.old_values, self.new_values);
        }
        self
    }

    /// Evaluates and returns `Some(f(old_vals, new_vals))` if any axis was committed with a non-trivial change,
    /// or `None` otherwise.
    #[inline]
    pub fn map_commit<T>(self, f: impl FnOnce([f32; 3], [f32; 3]) -> T) -> Option<T> {
        (self.committed() && (self.new_values != self.old_values))
            .then(|| f(self.old_values, self.new_values))
    }
}

impl std::ops::Deref for PropertyVec3Response {
    type Target = WidgetResponse;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

/// Configuration options for 2D coordinate vector property rows.
#[derive(Debug, Clone)]
pub struct PropertyVec2Options<'a> {
    /// Multiplier applied to continuous pointer drag displacement.
    pub speed: f32,
    /// Display precision decimals for formatted floating-point values.
    pub decimals: usize,
    /// Column width allocated for the left-aligned title caption.
    pub label_width: f32,
    /// Axis prefix labels displayed in each interactive channel pill (e.g. `["X", "Y"]`, `["W", "H"]`).
    pub axis_labels: [&'a str; 2],
    /// Physical unit suffix string (e.g. `"px"`, `""`).
    pub unit: &'a str,
    /// Valid numerical bounding constraints for both scalar components.
    pub range: std::ops::RangeInclusive<f32>,
    /// Pointer drag scrubbing acceleration/deceleration response profile.
    pub curve: DragCurve,
    /// Reference displacement pixels required per step unit.
    pub pixels_per_step: f32,
}

impl<'a> PropertyVec2Options<'a> {
    /// Constructs default configuration options with the specified drag speed.
    #[inline]
    #[must_use]
    pub const fn new(speed: f32) -> Self {
        Self {
            speed,
            decimals: 2,
            label_width: 52.0,
            axis_labels: ["X", "Y"],
            unit: "",
            range: -10000.0..=10000.0,
            curve: DragCurve::Linear,
            pixels_per_step: crate::controls::numeric_drag::DEFAULT_PIXELS_PER_STEP as f32,
        }
    }
}

/// Comprehensive interaction response returned by evaluating a 2D vector property row.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PropertyVec2Response {
    /// Aggregated composite widget response across the entire row.
    pub response: WidgetResponse,
    /// Allocated widget identifiers for each axis numeric input pill.
    pub axes: [WidgetId; 2],
    /// 0-indexed axis that changed during this frame (0 for axis 0, 1 for axis 1).
    pub changed_axis: Option<usize>,
    /// 0-indexed axis that was committed during this frame upon pointer release.
    pub committed_axis: Option<usize>,
    /// The pre-edit or baseline 2-channel vector values prior to this interaction.
    pub old_values: [f32; 2],
    /// The updated 2-channel vector values after evaluating interaction for this frame.
    pub new_values: [f32; 2],
}

impl PropertyVec2Response {
    /// Returns the baseline or pre-edit 2-channel vector values before this interaction.
    #[inline]
    #[must_use]
    pub const fn old_values(&self) -> [f32; 2] {
        self.old_values
    }

    /// Returns the updated 2-channel vector values after this interaction.
    #[inline]
    #[must_use]
    pub const fn new_values(&self) -> [f32; 2] {
        self.new_values
    }

    /// Returns `true` if any vector axis value was committed during this frame.
    #[inline]
    #[must_use]
    pub const fn committed(&self) -> bool {
        self.committed_axis.is_some()
    }

    /// Returns `true` if any vector axis value was modified during this frame.
    #[inline]
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.changed_axis.is_some() || self.response.changed()
    }

    /// Invokes `f` with the newly committed 2-channel vector values if any axis was committed during this frame.
    #[inline]
    pub fn on_commit(self, f: impl FnOnce([f32; 2])) -> Self {
        if self.committed() {
            f(self.new_values);
        }
        self
    }

    /// Invokes `f(old_vals, new_vals)` if any axis was committed with a non-trivial change.
    #[inline]
    pub fn on_commit_change(self, f: impl FnOnce([f32; 2], [f32; 2])) -> Self {
        if self.committed() && (self.new_values != self.old_values) {
            f(self.old_values, self.new_values);
        }
        self
    }

    /// Evaluates and returns `Some(f(old_vals, new_vals))` if any axis was committed with a non-trivial change,
    /// or `None` otherwise.
    #[inline]
    pub fn map_commit<T>(self, f: impl FnOnce([f32; 2], [f32; 2]) -> T) -> Option<T> {
        (self.committed() && (self.new_values != self.old_values))
            .then(|| f(self.old_values, self.new_values))
    }
}

impl std::ops::Deref for PropertyVec2Response {
    type Target = WidgetResponse;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl<'a> UiScope<'a> {
    /// Emits a declarative two-way bound 3D vector property row modifying `values` (`[x, y, z]`) in-place.
    ///
    /// Renders distinct color-coded X (red), Y (green), and Z (blue) axis editors alongside an optional reset button.
    /// Returns a rich [`PropertyVec3Response`] with per-axis change tracking (`.changed()`, `.committed()`).
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left (supports `##` separator).
    /// * `values` - Mutable reference to the 3-element float array `[x, y, z]`.
    /// * `speed` - Drag sensitivity multiplier applied per horizontal delta pixel.
    /// * `reset_value` - Optional default value restored when the reset button `🔄` is clicked.
    pub fn prop_vec3(
        &mut self,
        label: &str,
        values: &mut [f32; 3],
        speed: f32,
        reset_value: Option<[f32; 3]>,
    ) -> PropertyVec3Response {
        self.prop_vec3_styled(label, values, speed, reset_value, None)
    }

    /// Emits a declarative two-way bound 3D vector property row with custom label column width.
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left (supports `##` separator).
    /// * `values` - Mutable reference to the 3-element float array `[x, y, z]`.
    /// * `speed` - Drag sensitivity multiplier applied per horizontal delta pixel.
    /// * `reset_value` - Optional default value restored when the reset button `🔄` is clicked.
    /// * `label_width` - Optional column width in physical pixels allocated for the label. Defaults to [`crate::declarative::widget::DEFAULT_PROPERTY_LABEL_WIDTH`].
    pub fn prop_vec3_styled(
        &mut self,
        label: &str,
        values: &mut [f32; 3],
        speed: f32,
        reset_value: Option<[f32; 3]>,
        label_width: Option<f32>,
    ) -> PropertyVec3Response {
        let mut opts = PropertyVec3Options::new(speed);
        opts.reset_value = reset_value;
        opts.label_width =
            label_width.unwrap_or(crate::declarative::widget::DEFAULT_PROPERTY_LABEL_WIDTH);
        self.property_vec3_with_options(label, values, opts)
    }

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
        self.prop_vec3(label, values, speed, None).response
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
        let initial_values = *values;
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

            if let Some(restored_vec) = slot
                .tree
                .event_state
                .numeric_undo
                .take_pending_vec3(base_tag)
            {
                *values = restored_vec;
                any_changed = true;
                changed_axis_out = Some(0);
                committed_axis_out = Some(0);
            }

            for (prefix, col, idx) in axes_meta {
                let axis_tag = property_vec3_axis_tag(base_tag, idx);
                let axis_name = axis_names[idx];
                let box_name = format!("NumBox_{}_{}", visible_label, axis_name);
                let sel_name = format!("NumSel_{}_{}", visible_label, axis_name);
                let txt_name = format!("NumText_{}_{}", visible_label, axis_name);

                let axis_box_id = slot.tree.create_node();
                if let Some(node) = slot.tree.get_mut(axis_box_id) {
                    node.tag = axis_tag;
                }
                let _ = slot.tree.add_child(slot.parent, axis_box_id);
                axes_out[idx] = axis_box_id;

                let (clicked, hovered, drag_delta) = slot.check_interaction(axis_box_id);

                let edit_res = if options.edit_states[idx].is_none() {
                    let config = super::numeric_input::NumericEditConfig::new(
                        f32::NEG_INFINITY,
                        f32::INFINITY,
                        options.decimals,
                    );
                    Some(super::numeric_input::handle_numeric_edit_interaction(
                        &mut *slot,
                        axis_box_id,
                        axis_tag,
                        &mut values[idx],
                        config,
                        clicked,
                    ))
                } else {
                    None
                };

                let is_editing = edit_res
                    .as_ref()
                    .map_or(options.edit_states[idx].is_some(), |r| r.is_editing);

                if let Some(ref res) = edit_res {
                    if res.value_changed {
                        any_changed = true;
                        changed_axis_out = Some(idx);
                    }
                    if res.submitted {
                        committed_axis_out = Some(idx);
                    }
                }

                let is_dragging = slot.tree.event_state.is_pointer_drag_active
                    && (slot.tree.event_state.pointer_capture == Some(axis_box_id)
                        || (axis_tag != 0
                            && slot.tree.event_state.pointer_capture_tag == Some(axis_tag)));

                if !is_editing && let Some(delta) = drag_delta {
                    slot.tree
                        .event_state
                        .numeric_undo
                        .begin_drag(axis_tag, values[idx]);
                    let next = values[idx]
                        + options.curve.compute_delta_f32(
                            delta.x,
                            options.speed,
                            options.pixels_per_step,
                            &slot.tree.event_state,
                        );
                    if (next - values[idx]).abs() > f32::EPSILON {
                        values[idx] = next;
                        any_changed = true;
                        changed_axis_out = Some(idx);
                    }
                }

                let is_drag_released = (slot.tree.event_state.frame_drag_released_tag
                    == Some(axis_tag)
                    && axis_tag != 0)
                    || slot.tree.event_state.frame_drag_released_id == Some(axis_box_id);
                if is_drag_released {
                    slot.tree
                        .event_state
                        .numeric_undo
                        .commit_drag(axis_tag, values[idx]);
                    committed_axis_out = Some(idx);
                    if !is_editing {
                        slot.tree.event_state.focused = Some(axis_box_id);
                        if axis_tag != 0 {
                            slot.tree.event_state.focused_tag = Some(axis_tag);
                        }
                    }
                }

                any_clicked |= clicked;
                any_hovered |= hovered;

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

                let txt_col = if is_editing { Color::WHITE } else { col };

                let (_raw_value_str, display_str, is_all_selected) =
                    if let Some(s) = options.edit_states[idx] {
                        let buf = s.buffer;
                        let cursor = s.cursor_idx.min(buf.len());
                        let (left, right) = buf.split_at(cursor);
                        let (raw_val, str) = if s.is_all_selected {
                            (buf.to_string(), format!("{}{}", prefix, buf))
                        } else if s.blink_caret {
                            (
                                format!("{}|{}", left, right),
                                format!("{}{}|{}", prefix, left, right),
                            )
                        } else {
                            (
                                format!("{}{}", left, right),
                                format!("{}{}{}", prefix, left, right),
                            )
                        };
                        (raw_val, str, s.is_all_selected)
                    } else if let Some(ref res) = edit_res {
                        let is_all = res
                            .selection
                            .is_some_and(|(s, e)| s == 0 && e == res.raw_text.len());
                        (
                            res.raw_text.clone(),
                            format!("{}{}", prefix, res.display_text),
                            is_all,
                        )
                    } else {
                        let formatted = crate::declarative::types::format_compact_float(
                            values[idx],
                            options.decimals,
                        );
                        (formatted.clone(), format!("{}{}", prefix, formatted), false)
                    };

                let pill_style = Style::new()
                    .width(54.0)
                    .height(20.0)
                    .background(bg_col)
                    .border(1.0, border_col)
                    .border_radius(5.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center);

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

                {
                    let mut pill_scope = slot.child_scope(axis_box_id);
                    if is_all_selected {
                        let approx_char_w = 6.2;
                        let val_w = (display_str.len().saturating_sub(prefix.len()) as f32
                            * approx_char_w)
                            .max(12.0);
                        let prefix_w = prefix.len() as f32 * approx_char_w;
                        let tot_w = display_str.len() as f32 * approx_char_w;
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
                    slot.tree
                        .event_state
                        .numeric_undo
                        .push_vec3(base_tag, *values, reset_val);
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
            old_values: initial_values,
            new_values: *values,
        }
    }

    /// Emits a declarative two-way bound 2D vector property row with custom axis labels and drag speed.
    ///
    /// Modifies `values` (`[axis 0, axis 1]`) in-place upon horizontal dragging, clamping within range,
    /// and returns a rich [`PropertyVec2Response`] with per-axis change tracking (`.changed()`, `.committed()`).
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left (supports `##` separator).
    /// * `values` - Mutable reference to the 2-element float array.
    /// * `axis_labels` - Two-character or string labels for each channel (e.g. `["X", "Y"]` or `["W", "H"]`).
    /// * `speed` - Drag sensitivity multiplier applied per horizontal delta pixel.
    pub fn prop_vec2(
        &mut self,
        label: &str,
        values: &mut [f32; 2],
        axis_labels: [&str; 2],
        speed: f32,
    ) -> PropertyVec2Response {
        self.prop_vec2_styled(label, values, axis_labels, speed, None, None)
    }

    /// Emits a declarative two-way bound 2D vector property row with custom column width and numerical range constraints.
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left (supports `##` separator).
    /// * `values` - Mutable reference to the 2-element float array.
    /// * `axis_labels` - Two-character or string labels for each channel (e.g. `["X", "Y"]` or `["W", "H"]`).
    /// * `speed` - Drag sensitivity multiplier applied per horizontal delta pixel.
    /// * `label_width` - Optional column width in physical pixels allocated for the label. Defaults to [`crate::declarative::widget::DEFAULT_PROPERTY_LABEL_WIDTH`].
    /// * `range` - Optional inclusive numerical bounds for both scalar components.
    pub fn prop_vec2_styled(
        &mut self,
        label: &str,
        values: &mut [f32; 2],
        axis_labels: [&str; 2],
        speed: f32,
        label_width: Option<f32>,
        range: Option<std::ops::RangeInclusive<f32>>,
    ) -> PropertyVec2Response {
        let mut opts = PropertyVec2Options::new(speed);
        opts.axis_labels = axis_labels;
        opts.label_width =
            label_width.unwrap_or(crate::declarative::widget::DEFAULT_PROPERTY_LABEL_WIDTH);
        if let Some(r) = range {
            opts.range = r;
        }
        self.property_vec2_with_options(label, values, opts)
    }

    /// Emits a standard `["X", "Y"]` declarative two-way bound 2D coordinate property row.
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left (supports `##` separator).
    /// * `values` - Mutable reference to the 2-element float array `[x, y]`.
    /// * `speed` - Drag sensitivity multiplier applied per horizontal delta pixel.
    pub fn prop_vec2_xy(
        &mut self,
        label: &str,
        values: &mut [f32; 2],
        speed: f32,
    ) -> PropertyVec2Response {
        self.prop_vec2(label, values, ["X", "Y"], speed)
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
        self.prop_vec2_xy(label, values, speed).response
    }

    /// Emits an inline horizontal 2D vector editor with customizable axis labels, units, and ranges.
    ///
    /// # Arguments
    /// * `label` - Property description displayed on the left (supports `##` separator).
    /// * `values` - Mutable reference to the 2-element float array.
    /// * `options` - Custom configuration options (speed, decimals, axis_labels, unit, range).
    pub fn property_vec2_with_options(
        &mut self,
        label: &str,
        values: &mut [f32; 2],
        options: PropertyVec2Options<'_>,
    ) -> PropertyVec2Response {
        let initial_values = *values;
        let (visible_label, _) = crate::declarative::types::split_label_id(label);
        let base_tag = self.tag_for(label);
        let mut widget_response = WidgetResponse::default();
        let mut axes_out = [WidgetId::default(); 2];
        let mut changed_axis_out = None;
        let mut committed_axis_out = None;
        let mut slot_executed = false;

        let row_id = self.property_row_styled(label, Some(options.label_width), |slot| {
            let colors = [
                Color::rgba(0.90, 0.35, 0.35, 1.0),
                Color::rgba(0.35, 0.85, 0.45, 1.0),
            ];

            let mut any_changed = false;
            let mut any_clicked = false;
            let mut any_hovered = false;

            for idx in 0..2 {
                let axis_tag = property_vec2_axis_tag(base_tag, idx);
                let axis_name = options.axis_labels[idx];
                let box_name = format!("NumBox_{}_{}", visible_label, axis_name);
                let txt_name = format!("NumText_{}_{}", visible_label, axis_name);

                let axis_box_id = slot.tree.create_node();
                if let Some(node) = slot.tree.get_mut(axis_box_id) {
                    node.tag = axis_tag;
                }
                let _ = slot.tree.add_child(slot.parent, axis_box_id);
                axes_out[idx] = axis_box_id;

                let (clicked, hovered, drag_delta) = slot.check_interaction(axis_box_id);

                let min_val = *options.range.start();
                let max_val = *options.range.end();
                if let Some(restored) = slot
                    .tree
                    .event_state
                    .numeric_undo
                    .take_pending_scalar(axis_tag)
                {
                    let next = restored.clamp(min_val, max_val);
                    if (next - values[idx]).abs() > f32::EPSILON {
                        values[idx] = next;
                        any_changed = true;
                        changed_axis_out = Some(idx);
                        committed_axis_out = Some(idx);
                    }
                }

                let is_dragging = slot.tree.event_state.is_pointer_drag_active
                    && (slot.tree.event_state.pointer_capture == Some(axis_box_id)
                        || (axis_tag != 0
                            && slot.tree.event_state.pointer_capture_tag == Some(axis_tag)));

                if let Some(delta) = drag_delta {
                    slot.tree
                        .event_state
                        .numeric_undo
                        .begin_drag(axis_tag, values[idx]);
                    let step = options.curve.compute_delta_f32(
                        delta.x,
                        options.speed,
                        options.pixels_per_step,
                        &slot.tree.event_state,
                    );
                    let next = (values[idx] + step).clamp(min_val, max_val);
                    if (next - values[idx]).abs() > f32::EPSILON {
                        values[idx] = next;
                        any_changed = true;
                        changed_axis_out = Some(idx);
                    }
                }

                let is_drag_released = (slot.tree.event_state.frame_drag_released_tag
                    == Some(axis_tag)
                    && axis_tag != 0)
                    || slot.tree.event_state.frame_drag_released_id == Some(axis_box_id);
                if is_drag_released {
                    slot.tree
                        .event_state
                        .numeric_undo
                        .commit_drag(axis_tag, values[idx]);
                    committed_axis_out = Some(idx);
                    slot.tree.event_state.focused = Some(axis_box_id);
                    if axis_tag != 0 {
                        slot.tree.event_state.focused_tag = Some(axis_tag);
                    }
                }

                any_clicked |= clicked;
                any_hovered |= hovered;

                let pill_style = Style::new()
                    .width(70.0)
                    .height(20.0)
                    .background(Color::rgba(0.125, 0.133, 0.153, 0.98))
                    .border(
                        1.0,
                        if hovered || is_dragging {
                            Color::rgba(0.35, 0.50, 0.70, 0.95)
                        } else {
                            Color::rgba(0.180, 0.192, 0.227, 0.85)
                        },
                    )
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .padding_insets(Insets::new(0.0, 4.0, 0.0, 4.0));

                if let Some(node) = slot.tree.get_mut(axis_box_id) {
                    node.set_name(box_name);
                    node.set_style(pill_style);
                    node.role = WidgetRole::NumericInput;
                    node.interactive = true;
                    node.cursor = Some(WidgetCursor::EwResize);
                }

                let formatted_val =
                    crate::declarative::types::format_compact_float(values[idx], options.decimals);
                let text_val = if options.unit.is_empty() {
                    format!("{}: {}", axis_name, formatted_val)
                } else {
                    format!("{}: {} {}", axis_name, formatted_val, options.unit)
                };

                let mut pill_scope = slot.child_scope(axis_box_id);
                pill_scope.label_styled_passive(
                    txt_name,
                    text_val,
                    10.5,
                    colors[idx],
                    TextAlign::Center,
                    Style::new().width(64.0).height(16.0),
                );
            }

            widget_response =
                WidgetResponse::new(slot.parent, any_clicked, any_hovered, any_changed);
            slot_executed = true;
        });

        if !slot_executed {
            widget_response = self.response(row_id);
        }

        PropertyVec2Response {
            response: widget_response,
            axes: axes_out,
            changed_axis: changed_axis_out,
            committed_axis: committed_axis_out,
            old_values: initial_values,
            new_values: *values,
        }
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
                if let Some(restored) = slot
                    .tree
                    .event_state
                    .numeric_undo
                    .take_pending_scalar(axis_tag)
                {
                    let next = restored.clamp(0.0, 1.0);
                    if (next - rgb[idx]).abs() > f32::EPSILON {
                        rgb[idx] = next;
                        any_changed = true;
                    }
                }
                if let Some(delta) = drag_delta {
                    slot.tree
                        .event_state
                        .numeric_undo
                        .begin_drag(axis_tag, rgb[idx]);
                    let next = (rgb[idx] + delta.x * speed).clamp(0.0, 1.0);
                    if (next - rgb[idx]).abs() > f32::EPSILON {
                        rgb[idx] = next;
                        any_changed = true;
                    }
                }
                let is_drag_released = (slot.tree.event_state.frame_drag_released_tag
                    == Some(axis_tag)
                    && axis_tag != 0)
                    || slot.tree.event_state.frame_drag_released_id == Some(axis_box_id);
                if is_drag_released {
                    slot.tree
                        .event_state
                        .numeric_undo
                        .commit_drag(axis_tag, rgb[idx]);
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