// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Multi-line Text Subsystem (`controls::multiline`)
//!
//! Autonomous multi-line text editing infrastructure, cursor topology, line start cache,
//! and vertical navigation primitives for code editors, logs, and multi-line inputs.

pub mod navigation;
pub mod state;

pub use navigation::{
    DEFAULT_GUTTER_PADDING, GutterMetrics, GutterStyle, MIN_GUTTER_DIGITS, MultilineNavOutcome,
};
pub use state::MultilineTextState;