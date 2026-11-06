// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative 2D Vector Drag Builder & Response (`DragVec2`, `Vec2Response`)
//!
//! Provides the fluent [`DragVec2`] widget builder implementing [`Widget`] and yielding
//! a rich [`Vec2Response`] with per-axis change tracking and immediate in-place mutation
//! without leaking internal tag arithmetic.

use std::ops::{Deref, RangeInclusive};

use iris_core::{WidgetId, WidgetResponse};

use crate::controls::numeric_drag::{DEFAULT_PIXELS_PER_STEP, DragCurve};
use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::properties::PropertyVec2Options;
use crate::declarative::widget::Widget;

/// Comprehensive interaction response returned by evaluating a [`DragVec2`] widget.
///
/// Implements [`Deref<Target = WidgetResponse>`] for transparent access to generic
/// interaction flags (`clicked()`, `hovered()`, `changed()`) while augmenting them with
/// fine-grained axis identification.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2Response {
    /// Underlying generic widget response for the entire composite row.
    pub response: WidgetResponse,
    /// 0-indexed axis that changed during this frame (0 for axis 0, 1 for axis 1).
    pub changed_axis: Option<usize>,
    /// 0-indexed axis that was committed during this frame (via Enter, blur, or drag release).
    pub committed_axis: Option<usize>,
    /// Allocated widget identifiers for the two axis numeric inputs.
    pub axes: [WidgetId; 2],
}

impl Deref for Vec2Response {
    type Target = WidgetResponse;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl Vec2Response {
    /// Returns whether any axis value in the vector changed or was edited during this frame.
    #[inline]
    #[must_use]
    pub fn changed(&self) -> bool {
        self.response.changed() || self.changed_axis.is_some()
    }

    /// Returns whether the overall row was clicked during this frame.
    #[inline]
    #[must_use]
    pub fn clicked(&self) -> bool {
        self.response.clicked()
    }

    /// Returns the 0-indexed axis that changed during this frame, if any.
    #[inline]
    #[must_use]
    pub const fn changed_axis(&self) -> Option<usize> {
        self.changed_axis
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
}

/// Declarative 2D coordinate vector property drag builder implementing [`Widget`].
///
/// Modifies `values` (`[x, y]`) in place based on continuous pointer drag deltas
/// and direct keyboard typing.
///
/// # Example
/// ```ignore
/// let resp = scope.add(
///     DragVec2::new("Offset", &mut offset)
///         .axis_labels(["X", "Y"])
///         .speed(1.0)
///         .unit("px")
/// );
/// ```
pub struct DragVec2<'a> {
    label: &'a str,
    values: &'a mut [f32; 2],
    axis_labels: [&'a str; 2],
    unit: &'a str,
    speed: f32,
    range: RangeInclusive<f32>,
    decimals: usize,
    label_width: f32,
    curve: DragCurve,
    pixels_per_step: f32,
}

impl<'a> DragVec2<'a> {
    /// Creates a new [`DragVec2`] builder for the given label and mutable 2-element float array.
    ///
    /// Defaults to `axis_labels: ["X", "Y"]`, `unit: ""`, `speed: 0.1`, `range: -10000.0..=10000.0`,
    /// `decimals: 2`, and `label_width: 52.0`.
    #[inline]
    pub fn new(label: &'a str, values: &'a mut [f32; 2]) -> Self {
        Self {
            label,
            values,
            axis_labels: ["X", "Y"],
            unit: "",
            speed: 0.1,
            range: -10000.0..=10000.0,
            decimals: 2,
            label_width: 52.0,
            curve: DragCurve::Linear,
            pixels_per_step: DEFAULT_PIXELS_PER_STEP as f32,
        }
    }

    /// Sets the custom axis prefix tags (e.g. `["X", "Y"]` or `["W", "H"]`).
    #[inline]
    #[must_use]
    pub fn axis_labels(mut self, labels: [&'a str; 2]) -> Self {
        self.axis_labels = labels;
        self
    }

    /// Sets the physical unit suffix string (e.g. `"px"`, `""`).
    #[inline]
    #[must_use]
    pub fn unit(mut self, unit: &'a str) -> Self {
        self.unit = unit;
        self
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

    /// Sets the number of decimal places displayed for non-editing values.
    #[inline]
    #[must_use]
    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    /// Sets the fixed column width in pixels for the left-aligned property label.
    #[inline]
    #[must_use]
    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = width;
        self
    }
}

impl<'a> Widget for DragVec2<'a> {
    type Response = Vec2Response;

    fn ui(self, scope: &mut UiScope<'_>) -> Self::Response {
        let opts = PropertyVec2Options {
            speed: self.speed,
            decimals: self.decimals,
            label_width: self.label_width,
            axis_labels: self.axis_labels,
            unit: self.unit,
            range: self.range.clone(),
            curve: self.curve,
            pixels_per_step: self.pixels_per_step,
        };

        let min_val = *self.range.start();
        let max_val = *self.range.end();

        let prop_resp = scope.property_vec2_with_options(self.label, self.values, opts);
        for v in self.values.iter_mut() {
            *v = v.clamp(min_val, max_val);
        }

        Vec2Response {
            response: prop_resp.response,
            changed_axis: prop_resp.changed_axis,
            committed_axis: prop_resp.committed_axis,
            axes: prop_resp.axes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::UiTree;

    #[test]
    fn test_drag_vec2_builder_and_response() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        let mut values = [10.0, 20.0];
        let resp = scope.add(
            DragVec2::new("Offset", &mut values)
                .axis_labels(["X", "Y"])
                .unit("px")
                .speed(0.5)
                .range(0.0..=100.0)
                .decimals(0)
                .label_width(60.0),
        );

        assert!(!resp.changed());
        assert_eq!(resp.changed_axis(), None);
        assert_ne!(resp.axes[0], WidgetId::default());
        assert_ne!(resp.axes[1], WidgetId::default());
        assert_eq!(values, [10.0, 20.0]);
    }

    #[test]
    fn test_drag_vec2_range_clamping() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        let mut values = [-50.0, 150.0];
        let _ = scope.add(DragVec2::new("Bounded", &mut values).range(0.0..=100.0));

        assert_eq!(values, [0.0, 100.0]);
    }
}