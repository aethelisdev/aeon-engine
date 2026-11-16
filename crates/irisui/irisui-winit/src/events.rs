// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Platform Event Normalization Subsystem
//!
//! Provides hardware-independent event normalization, HighDPI coordinate transformation,
//! and high-precision mouse wheel / touchpad translation.

pub mod scroll;

#[cfg(test)]
mod scroll_tests;

pub use scroll::{
    DEFAULT_SCROLL_LINE_STEP, MIN_SAFE_SCALE_FACTOR, normalize_scroll_delta,
    normalize_scroll_delta_with_line_step,
};