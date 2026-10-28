// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Frameless Window Subsystem
//!
//! Provides boundary geometry evaluation, edge tolerance calculation, and
//! hardware-independent window resizing helpers for custom decorated or frameless windows.

pub mod geometry;

#[cfg(test)]
mod tests;

pub use geometry::{
    DEFAULT_BORDER_THICKNESS, DEFAULT_CORNER_SIZE, ResizeDirection, evaluate_resize_direction,
};