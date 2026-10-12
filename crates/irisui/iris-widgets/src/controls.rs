// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Interactive Control Primitives (`controls`)
//!
//! Subsystem providing autonomous pointer capture, numeric scrubbers, and focus management
//! for rich interactive controls.
//!

pub mod numeric_drag;

pub use numeric_drag::{
    NUMERIC_DRAG_COARSE_MULTIPLIER, NUMERIC_DRAG_FINE_MULTIPLIER, NUMERIC_DRAG_THRESHOLD,
    NumericDragResponse, NumericDragState, handle_numeric_drag_event,
};