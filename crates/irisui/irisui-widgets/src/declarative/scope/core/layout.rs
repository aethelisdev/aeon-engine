// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scope Layout Resolution & Subtree Geometry
//!
//! Provides layout execution, parent bounds evaluation, text measurement,
//! container configuration, and scroll boundary configuration for [`UiScope`].

use super::context::UiScope;
use crate::declarative::layout::layout_subtree;
use crate::declarative::widget::FloatingDragPreview;
use irisui_core::{Point, Rect, Size, Style, UiTree, WidgetId, WidgetResponse};

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
    /// Retrieves the resolved bounding rectangle ([`irisui_core::WidgetNode::computed_rect`]) of the parent
    /// container within the unified [`UiTree`] coordinate space.
    #[must_use]
    pub fn parent_rect(&self) -> Rect {
        self.tree
            .get(self.parent)
            .map(|n| n.computed_rect)
            .unwrap_or(Rect::ZERO)
    }

    /// Evaluates whether the parent container is collapsed or too small to render meaningful content.
    ///
    /// Evaluates if the parent's resolved layout bounds have width or height less than or equal to `min_dimension`.
    ///
    /// # Arguments
    /// * `min_dimension` - Minimum physical threshold in pixels for both width and height.
    #[inline]
    #[must_use]
    pub fn is_parent_collapsed(&self, min_dimension: f32) -> bool {
        let rect = self.parent_rect();
        rect.width <= min_dimension || rect.height <= min_dimension
    }

    /// Evaluates whether the parent container is collapsed below the canonical 20.0px viewport threshold.
    ///
    /// Standard early-return check for dock panels and floating overlays before assembling children.
    #[inline]
    #[must_use]
    pub fn is_collapsed(&self) -> bool {
        self.is_parent_collapsed(20.0)
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
        irisui_text::measure_text_advance(text, font_size)
    }

    /// Configures the default font family name for thread-local dynamic text layout measurements.
    ///
    /// # Arguments
    /// * `family` - Target font family name (e.g. `"Inter"`, `"Roboto"`, `"sans-serif"`).
    #[inline]
    pub fn set_default_font_family(&self, family: &str) {
        irisui_text::set_default_text_family(family);
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
        self.tree.refresh_hover_from_cursor();
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
        tree.refresh_hover_from_cursor();
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
    /// Configures virtual scrolling boundaries without leaking direct [`irisui_core::WidgetNode`] mutable access.
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

    /// Emits a non-interactive floating drag-and-drop feedback card onto the elevated [`irisui_core::UiLayer::Tooltip`] layer.
    ///
    /// # Arguments
    /// * `preview` - Configured [`FloatingDragPreview`] descriptor.
    pub fn floating_drag_preview(&mut self, preview: FloatingDragPreview) -> WidgetResponse {
        self.add(preview)
    }

    /// Emits a declarative viewport drop target overlay combining an automatic viewport boundary
    /// hit-test, projected landing ring indicator, and floating drag preview capsule.
    ///
    /// Resolves the live pointer coordinate from the retained UI tree (falling back to
    /// `desc.cursor_fallback`), evaluates `resolve_landing_point(cursor)` only when the cursor
    /// lies inside a valid non-collapsed `desc.viewport_rect`, and emits the floating preview card.
    pub fn viewport_drop_overlay<F>(
        &mut self,
        desc: crate::declarative::widget::ViewportDropOverlay<'_>,
        resolve_landing_point: F,
    ) -> WidgetResponse
    where
        F: FnOnce(Point) -> Option<Point>,
    {
        let cursor = self.resolved_cursor_pos(desc.cursor_fallback);
        if desc.viewport_rect.width > 20.0
            && desc.viewport_rect.height > 20.0
            && desc.viewport_rect.contains_point(cursor)
            && let Some(landing_pt) = resolve_landing_point(cursor)
        {
            self.landing_indicator(
                landing_pt,
                desc.landing_outer_radius,
                desc.landing_inner_radius,
                desc.landing_ring_color,
            );
        }
        self.floating_drag_preview(
            FloatingDragPreview::new(desc.title)
                .with_cursor(cursor)
                .with_badge(desc.badge_text, desc.badge_color),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::tree::UiTree;

    #[test]
    fn test_ui_scope_is_collapsed() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");

        // 1. Initial zero rect is collapsed
        let scope = UiScope::new(&mut tree, root);
        assert!(scope.is_collapsed());
        assert!(scope.is_parent_collapsed(20.0));

        // 2. Set dimensions > 20px -> not collapsed
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        }
        let scope = UiScope::new(&mut tree, root);
        assert!(!scope.is_collapsed());

        // 3. One dimension <= 20px -> collapsed
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 10.0, 100.0);
        }
        let scope = UiScope::new(&mut tree, root);
        assert!(scope.is_collapsed());
    }
}