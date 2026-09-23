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
    /// # Arguments
    /// * `label` - Display text beside the checkbox indicator.
    /// * `checked` - Mutable reference to the underlying boolean value.
    pub fn checkbox(&mut self, label: impl Into<String>, checked: &mut bool) -> WidgetResponse {
        let node_id = self.tree.create_node();
        let indicator = if *checked { "[x]" } else { "[ ]" };
        let full_text = format!("{} {}", indicator, label.into());

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
            } else {
                Color::rgba(0.70, 0.72, 0.78, 1.0)
            };
            node.set_style(
                Style::new()
                    .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
                    .align_items(AlignItems::Center),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let (clicked, hovered, _) = self.check_interaction(node_id);
        let mut changed = false;
        if clicked {
            *checked = !*checked;
            changed = true;
        }

        WidgetResponse::new(node_id, clicked, hovered, changed)
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
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::NumericInput;
            node.cursor = Some(WidgetCursor::EwResize);
            node.set_text(format!("{}: {:.2}", label, *value));
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

        let (clicked, hovered, drag_delta) = self.check_interaction(node_id);
        let mut changed = false;
        if let Some(delta) = drag_delta {
            let next = (*value + delta.x * speed).clamp(min, max);
            if (next - *value).abs() > f32::EPSILON {
                *value = next;
                changed = true;
            }
        }

        WidgetResponse::new(node_id, clicked, hovered, changed)
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

            any_changed = rx.changed || ry.changed || rz.changed;
            any_clicked = rx.clicked || ry.clicked || rz.clicked;
            any_hovered = rx.hovered || ry.hovered || rz.hovered;
        });

        WidgetResponse::new(row_id, any_clicked, any_hovered, any_changed)
    }

    /// Emits a styled text input display box with placeholder support and an optional blinking caret.
    ///
    /// # Arguments
    /// * `text` - Currently entered string value.
    /// * `placeholder` - Prompt string displayed when `text` is empty.
    /// * `text_width` - Measured width of the active text string for caret positioning.
    /// * `cursor_blink_visible` - Whether the editing caret is rendered in this animation cycle.
    ///
    /// Returns the allocated [`WidgetId`] of the input box container.
    pub fn input_box(
        &mut self,
        text: &str,
        placeholder: &str,
        text_width: f32,
        cursor_blink_visible: bool,
    ) -> WidgetId {
        let box_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(box_id) {
            node.role = WidgetRole::TextInput;
            node.cursor = Some(WidgetCursor::Text);
            node.set_style(
                Style::new()
                    .height(28.0)
                    .border_radius(4.0)
                    .border(1.0, Color::rgba(0.0, 0.85, 0.95, 0.80))
                    .background(Color::rgba(0.05, 0.05, 0.07, 1.0)),
            );
        }
        let _ = self.tree.add_child(self.parent, box_id);

        let text_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(text_id) {
            node.role = WidgetRole::Default;
            let display = if text.is_empty() { placeholder } else { text };
            node.set_text(display);
            node.font_size = 12.0;
            node.line_height = 28.0;
            node.text_align = TextAlign::Left;
            node.text_color = if text.is_empty() {
                Color::rgba(0.45, 0.45, 0.52, 1.0)
            } else {
                Color::WHITE
            };
        }
        let _ = self.tree.add_child(box_id, text_id);

        if cursor_blink_visible {
            let caret_offset = if text.is_empty() {
                0.0_f32
            } else {
                text_width + 1.0
            };
            let caret_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(caret_id) {
                node.role = WidgetRole::Default;
                node.tag = caret_offset.to_bits() as u64;
                node.set_style(
                    Style::new()
                        .background(Color::rgba(0.0, 0.90, 1.0, 0.95))
                        .border_radius(0.75),
                );
            }
            let _ = self.tree.add_child(box_id, caret_id);
        }

        box_id
    }
}