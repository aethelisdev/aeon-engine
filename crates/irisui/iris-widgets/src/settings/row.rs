// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Settings Property Row Layout Builder
//!
//! Standardizes property setting row layout geometry with a left-aligned descriptive label
//! and a right-aligned control slot for interactive widgets (sliders, toggles, dropdowns).

use super::types::{SettingRowFrame, SettingRowStyle};
use iris_core::geometry::Rect;
use iris_core::id::WidgetId;
use iris_core::node::UiLayer;
use iris_core::tree::UiTree;

/// Fluent builder for creating structured setting property rows.
pub struct SettingRowBuilder {
    rect: Rect,
    label: String,
    style: SettingRowStyle,
}

impl SettingRowBuilder {
    /// Creates a new row builder with the given bounding rectangle and property label.
    #[inline]
    pub fn new(rect: Rect, label: impl Into<String>) -> Self {
        Self {
            rect,
            label: label.into(),
            style: SettingRowStyle::default(),
        }
    }

    /// Overrides the left label column width in logical pixels.
    #[inline]
    pub fn label_width(mut self, width: f32) -> Self {
        self.style.label_width = width;
        self
    }

    /// Overrides the visual styling parameters of the setting row.
    #[inline]
    pub fn style(mut self, style: SettingRowStyle) -> Self {
        self.style = style;
        self
    }

    /// Compiles the row into the provided [`UiTree`] and attaches it to `parent_id`.
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> SettingRowFrame {
        let row_id = tree.create_node();
        if let Some(node) = tree.get_mut(row_id) {
            node.set_name("SettingRow");
            node.layer = UiLayer::Content;
            node.computed_rect = self.rect;
        }
        let _ = tree.add_child(parent_id, row_id);

        let label_w = self.style.label_width.min(self.rect.width * 0.6);
        let label_rect = Rect::new(self.rect.x, self.rect.y, label_w, self.rect.height);

        let label_id = tree.create_node();
        if let Some(node) = tree.get_mut(label_id) {
            node.set_name("SettingRowLabel");
            node.set_text(&self.label);
            node.font_size = self.style.label_font_size;
            node.line_height = self.rect.height;
            node.text_color = self.style.label_color;
            node.computed_rect = label_rect;
        }
        let _ = tree.add_child(row_id, label_id);

        let control_x = self.rect.x + label_w;
        let control_w = (self.rect.width - label_w).max(0.0);
        let control_rect = Rect::new(control_x, self.rect.y, control_w, self.rect.height);

        SettingRowFrame {
            row_id,
            label_rect,
            control_rect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_row_builder_layout() {
        let mut tree = UiTree::new();
        let root_id = tree.create_node();

        let frame = SettingRowBuilder::new(Rect::new(20.0, 30.0, 400.0, 24.0), "Field Name")
            .label_width(150.0)
            .build(&mut tree, root_id);

        assert_eq!(frame.label_rect.width, 150.0);
        assert_eq!(frame.control_rect.x, 20.0 + 150.0);
        assert_eq!(frame.control_rect.width, 250.0);
    }
}