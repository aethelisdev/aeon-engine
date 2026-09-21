// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Canvas & External GPU Texture Widget (`iris-widgets::viewport_canvas`)
//!
//! Provides a first-class, retained-mode container widget for embedding hardware-rendered
//! 3D viewports, game camera outputs, and external GPU texture surfaces into Iris UI.
//!

use iris_core::color::Color;
use iris_core::geometry::Rect;
use iris_core::id::WidgetId;
use iris_core::node::{ExternalTextureId, UiLayer, WidgetCursor, WidgetRole};
use iris_core::style::Style;
use iris_core::tree::UiTree;

/// Retained builder for constructing hardware-rendered 3D viewport canvas surfaces in [`UiTree`].
///
/// Encapsulates external texture binding ([`ExternalTextureId`]), viewport background styling,
/// coordinate bounding boxes, and elevation layer configuration without exposing raw node creation
/// to host application layers.
pub struct ViewportCanvasBuilder<'a> {
    tree: &'a mut UiTree,
    rect: Rect,
    external_texture: Option<ExternalTextureId>,
    background_color: Color,
    border_color: Option<Color>,
    border_width: f32,
    corner_radius: f32,
    layer: UiLayer,
    cursor: WidgetCursor,
    name: &'static str,
    interactive: bool,
}

impl<'a> ViewportCanvasBuilder<'a> {
    /// Creates a new `ViewportCanvasBuilder` for the designated layout bounding box.
    ///
    /// By default, `interactive` is configured to `false` so underlying 3D camera controls
    /// and scene raycasting are unobstructed.
    #[must_use]
    pub fn new(tree: &'a mut UiTree, rect: Rect) -> Self {
        Self {
            tree,
            rect,
            external_texture: None,
            background_color: Color::rgba(0.08, 0.08, 0.10, 1.0),
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            layer: UiLayer::Background,
            cursor: WidgetCursor::Default,
            name: "ViewportCanvas",
            interactive: false,
        }
    }

    /// Attaches an external GPU texture handle for rendering 3D graphics pass output.
    #[must_use]
    pub fn external_texture(mut self, texture_id: ExternalTextureId) -> Self {
        self.external_texture = Some(texture_id);
        self
    }

    /// Sets the clear or fallback background color beneath the 3D surface.
    #[must_use]
    pub fn background(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    /// Sets an optional outline border stroke for the canvas boundaries.
    #[must_use]
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }

    /// Sets the rounded corner radius for the viewport canvas quad.
    #[must_use]
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Sets the explicit stacking elevation layer (defaults to [`UiLayer::Background`]).
    #[must_use]
    pub fn layer(mut self, layer: UiLayer) -> Self {
        self.layer = layer;
        self
    }

    /// Sets the default mouse cursor shape displayed over the viewport.
    #[must_use]
    pub fn cursor(mut self, cursor: WidgetCursor) -> Self {
        self.cursor = cursor;
        self
    }

    /// Sets whether the viewport canvas node intercepts mouse and cursor events.
    ///
    /// When `false` (default), the viewport canvas acts as a visual backdrop without blocking
    /// 3D scene camera navigation, WASD flight, or entity raycast selection.
    #[must_use]
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Sets a debug name label for the canvas node in the UI hierarchy.
    #[must_use]
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Assembles the viewport canvas node in the [`UiTree`] and returns its [`WidgetId`].
    ///
    /// If `parent_id` is supplied, the newly created canvas node is appended to the parent's children.
    pub fn build(self, parent_id: Option<WidgetId>) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.name = Some(self.name.to_string());
            node.computed_rect = self.rect;
            node.layer = self.layer;
            node.cursor = Some(self.cursor);
            node.role = WidgetRole::Default;
            node.external_texture = self.external_texture;
            node.interactive = self.interactive;

            let mut style = Style::new()
                .background(self.background_color)
                .width(self.rect.width)
                .height(self.rect.height)
                .flex_grow(1.0)
                .flex_shrink(1.0);
            if let Some(bc) = self.border_color {
                style = style.border(self.border_width, bc);
            }
            if self.corner_radius > 0.0 {
                style = style.border_radius(self.corner_radius);
            }
            node.style = style;
        }

        if let Some(parent) = parent_id {
            let _ = self.tree.add_child(parent, node_id);
        }

        node_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_canvas_builder_hierarchy_and_properties() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let rect = Rect::new(0.0, 0.0, 1280.0, 720.0);
        let tex_id = ExternalTextureId(42);

        let canvas_id = ViewportCanvasBuilder::new(&mut tree, rect)
            .external_texture(tex_id)
            .background(Color::rgba(0.05, 0.05, 0.08, 1.0))
            .border(Color::WHITE, 1.0)
            .corner_radius(4.0)
            .layer(UiLayer::Background)
            .cursor(WidgetCursor::Grab)
            .name("Main3DViewport")
            .build(Some(root));

        let node = tree
            .get(canvas_id)
            .expect("Viewport canvas node must exist");
        assert_eq!(node.name.as_deref(), Some("Main3DViewport"));
        assert_eq!(node.computed_rect, rect);
        assert_eq!(node.layer, UiLayer::Background);
        assert_eq!(node.cursor, Some(WidgetCursor::Grab));
        assert_eq!(node.external_texture, Some(tex_id));
        assert!(!node.interactive);
        assert_eq!(
            node.style.background_color,
            Color::rgba(0.05, 0.05, 0.08, 1.0)
        );

        let root_node = tree.get(root).expect("Root node must exist");
        assert!(root_node.children.contains(&canvas_id));

        // Viewport canvas does not steal hit-test events from 3D camera controls
        let hit = tree.hit_test_target(iris_core::geometry::Point::new(640.0, 360.0));
        assert!(hit.is_none());
    }
}