// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dirty state tracking flags for Retained-Mode caching in Iris UI.
//!
//! Dirty flags allow the UI engine to skip unchanged nodes during layout resolution,
//! geometry generation, and GPU buffer uploads, maintaining near-zero CPU overhead on idle frames.

use bitflags::bitflags;

bitflags! {
    /// Bitflags representing fine-grained dirty states of a widget node.
    /// When properties change on a widget, only the relevant dirty flags are raised,
    /// enabling the layout and render engines to selectively update cached computations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct DirtyFlags: u32 {
        /// Indicates that the widget's layout (dimensions, padding, flexbox constraints) needs recalculation.
        const LAYOUT = 1 << 0;
        /// Indicates that the widget's visual appearance (color, border, shadow) changed and requires redraw.
        const PAINT = 1 << 1;
        /// Indicates that the styling rules or theme overrides have changed.
        const STYLE = 1 << 2;
        /// Indicates that child hierarchy has been mutated (children added, removed, or reordered).
        const CHILDREN = 1 << 3;
        /// Indicates that transformation matrices or absolute positions have changed.
        const TRANSFORM = 1 << 4;
        /// Indicates that text content, glyph layout, or font parameters require reshaping.
        const TEXT = 1 << 5;
        /// All dirty flags combined, indicating a full invalidation of the widget state.
        const ALL = Self::LAYOUT.bits() | Self::PAINT.bits() | Self::STYLE.bits() | Self::CHILDREN.bits() | Self::TRANSFORM.bits() | Self::TEXT.bits();
    }
}

impl Default for DirtyFlags {
    #[inline]
    fn default() -> Self {
        Self::ALL
    }
}