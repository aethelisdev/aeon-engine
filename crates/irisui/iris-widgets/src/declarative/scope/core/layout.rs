// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scope Layout Resolution & Subtree Geometry
//!
//! Provides layout execution, parent bounds evaluation, text measurement,
//! container configuration, and scroll boundary configuration for [`UiScope`].

use super::context::UiScope;
use crate::declarative::layout::layout_subtree;
use crate::declarative::scope::hover::update_hover_styles_recursive;
use crate::declarative::widget::FloatingDragPreview;
use iris_core::{Point, Rect, Size, Style, UiTree, WidgetId, WidgetResponse};

impl<'a> UiScope<'a> {
    /// Returns the accumulated screen-space origin `(x, y)` of this scope's parent hierarchy.
    ///
    /// Climbs the ancestor chain to sum explicit positioning and computed offsets.
    #[must_use]
    pub fn parent_screen_origin(&self) -> Point {
        let mut curr = Some(self.parent);
        let mut x = 0.0;
        let mut y = 0.0;
        while let Some(id) = curr {
            if let Some(node) = self.tree.get(id) {
                if node.computed_rect.width > 0.0 && node.computed_rect.height > 0.0 {
                    x += node.computed_rect.x;
                    y += node.computed_rect.y;
                    break;
                }
                if let Some(left) = node.style.inset_left {
                    x += left;
                }
                if let Some(top) = node.style.inset_top {
                    y += top;
                }
                curr = node.parent;
            } else {
                break;
            }
        }
        Point::new(x, y)
    }

    /// Returns the computed physical layout bounds of this scope's parent node if present.
    ///
    /// Retrieves the resolved bounding rectangle ([`iris_core::WidgetNode::computed_rect`]) of the parent
    /// container within the unified [`UiTree`] coordinate space.
    #[must_use]
    pub fn parent_rect(&self) -> Rect {
        self.tree
            .get(self.parent)
            .map(|n| n.computed_rect)
            .unwrap_or(Rect::ZERO)
    }

    /// Measures the horizontal layout advance width of a string in physical pixels at the given font size.
    ///
    /// Dynamically evaluates font glyph advances via the underlying typography engine.
    ///
    /// # Arguments
    /// * `text` - Text string to measure.
    /// * `font_size` - Font size in physical pixels.
    ///
    /// # Returns
    /// Total horizontal advance width in pixels.
    #[inline]
    pub fn measure_text_advance(&self, text: &str, font_size: f32) -> f32 {
        iris_text::measure_text_advance(text, font_size)
    }

    /// Configures the default font family name for thread-local dynamic text layout measurements.
    ///
    /// # Arguments
    /// * `family` - Target font family name (e.g. `"Inter"`, `"Roboto"`, `"sans-serif"`).
    #[inline]
    pub fn set_default_font_family(&self, family: &str) {
        iris_text::set_default_text_family(family);
    }

    /// Returns the computed bounding rectangle of an emitted widget after layout resolution.
    ///
    /// # Arguments
    /// * `id` - Arena node identifier of the target widget.
    #[inline]
    pub fn computed_rect(&self, id: WidgetId) -> Rect {
        self.tree
            .get(id)
            .map(|node| node.computed_rect)
            .unwrap_or(Rect::ZERO)
    }

    /// Calculates and applies recursive flow layout for this scope's subtree across `bounds`.
    /// Automatically reconciles visual hover styling on interactive descendants if a pointer
    /// position is registered in the UI tree, eliminating manual pointer coordinate plumbing.
    pub fn finish_layout(&mut self, bounds: Rect) {
        layout_subtree(self.tree, self.parent, bounds);
        if let Some(cursor) = self.tree.cursor_position() {
            update_hover_styles_recursive(self.tree, self.parent, cursor);
        }
    }

    /// Executes a declarative UI builder closure within a scoped hierarchy on `parent`,
    /// automatically executing recursive layout finalization across `bounds` upon closure exit.
    ///
    /// Eliminates boilerplate manual calls to `finish_layout(bounds)` across panels and viewports.
    ///
    /// # Arguments
    /// * `tree` - Mutable reference to the UI node hierarchy.
    /// * `parent` - Target parent widget under which nodes are constructed.
    /// * `bounds` - Bounding layout rectangle constraining the subtree.
    /// * `f` - Declarative builder closure.
    pub fn scoped<F, R>(tree: &mut UiTree, parent: WidgetId, bounds: Rect, f: F) -> R
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let mut scope = UiScope::new(tree, parent);
        let result = f(&mut scope);
        layout_subtree(tree, parent, bounds);
        if let Some(cursor) = tree.cursor_position() {
            update_hover_styles_recursive(tree, parent, cursor);
        }
        result
    }

    /// Sets the vertical scroll offset in pixels on the active parent container node.
    #[inline]
    pub fn set_scroll_offset_y(&mut self, offset_y: f32) {
        if let Some(node) = self.tree.get_mut(self.parent) {
            node.style.scroll_offset_y = offset_y;
        }
    }

    /// Sets the scrollable content extent size in physical pixels on the specified container node.
    ///
    /// Configures virtual scrolling boundaries without leaking direct [`iris_core::WidgetNode`] mutable access.
    ///
    /// # Arguments
    /// * `widget_id` - Identifier of the container node whose content size is configured.
    /// * `size` - Content extent size in physical pixels.
    #[inline]
    pub fn set_content_size(&mut self, widget_id: WidgetId, size: Size) {
        if let Some(node) = self.tree.get_mut(widget_id) {
            node.content_size = size;
        }
    }

    /// Configures the active parent container's debug name, layout style, and interactivity.
    ///
    /// Provides declarative setup for root nodes and container scopes without
    /// raw `WidgetNode` imperative manipulation in the consumer application layer.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier assigned to the container node.
    /// * `style` - Flexbox layout constraints and visual properties applied to the container.
    /// * `interactive` - Whether the container node responds to pointer interactions.
    #[inline]
    pub fn configure_container(
        &mut self,
        name: impl Into<String>,
        style: Style,
        interactive: bool,
    ) {
        if let Some(node) = self.tree.get_mut(self.parent) {
            node.set_name(name);
            node.interactive = interactive;
            node.set_style(style);
        }
    }

    /// Returns the available bounding layout rectangle of the active parent container.
    ///
    /// Evaluates `computed_rect`, `style` width/height constraints, or cached `viewport_sizes`.
    pub fn available_rect(&self) -> Rect {
        if let Some(node) = self.tree.get(self.parent) {
            let w = if node.computed_rect.width > 0.0 {
                node.computed_rect.width
            } else if let Some(sw) = node.style.width {
                sw
            } else if node.tag != 0 {
                self.tree
                    .viewport_size(node.tag)
                    .map(|s| s.width)
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            let h = if node.computed_rect.height > 0.0 {
                node.computed_rect.height
            } else if let Some(sh) = node.style.height {
                sh
            } else if node.tag != 0 {
                self.tree
                    .viewport_size(node.tag)
                    .map(|s| s.height)
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            if w > 0.0 && h > 0.0 {
                return Rect::new(0.0, 0.0, w, h);
            }
        }
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    /// Emits a non-interactive floating drag-and-drop feedback card onto the elevated [`iris_core::UiLayer::Tooltip`] layer.
    ///
    /// # Arguments
    /// * `preview` - Configured [`FloatingDragPreview`] descriptor.
    pub fn floating_drag_preview(&mut self, preview: FloatingDragPreview) -> WidgetResponse {
        self.add(preview)
    }
}