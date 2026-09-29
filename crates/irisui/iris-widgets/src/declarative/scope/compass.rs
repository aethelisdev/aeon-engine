// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Orientation Compass Scope Extension
//!
//! Provides interactive orientation compass knobs and negative axis dots for 3D viewport
//! navigation gizmos via declarative [`UiScope`] methods.
//!

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{Color, Style, TextAlign, WidgetCursor, WidgetRole};

impl<'a> UiScope<'a> {
    /// Emits a circular interactive compass knob button representing a positive axis endpoint.
    ///
    /// Positioned absolutely within its parent compass canvas container. Applies circular
    /// geometry, pointer cursor, and interactive event response for clicking and hover highlighting.
    ///
    /// # Arguments
    /// * `label` - Single-character axis indicator caption (e.g. "X", "Y", "Z").
    /// * `tag` - Unique 64-bit semantic tag for hit-testing and action dispatching.
    /// * `x` - Left coordinate offset relative to parent compass canvas in logical points.
    /// * `y` - Top coordinate offset relative to parent compass canvas in logical points.
    /// * `size` - Diameter of the circular knob button in logical points.
    /// * `color` - Idle background fill color representing the target axis.
    pub fn compass_knob(
        &mut self,
        label: impl Into<String>,
        tag: u64,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
    ) -> WidgetResponse {
        let label_str = label.into();
        let knob_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(knob_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, knob_id);
        let (clicked, hovered, _) = self.check_interaction(knob_id);

        if let Some(node) = self.tree.get_mut(knob_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_name("CompassKnob");
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(x)
                    .top(y)
                    .width(size)
                    .height(size)
                    .background(if hovered { Color::WHITE } else { color })
                    .border_radius(size * 0.5),
            );
        }

        let txt_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(txt_id) {
            node.interactive = false;
            node.set_name("CompassKnobText");
            node.set_text(label_str);
            node.font_size = 8.5;
            node.line_height = size;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered { Color::BLACK } else { Color::WHITE };
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(0.0)
                    .top(0.0)
                    .width(size)
                    .height(size),
            );
        }
        let _ = self.tree.add_child(knob_id, txt_id);

        WidgetResponse::new(knob_id, clicked, hovered, false)
    }

    /// Emits a small circular interactive compass dot representing a negative axis endpoint.
    ///
    /// Positioned absolutely within its parent compass canvas container. Applies circular
    /// geometry, pointer cursor, and interactive event response for clicking and hover highlighting.
    ///
    /// # Arguments
    /// * `tag` - Unique 64-bit semantic tag for hit-testing and action dispatching.
    /// * `x` - Left coordinate offset relative to parent compass canvas in logical points.
    /// * `y` - Top coordinate offset relative to parent compass canvas in logical points.
    /// * `size` - Diameter of the circular dot in logical points.
    /// * `color` - Idle background fill color representing the negative axis.
    pub fn compass_dot(
        &mut self,
        tag: u64,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
    ) -> WidgetResponse {
        let dot_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(dot_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, dot_id);
        let (clicked, hovered, _) = self.check_interaction(dot_id);

        if let Some(node) = self.tree.get_mut(dot_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_name("CompassDot");
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(x)
                    .top(y)
                    .width(size)
                    .height(size)
                    .background(if hovered { Color::WHITE } else { color })
                    .border_radius(size * 0.5),
            );
        }

        WidgetResponse::new(dot_id, clicked, hovered, false)
    }
}