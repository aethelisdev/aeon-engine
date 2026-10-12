// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Iris Winit Integration
//!
//! Verifies event translation invariants, cursor roundtrips, scale factor transformations,
//! and modifier state tracking.

use crate::adapter::{
    winit_ime_to_iris, winit_key_to_iris, winit_mouse_button_to_iris, winit_to_ui_event,
};
use crate::cursor::{widget_cursor_to_winit, winit_to_widget_cursor};
use crate::modifiers::ModifiersState;
use crate::state::WinitState;
use iris_core::event::{ImeEvent, KeyCode, MouseButton, UiEvent};
use iris_core::geometry::Point;
use iris_core::node::WidgetCursor;
use winit::dpi::PhysicalPosition;
use winit::event::{
    DeviceId, ElementState, Ime, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent,
};
use winit::keyboard::{Key, ModifiersState as WinitModifiersState, NamedKey, SmolStr};
use winit::window::CursorIcon;

#[test]
fn test_cursor_roundtrip_all_variants() {
    let variants = [
        WidgetCursor::Default,
        WidgetCursor::Pointer,
        WidgetCursor::Text,
        WidgetCursor::Crosshair,
        WidgetCursor::Grab,
        WidgetCursor::Grabbing,
        WidgetCursor::ColResize,
        WidgetCursor::RowResize,
        WidgetCursor::EwResize,
        WidgetCursor::NsResize,
        WidgetCursor::NeswResize,
        WidgetCursor::NwseResize,
        WidgetCursor::NotAllowed,
    ];

    for cur in variants {
        let winit_icon = widget_cursor_to_winit(cur);
        let roundtrip = winit_to_widget_cursor(winit_icon);
        assert_eq!(roundtrip, Some(cur));
    }

    assert_eq!(winit_to_widget_cursor(CursorIcon::Wait), None);
    assert_eq!(winit_to_widget_cursor(CursorIcon::Progress), None);
}

#[test]
fn test_mouse_button_translation() {
    assert_eq!(
        winit_mouse_button_to_iris(WinitMouseButton::Left),
        MouseButton::Left
    );
    assert_eq!(
        winit_mouse_button_to_iris(WinitMouseButton::Right),
        MouseButton::Right
    );
    assert_eq!(
        winit_mouse_button_to_iris(WinitMouseButton::Middle),
        MouseButton::Middle
    );
    assert_eq!(
        winit_mouse_button_to_iris(WinitMouseButton::Other(4)),
        MouseButton::Other(4)
    );
}

#[test]
fn test_mouse_input_and_move_events() {
    let p = Point::new(120.0, 85.0);

    let move_ev = WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(120.0, 85.0),
    };
    assert_eq!(
        winit_to_ui_event(&move_ev, p),
        Some(UiEvent::MouseMove { point: p })
    );

    let down_ev = WindowEvent::MouseInput {
        device_id: DeviceId::dummy(),
        state: ElementState::Pressed,
        button: WinitMouseButton::Left,
    };
    assert_eq!(
        winit_to_ui_event(&down_ev, p),
        Some(UiEvent::MouseDown {
            button: MouseButton::Left,
            point: p
        })
    );

    let up_ev = WindowEvent::MouseInput {
        device_id: DeviceId::dummy(),
        state: ElementState::Released,
        button: WinitMouseButton::Right,
    };
    assert_eq!(
        winit_to_ui_event(&up_ev, p),
        Some(UiEvent::MouseUp {
            button: MouseButton::Right,
            point: p
        })
    );
}

#[test]
fn test_mouse_scroll_translation() {
    let p = Point::new(42.0, 99.0);

    let line_scroll = WindowEvent::MouseWheel {
        device_id: DeviceId::dummy(),
        delta: MouseScrollDelta::LineDelta(2.0, -3.0),
        phase: winit::event::TouchPhase::Moved,
    };
    assert_eq!(
        winit_to_ui_event(&line_scroll, p),
        Some(UiEvent::MouseScroll {
            delta_x: 2.0,
            delta_y: -3.0,
            point: p,
        })
    );

    let pixel_scroll = WindowEvent::MouseWheel {
        device_id: DeviceId::dummy(),
        delta: MouseScrollDelta::PixelDelta(PhysicalPosition::new(48.0, -72.0)),
        phase: winit::event::TouchPhase::Moved,
    };
    assert_eq!(
        winit_to_ui_event(&pixel_scroll, p),
        Some(UiEvent::MouseScroll {
            delta_x: 2.0,
            delta_y: -3.0,
            point: p,
        })
    );
}

#[test]
fn test_keyboard_key_translations() {
    let (key_enter, _) = winit_key_to_iris(&Key::Named(NamedKey::Enter));
    assert_eq!(key_enter, KeyCode::Enter);

    let (key_esc, _) = winit_key_to_iris(&Key::Named(NamedKey::Escape));
    assert_eq!(key_esc, KeyCode::Escape);

    let (key_arrow_l, _) = winit_key_to_iris(&Key::Named(NamedKey::ArrowLeft));
    assert_eq!(key_arrow_l, KeyCode::ArrowLeft);

    let (key_arrow_r, _) = winit_key_to_iris(&Key::Named(NamedKey::ArrowRight));
    assert_eq!(key_arrow_r, KeyCode::ArrowRight);

    let (key_arrow_u, _) = winit_key_to_iris(&Key::Named(NamedKey::ArrowUp));
    assert_eq!(key_arrow_u, KeyCode::ArrowUp);

    let (key_arrow_d, _) = winit_key_to_iris(&Key::Named(NamedKey::ArrowDown));
    assert_eq!(key_arrow_d, KeyCode::ArrowDown);

    let (key_home, _) = winit_key_to_iris(&Key::Named(NamedKey::Home));
    assert_eq!(key_home, KeyCode::Home);

    let (key_end, _) = winit_key_to_iris(&Key::Named(NamedKey::End));
    assert_eq!(key_end, KeyCode::End);

    let (key_back, _) = winit_key_to_iris(&Key::Named(NamedKey::Backspace));
    assert_eq!(key_back, KeyCode::Backspace);

    let (key_del, _) = winit_key_to_iris(&Key::Named(NamedKey::Delete));
    assert_eq!(key_del, KeyCode::Delete);

    let (key_tab, _) = winit_key_to_iris(&Key::Named(NamedKey::Tab));
    assert_eq!(key_tab, KeyCode::Tab);

    let (key_space, _) = winit_key_to_iris(&Key::Named(NamedKey::Space));
    assert_eq!(key_space, KeyCode::Space);

    let (key_char, text) = winit_key_to_iris(&Key::Character(SmolStr::new("x")));
    assert_eq!(key_char, KeyCode::Character('x'));
    assert_eq!(text, Some("x".to_string()));

    let (key_unicode, unicode_text) = winit_key_to_iris(&Key::Character(SmolStr::new("ç")));
    assert_eq!(key_unicode, KeyCode::Character('ç'));
    assert_eq!(unicode_text, Some("ç".to_string()));
}

#[test]
fn test_ime_events() {
    assert_eq!(winit_ime_to_iris(&Ime::Enabled), ImeEvent::Enabled);
    assert_eq!(
        winit_ime_to_iris(&Ime::Preedit("composer".to_string(), Some((0, 4)))),
        ImeEvent::Preedit("composer".to_string(), Some((0, 4)))
    );
    assert_eq!(
        winit_ime_to_iris(&Ime::Commit("result".to_string())),
        ImeEvent::Commit("result".to_string())
    );
    assert_eq!(winit_ime_to_iris(&Ime::Disabled), ImeEvent::Disabled);

    let ime_ev = WindowEvent::Ime(Ime::Commit("commit_str".to_string()));
    assert_eq!(
        winit_to_ui_event(&ime_ev, Point::new(0.0, 0.0)),
        Some(UiEvent::Ime(ImeEvent::Commit("commit_str".to_string())))
    );
}

#[test]
fn test_modifiers_state() {
    let mut mods = ModifiersState::new();
    assert!(mods.is_empty());
    assert!(!mods.any());

    mods.shift = true;
    assert!(mods.shift());
    assert!(mods.any());
    assert!(!mods.is_empty());

    let winit_mods = WinitModifiersState::CONTROL | WinitModifiersState::ALT;
    let converted = ModifiersState::from_winit_state(winit_mods);
    assert!(converted.ctrl());
    assert!(converted.alt());
    assert!(!converted.shift());
    assert!(!converted.super_key());
}

#[test]
fn test_winit_state_dpi_and_cursor_lifecycle() {
    let mut state = WinitState::new(2.0);
    assert_eq!(state.scale_factor(), 2.0);
    assert_eq!(state.scale_factor_f64(), 2.0);
    assert_eq!(state.cursor_pos(), None);

    let move_ev = WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(200.0, 100.0),
    };
    let event = state.on_window_event(&move_ev);
    assert_eq!(state.cursor_pos(), Some(Point::new(100.0, 50.0)));
    assert_eq!(
        event,
        Some(UiEvent::MouseMove {
            point: Point::new(100.0, 50.0)
        })
    );

    let leave_ev = WindowEvent::CursorLeft {
        device_id: DeviceId::dummy(),
    };
    state.on_window_event(&leave_ev);
    assert_eq!(state.cursor_pos(), None);

    let focus_ev = WindowEvent::Focused(false);
    state.on_window_event(&focus_ev);
    assert!(!state.is_focused());

    // Scale factor update
    state.set_scale_factor(1.5);
    assert_eq!(state.scale_factor(), 1.5);

    // Guard against negative / zero scale factor
    state.set_scale_factor(0.0);
    assert_eq!(state.scale_factor(), 1.0);
}