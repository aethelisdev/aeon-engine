// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Virtualized Linear Item List Geometry (`iris-widgets::scroll_area::virtual_list`)
//!
//! Provides an $O(1)$ zero-allocation virtualized list windowing engine that calculates
//! visible item indices and vertical offsets for arbitrarily large collections (100k+ rows).
//!

use super::types::VirtualSlice;

/// High-performance $O(1)$ virtualized windowing calculator for uniform-height items.
///
/// Translates total item counts, row strides, and viewport bounds into visible index slices
/// with bi-directional overscan buffer margins, without allocating heap memory or traversing full collections.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VirtualList {
    total_items: usize,
    item_stride: f32,
    overscan: usize,
}

impl VirtualList {
    /// Creates a new virtualized list calculator for the specified item count and stride.
    ///
    /// The `item_stride` represents the physical height of an individual item plus any spacing gap.
    /// Sets a default overscan buffer margin of 2 items above and below the visible viewport.
    #[must_use]
    pub fn new(total_items: usize, item_stride: f32) -> Self {
        Self {
            total_items,
            item_stride: item_stride.max(1.0),
            overscan: 2,
        }
    }

    /// Sets the number of overscan buffer items to render above and below the visible viewport.
    #[must_use]
    pub const fn with_overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }

    /// Returns the number of overscan buffer items rendered beyond the viewport boundaries.
    #[must_use]
    pub const fn overscan(&self) -> usize {
        self.overscan
    }

    /// Returns the total number of items in the underlying collection.
    #[must_use]
    pub const fn total_items(&self) -> usize {
        self.total_items
    }

    /// Returns the vertical stride (height + gap) per item in physical pixels.
    #[must_use]
    pub const fn item_stride(&self) -> f32 {
        self.item_stride
    }

    /// Computes the total content height in physical pixels required by all items.
    #[must_use]
    pub fn total_content_height(&self) -> f32 {
        self.total_items as f32 * self.item_stride
    }

    /// Calculates the maximum vertical scroll offset before reaching the bottom edge.
    #[must_use]
    pub fn max_scroll_y(&self, viewport_height: f32) -> f32 {
        (self.total_content_height() - viewport_height).max(0.0)
    }

    /// Calculates the visible window of item indices intersecting the given viewport with overscan margin.
    ///
    /// Applies overscan buffer rows both above `start_idx` and below `end_idx` to eliminate white flash
    /// and pop-in artifacts during rapid trackpad/mousewheel scrolling.
    #[must_use]
    pub fn compute_slice(&self, viewport_height: f32, scroll_y: f32) -> VirtualSlice {
        if self.total_items == 0 || viewport_height <= 0.0 {
            return VirtualSlice::empty();
        }

        let max_scroll = self.max_scroll_y(viewport_height);
        let safe_scroll = scroll_y.clamp(0.0, max_scroll);

        let raw_start = (safe_scroll / self.item_stride).floor() as usize;
        let start_idx = raw_start.saturating_sub(self.overscan);
        let raw_visible = (viewport_height / self.item_stride).ceil() as usize;
        let end_idx = (raw_start + raw_visible + self.overscan).min(self.total_items);

        VirtualSlice::new(start_idx, end_idx)
    }

    /// Computes the relative sub-pixel vertical scroll offset applied to the container
    /// given the current global scroll offset and the window slice `start_idx`.
    #[must_use]
    pub fn compute_scroll_offset(&self, scroll_y: f32, start_idx: usize) -> f32 {
        let first_item_y = start_idx as f32 * self.item_stride;
        (scroll_y - first_item_y).max(0.0)
    }

    /// Computes the physical screen Y coordinate for the item at the specified index.
    #[must_use]
    pub fn item_y(&self, index: usize, viewport_y: f32, scroll_y: f32) -> f32 {
        viewport_y + (index as f32 * self.item_stride) - scroll_y
    }

    /// Returns whether the item at the specified index intersects the visible viewport.
    #[must_use]
    pub fn is_item_visible(&self, index: usize, viewport_height: f32, scroll_y: f32) -> bool {
        if index >= self.total_items {
            return false;
        }
        let item_top = (index as f32 * self.item_stride) - scroll_y;
        item_top + self.item_stride > 0.0 && item_top < viewport_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_list_empty() {
        let vlist = VirtualList::new(0, 24.0);
        assert_eq!(vlist.total_content_height(), 0.0);
        assert_eq!(vlist.max_scroll_y(200.0), 0.0);

        let slice = vlist.compute_slice(200.0, 0.0);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_virtual_list_slicing() {
        // 100 items with 20px stride = 2000px total content height
        let vlist = VirtualList::new(100, 20.0);
        assert_eq!(vlist.total_content_height(), 2000.0);
        assert_eq!(vlist.max_scroll_y(200.0), 1800.0);

        // At scroll_y = 0: viewport of 200px fits 10 items + 2 bottom overscan = [0..12)
        let slice_0 = vlist.compute_slice(200.0, 0.0);
        assert_eq!(slice_0.start_idx, 0);
        assert_eq!(slice_0.end_idx, 12);
        assert_eq!(slice_0.visible_count, 12);

        // At scroll_y = 100px: raw_start = 5 -> with 2 overscan, start_idx = 3, end_idx = 17
        let slice_100 = vlist.compute_slice(200.0, 100.0);
        assert_eq!(slice_100.start_idx, 3);
        assert_eq!(slice_100.end_idx, 17);
        assert_eq!(slice_100.visible_count, 14);

        // At maximum scroll (1800px): raw_start = 90 -> start_idx = 88, end_idx clamped to 100
        let slice_max = vlist.compute_slice(200.0, 1800.0);
        assert_eq!(slice_max.start_idx, 88);
        assert_eq!(slice_max.end_idx, 100);
        assert_eq!(slice_max.visible_count, 12);

        // Explicit overscan override (e.g. 0 overscan)
        let vlist_no_overscan = vlist.with_overscan(0);
        let slice_exact = vlist_no_overscan.compute_slice(200.0, 100.0);
        assert_eq!(slice_exact.start_idx, 5);
        assert_eq!(slice_exact.end_idx, 15);
    }

    #[test]
    fn test_compute_scroll_offset() {
        let vlist = VirtualList::new(100, 26.0).with_overscan(2);
        // scroll_y = 78.0 (item 3 is at top), overscan = 2 -> start_idx = 1
        // first rendered item is item 1 at 26.0px -> sub-pixel offset = 78.0 - 26.0 = 52.0px
        let offset = vlist.compute_scroll_offset(78.0, 1);
        assert_eq!(offset, 52.0);

        // At scroll_y = 0.0 and start_idx = 0 -> 0.0px
        assert_eq!(vlist.compute_scroll_offset(0.0, 0), 0.0);
    }

    #[test]
    fn test_item_position_and_visibility() {
        let vlist = VirtualList::new(50, 30.0);
        let vp_y = 100.0;
        let scroll_y = 60.0; // scrolled down by 2 items

        // Item 0 is above viewport (top = 100 + 0 - 60 = 40, bottom = 70 < 100)
        assert_eq!(vlist.item_y(0, vp_y, scroll_y), 40.0);
        assert!(!vlist.is_item_visible(0, 200.0, scroll_y));

        // Item 2 is at top of viewport (top = 100 + 60 - 60 = 100)
        assert_eq!(vlist.item_y(2, vp_y, scroll_y), 100.0);
        assert!(vlist.is_item_visible(2, 200.0, scroll_y));

        // Item 5 is inside viewport
        assert!(vlist.is_item_visible(5, 200.0, scroll_y));

        // Beyond total items
        assert!(!vlist.is_item_visible(999, 200.0, scroll_y));
    }
}