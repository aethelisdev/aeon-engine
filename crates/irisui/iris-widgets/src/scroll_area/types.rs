// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # ScrollArea and VirtualList Core Types (`iris-widgets::scroll_area::types`)
//!
//! Provides orientation modes, visibility policies, virtualized item slice descriptors,
//! and hit-testing states for scroll containers in Iris UI.
//!

/// Supported scrolling directions for scroll areas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScrollDirection {
    /// Vertical scrolling along the Y axis only.
    #[default]
    Vertical,
    /// Horizontal scrolling along the X axis only.
    Horizontal,
    /// Simultaneous two-dimensional scrolling along both X and Y axes.
    Both,
}

/// Visibility policy for scrollbar track and thumb indicators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScrollBarVisibility {
    /// Render the scrollbar only when total content size exceeds viewport bounds.
    #[default]
    Auto,
    /// Keep the scrollbar permanently visible even if content fits within the viewport.
    Always,
    /// Completely suppress the scrollbar from rendering.
    Hidden,
}

/// Description of an active virtualized window of visible items within a collection.
///
/// Computed by virtualized layout routines to prune non-visible elements from the UI tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct VirtualSlice {
    /// Zero-based index of the first item that intersects the visible viewport.
    pub start_idx: usize,
    /// Zero-based exclusive ending index of items intersecting the visible viewport.
    pub end_idx: usize,
    /// Number of items within the visible slice (`end_idx - start_idx`).
    pub visible_count: usize,
}

impl VirtualSlice {
    /// Creates an empty slice with zero visible items.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            start_idx: 0,
            end_idx: 0,
            visible_count: 0,
        }
    }

    /// Creates a slice covering the range `[start_idx..end_idx)`.
    #[must_use]
    pub fn new(start_idx: usize, end_idx: usize) -> Self {
        let safe_end = end_idx.max(start_idx);
        Self {
            start_idx,
            end_idx: safe_end,
            visible_count: safe_end - start_idx,
        }
    }

    /// Returns whether this slice contains zero items.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.visible_count == 0
    }
}

/// Hit-testing targets on a scrollbar during cursor interaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScrollBarHit {
    /// Cursor is not over any scrollbar element.
    #[default]
    None,
    /// Cursor is positioned over the stationary scrollbar track background.
    Track,
    /// Cursor is positioned over the draggable scrollbar thumb indicator.
    Thumb,
}

/// Sizing strategy for elements inside a virtualized scroll container.
///
/// Supports fixed-height uniform elements as well as estimated heights prepared
/// for dynamic/variable-height content such as multi-line log entries and stack traces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VirtualItemHeight {
    /// Uniform static row height across all items in physical pixels.
    Fixed(f32),
    /// Estimated row height for variable-height items (prepared for future multi-line stack traces).
    Estimated(f32),
}

impl Default for VirtualItemHeight {
    fn default() -> Self {
        Self::Fixed(26.0)
    }
}

use iris_core::color::Color;

/// Sizing and overscan configuration descriptor for virtualized scroll viewports.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VirtualScrollConfig {
    /// Total count of elements in the collection.
    pub total_items: usize,
    /// Sizing mode (fixed uniform height or estimated for dynamic items).
    pub item_height: VirtualItemHeight,
    /// Number of buffer rows rendered above and below the visible viewport boundaries.
    pub overscan: usize,
    /// Background color applied to the scroll viewport container.
    pub bg: Color,
    /// Optional vertical scroll position override. If `None`, uses persistent scroll state from `UiTree`.
    pub scroll_y: Option<f32>,
    /// Optional explicit viewport height override in physical pixels. If `None`, autonomously resolves height.
    pub viewport_height: Option<f32>,
    /// Optional semantic tags `(track_tag, thumb_tag)` enabling autonomous scrollbar rendering.
    pub scrollbar_tags: Option<(u64, u64)>,
}

impl VirtualScrollConfig {
    /// Creates a virtual scroll configuration with fixed uniform item height.
    ///
    /// Defaults to an overscan buffer of 2 rows above and below the visible window
    /// and a transparent viewport background.
    #[must_use]
    pub const fn fixed(total_items: usize, row_height: f32) -> Self {
        Self {
            total_items,
            item_height: VirtualItemHeight::Fixed(row_height),
            overscan: 2,
            bg: Color::TRANSPARENT,
            scroll_y: None,
            viewport_height: None,
            scrollbar_tags: None,
        }
    }

    /// Creates a virtual scroll configuration with estimated height prepared for dynamic multi-line traces.
    ///
    /// Defaults to an overscan buffer of 2 rows above and below the visible window
    /// and a transparent viewport background.
    #[must_use]
    pub const fn estimated(total_items: usize, estimated_row_height: f32) -> Self {
        Self {
            total_items,
            item_height: VirtualItemHeight::Estimated(estimated_row_height),
            overscan: 2,
            bg: Color::TRANSPARENT,
            scroll_y: None,
            viewport_height: None,
            scrollbar_tags: None,
        }
    }

    /// Configures an explicit overscan buffer margin count.
    #[must_use]
    pub const fn with_overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }

    /// Sets a custom background color for the scroll viewport container.
    #[must_use]
    pub const fn with_bg(mut self, bg: Color) -> Self {
        self.bg = bg;
        self
    }

    /// Configures an explicit vertical scroll position override.
    #[must_use]
    pub const fn with_scroll_y(mut self, scroll_y: f32) -> Self {
        self.scroll_y = Some(scroll_y);
        self
    }

    /// Configures an explicit visible viewport height in physical pixels, bypassing autonomous resolution.
    #[must_use]
    pub const fn with_viewport_height(mut self, height: f32) -> Self {
        self.viewport_height = Some(height);
        self
    }

    /// Configures semantic tags for autonomous scrollbar indicator rendering.
    #[must_use]
    pub const fn with_scrollbar_tags(mut self, track: u64, thumb: u64) -> Self {
        self.scrollbar_tags = Some((track, thumb));
        self
    }

    /// Returns the effective item stride height in physical pixels.
    #[must_use]
    pub const fn stride(&self) -> f32 {
        match self.item_height {
            VirtualItemHeight::Fixed(h) | VirtualItemHeight::Estimated(h) => h,
        }
    }
}

/// Comprehensive result returned by [`crate::declarative::UiScope::virtual_scroll_area`].
///
/// Encapsulates maximum scroll limit, active clamped scroll offset,
/// resolved visible index range, and allocated container identifier.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VirtualScrollResponse {
    /// Maximum vertical scroll limit before reaching content bottom edge.
    pub max_scroll_y: f32,
    /// Clamped vertical scroll offset applied to the container node.
    pub current_scroll_y: f32,
    /// Effective visible viewport height in physical pixels.
    pub viewport_height: f32,
    /// Total virtual height of all collection items.
    pub content_height: f32,
    /// Zero-based index of the first rendered item.
    pub visible_start: usize,
    /// Zero-based non-inclusive index of the last rendered item.
    pub visible_end: usize,
    /// Allocated widget node ID for the viewport container.
    pub container_id: iris_core::WidgetId,
}

impl VirtualScrollResponse {
    /// Returns the maximum vertical scroll limit in physical pixels.
    #[inline]
    #[must_use]
    pub fn max_scroll(&self) -> f32 {
        self.max_scroll_y
    }

    /// Returns `true` if the viewport is scrolled to the bottom edge.
    #[inline]
    #[must_use]
    pub fn is_at_bottom(&self) -> bool {
        (self.current_scroll_y - self.max_scroll_y).abs() < 1.0
    }
}

impl From<VirtualScrollResponse> for f32 {
    #[inline]
    fn from(resp: VirtualScrollResponse) -> Self {
        resp.max_scroll_y
    }
}

/// Response returned by an autonomous declarative [`scroll_area`] call.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollAreaResponse<R = ()> {
    /// Inner value returned by the scroll area child closure.
    pub inner: R,
    /// Allocated widget node ID for the viewport container.
    pub container_id: iris_core::WidgetId,
    /// Maximum vertical scroll limit in physical pixels (`(content_height - viewport_height).max(0.0)`).
    pub max_scroll_y: f32,
    /// Total measured or estimated vertical content height in physical pixels.
    pub content_height: f32,
    /// Autonomously resolved visible viewport height in physical pixels.
    pub viewport_height: f32,
}

impl<R> ScrollAreaResponse<R> {
    /// Returns the maximum vertical scroll limit in physical pixels.
    #[inline]
    #[must_use]
    pub fn max_scroll(&self) -> f32 {
        self.max_scroll_y
    }
}

impl<R> std::ops::Deref for ScrollAreaResponse<R> {
    type Target = R;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<R> From<ScrollAreaResponse<R>> for f32 {
    #[inline]
    fn from(resp: ScrollAreaResponse<R>) -> Self {
        resp.max_scroll_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_slice_construction() {
        let slice = VirtualSlice::new(5, 12);
        assert_eq!(slice.start_idx, 5);
        assert_eq!(slice.end_idx, 12);
        assert_eq!(slice.visible_count, 7);
        assert!(!slice.is_empty());

        let empty = VirtualSlice::empty();
        assert_eq!(empty.visible_count, 0);
        assert!(empty.is_empty());

        let inverted = VirtualSlice::new(10, 5);
        assert_eq!(inverted.start_idx, 10);
        assert_eq!(inverted.end_idx, 10);
        assert_eq!(inverted.visible_count, 0);
        assert!(inverted.is_empty());
    }

    #[test]
    fn test_enum_defaults() {
        assert_eq!(ScrollDirection::default(), ScrollDirection::Vertical);
        assert_eq!(ScrollBarVisibility::default(), ScrollBarVisibility::Auto);
        assert_eq!(ScrollBarHit::default(), ScrollBarHit::None);
    }
}