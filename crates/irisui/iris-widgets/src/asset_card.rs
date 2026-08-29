// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Scene hierarchy tree items and asset browser thumbnail card widgets.

use iris_core::{AlignItems, Color, JustifyContent, Style, TextAlign, UiTree, WidgetId};

/// Helper builder for scene hierarchy tree items.
pub struct TreeItemBuilder {
    node_id: WidgetId,
}

impl TreeItemBuilder {
    /// Creates a hierarchy tree item with name, icon, and selection highlight.
    pub fn new(tree: &mut UiTree, name: impl Into<String>, is_selected: bool) -> Self {
        let node_id = tree.create_node();
        if let Some(node) = tree.get_mut(node_id) {
            node.set_text(format!("  🔷  {}", name.into()));
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_color = if is_selected {
                Color::WHITE
            } else {
                Color::hex("#cbd5e1")
            };

            let bg = if is_selected {
                Color::hex("#1e3a8a")
            } else {
                Color::TRANSPARENT
            };
            let border_color = if is_selected {
                Color::hex("#3b82f6")
            } else {
                Color::TRANSPARENT
            };

            node.set_style(
                Style::new()
                    .padding(3.0)
                    .margin(1.0)
                    .background(bg)
                    .border(1.0, border_color)
                    .border_radius(2.0)
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

/// Helper builder for asset browser thumbnail cards.
pub struct AssetCardBuilder {
    node_id: WidgetId,
}

impl AssetCardBuilder {
    /// Creates an asset thumbnail card with icon badge and filename.
    pub fn new(
        tree: &mut UiTree,
        name: impl Into<String>,
        tag: &'static str,
        tag_color: Color,
    ) -> Self {
        let node_id = tree.create_node();
        if let Some(node) = tree.get_mut(node_id) {
            node.set_text(format!("[{}]\n{}", tag, name.into()));
            node.font_size = 10.0;
            node.line_height = 14.0;
            node.text_color = tag_color;
            node.text_align = TextAlign::Center;
            node.set_style(
                Style::new()
                    .width(76.0)
                    .height(64.0)
                    .padding(4.0)
                    .background(Color::hex("#12121c"))
                    .border(1.0, Color::hex("#202030"))
                    .border_radius(4.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.3)),
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