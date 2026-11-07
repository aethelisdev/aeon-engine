// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative 3D Vector Drag Builder & Response (`DragVec3`, `Vec3Response`)
//!
//! Provides the fluent [`DragVec3`] widget builder implementing [`Widget`] and yielding
//! a rich [`Vec3Response`] with per-axis change tracking, immediate in-place mutation,
//! and dedicated reset action state without leaking internal tag arithmetic.

use std::ops::{Deref, RangeInclusive};

use iris_core::{WidgetId, WidgetResponse};

use crate::controls::numeric_drag::{DEFAULT_PIXELS_PER_STEP, DragCurve};
use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::properties::PropertyVec3Options;
use crate::declarative::widget::Widget;
use crate::numeric_input::NumericInputEditState;

/// Comprehensive interaction response returned by evaluating a [`DragVec3`] widget.
///
/// Implements [`Deref<Target = WidgetResponse>`] for transparent access to generic
/// interaction flags (`clicked()`, `hovered()`, `changed()`) while augmenting them with
/// fine-grained axis identification and dedicated reset button click detection.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec3Response {
    /// Underlying generic widget response for the entire composite row.
    pub response: WidgetResponse,
    /// 0-indexed axis that changed during this frame (0 for X, 1 for Y, 2 for Z).
    pub changed_axis: Option<usize>,
    /// 0-indexed axis that was committed during this frame (via Enter, blur, or drag release).
    pub committed_axis: Option<usize>,
    /// Whether the dedicated reset button was clicked during this frame.
    pub reset_clicked: bool,
    /// Whether any axis input in this row is currently being actively edited via keyboard input.
    pub editing_text: bool,
    /// Allocated widget identifiers for the X, Y, and Z axis numeric inputs.
    pub axes: [WidgetId; 3],
    /// Allocated widget identifier for the axis reset button, if enabled.
    pub reset_btn: Option<WidgetId>,
    /// Pre-interaction `[x, y, z]` vector snapshot captured at the start of the frame.
    pub old_values: [f32; 3],
    /// Post-interaction `[x, y, z]` vector snapshot captured after frame evaluation and range clamping.
    pub new_values: [f32; 3],
}

impl Deref for Vec3Response {
    type Target = WidgetResponse;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl Vec3Response {
    /// Returns whether any axis value in the vector changed or was edited during this frame.
    #[inline]
    #[must_use]
    pub fn changed(&self) -> bool {
        self.response.changed() || self.changed_axis.is_some()
    }

    /// Returns whether any vector scalar component numerically differs between `old_values` and `new_values`.
    #[inline]
    #[must_use]
    pub fn value_changed(&self) -> bool {
        (self.new_values[0] - self.old_values[0]).abs() > f32::EPSILON
            || (self.new_values[1] - self.old_values[1]).abs() > f32::EPSILON
            || (self.new_values[2] - self.old_values[2]).abs() > f32::EPSILON
    }

    /// Returns whether the overall row or its reset button was clicked during this frame.
    #[inline]
    #[must_use]
    pub fn clicked(&self) -> bool {
        self.response.clicked() || self.reset_clicked
    }

    /// Returns the 0-indexed axis that changed during this frame, if any.
    #[inline]
    #[must_use]
    pub const fn changed_axis(&self) -> Option<usize> {
        self.changed_axis
    }

    /// Returns whether the dedicated reset button was clicked during this frame.
    #[inline]
    #[must_use]
    pub const fn reset_clicked(&self) -> bool {
        self.reset_clicked
    }

    /// Returns whether any axis value in the vector was committed during this frame.
    #[inline]
    #[must_use]
    pub fn committed(&self) -> bool {
        self.response
            .flags
            .contains(iris_core::response::ResponseFlags::SUBMITTED)
            || self.committed_axis.is_some()
    }

    /// Returns the 0-indexed axis that was committed during this frame, if any.
    #[inline]
    #[must_use]
    pub const fn committed_axis(&self) -> Option<usize> {
        self.committed_axis
    }

    /// Invokes `f(old_values, new_values)` if the dedicated reset button was clicked during this frame.
    #[inline]
    pub fn on_reset(self, f: impl FnOnce([f32; 3], [f32; 3])) -> Self {
        if self.reset_clicked {
            f(self.old_values, self.new_values);
        }
        self
    }

    /// Invokes `f(old_values, new_values)` if any vector component changed or the reset button was clicked.
    #[inline]
    pub fn on_change(self, f: impl FnOnce([f32; 3], [f32; 3])) -> Self {
        if self.value_changed() || self.reset_clicked {
            f(self.old_values, self.new_values);
        }
        self
    }

    /// Invokes `f(old_values, new_values)` during an un-committed live interaction (such as active pointer dragging).
    #[inline]
    pub fn on_live_edit(self, f: impl FnOnce([f32; 3], [f32; 3])) -> Self {
        if !self.reset_clicked && self.value_changed() && !self.committed() {
            f(self.old_values, self.new_values);
        }
        self
    }

    /// Invokes `f(old_values, new_values)` when an edit is atomically committed (via reset button click, Enter confirmation, or drag release with delta).
    #[inline]
    pub fn on_commit_edit(self, f: impl FnOnce([f32; 3], [f32; 3])) -> Self {
        if self.reset_clicked || (self.value_changed() && self.committed()) {
            f(self.old_values, self.new_values);
        }
        self
    }

    /// Invokes `f()` when an interaction session is committed on a frame without value delta (such as pointer drag release after prior live drag frames).
    #[inline]
    pub fn on_release(self, f: impl FnOnce()) -> Self {
        if !self.reset_clicked && !self.value_changed() && self.committed() {
            f();
        }
        self
    }
}

/// Internal storage target for [`DragVec3`], supporting both mutable borrows and owned value expressions.
enum Vec3Target<'a> {
    Borrowed(&'a mut [f32; 3]),
    Owned([f32; 3]),
}

/// Declarative 3D coordinate vector property drag builder implementing [`Widget`].
///
/// Modifies `values` (`[x, y, z]`) in place based on continuous pointer drag deltas,
/// direct keyboard typing, and optional reset button clicks.
///
/// # Example
/// ```ignore
/// let resp = scope.add(
///     DragVec3::new("Position", &mut position)
///         .speed(0.1)
///         .range(-1000.0..=1000.0)
///         .reset([0.0, 0.0, 0.0])
/// );
/// if let Some(axis) = resp.changed_axis() {
///     // Handle axis change
/// }
/// ```
pub struct DragVec3<'a> {
    label: &'a str,
    values: Vec3Target<'a>,
    speed: f32,
    range: RangeInclusive<f32>,
    decimals: usize,
    reset_value: Option<[f32; 3]>,
    label_width: f32,
    edit_states: [Option<NumericInputEditState<'a>>; 3],
    curve: DragCurve,
    pixels_per_step: f32,
}

impl<'a> DragVec3<'a> {
    /// Creates a new [`DragVec3`] builder for the given label and mutable 3-element float array.
    ///
    /// Defaults to `speed: 0.1`, `range: -10000.0..=10000.0`, `decimals: 3`, and `label_width: 52.0`.
    #[inline]
    pub fn new(label: &'a str, values: &'a mut [f32; 3]) -> Self {
        Self {
            label,
            values: Vec3Target::Borrowed(values),
            speed: 0.1,
            range: -10000.0..=10000.0,
            decimals: 3,
            reset_value: None,
            label_width: 52.0,
            edit_states: [None, None, None],
            curve: DragCurve::Linear,
            pixels_per_step: DEFAULT_PIXELS_PER_STEP as f32,
        }
    }

    /// Creates a new [`DragVec3`] builder initialized from an owned `[f32; 3]` value expression.
    ///
    /// Enables pure declarative binding without intermediate local `let mut` buffers when paired
    /// with [`Vec3Response::on_change`], [`Vec3Response::on_live_edit`], [`Vec3Response::on_commit_edit`],
    /// and [`Vec3Response::on_reset`].
    #[inline]
    #[must_use]
    pub fn from_val(label: &'a str, values: [f32; 3]) -> Self {
        Self {
            label,
            values: Vec3Target::Owned(values),
            speed: 0.1,
            range: -10000.0..=10000.0,
            decimals: 3,
            reset_value: None,
            label_width: 52.0,
            edit_states: [None, None, None],
            curve: DragCurve::Linear,
            pixels_per_step: DEFAULT_PIXELS_PER_STEP as f32,
        }
    }

    /// Sets the response curve for pointer drag scrubbing.
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

    /// Sets the continuous drag sensitivity multiplier applied per horizontal delta pixel.
    #[inline]
    #[must_use]
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Constrains all vector components within an inclusive range `[min, max]`.
    #[inline]
    #[must_use]
    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    /// Sets the minimum allowable value for all vector components.
    #[inline]
    #[must_use]
    pub fn min(mut self, min: f32) -> Self {
        self.range = min..=*self.range.end();
        self
    }

    /// Sets the maximum allowable value for all vector components.
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

    /// Enables the dedicated reset button reverting the vector to `reset_value` upon click.
    #[inline]
    #[must_use]
    pub fn reset(mut self, reset_value: [f32; 3]) -> Self {
        self.reset_value = Some(reset_value);
        self
    }

    /// Sets the fixed column width in pixels for the left-aligned property label.
    #[inline]
    #[must_use]
    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = width;
        self
    }

    /// Attaches optional active numeric input inline editing states for the X, Y, and Z channels.
    #[inline]
    #[must_use]
    pub fn edit_states(mut self, edit_states: [Option<NumericInputEditState<'a>>; 3]) -> Self {
        self.edit_states = edit_states;
        self
    }
}

impl<'a> Widget for DragVec3<'a> {
    type Response = Vec3Response;

    fn ui(mut self, scope: &mut UiScope<'_>) -> Self::Response {
        let is_any_editing = self.edit_states.iter().any(|s| s.is_some());
        let mut opts = PropertyVec3Options::new(self.speed)
            .with_decimals(self.decimals)
            .with_label_width(self.label_width)
            .with_edit_states(self.edit_states)
            .with_curve(self.curve)
            .with_pixels_per_step(self.pixels_per_step);

        if let Some(rv) = self.reset_value {
            opts = opts.with_reset(rv);
        }

        let min_val = *self.range.start();
        let max_val = *self.range.end();

        let val_ref: &mut [f32; 3] = match &mut self.values {
            Vec3Target::Borrowed(r) => r,
            Vec3Target::Owned(arr) => arr,
        };
        let old_values = *val_ref;

        let prop_resp = scope.property_vec3_with_options(self.label, val_ref, opts);
        for v in val_ref.iter_mut() {
            *v = v.clamp(min_val, max_val);
        }
        let new_values = *val_ref;

        Vec3Response {
            response: prop_resp.response,
            changed_axis: prop_resp.changed_axis,
            committed_axis: prop_resp.committed_axis,
            reset_clicked: prop_resp.reset_clicked,
            editing_text: is_any_editing,
            axes: prop_resp.axes,
            reset_btn: prop_resp.reset_btn,
            old_values,
            new_values,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::UiTree;

    #[test]
    fn test_drag_vec3_builder_and_response() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        let mut values = [1.0, 2.0, 3.0];
        let resp = scope.add(
            DragVec3::new("Scale", &mut values)
                .speed(0.05)
                .range(0.1..=10.0)
                .decimals(2)
                .reset([1.0, 1.0, 1.0])
                .label_width(90.0),
        );

        assert!(!resp.changed());
        assert!(!resp.reset_clicked());
        assert_eq!(resp.changed_axis(), None);
        assert!(!resp.editing_text);
        assert_ne!(resp.axes[0], WidgetId::default());
        assert_ne!(resp.axes[1], WidgetId::default());
        assert_ne!(resp.axes[2], WidgetId::default());
        assert!(resp.reset_btn.is_some());
        assert_eq!(values, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_drag_vec3_range_clamping() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        let mut values = [-50.0, 150.0, 5.0];
        let _ = scope.add(DragVec3::new("Bounded", &mut values).range(0.0..=100.0));

        assert_eq!(values, [0.0, 100.0, 5.0]);
    }

    #[test]
    fn test_drag_vec3_shorthand_on_scope() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        let mut values = [10.0, 20.0, 30.0];
        let resp = scope.drag_vec3("Position", &mut values, 0.2);

        assert_eq!(values, [10.0, 20.0, 30.0]);
        assert_eq!(resp.changed_axis(), None);
    }

    #[test]
    fn test_drag_vec3_enter_confirmation_and_committed_axis() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");

        let mut values = [1.0, 2.0, 3.0];
        let label = "TransformPos";
        let base_tag = {
            let scope = UiScope::new(&mut tree, root);
            scope.tag_for(label)
        };
        let y_axis_tag =
            crate::declarative::scope::properties::vectors::property_vec3_axis_tag(base_tag, 1);

        // Simulate active editing session on Y axis
        tree.event_state
            .start_text_edit(None, y_axis_tag, "45.0", 4, None);
        tree.event_state.focused_tag = Some(y_axis_tag);
        tree.event_state.frame_key = Some(iris_core::event::KeyCode::Enter);
        tree.event_state.frame_key_pressed = true;

        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(DragVec3::new(label, &mut values).speed(0.1));

        assert_eq!(values, [1.0, 45.0, 3.0]);
        assert!(resp.changed());
        assert!(resp.committed());
        assert_eq!(resp.changed_axis(), Some(1));
        assert_eq!(resp.committed_axis(), Some(1));
        assert!(tree.event_state.active_text_edit.is_none());
    }

    #[test]
    fn test_drag_vec3_from_val_and_fluent_callbacks_on_reset_and_commit() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");

        let label = "Position";
        let base_tag = {
            let scope = UiScope::new(&mut tree, root);
            scope.tag_for(label)
        };
        let reset_tag =
            crate::declarative::scope::properties::vectors::property_vec3_reset_tag(base_tag);

        tree.event_state.frame_clicked_tag = Some(reset_tag);

        let mut observed_change = None;
        let mut observed_commit = None;
        let mut observed_reset = None;
        let mut observed_live = false;

        let mut scope = UiScope::new(&mut tree, root);
        scope
            .add(
                DragVec3::from_val(label, [3.0, 4.0, 5.0])
                    .speed(0.1)
                    .reset([0.0, 0.0, 0.0]),
            )
            .on_change(|old_v, new_v| {
                observed_change = Some((old_v, new_v));
            })
            .on_reset(|old_v, new_v| {
                observed_reset = Some((old_v, new_v));
            })
            .on_live_edit(|_, _| {
                observed_live = true;
            })
            .on_commit_edit(|old_v, new_v| {
                observed_commit = Some((old_v, new_v));
            });

        assert_eq!(observed_change, Some(([3.0, 4.0, 5.0], [0.0, 0.0, 0.0])));
        assert_eq!(observed_reset, Some(([3.0, 4.0, 5.0], [0.0, 0.0, 0.0])));
        assert_eq!(observed_commit, Some(([3.0, 4.0, 5.0], [0.0, 0.0, 0.0])));
        assert!(!observed_live);
    }
}