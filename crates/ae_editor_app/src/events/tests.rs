// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for InputArbiter and PointerCapture state transitions.

use super::arbiter::{InputArbiter, PointerCapture};
use winit::event::MouseButton;

#[test]
fn test_input_arbiter_defaults_and_pointer_capture_reset() {
    let mut arbiter = InputArbiter::new();
    assert_eq!(arbiter.pointer_capture(), PointerCapture::None);

    let ui_cap = PointerCapture::Ui(MouseButton::Left);
    let vp_cap = PointerCapture::Viewport(MouseButton::Right);
    assert_ne!(ui_cap, vp_cap);
    assert_ne!(ui_cap, PointerCapture::None);

    arbiter.reset_pointer_capture();
    assert_eq!(arbiter.pointer_capture(), PointerCapture::None);
}