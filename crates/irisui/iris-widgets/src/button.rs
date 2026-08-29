// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Push button and docking tab widget builders.

use iris_core::{AlignItems, Color, JustifyContent, Style, TextAlign, UiTree, WidgetId};

/// Helper builder for push buttons with text and default theme styling.
pub struct ButtonBuilder<'a> {
    tree: &'a mut UiTree,
    node_id: WidgetId,
}

impl<'a> ButtonBuilder<'a> {
    /// Creates a new button with text label.
    pub fn new(tree: &'a mut UiTree, label: impl Into<String>) -> Self {
        let node_id = tree.create_node();
        if let Some(node) = tree.get_mut(node_id) {
            node.set_text(label);
            node.text_align = TextAlign::Center;
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_color = Color::hex("#e2e8f0");
            node.set_style(
                Style::new()
                    .padding(4.0)
                    .background(Color::hex("#1e1e2c"))
                    .border(1.0, Color::hex("#2c2c3e"))
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center),
            );
        }
        Self { tree, node_id }
    }

    /// Returns the allocated `WidgetId` of this button.
    #[inline]
    pub fn id(&self) -> WidgetId {
        self.node_id
    }

    /// Consumes the builder and returns the configured `WidgetId`.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.node_id
    }

    /// Applies a style modification closure to the button.
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

/// Helper builder for tab bar headers with active selection highlights.
pub struct TabBuilder {
    node_id: WidgetId,
}

impl TabBuilder {
    /// Creates a new tab item with title and active state styling.
    pub fn new(tree: &mut UiTree, title: impl Into<String>, active: bool) -> Self {
        let node_id = tree.create_node();
        if let Some(node) = tree.get_mut(node_id) {
            node.set_text(title);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_color = if active {
                Color::WHITE
            } else {
                Color::hex("#94a3b8")
            };

            let bg = if active {
                Color::hex("#1e1e2c")
            } else {
                Color::hex("#121218")
            };
            let border_color = if active {
                Color::hex("#38bdf8")
            } else {
                Color::hex("#1c1c24")
            };

            node.set_style(
                Style::new()
                    .padding(5.0)
                    .background(bg)
                    .border(1.0, border_color)
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center),
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