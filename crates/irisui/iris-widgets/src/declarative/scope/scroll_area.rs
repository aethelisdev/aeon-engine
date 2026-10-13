// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative ScrollArea and Virtualized List Scope Primitives
//!
//! Exposes fluent declarative [`UiScope`] methods for embedding hardware-clipped
//! scrollable containers and $O(1)$ virtualized lists into the retainable UI tree.
//!

use super::core::UiScope;
use crate::scroll_area::area::{ScrollAreaBuilder, ScrollAreaFrame};
use crate::scroll_area::style::ScrollAreaStyle;
use crate::scroll_area::types::{ScrollBarVisibility, VirtualScrollConfig, VirtualSlice};
use crate::scroll_area::virtual_list::VirtualList;
use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;
use iris_core::node::WidgetRole;
use iris_core::style::Style;

/// Configuration options customizing a declarative scroll area container.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollAreaConfig {
    /// Debug identifier assigned to the container node in the UI tree.
    pub name: &'static str,
    /// Current vertical scroll position in logical pixels.
    pub scroll_y: f32,
    /// Total virtual or measured height of all contained content in logical pixels.
    pub content_height: f32,
    /// Visual style parameters controlling scrollbar geometry, colors, and margins.
    pub style: ScrollAreaStyle,
    /// Visibility policy determining when the scrollbar indicator is rendered.
    pub visibility: ScrollBarVisibility,
    /// Optional background fill color applied to the viewport container node.
    pub background: Option<Color>,
    /// Optional cursor position in logical pixels for thumb hover evaluation.
    pub cursor_pos: Option<Point>,
    /// Whether the scrollbar thumb is actively undergoing dragging.
    pub is_dragging: bool,
}

impl Default for ScrollAreaConfig {
    #[inline]
    fn default() -> Self {
        Self {
            name: "ScrollArea",
            scroll_y: 0.0,
            content_height: 0.0,
            style: ScrollAreaStyle::default(),
            visibility: ScrollBarVisibility::Auto,
            background: None,
            cursor_pos: None,
            is_dragging: false,
        }
    }
}

impl ScrollAreaConfig {
    /// Creates a default scroll area configuration with the given content height and scroll offset.
    #[inline]
    pub fn new(content_height: f32, scroll_y: f32) -> Self {
        Self {
            content_height: content_height.max(0.0),
            scroll_y: scroll_y.max(0.0),
            ..Default::default()
        }
    }

    /// Sets the debug and telemetry name for the container node.
    #[inline]
    pub const fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Sets an explicit background color for the viewport container.
    #[inline]
    pub const fn with_background(mut self, bg: Color) -> Self {
        self.background = Some(bg);
        self
    }

    /// Overrides the visual style of the scrollbar indicator.
    #[inline]
    pub const fn with_style(mut self, style: ScrollAreaStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the scrollbar visibility policy.
    #[inline]
    pub const fn with_visibility(mut self, visibility: ScrollBarVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Sets the active dragging flag for the scrollbar thumb.
    #[inline]
    pub const fn with_dragging(mut self, is_dragging: bool) -> Self {
        self.is_dragging = is_dragging;
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a scrollable container with flex-grow expansion and hardware child clipping.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the scroll viewport node.
    /// * `bg` - Background fill color.
    /// * `f` - Closure executing within the scroll viewport scope.
    pub fn scroll_area<F>(&mut self, name: &'static str, bg: Color, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let style = Style::new()
            .flex_col()
            .flex_grow(1.0)
            .clip_children(true)
            .background(bg);
        self.container_named(name, style, f)
    }

    /// Emits a tagged scrollable container with flex-grow expansion and hardware child clipping.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the scroll viewport node.
    /// * `tag` - Semantic tag for tracking interactions and event routing.
    /// * `bg` - Background fill color.
    /// * `f` - Closure executing within the scroll viewport scope.
    pub fn scroll_area_tagged<F>(
        &mut self,
        name: &'static str,
        tag: u64,
        bg: Color,
        f: F,
    ) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let style = Style::new()
            .flex_col()
            .flex_grow(1.0)
            .clip_children(true)
            .background(bg);
        self.container_tagged(name, style, WidgetRole::ScrollAreaViewport, tag, f)
    }

    /// Emits a high-performance virtualized scrollable container that renders only the visible slice of items.
    ///
    /// Automatically manages [`VirtualList`](crate::scroll_area::VirtualList) windowing, overscan buffer padding,
    /// sub-pixel scroll offset application, and child iteration via declarative closures.
    ///
    /// Returns the computed maximum vertical scroll limit in physical pixels.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the scroll viewport node.
    /// * `tag` - Semantic tag for tracking interactions and event routing.
    /// * `viewport_height` - Available visible height in physical pixels.
    /// * `scroll_y` - Current vertical scroll offset in physical pixels.
    /// * `config` - Sizing, overscan, and container styling configuration descriptor.
    /// * `item_renderer` - Closure called for each visible index: `(scope, index)`.
    pub fn virtual_scroll_area<F>(
        &mut self,
        name: &'static str,
        tag: u64,
        viewport_height: f32,
        scroll_y: f32,
        config: VirtualScrollConfig,
        mut item_renderer: F,
    ) -> f32
    where
        F: FnMut(&mut UiScope<'_>, usize),
    {
        let stride = config.stride();
        let vlist = VirtualList::new(config.total_items, stride).with_overscan(config.overscan);
        let max_scroll = vlist.max_scroll_y(viewport_height);
        let safe_scroll = scroll_y.clamp(0.0, max_scroll);
        let slice = vlist.compute_slice(viewport_height, safe_scroll);
        let scroll_offset = vlist.compute_scroll_offset(safe_scroll, slice.start_idx);

        self.scroll_area_tagged(name, tag, config.bg, |vp| {
            vp.set_scroll_offset_y(scroll_offset);
            for idx in slice.start_idx..slice.end_idx {
                item_renderer(vp, idx);
            }
        });

        max_scroll
    }

    /// Emits a declarative hardware-clipped scroll container into the UI tree with explicit bounding geometry.
    ///
    /// Constructs a viewport node with `.clip_children(true)`, calculates proportional
    /// scrollbar thumb geometry if the content overflows, and scopes child additions
    /// to the viewport container node.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier for the container node.
    /// * `viewport_rect` - Bounding rectangle of the visible viewport in logical pixels.
    /// * `scroll_y` - Current vertical scroll offset in logical pixels.
    /// * `content_height` - Total measured or virtual height of all contained items.
    /// * `f` - Closure executing within the scrollable child scope.
    pub fn scroll_area_bounded<F>(
        &mut self,
        name: &'static str,
        viewport_rect: Rect,
        scroll_y: f32,
        content_height: f32,
        f: F,
    ) -> ScrollAreaFrame
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let config = ScrollAreaConfig::new(content_height, scroll_y).with_name(name);
        self.scroll_area_with_config(viewport_rect, &config, f)
    }

    /// Emits a declarative scroll container using a pre-configured [`ScrollAreaConfig`].
    ///
    /// # Arguments
    /// * `viewport_rect` - Bounding rectangle of the visible viewport in logical pixels.
    /// * `config` - Comprehensive configuration specifying styling, visibility, and heights.
    /// * `f` - Closure executing within the scrollable child scope.
    pub fn scroll_area_with_config<F>(
        &mut self,
        viewport_rect: Rect,
        config: &ScrollAreaConfig,
        f: F,
    ) -> ScrollAreaFrame
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let mut builder = ScrollAreaBuilder::new(viewport_rect, config.content_height)
            .name(config.name)
            .scroll_y(config.scroll_y)
            .style(config.style)
            .visibility(config.visibility)
            .is_dragging(config.is_dragging)
            .cursor_pos(config.cursor_pos);

        if let Some(bg) = config.background {
            builder = builder.background(bg);
        }

        let frame = builder.build(self.tree, self.parent);

        let mut child_scope = self.child_scope(frame.container_id);
        f(&mut child_scope);

        frame
    }

    /// Emits a zero-allocation, hardware-clipped virtualized list container with explicit viewport bounding box.
    ///
    /// Uses [`VirtualList`] to compute an $O(1)$ visible slice `[start_idx..end_idx]`
    /// and invokes `render_item` solely for items intersecting the visible viewport window,
    /// pruning thousands of off-screen nodes from the tree.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier for the container.
    /// * `viewport_rect` - Bounding rectangle of the visible viewport.
    /// * `scroll_y` - Current vertical scroll offset in logical pixels.
    /// * `total_items` - Total count of elements in the underlying collection.
    /// * `item_stride` - Uniform vertical height + margin gap per item in logical pixels.
    /// * `render_item` - Closure invoked per visible element with `(scope, index, item_rect)`.
    pub fn virtual_scroll_area_bounded<F>(
        &mut self,
        name: &'static str,
        viewport_rect: Rect,
        scroll_y: f32,
        total_items: usize,
        item_stride: f32,
        mut render_item: F,
    ) -> (ScrollAreaFrame, VirtualSlice)
    where
        F: FnMut(&mut UiScope<'_>, usize, Rect),
    {
        let vlist = VirtualList::new(total_items, item_stride);
        let content_height = vlist.total_content_height();
        let slice = vlist.compute_slice(viewport_rect.height, scroll_y);

        let config = ScrollAreaConfig::new(content_height, scroll_y).with_name(name);

        let frame = self.scroll_area_with_config(viewport_rect, &config, |scope| {
            let item_w = if frame_scrollbar_active(content_height, viewport_rect.height) {
                (viewport_rect.width - config.style.thickness - config.style.inset * 2.0).max(10.0)
            } else {
                viewport_rect.width
            };

            for idx in slice.start_idx..slice.end_idx {
                let item_top = viewport_rect.y + (idx as f32 * item_stride) - scroll_y;
                let item_rect = Rect::new(viewport_rect.x, item_top, item_w, item_stride);
                render_item(scope, idx, item_rect);
            }
        });

        (frame, slice)
    }
}

#[inline]
fn frame_scrollbar_active(content_height: f32, viewport_height: f32) -> bool {
    content_height > viewport_height && viewport_height > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::tree::UiTree;

    #[test]
    fn test_scroll_area_virtual_culling_slice() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root must exist");

        let vp = Rect::new(0.0, 0.0, 200.0, 400.0);
        let total_items = 10_000;
        let item_stride = 20.0; // 10,000 * 20 = 200,000 px content height
        let scroll_y = 1_000.0; // 50 items scrolled down

        let mut rendered_indices = Vec::new();

        let (_frame, slice) = {
            let mut scope = UiScope::new(&mut tree, root);
            scope.virtual_scroll_area_bounded(
                "TestVirtualOutliner",
                vp,
                scroll_y,
                total_items,
                item_stride,
                |_s, idx, rect| {
                    rendered_indices.push(idx);
                    assert_eq!(
                        rect.width,
                        vp.width
                            - ScrollAreaStyle::default().thickness
                            - ScrollAreaStyle::default().inset * 2.0
                    );
                },
            )
        };

        // 10,000 items in collection, but only ~24 items rendered in UI tree!
        assert!(slice.visible_count < 30);
        assert_eq!(rendered_indices.len(), slice.visible_count);
        assert_eq!(rendered_indices[0], slice.start_idx);
        assert_eq!(*rendered_indices.last().unwrap(), slice.end_idx - 1);
    }

    #[test]
    fn test_declarative_scroll_area_scope() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root must exist");

        let vp = Rect::new(10.0, 10.0, 300.0, 400.0);
        let content_height = 800.0;
        let scroll_y = 100.0;

        let frame = {
            let mut scope = UiScope::new(&mut tree, root);
            scope.scroll_area_bounded("MyScrollContainer", vp, scroll_y, content_height, |s| {
                s.text("Header item inside scroll");
                s.text("Second item inside scroll");
            })
        };

        // Viewport container was created with clip_children
        let container_node = tree.get(frame.container_id).unwrap();
        assert!(container_node.style.clip_children);
        assert_eq!(container_node.computed_rect, vp);
        assert!(container_node.children.len() >= 2);

        // Scrollbar was created because content_height 800 > viewport 400
        assert!(frame.scrollbar.is_some());
        let geom = frame.scrollbar.unwrap();
        assert_eq!(geom.track_rect.y, vp.y + ScrollAreaStyle::default().inset);
    }

    #[test]
    fn test_declarative_scroll_area_fits_no_scrollbar() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root must exist");

        let vp = Rect::new(0.0, 0.0, 200.0, 500.0);
        let content_height = 300.0; // Content fits in viewport

        let frame = {
            let mut scope = UiScope::new(&mut tree, root);
            scope.scroll_area_bounded("FittingContainer", vp, 0.0, content_height, |s| {
                s.text("Fits completely");
            })
        };

        // No scrollbar indicator when content fits
        assert!(frame.scrollbar.is_none());
        assert_eq!(frame.max_scroll_y, 0.0);
    }
}