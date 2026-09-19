// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Floating window subsystem enabling detachable, free-floating panel surfaces.
//!
//! Organizes independent floating window hierarchies, edge resize detection,
//! boundary clamping constraints, and native UI widget tree construction.
//!

pub mod bounds;
pub mod cursor;
pub mod layer;
pub mod model;

pub use bounds::{FloatingWindowClampBounds, clamp_floating_windows};
pub use cursor::{
    DEFAULT_RESIZE_MARGIN, FloatingResizeEdge, FloatingWindowCursor, detect_resize_edge,
    evaluate_floating_resize_cursor,
};
pub use layer::build_floating_windows_layer;
pub use model::{
    FloatingDragState, FloatingWindow, FloatingWindowClickAction, FloatingWindowStyle,
    evaluate_floating_window_click, find_active_tab_content_rect,
};