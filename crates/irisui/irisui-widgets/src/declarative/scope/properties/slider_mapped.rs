// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit-Mapped Bidirectional Property Sliders
//!
//! Extends [`UiScope`] with sliders that expose converted display units (such as
//! degrees for stored radians) without requiring local temporary variables in views.

use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::properties::PropertySliderOptions;
use crate::declarative::types::WidgetResponse;

impl<'a> UiScope<'a> {
    /// Emits a two-way property slider performing bidirectional unit mapping around `stored_value`.
    ///
    /// Converts `*stored_value` via `to_display` before drawing the slider, then writes back
    /// `from_display(display_val)` when the edited display value diverges from the converted input.
    ///
    /// # Arguments
    /// * `label` - Property label displayed on the left (supports `##` separator).
    /// * `stored_value` - Persistent scalar reference stored in canonical units (e.g. radians).
    /// * `to_display` - Conversion from canonical units to display slider units (e.g. `f32::to_degrees`).
    /// * `from_display` - Inverse conversion from display slider units to canonical units (e.g. `f32::to_radians`).
    /// * `options` - Slider bounds, step speed, format string, and optional subtitle in display units.
    pub fn property_slider_mapped(
        &mut self,
        label: &str,
        stored_value: &mut f32,
        to_display: fn(f32) -> f32,
        from_display: fn(f32) -> f32,
        options: PropertySliderOptions<'_>,
    ) -> WidgetResponse {
        let initial_display = to_display(*stored_value);
        let mut display_value = initial_display;
        let resp = self.property_slider_with_options(label, &mut display_value, options);
        if (display_value - initial_display).abs() > 1e-4 {
            *stored_value = from_display(display_value);
        }
        resp
    }
}

#[cfg(test)]
mod tests {
    use irisui_core::{InteractionEvent, Point, Rect, UiTree};

    use crate::declarative::scope::UiScope;
    use crate::declarative::scope::properties::PropertySliderOptions;

    #[test]
    fn test_property_slider_mapped_converts_radians_and_degrees_bidirectionally() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let slider_tag = {
            let scope = UiScope::new(&mut tree, root);
            scope.tag_for("Sun Pitch")
        };
        tree.tag_rects
            .insert(slider_tag, Rect::new(0.0, 0.0, 240.0, 20.0));

        let mut pitch_rad = 0.0_f32;
        let events = [(
            slider_tag,
            InteractionEvent::Drag {
                delta: Point::new(30.0, 0.0),
            },
        )];
        let mut scope = UiScope::with_tagged_interactions(&mut tree, root, &events, None);
        scope.property_slider_mapped(
            "Sun Pitch",
            &mut pitch_rad,
            f32::to_degrees,
            f32::to_radians,
            PropertySliderOptions::new(-180.0, 180.0, 1.0),
        );

        let pitch_deg = pitch_rad.to_degrees();
        assert!(
            (pitch_deg - 30.0).abs() < 1e-2,
            "Expected stored radians to reflect +30 deg drag, got {pitch_deg} deg"
        );
    }
}