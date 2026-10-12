// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Interactive Control Primitives (`controls`)
//!
//! Subsystem providing autonomous pointer capture, numeric scrubbers, and focus management
//! for rich interactive controls.
//!

pub mod focus;
pub mod numeric_drag;

pub use focus::{
    FocusEventResponse, FocusModifiers, FocusPolicy, FocusState, FocusTarget, FocusableArea,
    handle_focus_event, handle_focus_event_with_modifiers,
};
pub use numeric_drag::{
    NUMERIC_DRAG_COARSE_MULTIPLIER, NUMERIC_DRAG_FINE_MULTIPLIER, NUMERIC_DRAG_THRESHOLD,
    NumericDragResponse, NumericDragState, handle_numeric_drag_event,
};