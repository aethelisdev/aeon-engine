// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Inspector property rows, XYZ vector editors, color pickers, and dropdown selectors.

use crate::input::DragValueBuilder;
use iris_core::{AlignItems, Color, Style, TextAlign, UiTree, WidgetId};

/// Helper builder for inspector property rows (e.g. `Position [X] [Y] [Z]`).
pub struct PropertyRowBuilder {
    container_id: WidgetId,
    x_id: Option<WidgetId>,
    y_id: Option<WidgetId>,
    z_id: Option<WidgetId>,
}

impl PropertyRowBuilder {
    /// Creates a complete 3-axis property row container with label and X/Y/Z drag fields.
    pub fn new_xyz(tree: &mut UiTree, label: impl Into<String>, x: f32, y: f32, z: f32) -> Self {
        let container_id = tree.create_node();
        if let Some(node) = tree.get_mut(container_id) {
            node.set_style(
                Style::new()
                    .flex_row()
                    .gap(4.0)
                    .align_items(AlignItems::Center),
            );
        }

        let label_id = tree.create_node();
        if let Some(node) = tree.get_mut(label_id) {
            node.set_text(label.into());
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_color = Color::hex("#94a3b8");
            node.set_style(Style::new().width(56.0).align_items(AlignItems::Center));
        }
        let _ = tree.add_child(container_id, label_id);

        let x_id = DragValueBuilder::new(tree, "X", x, Color::hex("#ef4444"), false).build();
        let _ = tree.add_child(container_id, x_id);

        let y_id = DragValueBuilder::new(tree, "Y", y, Color::hex("#22c55e"), false).build();
        let _ = tree.add_child(container_id, y_id);

        let z_id = DragValueBuilder::new(tree, "Z", z, Color::hex("#3b82f6"), false).build();
        let _ = tree.add_child(container_id, z_id);

        Self {
            container_id,
            x_id: Some(x_id),
            y_id: Some(y_id),
            z_id: Some(z_id),
        }
    }

    /// Returns the X axis widget ID.
    #[inline]
    pub fn x_id(&self) -> Option<WidgetId> {
        self.x_id
    }

    /// Returns the Y axis widget ID.
    #[inline]
    pub fn y_id(&self) -> Option<WidgetId> {
        self.y_id
    }

    /// Returns the Z axis widget ID.
    #[inline]
    pub fn z_id(&self) -> Option<WidgetId> {
        self.z_id
    }

    /// Returns a tuple of `(X, Y, Z)` widget IDs for input event targeting.
    #[inline]
    pub fn inputs(&self) -> (WidgetId, WidgetId, WidgetId) {
        (
            self.x_id.unwrap_or_default(),
            self.y_id.unwrap_or_default(),
            self.z_id.unwrap_or_default(),
        )
    }

    /// Consumes the builder and returns the configured container `WidgetId`.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.container_id
    }
}

/// Helper builder for color picker property swatches.
pub struct ColorPickerBuilder {
    node_id: WidgetId,
}

impl ColorPickerBuilder {
    /// Creates a color picker swatch showing the hex value and color preview.
    pub fn new(tree: &mut UiTree, label: impl Into<String>, color: Color) -> Self {
        let node_id = tree.create_node();
        let hex_str = format!(
            "#{:02X}{:02X}{:02X}",
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8
        );

        if let Some(node) = tree.get_mut(node_id) {
            node.set_text(format!("{}  [{}]", label.into(), hex_str));
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_color = color;
            node.text_align = TextAlign::Center;

            node.set_style(
                Style::new()
                    .padding(3.0)
                    .margin(1.0)
                    .background(Color::hex("#121218"))
                    .border(1.0, color)
                    .border_radius(3.0)
                    .align_items(AlignItems::Center),
            );
        }
        Self { node_id }
    }

    /// Consumes the builder and returns the configured `WidgetId`.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.node_id
    }
}

/// Helper builder for selectable dropdown option selectors (ComboBox).
pub struct DropdownBuilder {
    node_id: WidgetId,
}

impl DropdownBuilder {
    /// Creates a dropdown selector widget with current option and expand chevron.
    pub fn new(
        tree: &mut UiTree,
        label: impl Into<String>,
        selected_option: impl Into<String>,
        is_open: bool,
    ) -> Self {
        let node_id = tree.create_node();
        let chevron = if is_open { "▲" } else { "▼" };
        let display = format!("{}: {}  {}", label.into(), selected_option.into(), chevron);

        if let Some(node) = tree.get_mut(node_id) {
            node.set_text(display);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_color = Color::WHITE;
            node.text_align = TextAlign::Center;

            let border_color = if is_open {
                Color::hex("#38bdf8")
            } else {
                Color::hex("#2a2a3e")
            };

            node.set_style(
                Style::new()
                    .padding(4.0)
                    .margin(1.0)
                    .background(Color::hex("#161622"))
                    .border(1.0, border_color)
                    .border_radius(3.0)
                    .align_items(AlignItems::Center),
            );
        }
        Self { node_id }
    }

    /// Consumes the builder and returns the configured `WidgetId`.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.node_id
    }
}