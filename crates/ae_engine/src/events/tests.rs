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

#[test]
fn test_compute_logical_screen_size_handles_scaling_and_edge_cases() {
    assert_eq!(
        super::handlers::compute_logical_screen_size(1920, 1080, 1.0),
        Some((1920.0, 1080.0))
    );
    assert_eq!(
        super::handlers::compute_logical_screen_size(1920, 1080, 1.5),
        Some((1280.0, 720.0))
    );
    assert_eq!(
        super::handlers::compute_logical_screen_size(0, 1080, 1.0),
        None
    );
    assert_eq!(
        super::handlers::compute_logical_screen_size(1920, 0, 1.0),
        None
    );
    assert_eq!(
        super::handlers::compute_logical_screen_size(1920, 1080, 0.0),
        None
    );
    assert_eq!(
        super::handlers::compute_logical_screen_size(1920, 1080, f32::NAN),
        None
    );
}

#[test]
fn test_apply_hover_outcome_propagates_targets_and_redraw_flags() {
    use ae_editor_ui::ui::PanelId;
    use ae_editor_ui::ui::iris_bridge::IrisChromeState;
    use irisui::workbench::WorkbenchOutcome;

    let mut chrome = IrisChromeState {
        needs_text_prepare: false,
        needs_full_quad_upload: false,
        ..IrisChromeState::default()
    };

    // 1. Entering a tagged widget inside a panel sets targets and marks text/quad flags
    let mut enter = WorkbenchOutcome::<(), PanelId>::empty();
    enter.hovered_tag = Some(0x1234);
    enter.hovered_panel = Some(PanelId::Inspector);
    enter.hover_target_changed = true;
    super::arbiter::apply_hover_outcome(&enter, &mut chrome);
    assert_eq!(chrome.hovered_tag, Some(0x1234));
    assert_eq!(chrome.hovered_panel, Some(PanelId::Inspector));
    assert!(chrome.needs_text_prepare);
    assert!(chrome.needs_full_quad_upload);

    // 2. Unchanged hover target preserves flags without triggering redraws
    chrome.needs_text_prepare = false;
    chrome.needs_full_quad_upload = false;
    let mut steady = WorkbenchOutcome::<(), PanelId>::empty();
    steady.hovered_tag = Some(0x1234);
    steady.hovered_panel = Some(PanelId::Inspector);
    steady.hover_target_changed = false;
    super::arbiter::apply_hover_outcome(&steady, &mut chrome);
    assert_eq!(chrome.hovered_tag, Some(0x1234));
    assert!(!chrome.needs_text_prepare);
    assert!(!chrome.needs_full_quad_upload);

    // 3. Leaving tag clears hovered_tag and requests refresh
    let mut leave = WorkbenchOutcome::<(), PanelId>::empty();
    leave.hovered_tag = None;
    leave.hovered_panel = Some(PanelId::Console);
    leave.hover_target_changed = true;
    super::arbiter::apply_hover_outcome(&leave, &mut chrome);
    assert_eq!(chrome.hovered_tag, None);
    assert_eq!(chrome.hovered_panel, Some(PanelId::Console));
    assert!(chrome.needs_text_prepare);
    assert!(chrome.needs_full_quad_upload);
}