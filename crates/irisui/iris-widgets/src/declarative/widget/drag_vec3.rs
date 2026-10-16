// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative 3D Vector Drag Builder & Response (`DragVec3`, `Vec3Response`)
//!
//! Provides the fluent [`DragVec3`] widget builder implementing [`Widget`] and yielding
//! a rich [`Vec3Response`] with per-axis change tracking, immediate in-place mutation,
//! and dedicated reset action state without leaking internal tag arithmetic.
//!

use std::ops::{Deref, RangeInclusive};

use iris_core::{WidgetId, WidgetResponse};

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
    /// Whether the dedicated reset button was clicked during this frame.
    pub reset_clicked: bool,
    /// Whether any axis input in this row is currently being actively edited via keyboard input.
    pub editing_text: bool,
    /// Allocated widget identifiers for the X, Y, and Z axis numeric inputs.
    pub axes: [WidgetId; 3],
    /// Allocated widget identifier for the axis reset button, if enabled.
    pub reset_btn: Option<WidgetId>,
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
    values: &'a mut [f32; 3],
    speed: f32,
    range: RangeInclusive<f32>,
    decimals: usize,
    reset_value: Option<[f32; 3]>,
    label_width: f32,
    edit_states: [Option<NumericInputEditState<'a>>; 3],
}

impl<'a> DragVec3<'a> {
    /// Creates a new [`DragVec3`] builder for the given label and mutable 3-element float array.
    ///
    /// Defaults to `speed: 0.1`, `range: -10000.0..=10000.0`, `decimals: 3`, and `label_width: 52.0`.
    #[inline]
    pub fn new(label: &'a str, values: &'a mut [f32; 3]) -> Self {
        Self {
            label,
            values,
            speed: 0.1,
            range: -10000.0..=10000.0,
            decimals: 3,
            reset_value: None,
            label_width: 52.0,
            edit_states: [None, None, None],
        }
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

    fn ui(self, scope: &mut UiScope<'_>) -> Self::Response {
        let is_any_editing = self.edit_states.iter().any(|s| s.is_some());
        let mut opts = PropertyVec3Options::new(self.speed)
            .with_decimals(self.decimals)
            .with_label_width(self.label_width)
            .with_edit_states(self.edit_states);

        if let Some(rv) = self.reset_value {
            opts = opts.with_reset(rv);
        }

        let min_val = *self.range.start();
        let max_val = *self.range.end();

        let prop_resp = scope.property_vec3_with_options(self.label, self.values, opts);
        for v in self.values.iter_mut() {
            *v = v.clamp(min_val, max_val);
        }

        Vec3Response {
            response: prop_resp.response,
            changed_axis: prop_resp.changed_axis,
            reset_clicked: prop_resp.reset_clicked,
            editing_text: is_any_editing,
            axes: prop_resp.axes,
            reset_btn: prop_resp.reset_btn,
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
}