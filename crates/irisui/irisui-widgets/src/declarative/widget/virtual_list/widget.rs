// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative VirtualList Widget Builder
//!
//! Provides the fluent [`VirtualListBuilder`] and [`UiScope::virtual_list`] API for
//! virtualized $O(1)$ list rendering over arbitrarily large datasets (100k+ to 10M+ items).
//! Automatically inserts top and bottom spacer nodes to simulate full scroll height
//! while emitting strictly visible rows ($K \sim 30$) into the retainable UI tree.

use super::math::{DEFAULT_OVERSCAN, VirtualSlice, VirtualSliceParams, compute_virtual_slice};
use crate::declarative::scope::core::UiScope;
use crate::scroll_area::ScrollBarGeometry;
use crate::scroll_area::style::ScrollAreaStyle;
use irisui_core::geometry::{Insets, Rect, Size};
use irisui_core::node::WidgetRole;
use irisui_core::{Color, Style, WidgetId};

/// Response returned after rendering a declarative virtual list pass.
///
/// Encapsulates visible slice metrics, viewport container ID, and layout bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VirtualListResponse {
    /// Calculated visible slice metrics (first/last indices, spacers, total height).
    pub slice: VirtualSlice,
    /// Viewport container node ID in the retainable UI tree.
    pub container_id: WidgetId,
    /// Maximum scroll offset before reaching the bottom edge ($H_{\text{total}} - V$).
    pub max_scroll_y: f32,
    /// Resolved visible viewport height in logical pixels ($V$).
    pub viewport_height: f32,
}

impl VirtualListResponse {
    /// Zero-based index of the first visible item (inclusive).
    #[inline]
    #[must_use]
    pub const fn first_index(&self) -> usize {
        self.slice.first_index
    }

    /// Zero-based ending index of visible items (exclusive).
    #[inline]
    #[must_use]
    pub const fn last_index(&self) -> usize {
        self.slice.last_index
    }

    /// Total count of items rendered in the current frame ($K = \text{last} - \text{first}$).
    #[inline]
    #[must_use]
    pub const fn visible_count(&self) -> usize {
        self.slice.visible_count()
    }

    /// Total virtual content height across all items in logical pixels ($N \times H$).
    #[inline]
    #[must_use]
    pub const fn total_height(&self) -> f32 {
        self.slice.total_height
    }

    /// Height in logical pixels of the top spacer element ($H_{\text{top}}$).
    #[inline]
    #[must_use]
    pub const fn top_spacer(&self) -> f32 {
        self.slice.top_spacer
    }

    /// Height in logical pixels of the bottom spacer element ($H_{\text{bottom}}$).
    #[inline]
    #[must_use]
    pub const fn bottom_spacer(&self) -> f32 {
        self.slice.bottom_spacer
    }
}

/// Fluent builder for declarative virtualized list containers.
///
/// Evaluates visible index slices and top/bottom spacer geometries in $O(1)$ constant time,
/// restricting active UI tree nodes to strictly visible elements even when operating over
/// massive datasets (1.000.000+ items).
#[must_use = "builders do nothing until .render_item() or .show() is called"]
pub struct VirtualListBuilder<'s, 't> {
    scope: &'s mut UiScope<'t>,
    item_count: usize,
    item_height: f32,
    visible_height: Option<f32>,
    scroll_y: Option<f32>,
    scroll_key: Option<u64>,
    overscan: usize,
    tag: u64,
    name: &'static str,
    background: Option<Color>,
    scrollbar_tags: Option<(u64, u64)>,
    width: Option<f32>,
    margin: Option<Insets>,
    gap: Option<f32>,
    empty_message: Option<&'static str>,
}

/// Type alias aligning with idiomatic widget naming conventions.
pub type VirtualList<'s, 't> = VirtualListBuilder<'s, 't>;

impl<'s, 't> VirtualListBuilder<'s, 't> {
    /// Creates a new virtual list builder attached to the given UI scope.
    ///
    /// # Arguments
    ///
    /// * `scope` - Mutable reference to the parent declarative UI scope.
    /// * `item_count` - Total number of elements in the collection ($N$).
    /// * `item_height` - Uniform height allocated to each item in logical pixels ($H$).
    #[inline]
    pub fn new(scope: &'s mut UiScope<'t>, item_count: usize, item_height: f32) -> Self {
        Self {
            scope,
            item_count,
            item_height,
            visible_height: None,
            scroll_y: None,
            scroll_key: None,
            overscan: DEFAULT_OVERSCAN,
            tag: 0,
            name: "VirtualList",
            background: None,
            scrollbar_tags: None,
            width: None,
            margin: None,
            gap: None,
            empty_message: None,
        }
    }

    /// Sets explicit width for the virtual list viewport container.
    #[inline]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets margin insets around the virtual list viewport container.
    #[inline]
    pub fn margin_insets(mut self, insets: Insets) -> Self {
        self.margin = Some(insets);
        self
    }

    /// Sets the gap in pixels between consecutive rendered child items in the viewport container.
    #[inline]
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    /// Assigns explicit hit-testing tags to the vertical scrollbar track and thumb nodes.
    ///
    /// Enables interactive mouse-drag and click-to-jump on the vertical scrollbar indicator.
    #[inline]
    pub fn scrollbar_tags(mut self, track_tag: u64, thumb_tag: u64) -> Self {
        self.scrollbar_tags = Some((track_tag, thumb_tag));
        self
    }

    /// Explicitly sets the visible viewport height in logical pixels ($V$).
    ///
    /// When omitted, the builder autonomously resolves available height via the parent container.
    #[inline]
    pub fn visible_height(mut self, height: f32) -> Self {
        self.visible_height = Some(height);
        self
    }

    /// Explicitly sets the vertical scroll position in logical pixels ($S_y$).
    ///
    /// When omitted, the builder queries persistent scroll state via [`UiTree::scroll_offset`].
    #[inline]
    pub fn scroll_y(mut self, scroll_y: f32) -> Self {
        self.scroll_y = Some(scroll_y);
        self
    }

    /// Associates a domain `key` with this virtual list so key changes reset scroll to `0.0`.
    #[inline]
    pub fn scroll_key(mut self, key: u64) -> Self {
        self.scroll_key = Some(key);
        self
    }

    /// Overrides the bidirectional overscan margin buffer count.
    ///
    /// By default, retains 2 buffer rows above and below the visible viewport to eliminate
    /// pop-in during rapid trackpad and mousewheel scrolling.
    #[inline]
    pub fn overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }

    /// Sets a persistent 64-bit semantic tag for tracking scroll offsets and interactions.
    #[inline]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Sets a custom node name for debugging and tree inspection.
    #[inline]
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Sets the viewport background fill color.
    #[inline]
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Sets an empty placeholder message displayed inside the viewport when `item_count == 0`.
    ///
    /// # Arguments
    ///
    /// * `message` - Static text label displayed in the center of the empty viewport.
    #[inline]
    pub fn empty_notice(mut self, message: &'static str) -> Self {
        self.empty_message = Some(message);
        self
    }

    /// Automatically manages continuous auto-scrolling to the bottom of the virtual list.
    ///
    /// When `*auto_scroll` is `true`, sets the scroll offset to `f32::MAX` to pin the list
    /// to the latest appended items.
    /// If an upward mouse scroll gesture is detected on this container during the current frame,
    /// `*auto_scroll` is automatically set to `false` to preserve the user's manual scroll position.
    ///
    /// # Arguments
    ///
    /// * `auto_scroll` - Mutable reference to the boolean auto-scroll state.
    #[inline]
    pub fn auto_scroll(mut self, auto_scroll: &mut bool) -> Self {
        if self.tag != 0 && self.scope.is_tag_scrolled_up(self.tag) {
            *auto_scroll = false;
        }
        if *auto_scroll {
            self.scroll_y = Some(f32::MAX);
        }
        self
    }

    /// Invokes the provided closure if an upward mouse wheel scroll was performed on this virtual list.
    ///
    /// # Arguments
    ///
    /// * `callback` - Closure executed when an upward scroll occurs on this list.
    #[inline]
    pub fn on_scroll_up<F>(self, mut callback: F) -> Self
    where
        F: FnMut(),
    {
        if self.tag != 0 && self.scope.is_tag_scrolled_up(self.tag) {
            callback();
        }
        self
    }

    /// Invokes the provided closure if a downward mouse wheel scroll was performed on this virtual list.
    ///
    /// # Arguments
    ///
    /// * `callback` - Closure executed when a downward scroll occurs on this list.
    #[inline]
    pub fn on_scroll_down<F>(self, mut callback: F) -> Self
    where
        F: FnMut(),
    {
        if self.tag != 0 && self.scope.is_tag_scrolled_down(self.tag) {
            callback();
        }
        self
    }

    /// Evaluates the visible slice and invokes the `item_renderer` closure for every visible item index.
    ///
    /// Inserts top and bottom spacer nodes to simulate the full scroll height while restricting
    /// rendered children to strictly visible rows ($K \sim 30$) for $O(1)$ memory usage.
    ///
    /// # Arguments
    ///
    /// * `item_renderer` - Closure receiving the child UI scope and item index: `|item_scope, index| { ... }`.
    pub fn render_item<F>(self, mut item_renderer: F) -> VirtualListResponse
    where
        F: FnMut(&mut UiScope<'_>, usize),
    {
        if let Some(key) = self.scroll_key {
            self.scope.sync_scroll_key(self.tag, key);
        }
        let viewport_height = self
            .visible_height
            .unwrap_or_else(|| self.scope.resolve_available_height(self.tag));

        let current_scroll = self.scroll_y.unwrap_or_else(|| {
            if self.tag != 0 {
                self.scope.tree.scroll_offset(self.tag)
            } else {
                0.0
            }
        });

        let total_content_height = (self.item_count as f64 * self.item_height as f64) as f32;
        let max_scroll = (total_content_height - viewport_height).max(0.0);
        let safe_scroll = current_scroll.clamp(0.0, max_scroll);

        let slice = compute_virtual_slice(VirtualSliceParams {
            item_count: self.item_count,
            item_height: self.item_height,
            viewport_height,
            scroll_y: safe_scroll,
            overscan: self.overscan,
        });

        if self.tag != 0 {
            self.scope.tree.set_scroll_offset(self.tag, safe_scroll);
            self.scope.tree.set_max_scroll(self.tag, max_scroll);
        }

        let vp_id = self.scope.tree.create_node();
        if let Some(node) = self.scope.tree.get_mut(vp_id) {
            node.set_name(self.name);
            node.tag = self.tag;
            node.role = WidgetRole::ScrollAreaViewport;
            node.interactive = true;
            node.content_size = Size::new(0.0, slice.total_height);

            let mut style = Style::new()
                .flex_col()
                .flex_grow(1.0)
                .min_height(0.0)
                .flex_shrink(1.0)
                .clip_children(true)
                .scroll_offset_y(safe_scroll);

            if let Some(w) = self.width {
                style = style.width(w);
            }
            if let Some(m) = self.margin {
                style = style.margin_insets(m);
            }
            if let Some(g) = self.gap {
                style = style.gap(g);
            }
            if let Some(vh) = self.visible_height {
                style = style.height(vh).max_height(vh);
            }
            if let Some(bg) = self.background {
                style = style.background(bg);
            }
            node.set_style(style);
        }
        let _ = self.scope.tree.add_child(self.scope.parent, vp_id);

        {
            let mut child = self.scope.child_scope(vp_id);
            child.set_scroll_offset_y(safe_scroll);

            if self.item_count == 0 {
                if let Some(msg) = self.empty_message {
                    child.empty_notice("VirtualListEmptyNotice", msg);
                }
            } else {
                // 1. Top Spacer: emitted only when first_index > 0
                if slice.top_spacer > 0.0 {
                    child.empty_box_passive_named(
                        "VirtualListTopSpacer",
                        Style::new()
                            .height(slice.top_spacer)
                            .min_height(slice.top_spacer)
                            .flex_shrink(0.0),
                    );
                }

                // 2. Visible Rows: strictly limited to the computed slice window
                for idx in slice.range() {
                    item_renderer(&mut child, idx);
                }

                // 3. Bottom Spacer: emitted only when last_index < item_count
                if slice.bottom_spacer > 0.0 {
                    child.empty_box_passive_named(
                        "VirtualListBottomSpacer",
                        Style::new()
                            .height(slice.bottom_spacer)
                            .min_height(slice.bottom_spacer)
                            .flex_shrink(0.0),
                    );
                }
            }
        }

        // 4. Autonomous vertical scrollbar emission
        let effective_scrollbar = self.scrollbar_tags.or_else(|| {
            if self.tag != 0 {
                Some((self.tag.wrapping_add(0x1000), self.tag.wrapping_add(0x2000)))
            } else {
                None
            }
        });

        if let Some((track_tag, thumb_tag)) = effective_scrollbar
            && slice.total_height > viewport_height
        {
            let scroll_style = ScrollAreaStyle::dark_default();
            let vp_rect = self.scope.tree.tag_rect(self.tag).unwrap_or(Rect::new(
                0.0,
                0.0,
                400.0,
                viewport_height,
            ));

            if let Some(geom) = ScrollBarGeometry::compute_vertical(
                vp_rect,
                slice.total_height,
                safe_scroll,
                &scroll_style,
            ) {
                let is_dragging = self
                    .scope
                    .tree
                    .event_state
                    .pointer_capture
                    .is_some_and(|cid| {
                        self.scope.tree.get(cid).is_some_and(|n| n.tag == thumb_tag)
                    });
                let cursor_pos = self.scope.tree.event_state.last_pointer_pos;

                self.scope.scrollbar_vertical(
                    geom,
                    vp_rect,
                    is_dragging,
                    cursor_pos,
                    track_tag,
                    thumb_tag,
                );
            }
        }

        VirtualListResponse {
            slice,
            container_id: vp_id,
            max_scroll_y: max_scroll,
            viewport_height,
        }
    }

    /// Alias for [`Self::render_item`] supporting the idiomatic `.show(...)` container pattern.
    #[inline]
    pub fn show<F>(self, item_renderer: F) -> VirtualListResponse
    where
        F: FnMut(&mut UiScope<'_>, usize),
    {
        self.render_item(item_renderer)
    }
}

impl<'a> UiScope<'a> {
    /// Spawns a declarative virtual list builder for high-performance $O(1)$ windowed rendering.
    ///
    /// Prunes non-visible rows from the retainable UI tree, emitting only top/bottom spacer
    /// nodes and currently visible rows ($K \sim 30$) regardless of collection size ($N = 100k+$).
    ///
    /// # Example
    ///
    /// ```ignore
    /// scope.virtual_list(100_000, 24.0)
    ///     .visible_height(600.0)
    ///     .scroll_y(1200.0)
    ///     .render_item(|item_scope, index| {
    ///         item_scope.label(format!("Row #{}", index));
    ///     });
    /// ```
    pub fn virtual_list<'s>(
        &'s mut self,
        item_count: usize,
        item_height: f32,
    ) -> VirtualListBuilder<'s, 'a> {
        VirtualListBuilder::new(self, item_count, item_height)
    }
}