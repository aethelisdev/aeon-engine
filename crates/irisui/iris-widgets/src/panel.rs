// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Panel and container widget builders for structural layout cards.

use iris_core::{
    Color, Rect, Style, TextAlign, UiLayer, UiTree, WidgetCursor, WidgetId, WidgetRole,
};

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

    /// Assigns a human-readable identifier and debug name to the panel node.
    pub fn name(self, name: impl Into<String>) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.set_name(name);
        }
        self
    }

    /// Sets the explicit computed layout rectangle for the panel node.
    pub fn rect(self, rect: Rect) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.computed_rect = rect;
        }
        self
    }

    /// Sets the user-defined numeric tag or action identifier on the panel node.
    pub fn tag(self, tag: u64) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.set_tag(tag);
        }
        self
    }

    /// Sets the hardware cursor shape override for this panel node.
    pub fn cursor(self, cursor: WidgetCursor) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.set_cursor(cursor);
        }
        self
    }

    /// Assigns the semantic role of this panel widget.
    pub fn role(self, role: WidgetRole) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            let _ = node.set_role(role);
        }
        self
    }

    /// Assigns the explicit rendering and stacking layer of this panel widget.
    pub fn layer(self, layer: UiLayer) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            let _ = node.set_layer(layer);
        }
        self
    }

    /// Sets the texture UV sub-rectangle for rendering an icon or image.
    pub fn texture_uv(self, uv: [f32; 4]) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.set_texture_uv(uv);
        }
        self
    }

    /// Sets the color tint applied to the texture atlas icon or image.
    pub fn texture_tint(self, tint: Color) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.set_texture_tint(tint);
        }
        self
    }

    /// Sets whether the text within this node wraps automatically at word boundaries.
    pub fn text_wrap(self, wrap: iris_core::TextWrap) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.text_wrap = wrap;
        }
        self
    }

    /// Sets whether this widget node participates in pointer and keyboard interactions.
    pub fn interactive(self, interactive: bool) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.interactive = interactive;
        }
        self
    }

    /// Sets the text content and typography properties for this panel node.
    pub fn text(
        self,
        text: impl Into<String>,
        font_size: f32,
        color: Color,
        align: TextAlign,
    ) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.set_text(text);
            node.set_text_properties(font_size, font_size * 1.25, color, align);
        }
        self
    }

    /// Sets explicit line height for text rendering.
    pub fn line_height(self, height: f32) -> Self {
        if let Some(node) = self.tree.get_mut(self.node_id) {
            node.line_height = height;
        }
        self
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