// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Typography, labels, and section header widget builders.

use iris_core::{AlignItems, Color, Style, TextAlign, UiTree, WidgetId};

/// Helper builder for text labels.
pub struct LabelBuilder<'a> {
    tree: &'a mut UiTree,
    node_id: WidgetId,
}

impl<'a> LabelBuilder<'a> {
    /// Creates a new text label widget with specified string content.
    pub fn new(tree: &'a mut UiTree, text: impl Into<String>) -> Self {
        let node_id = tree.create_node();
        if let Some(node) = tree.get_mut(node_id) {
            node.set_text(text);
        }
        Self { tree, node_id }
    }

    /// Returns the allocated `WidgetId` of this label.
    #[inline]
    pub fn id(&self) -> WidgetId {
        self.node_id
    }

    /// Consumes the builder and returns the configured `WidgetId`.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.node_id
    }

    /// Sets the label font size and line height in pixels.
    pub fn font_size(self, size: f32, line_height: f32) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.font_size = size;
            node.line_height = line_height;
        }
        self
    }

    /// Sets the label foreground text color.
    pub fn color(self, color: Color) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.text_color = color;
        }
        self
    }

    /// Sets the label text alignment.
    pub fn align(self, align: TextAlign) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.text_align = align;
        }
        self
    }

    /// Applies a style modification closure to the label.
    pub fn style<F>(self, modifier: F) -> Self
    where
        F: FnOnce(Style) -> Style,
    {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            let new_style = modifier(node.style);
            node.set_style(new_style);
        }
        self
    }
}

/// Helper builder for collapsible or colored section category headers.
pub struct SectionHeaderBuilder {
    node_id: WidgetId,
}

impl SectionHeaderBuilder {
    /// Creates a section header banner with category title and accent indicator.
    pub fn new(tree: &mut UiTree, title: impl Into<String>, accent_color: Color) -> Self {
        let node_id = tree.create_node();
        if let Some(node) = tree.get_mut(node_id) {
            node.set_text(format!("▼  {}", title.into()));
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_color = accent_color;
            node.set_style(
                Style::new()
                    .padding(3.0)
                    .margin(1.0)
                    .background(Color::hex("#14141e"))
                    .border(1.0, Color::hex("#1e1e2c"))
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