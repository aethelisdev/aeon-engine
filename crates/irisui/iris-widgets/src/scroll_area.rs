// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scroll Area & Virtualized Collection Widgets (`iris-widgets::scroll_area`)
//!
//! Exposes hardware-clipped scroll containers, proportional scrollbar indicators,
//! and $O(1)$ zero-allocation virtualized list windowing.
//!

pub mod area;
pub mod scroll_bar;
pub mod style;
pub mod types;
pub mod virtual_list;

pub use area::{ScrollAreaBuilder, ScrollAreaFrame};
pub use scroll_bar::ScrollBarGeometry;
pub use style::ScrollAreaStyle;
pub use types::{
    ScrollBarHit, ScrollBarVisibility, ScrollDirection, VirtualItemHeight, VirtualScrollConfig,
    VirtualSlice,
};
pub use virtual_list::VirtualList;