// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Frameless Window Subsystem
//!
//! Provides boundary geometry evaluation, edge tolerance calculation, and
//! hardware-independent window resizing helpers for custom decorated or frameless windows.

pub mod drag;
pub mod geometry;
pub mod host;
pub mod window_trait;

#[cfg(test)]
mod drag_tests;
#[cfg(test)]
mod geometry_tests;
#[cfg(test)]
mod host_tests;

pub use drag::{
    DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD, DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD, WindowDragHost,
    WindowDragOutcome, is_double_click, point_distance,
};
pub use geometry::{
    DEFAULT_BORDER_THICKNESS, DEFAULT_CORNER_SIZE, ResizeDirection, evaluate_resize_direction,
};
pub use host::FramelessWindowHost;
pub use window_trait::{WindowInterface, WindowOpError};