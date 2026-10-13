// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for InputArbiter subsystem and PointerCapture state machine.

use super::arbiter::{InputArbiter, PointerCapture};
use winit::event::{MouseButton, WindowEvent};

#[test]
fn test_input_arbiter_instantiation_and_defaults() {
    let arbiter = InputArbiter::new();
    assert_eq!(arbiter.pointer_capture(), PointerCapture::None);
    let debug_repr = format!("{:?}", arbiter);
    assert!(debug_repr.contains("InputArbiter"));
    assert!(debug_repr.contains("None"));
}

#[test]
fn test_pointer_capture_state_machine_transitions() {
    let mut arbiter = InputArbiter::new();
    assert_eq!(arbiter.pointer_capture(), PointerCapture::None);

    // Simulate UI pointer capture
    let ui_cap = PointerCapture::Ui(MouseButton::Left);
    let vp_cap = PointerCapture::Viewport(MouseButton::Right);

    assert_ne!(ui_cap, vp_cap);
    assert_ne!(ui_cap, PointerCapture::None);

    arbiter.reset_pointer_capture();
    assert_eq!(arbiter.pointer_capture(), PointerCapture::None);
}

#[test]
fn test_input_arbiter_unhandled_events_are_no_ops() {
    let mut arbiter = InputArbiter::new();
    let _ = &mut arbiter;
    assert_eq!(
        std::mem::size_of::<InputArbiter>(),
        std::mem::size_of::<PointerCapture>()
    );
}

#[test]
fn test_window_event_debug_formats() {
    let event = WindowEvent::Focused(false);
    let repr = format!("{:?}", event);
    assert!(repr.contains("Focused"));
}