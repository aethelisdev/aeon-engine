// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scroll Area & Virtualized Collection Widgets (`iris-widgets::scroll_area`)
//!
//! Exposes hardware-clipped scroll containers, proportional scrollbar indicators,
//! and $O(1)$ zero-allocation virtualized list windowing.

pub mod area;
pub mod events;
pub mod scroll_bar;
pub mod style;
pub mod types;
pub mod virtual_list;
pub mod wheel;

pub use area::{ScrollArea, ScrollAreaBuilder, ScrollAreaFrame};
pub use events::{
    ScrollAreaState, ScrollResponse, ScrollbarDragState, handle_scroll_area_event,
    handle_scrollbar_event,
};
pub use scroll_bar::ScrollBarGeometry;
pub use style::ScrollAreaStyle;
pub use types::{
    ScrollAreaResponse, ScrollBarHit, ScrollBarVisibility, ScrollDirection, VirtualItemHeight,
    VirtualScrollConfig, VirtualScrollResponse, VirtualSlice,
};
pub use virtual_list::VirtualList;
pub use wheel::{
    DEFAULT_SCROLL_STEP, WheelScrollConfig, WheelScrollResponse, evaluate_wheel_scroll,
};