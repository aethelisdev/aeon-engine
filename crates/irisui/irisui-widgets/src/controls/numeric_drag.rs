// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Numeric Scrubber Dragging & Modifier Control
//!
//! Provides autonomous horizontal scrubbing state, speed multipliers, modifier sensitivity
//! adjustments (Shift for fine-tuning, Ctrl for coarse increments), non-linear drag response
//! curves, and pointer capture management for numeric input fields.

pub mod curve;
pub mod state;

pub use curve::*;
pub use state::*;