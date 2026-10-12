// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Form Inputs, Checkboxes & Scalar Drag Editors
//!
//! Provides interactive property inspection inputs on [`UiScope`]: boolean checkboxes,
//! drag-float scalar editors, 3D vector inspectors, and text input boxes.
//!

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetId, WidgetRole,
};

impl<'a> UiScope<'a> {
    /// Emits an interactive boolean checkbox toggling state in-place upon click.
    ///
    /// Generates a deterministic persistent tag from `label` to support cross-frame
    /// interaction tracking, hover highlighting, and hit testing without manual identifier management.
    ///
    /// # Arguments
    /// * `label` - Display text beside the checkbox indicator.
    /// * `checked` - Mutable reference to the underlying boolean value.
    pub fn checkbox(&mut self, label: impl Into<String>, checked: &mut bool) -> WidgetResponse {
        let label_str = label.into();
        let (visible, _) = crate::declarative::types::split_label_id(&label_str);
        let tag = self.tag_for(&label_str);
        self.checkbox_tagged(visible, tag, checked)
    }

    /// Emits an interactive boolean checkbox with an explicit semantic tag.
    ///
    /// Automatically applies hover highlighting, pointer cursor, and toggles `*checked` state upon click.
    ///
    /// # Arguments
    /// * `label` - Display text beside the checkbox indicator.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    /// * `checked` - Mutable reference to the underlying boolean value.
    pub fn checkbox_tagged(
        &mut self,
        label: impl Into<String>,
        tag: u64,
        checked: &mut bool,
    ) -> WidgetResponse {
        let node_id = self.tree.create_node();
        let label_str = label.into();
        let indicator = if *checked { "[x]" } else { "[ ]" };
        let full_text = format!("{} {}", indicator, label_str);

        let _ = self.tree.add_child(self.parent, node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("Checkbox");
            node.tag = tag;
        }

        let (clicked, hovered, _) = self.check_interaction(node_id);
        let pressed = self.is_pressed(node_id);
        let mut changed = false;
        if clicked {
            *checked = !*checked;
            changed = true;
        }

        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Checkbox;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(full_text);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Left;
            node.text_color = if *checked {
                Color::rgba(0.2, 0.85, 0.65, 1.0)
            } else if hovered || pressed {
                Color::WHITE
            } else {
                Color::rgba(0.70, 0.72, 0.78, 1.0)
            };
            node.hover_text_color = Some(Color::WHITE);

            let mut style = Style::new()
                .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
                .align_items(AlignItems::Center);
            style.hover_background = Some(Color::rgba(0.25, 0.28, 0.35, 0.40));
            if pressed {
                style.background_color = Color::rgba(0.20, 0.22, 0.28, 0.60);
            } else if hovered && let Some(hbg) = style.hover_background {
                style.background_color = hbg;
            }
            node.set_style(style);
        }

        let mut resp = WidgetResponse::new(node_id, clicked, hovered, changed);
        if pressed {
            resp = resp.with_pressed(true);
        }
        resp
    }

    /// Emits a numeric drag-value pill component for floating-point scalar properties.
    ///
    /// # Arguments
    /// * `tag` - Semantic identifier to map hover, drag, and numeric scroll events.
    /// * `prefix` - Short axis or property prefix (e.g. `"X: "`, `"Y: "`, `"Z: "`).
    /// * `value` - Floating-point scalar value to display.
    pub fn drag_value(&mut self, tag: u64, prefix: &str, value: f32) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::NumericInput;
            node.cursor = Some(WidgetCursor::EwResize);
            node.tag = tag;
            node.set_text(format!("{}{:.2}", prefix, value));
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = Color::rgba(0.85, 0.88, 0.94, 1.0);
            node.set_style(
                Style::new()
                    .padding_insets(Insets::new(3.0, 6.0, 3.0, 6.0))
                    .background(Color::rgba(0.14, 0.15, 0.18, 0.95))
                    .border(1.0, Color::rgba(0.24, 0.25, 0.30, 0.8))
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits an interactive numeric scrubber dragging scalar values horizontally in-place.
    ///
    /// Generates a deterministic persistent tag from `label` to support cross-frame
    /// pointer capture, horizontal displacement tracking, and in-place value modification.
    /// Employs a default dragging speed multiplier of `0.1` and unbounded range (`f32::NEG_INFINITY..f32::INFINITY`).
    ///
    /// # Arguments
    /// * `label` - Descriptive property label displayed on the scrubber.
    /// * `value` - Mutable reference to the floating-point value modified in-place upon dragging.
    ///
    /// # Returns
    /// A [`WidgetResponse`] reflecting hovered, clicked, dragged, and changed state flags.
    pub fn numeric_drag(&mut self, label: impl Into<String>, value: &mut f32) -> WidgetResponse {
        self.numeric_drag_range(label, value, 0.1, f32::NEG_INFINITY, f32::INFINITY)
    }

    /// Emits an interactive numeric scrubber constrained within an explicit `[min, max]` range.
    ///
    /// Generates a deterministic persistent tag from `label` and delegates to [`Self::numeric_drag_tagged`].
    ///
    /// # Arguments
    /// * `label` - Descriptive property label displayed on the scrubber.
    /// * `value` - Mutable reference to the floating-point scalar value.
    /// * `speed` - Multiplier applied to pointer horizontal displacement delta.
    /// * `min` - Minimum allowable scalar value bound.
    /// * `max` - Maximum allowable scalar value bound.
    ///
    /// # Returns
    /// A [`WidgetResponse`] reflecting hovered, clicked, dragged, and changed state flags.
    pub fn numeric_drag_range(
        &mut self,
        label: impl Into<String>,
        value: &mut f32,
        speed: f32,
        min: f32,
        max: f32,
    ) -> WidgetResponse {
        let label_str = label.into();
        let (visible, _) = crate::declarative::types::split_label_id(&label_str);
        let tag = self.tag_for(&label_str);
        self.numeric_drag_tagged(visible, tag, value, speed, min, max)
    }

    /// Emits an interactive numeric scrubber with an explicit semantic tag and boundary constraints.
    ///
    /// Captures pointer events upon mouse press, displays horizontal resize cursor ([`WidgetCursor::EwResize`]),
    /// updates `*value` based on relative pointer deltas, and returns [`WidgetResponse`] with `.changed()` flag.
    ///
    /// # Arguments
    /// * `label` - Display text displayed alongside the scalar value.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    /// * `value` - Mutable reference to the scalar floating-point value.
    /// * `speed` - Multiplier applied to pointer horizontal displacement delta.
    /// * `min` - Minimum allowable scalar value bound.
    /// * `max` - Maximum allowable scalar value bound.
    ///
    /// # Returns
    /// A [`WidgetResponse`] reflecting hovered, clicked, dragged, and changed state flags.
    pub fn numeric_drag_tagged(
        &mut self,
        label: impl Into<String>,
        tag: u64,
        value: &mut f32,
        speed: f32,
        min: f32,
        max: f32,
    ) -> WidgetResponse {
        let node_id = self.tree.create_node();
        let label_str = label.into();
        let text = format!("{}: {:.2}", label_str, *value);

        let _ = self.tree.add_child(self.parent, node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("NumericDrag");
            node.tag = tag;
        }

        let (clicked, hovered, drag_delta) = self.check_interaction(node_id);
        let is_pressed = self.is_pressed(node_id);
        let is_dragging = drag_delta.is_some()
            || self.tree.event_state.pointer_capture == Some(node_id)
            || (tag != 0
                && self
                    .tree
                    .event_state
                    .pointer_capture
                    .and_then(|cid| self.tree.get(cid))
                    .is_some_and(|n| n.tag == tag));

        let mut changed = false;
        if let Some(delta) = drag_delta {
            let next = (*value + delta.x * speed).clamp(min, max);
            if (next - *value).abs() > f32::EPSILON {
                *value = next;
                changed = true;
            }
        }

        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::NumericInput;
            node.cursor = Some(WidgetCursor::EwResize);
            node.set_text(if changed {
                format!("{}: {:.2}", label_str, *value)
            } else {
                text
            });
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if is_dragging {
                Color::rgba(0.35, 0.85, 0.95, 1.0)
            } else if hovered || is_pressed {
                Color::WHITE
            } else {
                Color::rgba(0.85, 0.88, 0.94, 1.0)
            };

            let mut style = Style::new()
                .padding_insets(Insets::new(3.0, 6.0, 3.0, 6.0))
                .background(if is_dragging {
                    Color::rgba(0.20, 0.28, 0.38, 0.98)
                } else if is_pressed {
                    Color::rgba(0.18, 0.20, 0.26, 0.98)
                } else if hovered {
                    Color::rgba(0.18, 0.20, 0.25, 0.95)
                } else {
                    Color::rgba(0.14, 0.15, 0.18, 0.95)
                })
                .border(
                    1.0,
                    if is_dragging {
                        Color::rgba(0.35, 0.75, 0.95, 0.9)
                    } else if hovered {
                        Color::rgba(0.30, 0.35, 0.45, 0.8)
                    } else {
                        Color::rgba(0.24, 0.25, 0.30, 0.8)
                    },
                )
                .border_radius(3.0)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);
            style.hover_background = Some(Color::rgba(0.18, 0.20, 0.25, 0.95));
            node.set_style(style);
        }

        let mut resp = WidgetResponse::new(node_id, clicked, hovered, changed);
        if is_pressed {
            resp = resp.with_pressed(true);
        }
        if is_dragging {
            resp = resp.with_dragged(true);
        }
        resp
    }

    /// Emits a property row with an interactive drag-float scalar editor.
    ///
    /// Automatically updates `*value` upon drag interaction according to `speed`,
    /// clamped within the specified `[min, max]` range.
    ///
    /// # Arguments
    /// * `label` - Descriptive property label displayed on the left.
    /// * `value` - Mutable reference to the floating-point value.
    /// * `speed` - Multiplier applied to pointer horizontal displacement delta.
    /// * `min` - Minimum permissible scalar value.
    /// * `max` - Maximum permissible scalar value.
    pub fn drag_float(
        &mut self,
        label: &str,
        value: &mut f32,
        speed: f32,
        min: f32,
        max: f32,
    ) -> WidgetResponse {
        self.numeric_drag_range(label, value, speed, min, max)
    }

    /// Emits a 3D vector property row with inline X, Y, Z drag-value editors.
    ///
    /// # Arguments
    /// * `label` - Descriptive vector label (e.g. `"Position"`, `"Rotation"`).
    /// * `values` - Mutable slice containing `[x, y, z]` vector coordinates.
    /// * `speed` - Multiplier applied to drag displacement.
    pub fn drag_vec3(&mut self, label: &str, values: &mut [f32; 3], speed: f32) -> WidgetResponse {
        let mut any_changed = false;
        let mut any_clicked = false;
        let mut any_hovered = false;

        let row_id = self.row(|row| {
            row.text(label);
            let rx = row.drag_float("X", &mut values[0], speed, -10_000.0, 10_000.0);
            let ry = row.drag_float("Y", &mut values[1], speed, -10_000.0, 10_000.0);
            let rz = row.drag_float("Z", &mut values[2], speed, -10_000.0, 10_000.0);

            any_changed = rx.changed() || ry.changed() || rz.changed();
            any_clicked = rx.clicked() || ry.clicked() || rz.clicked();
            any_hovered = rx.hovered() || ry.hovered() || rz.hovered();
        });

        WidgetResponse::new(row_id, any_clicked, any_hovered, any_changed)
    }
}

pub use super::input_box::*;