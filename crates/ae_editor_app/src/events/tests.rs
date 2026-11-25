// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for InputArbiter and PointerCapture state transitions.

use super::arbiter::{InputArbiter, PointerCapture};
use ae_core::modules::EngineMode;
use ae_input::{ElementState, KeyCode};
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

#[test]
fn test_mode_transition_for_key_ignores_repeats_and_edit_escape() {
    assert_eq!(
        InputArbiter::mode_transition_for_key(
            EngineMode::Edit,
            KeyCode::F5,
            ElementState::Pressed,
            false
        ),
        Some(EngineMode::Play)
    );
    assert_eq!(
        InputArbiter::mode_transition_for_key(
            EngineMode::Play,
            KeyCode::F5,
            ElementState::Pressed,
            false
        ),
        Some(EngineMode::Edit)
    );
    assert_eq!(
        InputArbiter::mode_transition_for_key(
            EngineMode::Play,
            KeyCode::Escape,
            ElementState::Pressed,
            false
        ),
        Some(EngineMode::Edit)
    );
    // Negative cases: key repeats, releases, and Escape while already in Edit mode.
    assert_eq!(
        InputArbiter::mode_transition_for_key(
            EngineMode::Edit,
            KeyCode::F5,
            ElementState::Pressed,
            true
        ),
        None
    );
    assert_eq!(
        InputArbiter::mode_transition_for_key(
            EngineMode::Edit,
            KeyCode::F5,
            ElementState::Released,
            false
        ),
        None
    );
    assert_eq!(
        InputArbiter::mode_transition_for_key(
            EngineMode::Edit,
            KeyCode::Escape,
            ElementState::Pressed,
            false
        ),
        None
    );
}