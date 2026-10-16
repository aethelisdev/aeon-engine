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
use crate::scroll_area::types::{
    ScrollAreaResponse, ScrollBarVisibility, VirtualScrollConfig, VirtualScrollResponse,
    VirtualSlice,
};
use crate::scroll_area::virtual_list::VirtualList;
use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
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
    /// Emits a tagged scrollable container with flex-grow expansion and hardware child clipping.

    /// Emits an autonomous declarative scroll container that clips its children,
    /// measures content height, and emits a scrollbar when content overflows the viewport.
    ///
    /// Autonomously determines the visible viewport height via [`Self::resolve_available_height`]
    /// and retrieves persistent scroll state from [`UiTree::scroll_offset`], completely eliminating
    /// manual height subtraction boilerplate (`let vp_h = panel_rect.height - ...`) and manual
    /// scrollbar construction from callers.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the scroll viewport node.
    /// * `tag` - Semantic tag for tracking interactions, scroll offsets, and viewport size.
    /// * `scrollbar_tags` - Optional `(track_tag, thumb_tag)` for autonomous scrollbar rendering.
    /// * `f` - Closure executing within the scroll viewport scope.
    pub fn scroll_area<R, F>(
        &mut self,
        name: &'static str,
        tag: u64,
        scrollbar_tags: Option<(u64, u64)>,
        f: F,
    ) -> ScrollAreaResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.scroll_area_with_offset(name, tag, None, scrollbar_tags, f)
    }

    /// Emits an autonomous declarative scroll container with an optional explicit vertical scroll offset override.
    pub fn scroll_area_with_offset<R, F>(
        &mut self,
        name: &'static str,
        tag: u64,
        explicit_scroll_y: Option<f32>,
        scrollbar_tags: Option<(u64, u64)>,
        f: F,
    ) -> ScrollAreaResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let scroll_y = explicit_scroll_y.unwrap_or_else(|| {
            if tag != 0 {
                self.tree.scroll_offset(tag)
            } else {
                0.0
            }
        });

        let mut style = Style::new().flex_col().flex_grow(1.0).clip_children(true);
        style.scroll_offset_y = scroll_y;

        let vp_resp = self.container_tagged(name, style, WidgetRole::ScrollAreaViewport, tag, f);
        let vp_id = vp_resp.response.id;
        let inner = vp_resp.inner;

        let vp_h = self.resolve_available_height(tag);
        let content_h = crate::declarative::measure_content_height(self.tree, vp_id);
        let max_scroll = (content_h - vp_h).max(0.0);
        let safe_scroll = if max_scroll > 0.0 {
            scroll_y.clamp(0.0, max_scroll)
        } else {
            scroll_y
        };

        if tag != 0 {
            self.tree.set_max_scroll(tag, max_scroll);
        }

        if let Some(node) = self.tree.get_mut(vp_id) {
            node.content_size = iris_core::geometry::Size::new(0.0, content_h);
            node.style.scroll_offset_y = safe_scroll;
        }

        if let Some((track_tag, thumb_tag)) = scrollbar_tags.filter(|_| max_scroll > 0.0) {
            let vp_rect = self
                .tree
                .get(vp_id)
                .map(|n| n.computed_rect)
                .filter(|r| r.height > 0.0)
                .unwrap_or_else(|| {
                    let vp_w = self.tree.viewport_size(tag).map_or(100.0, |s| s.width);
                    Rect::new(0.0, 0.0, vp_w, vp_h)
                });

            let scroll_style = ScrollAreaStyle::dark_default();
            if let Some(geom) = crate::scroll_area::ScrollBarGeometry::compute_vertical(
                vp_rect,
                content_h,
                safe_scroll,
                &scroll_style,
            ) {
                let is_dragging = self
                    .tree
                    .event_state
                    .pointer_capture
                    .is_some_and(|cid| self.tree.get(cid).is_some_and(|n| n.tag == thumb_tag));
                let cursor_pos = self.tree.event_state.last_pointer_pos;
                let parent_rect = self
                    .tree
                    .get(self.parent)
                    .map(|n| n.computed_rect)
                    .unwrap_or(vp_rect);

                self.scrollbar_vertical(
                    geom,
                    parent_rect,
                    is_dragging,
                    cursor_pos,
                    track_tag,
                    thumb_tag,
                );
            }
        }

        ScrollAreaResponse {
            inner,
            container_id: vp_id,
            max_scroll_y: max_scroll,
            content_height: content_h,
            viewport_height: vp_h,
        }
    }

    /// Autonomously resolves the available visible vertical viewport height in physical pixels.
    ///
    /// Evaluates:
    /// 1. Preceding layout calculations cached in [`UiTree::viewport_sizes`].
    /// 2. Active parent container bounding geometry (`computed_rect.height` or `style.height`) minus
    ///    height consumed by preceding sibling nodes and gaps.
    /// 3. Grandparent/ancestor bounding geometry.
    /// 4. Fallback safe minimum height (400.0px).
    pub fn resolve_available_height(&self, tag: u64) -> f32 {
        let cached_h = if tag != 0 {
            self.tree
                .viewport_size(tag)
                .map(|s| s.height)
                .filter(|&h| h > 10.0)
        } else {
            None
        };
        if let Some(h) = cached_h {
            return h;
        }

        let mut current = Some(self.parent);
        while let Some(curr_id) = current {
            if let Some(node) = self.tree.get(curr_id) {
                let node_h = if node.computed_rect.height > 10.0 {
                    node.computed_rect.height
                } else {
                    node.style.height.unwrap_or(0.0)
                };

                if node_h > 10.0 {
                    let mut consumed_h = 0.0_f32;
                    for &cid in &node.children {
                        if let Some(child) = self.tree.get(cid) {
                            let ch = if let Some(h) = child.style.height {
                                h
                            } else if child.computed_rect.height > 0.0 {
                                child.computed_rect.height
                            } else {
                                0.0
                            };
                            let margin = child.style.margin;
                            consumed_h += ch + margin.top + margin.bottom;
                        }
                    }
                    let gaps = node.style.gap * node.children.len().saturating_sub(1) as f32;
                    let remaining = (node_h - consumed_h - gaps).max(20.0);
                    return remaining;
                }
                current = node.parent;
            } else {
                break;
            }
        }

        400.0
    }

    /// Emits a high-performance virtualized scrollable container that renders only the visible slice of items.
    ///
    /// Autonomously determines the visible viewport height via [`Self::resolve_available_height`]
    /// and retrieves persistent scroll state from [`UiTree::scroll_offset`], completely eliminating
    /// manual height subtraction boilerplate (`let vp_h = panel_rect.height - ...`) from callers.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the scroll viewport node.
    /// * `tag` - Semantic tag for tracking interactions, persistent scroll offsets, and viewport size.
    /// * `config` - Sizing, overscan, and container styling configuration descriptor.
    /// * `item_renderer` - Closure called for each visible index: `(scope, index)`.
    pub fn virtual_scroll_area<F>(
        &mut self,
        name: &'static str,
        tag: u64,
        config: VirtualScrollConfig,
        mut item_renderer: F,
    ) -> VirtualScrollResponse
    where
        F: FnMut(&mut UiScope<'_>, usize),
    {
        let viewport_height = config
            .viewport_height
            .unwrap_or_else(|| self.resolve_available_height(tag));

        let current_scroll = config
            .scroll_y
            .unwrap_or_else(|| self.tree.scroll_offset(tag));

        let stride = config.stride();
        let vlist = VirtualList::new(config.total_items, stride).with_overscan(config.overscan);
        let max_scroll = vlist.max_scroll_y(viewport_height);
        let safe_scroll = current_scroll.clamp(0.0, max_scroll);
        let slice = vlist.compute_slice(viewport_height, safe_scroll);

        if tag != 0 {
            self.tree.set_scroll_offset(tag, safe_scroll);
            self.tree.set_max_scroll(tag, max_scroll);
        }

        let vp_id = crate::declarative::widget::ScrollAreaView::new(name)
            .tag(tag)
            .background(config.bg)
            .show(self, |vp| {
                vp.set_scroll_offset_y(safe_scroll);
                if slice.start_idx > 0 {
                    let top_spacer_h = slice.start_idx as f32 * stride;
                    vp.empty_box(Style::new().height(top_spacer_h));
                }
                for idx in slice.start_idx..slice.end_idx {
                    item_renderer(vp, idx);
                }
                let remaining = config.total_items.saturating_sub(slice.end_idx);
                if remaining > 0 {
                    let bottom_spacer_h = remaining as f32 * stride;
                    vp.empty_box(Style::new().height(bottom_spacer_h));
                }
            })
            .response
            .id;

        if let Some(node) = self.tree.get_mut(vp_id) {
            node.content_size = iris_core::geometry::Size::new(0.0, vlist.total_content_height());
        }

        // Autonomous scrollbar indicator emission if configured
        if let Some((track_tag, thumb_tag)) = config.scrollbar_tags.filter(|_| max_scroll > 0.0) {
            let vp_rect = self
                .tree
                .get(vp_id)
                .map(|n| n.computed_rect)
                .filter(|r| r.height > 0.0)
                .unwrap_or_else(|| {
                    let vp_w = self.tree.viewport_size(tag).map_or(100.0, |s| s.width);
                    Rect::new(0.0, 0.0, vp_w, viewport_height)
                });

            let scroll_style = ScrollAreaStyle::dark_default();
            if let Some(geom) = crate::scroll_area::ScrollBarGeometry::compute_vertical(
                vp_rect,
                vlist.total_content_height(),
                safe_scroll,
                &scroll_style,
            ) {
                let is_dragging = self
                    .tree
                    .event_state
                    .pointer_capture
                    .is_some_and(|cid| self.tree.get(cid).is_some_and(|n| n.tag == thumb_tag));
                let cursor_pos = self.tree.event_state.last_pointer_pos;
                let parent_rect = self
                    .tree
                    .get(self.parent)
                    .map(|n| n.computed_rect)
                    .unwrap_or(vp_rect);

                self.scrollbar_vertical(
                    geom,
                    parent_rect,
                    is_dragging,
                    cursor_pos,
                    track_tag,
                    thumb_tag,
                );
            }
        }

        VirtualScrollResponse {
            max_scroll_y: max_scroll,
            current_scroll_y: safe_scroll,
            viewport_height,
            content_height: vlist.total_content_height(),
            visible_start: slice.start_idx,
            visible_end: slice.end_idx,
            container_id: vp_id,
        }
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

    #[test]
    fn test_autonomous_virtual_scroll_area_height_resolution_and_slicing() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root must exist");

        let test_tag = 0x9001;
        // Mock a previously computed viewport size in the tree
        tree.set_viewport_size(test_tag, iris_core::Size::new(240.0, 300.0));
        // Mock a scroll position of 100.0 px (5 items down with 20px item height)
        tree.set_scroll_offset(test_tag, 100.0);

        let mut rendered_indices = Vec::new();
        let config = VirtualScrollConfig::fixed(1000, 20.0).with_scrollbar_tags(0x9002, 0x9003);

        let resp = {
            let mut scope = UiScope::new(&mut tree, root);
            scope.virtual_scroll_area("AutonomousVirtualList", test_tag, config, |_s, idx| {
                rendered_indices.push(idx);
            })
        };

        assert_eq!(resp.viewport_height, 300.0);
        assert_eq!(resp.current_scroll_y, 100.0);
        assert_eq!(resp.content_height, 20_000.0);
        assert_eq!(resp.max_scroll_y, 20_000.0 - 300.0);
        assert!(!rendered_indices.is_empty());
        assert!(rendered_indices.contains(&5)); // item at y=100 is visible
        assert_eq!(resp.visible_start, rendered_indices[0]);
        assert_eq!(resp.visible_end, *rendered_indices.last().unwrap() + 1);

        // Verify scrollbar was emitted autonomously
        let thumb_node = tree.find_node_by_tag(0x9003);
        assert!(
            thumb_node.is_some(),
            "autonomous scrollbar thumb should be emitted"
        );
    }

    #[test]
    fn test_autonomous_height_resolution_parent_minus_siblings() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root must exist");

        // Set root panel rect to 600px height
        if let Some(r) = tree.get_mut(root) {
            r.computed_rect = Rect::new(0.0, 0.0, 400.0, 600.0);
        }

        let resolved_h = {
            let mut scope = UiScope::new(&mut tree, root);
            // Add a fixed 40px toolbar sibling
            scope.container(Style::new().height(40.0), |_s| {});
            // Add a 10px spacing sibling
            scope.container(Style::new().height(10.0), |_s| {});
            // Resolve available height for next child with unrecorded tag
            scope.resolve_available_height(0x9999)
        };

        // 600 - 40 - 10 = 550.0
        assert_eq!(resolved_h, 550.0);
    }

    #[test]
    fn test_autonomous_scroll_area_measures_content_and_emits_scrollbar() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root must exist");

        let test_tag = 0x9010;
        let track_tag = 0x9011;
        let thumb_tag = 0x9012;

        tree.set_viewport_size(test_tag, iris_core::Size::new(300.0, 200.0));

        let resp = {
            let mut scope = UiScope::new(&mut tree, root);
            scope.scroll_area(
                "AutonomousContentScroll",
                test_tag,
                Some((track_tag, thumb_tag)),
                |vp| {
                    // 5 children of 80px each = 400px content
                    for _ in 0..5 {
                        vp.container(Style::new().height(80.0), |_s| {});
                    }
                },
            )
        };

        assert_eq!(resp.viewport_height, 200.0);
        assert_eq!(resp.content_height, 400.0);
        assert_eq!(resp.max_scroll_y, 200.0);

        let thumb_node = tree.find_node_by_tag(thumb_tag);
        assert!(
            thumb_node.is_some(),
            "autonomous scrollbar thumb should be emitted for overflowing content"
        );
    }
}