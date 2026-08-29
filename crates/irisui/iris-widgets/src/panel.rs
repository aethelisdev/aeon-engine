// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Panel and container widget builders for structural layout cards.

use iris_core::{Color, Style, UiTree, WidgetId};

/// Helper builder for creating and configuring structured UI panels and cards.
pub struct PanelBuilder<'a> {
    tree: &'a mut UiTree,
    node_id: WidgetId,
}

impl<'a> PanelBuilder<'a> {
    /// Creates a new panel widget attached to a parent or as a standalone branch.
    pub fn new(tree: &'a mut UiTree) -> Self {
        let node_id = tree.create_node();
        Self { tree, node_id }
    }

    /// Returns the allocated `WidgetId` of this panel.
    #[inline]
    pub fn id(&self) -> WidgetId {
        self.node_id
    }

    /// Consumes the builder and returns the configured `WidgetId`.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.node_id
    }

    /// Applies a style modification closure to the panel.
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

    /// Configures the panel with dark theme preset defaults.
    pub fn dark_theme(self) -> Self {
        self.style(|s| {
            s.background(Color::hex("#101016"))
                .border(1.0, Color::hex("#1c1c28"))
                .border_radius(4.0)
                .padding(6.0)
                .box_shadow(0.0, 4.0, 16.0, Color::rgba(0.0, 0.0, 0.0, 0.4))
        })
    }
}