// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative ScrollArea and Virtualized List Scope Primitives
//!
//! Exposes fluent declarative [`UiScope`] methods for embedding hardware-clipped
//! scrollable containers and $O(1)$ virtualized lists into the retainable UI tree.

pub use super::scroll_area_options::{ScrollAreaConfig, ScrollAreaOptions};

use super::core::UiScope;
use crate::scroll_area::area::{ScrollAreaBuilder, ScrollAreaFrame};
use crate::scroll_area::style::ScrollAreaStyle;
use crate::scroll_area::types::{
    ScrollAreaResponse, VirtualScrollConfig, VirtualScrollResponse, VirtualSlice,
};
use crate::scroll_area::virtual_list::VirtualList;
use irisui_core::geometry::Rect;
use irisui_core::node::WidgetRole;
use irisui_core::style::{FlexDirection, Style};

impl<'a> UiScope<'a> {
    /// Emits an autonomous declarative scroll container that clips its children,
    /// measures content height, and emits a scrollbar when content overflows the viewport.
    ///
    /// Autonomously determines the visible viewport height via [`Self::resolve_available_height`]
    /// and retrieves persistent scroll state from [`irisui_core::UiTree::scroll_offset`], completely eliminating
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

    /// Emits an autonomous declarative scroll container with continuous auto-scrolling management.
    ///
    /// When `*auto_scroll` is `true`, pins the scroll offset to the bottom of the container.
    /// If an upward mouse scroll gesture is detected on this container during the current frame,
    /// `*auto_scroll` is automatically set to `false` to respect the user's manual inspection.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier for the container node.
    /// * `tag` - 64-bit semantic tag of the container node.
    /// * `auto_scroll` - Mutable reference to the boolean auto-scroll state.
    /// * `scrollbar_tags` - Optional `(track_tag, thumb_tag)` for interactive scrollbar indicators.
    /// * `f` - Closure executing within the scroll viewport scope.
    pub fn scroll_area_auto_scroll<R, F>(
        &mut self,
        name: &'static str,
        tag: u64,
        auto_scroll: &mut bool,
        scrollbar_tags: Option<(u64, u64)>,
        f: F,
    ) -> ScrollAreaResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if tag != 0 && self.is_tag_scrolled_up(tag) {
            *auto_scroll = false;
        }
        let explicit_offset = if *auto_scroll { Some(f32::MAX) } else { None };
        self.scroll_area_with_offset(name, tag, explicit_offset, scrollbar_tags, f)
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
        let mut opts = ScrollAreaOptions::new(name, tag);
        opts.explicit_scroll_y = explicit_scroll_y;
        opts.scrollbar_tags = scrollbar_tags;
        self.scroll_area_configured(opts, f)
    }

    /// Emits an autonomous declarative scroll container customized via [`ScrollAreaOptions`],
    /// measuring inner child heights automatically and rendering a vertical scrollbar on overflow.
    pub fn scroll_area_configured<R, F>(
        &mut self,
        opts: ScrollAreaOptions,
        f: F,
    ) -> ScrollAreaResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if let Some(key) = opts.scroll_key {
            self.tree.sync_scroll_key(opts.tag, key);
        }
        let scroll_y = opts.explicit_scroll_y.unwrap_or_else(|| {
            if opts.tag != 0 {
                self.tree.scroll_offset(opts.tag)
            } else {
                0.0
            }
        });

        let avail_w = self
            .resolve_remaining_axis_from_ancestors(true)
            .filter(|&w| w > 10.0);
        let vp_h = opts
            .height
            .unwrap_or_else(|| self.resolve_available_height(opts.tag));

        let parent_is_row = self.tree.get(self.parent).is_some_and(|n| {
            matches!(
                n.style.flex_direction,
                FlexDirection::Row | FlexDirection::RowReverse
            )
        });
        let grow_along_parent = if parent_is_row {
            opts.width.is_none()
        } else {
            opts.height.is_none()
        };

        let mut host_style = Style::new()
            .flex_col()
            .flex_shrink(1.0)
            .min_width(0.0)
            .min_height(0.0);
        if grow_along_parent {
            host_style = host_style.flex_grow(1.0);
        }
        if let Some(w) = opts.width {
            host_style = host_style.width(w);
        }
        if let Some(h) = opts.height {
            host_style = host_style.height(h);
        }
        if let Some(bg) = opts.background {
            host_style = host_style.background(bg);
        }

        let host_id = self.tree.create_node();
        if let Some(host) = self.tree.get_mut(host_id) {
            host.set_name("ScrollAreaHost");
            host.set_style(host_style);
        }
        let _ = self.tree.add_child(self.parent, host_id);

        let mut vp_style = Style::new()
            .flex_col()
            .flex_grow(1.0)
            .flex_shrink(1.0)
            .min_width(0.0)
            .min_height(0.0)
            .clip_children(true)
            .padding_insets(opts.padding)
            .gap(opts.gap);
        vp_style.scroll_offset_y = scroll_y;

        let vp_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(vp_id) {
            node.set_name(opts.name);
            node.set_style(vp_style);
            node.role = WidgetRole::ScrollAreaViewport;
            node.tag = opts.tag;
            node.interactive = true;
        }
        let _ = self.tree.add_child(host_id, vp_id);

        let mut child_scope = self.child_scope(vp_id);
        let inner = f(&mut child_scope);

        let width_hint = opts
            .width
            .or(avail_w)
            .or_else(|| {
                (opts.tag != 0)
                    .then(|| {
                        self.tree
                            .viewport_size(opts.tag)
                            .map(|s| s.width)
                            .or_else(|| self.tree.tag_rect(opts.tag).map(|r| r.width))
                    })
                    .flatten()
            })
            .or_else(|| {
                self.tree
                    .get(self.parent)
                    .map(|n| n.computed_rect.width)
                    .filter(|&w| w > 0.0)
            });
        let content_h =
            crate::declarative::measure_content_height_constrained(self.tree, vp_id, width_hint);
        let max_scroll = (content_h - vp_h).max(0.0);
        let safe_scroll = scroll_y.clamp(0.0, max_scroll);

        if opts.tag != 0 {
            self.tree.set_scroll_offset(opts.tag, safe_scroll);
            self.tree.set_max_scroll(opts.tag, max_scroll);
        }
        if let Some(node) = self.tree.get_mut(vp_id) {
            node.content_size = irisui_core::geometry::Size::new(0.0, content_h);
            node.style.scroll_offset_y = safe_scroll;
        }

        let origin = self.parent_screen_origin();
        let est_w = width_hint.unwrap_or(400.0);
        let fallback_rect = Rect::new(origin.x, origin.y, est_w, vp_h);
        let mut host_scope = self.child_scope(host_id);
        host_scope.emit_configured_scrollbar(
            &opts,
            fallback_rect,
            content_h,
            max_scroll,
            safe_scroll,
        );

        ScrollAreaResponse {
            inner,
            container_id: vp_id,
            max_scroll_y: max_scroll,
            content_height: content_h,
            viewport_height: vp_h,
        }
    }

    fn emit_configured_scrollbar(
        &mut self,
        opts: &ScrollAreaOptions,
        vp_rect: Rect,
        content_h: f32,
        max_scroll: f32,
        safe_scroll: f32,
    ) {
        if max_scroll <= 0.0 {
            return;
        }
        let (track_tag, thumb_tag) = opts.scrollbar_tags.unwrap_or_else(|| {
            let base = if opts.tag != 0 {
                opts.tag
            } else {
                crate::declarative::types::hash_label(opts.name)
            };
            (base.wrapping_add(0x1000), base.wrapping_add(0x2000))
        });

        if let Some(geom) = crate::scroll_area::ScrollBarGeometry::compute_vertical(
            vp_rect,
            content_h,
            safe_scroll,
            &opts.scrollbar_style,
        ) {
            let is_dragging = self
                .tree
                .event_state
                .pointer_capture
                .is_some_and(|cid| self.tree.get(cid).is_some_and(|n| n.tag == thumb_tag));
            let cursor_pos = opts.cursor_pos.or(self.tree.event_state.last_pointer_pos);
            self.scrollbar_vertical(geom, vp_rect, is_dragging, cursor_pos, track_tag, thumb_tag);
        }
    }

    /// Autonomously resolves the available visible vertical viewport height in physical pixels.
    ///
    /// Evaluates:
    /// 1. Active ancestor container bounding geometry (`computed_rect.height` or `style.height`) minus
    ///    height consumed by siblings, padding, and gaps along the container chain.
    /// 2. Preceding layout calculations cached in [`irisui_core::UiTree::viewport_sizes`] or [`irisui_core::UiTree::tag_rects`].
    /// 3. Active screen bounds height or fallback safe minimum height (`400.0px`).
    pub fn resolve_available_height(&self, tag: u64) -> f32 {
        if let Some(h) = self
            .resolve_remaining_axis_from_ancestors(false)
            .filter(|&h| h > 10.0)
        {
            return h.max(20.0);
        }

        if tag != 0
            && let Some(h) = self
                .tree
                .viewport_size(tag)
                .map(|s| s.height)
                .filter(|&h| h > 10.0)
                .or_else(|| {
                    self.tree
                        .tag_rect(tag)
                        .map(|r| r.height)
                        .filter(|&h| h > 10.0)
                })
        {
            return h;
        }

        let screen_h = self.screen_bounds().height;
        if screen_h > 10.0 {
            return screen_h;
        }

        400.0
    }

    /// Emits a high-performance virtualized scrollable container that renders only the visible slice of items.
    ///
    /// Autonomously determines the visible viewport height via [`Self::resolve_available_height`]
    /// and retrieves persistent scroll state from [`irisui_core::UiTree::scroll_offset`], completely eliminating
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

        let vp_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(vp_id) {
            node.set_name(name);
            node.tag = tag;
            node.role = WidgetRole::ScrollAreaViewport;
            node.interactive = true;
            node.set_style(
                Style::new()
                    .flex_col()
                    .flex_grow(1.0)
                    .min_height(0.0)
                    .flex_shrink(1.0)
                    .clip_children(true)
                    .background(config.bg),
            );
        }
        let _ = self.tree.add_child(self.parent, vp_id);

        {
            let mut vp = self.child_scope(vp_id);
            vp.set_scroll_offset_y(safe_scroll);
            if slice.start_idx > 0 {
                let top_spacer_h = slice.start_idx as f32 * stride;
                vp.empty_box(
                    Style::new()
                        .height(top_spacer_h)
                        .min_height(top_spacer_h)
                        .flex_shrink(0.0),
                );
            }
            for idx in slice.start_idx..slice.end_idx {
                item_renderer(&mut vp, idx);
            }
        }

        if let Some(node) = self.tree.get_mut(vp_id) {
            node.content_size = irisui_core::geometry::Size::new(0.0, vlist.total_content_height());
        }

        // Autonomous scrollbar indicator emission if configured or defaulting to derived tags
        let effective_scrollbar_tags = config.scrollbar_tags.or_else(|| {
            if tag != 0 {
                Some((tag.wrapping_add(0x1000), tag.wrapping_add(0x2000)))
            } else {
                let derived = crate::declarative::types::hash_label(name);
                Some((derived.wrapping_add(0x1000), derived.wrapping_add(0x2000)))
            }
        });

        if let Some((track_tag, thumb_tag)) = effective_scrollbar_tags.filter(|_| max_scroll > 0.0)
        {
            let parent_rect = self
                .tree
                .get(self.parent)
                .map(|n| n.computed_rect)
                .filter(|r| r.height > 0.0)
                .unwrap_or_else(|| {
                    let vp_w = self.tree.viewport_size(tag).map_or(400.0, |s| s.width);
                    Rect::new(0.0, 0.0, vp_w, viewport_height)
                });

            let vp_rect = self
                .tree
                .get(vp_id)
                .map(|n| n.computed_rect)
                .filter(|r| r.height > 0.0)
                .unwrap_or_else(|| {
                    Rect::new(
                        parent_rect.x,
                        parent_rect.y,
                        parent_rect.width,
                        viewport_height,
                    )
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

    /// Computes the visible range of items `start_idx..end_idx` for virtualized collections
    /// by autonomously reading the enclosing scroll container's scroll offset and available viewport height.
    ///
    /// Eliminates manual viewport height subtractions and explicit tag querying from panel callers.
    ///
    /// # Arguments
    /// * `item_height` - Uniform vertical height (stride) of each item in logical pixels.
    /// * `total_count` - Total number of items in the underlying collection.
    pub fn visible_range(&self, item_height: f32, total_count: usize) -> std::ops::Range<usize> {
        if total_count == 0 || item_height <= 0.0 {
            return 0..0;
        }

        let mut scroll_y = 0.0;
        let mut vp_h = 400.0;

        let mut current = Some(self.parent);
        while let Some(curr_id) = current {
            if let Some(node) = self.tree.get(curr_id) {
                if node.role == WidgetRole::ScrollAreaViewport || node.style.clip_children {
                    scroll_y = if node.tag != 0 {
                        self.tree.scroll_offset(node.tag)
                    } else {
                        node.style.scroll_offset_y
                    };
                    vp_h = self.resolve_available_height(node.tag);
                    break;
                }
                current = node.parent;
            } else {
                break;
            }
        }

        let vlist = VirtualList::new(total_count, item_height).with_overscan(2);
        let slice = vlist.compute_slice(vp_h, scroll_y);
        slice.start_idx..slice.end_idx
    }
}

#[inline]
fn frame_scrollbar_active(content_height: f32, viewport_height: f32) -> bool {
    content_height > viewport_height && viewport_height > 0.0
}