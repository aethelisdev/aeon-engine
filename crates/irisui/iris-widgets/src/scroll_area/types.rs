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

    /// Returns the effective item stride height in physical pixels.
    #[must_use]
    pub const fn stride(&self) -> f32 {
        match self.item_height {
            VirtualItemHeight::Fixed(h) | VirtualItemHeight::Estimated(h) => h,
        }
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