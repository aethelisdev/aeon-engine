// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Winit Event to Iris UI Event Adapter
//!
//! Provides stateless translation from platform window events ([`winit::event::WindowEvent`])
//! into hardware-independent [`iris_core::event::UiEvent`] primitives.
//!

use iris_core::event::{
    ImeEvent as IrisImeEvent, KeyCode as IrisKeyCode, MouseButton as IrisMouseButton, UiEvent,
};
use iris_core::geometry::Point;
use winit::event::{
    ElementState, Ime as WinitIme, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent,
};
use winit::keyboard::{Key, NamedKey};

/// Translates a native Winit [`WindowEvent`] into an Iris UI [`UiEvent`].
///
/// Accepts the current cursor position in logical UI points to populate coordinate fields
/// on mouse gestures, presses, and wheel motions.
///
/// Returns `Some(UiEvent)` if the platform event corresponds to an active UI gesture
/// (mouse motion, mouse button, mouse wheel, keyboard input, or IME composition).
///
/// # Arguments
///
/// * `event` - Reference to the active window event from the platform event loop.
/// * `cursor_pos` - Current mouse cursor position in logical screen points.
#[must_use]
pub fn winit_to_ui_event(event: &WindowEvent, cursor_pos: Point) -> Option<UiEvent> {
    match event {
        WindowEvent::CursorMoved { .. } => Some(UiEvent::MouseMove { point: cursor_pos }),
        WindowEvent::MouseInput { state, button, .. } => {
            let btn = winit_mouse_button_to_iris(*button);
            match state {
                ElementState::Pressed => Some(UiEvent::MouseDown {
                    button: btn,
                    point: cursor_pos,
                }),
                ElementState::Released => Some(UiEvent::MouseUp {
                    button: btn,
                    point: cursor_pos,
                }),
            }
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let (dx, dy) = match delta {
                MouseScrollDelta::LineDelta(x, y) => (*x * 28.0, *y * 28.0),
                MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
            };
            Some(UiEvent::MouseScroll {
                delta_x: dx,
                delta_y: dy,
                point: cursor_pos,
            })
        }
        WindowEvent::KeyboardInput { event: key_ev, .. } => {
            let is_pressed = key_ev.state == ElementState::Pressed;
            let (key, mapped_text) = winit_key_to_iris(&key_ev.logical_key);
            let text = if is_pressed {
                if matches!(
                    key,
                    IrisKeyCode::Enter
                        | IrisKeyCode::Escape
                        | IrisKeyCode::Backspace
                        | IrisKeyCode::Delete
                        | IrisKeyCode::Tab
                ) {
                    None
                } else {
                    key_ev.text.as_ref().map(|t| t.to_string()).or(mapped_text)
                }
            } else {
                None
            };
            Some(UiEvent::KeyboardInput {
                key,
                text,
                is_pressed,
            })
        }
        WindowEvent::Ime(ime) => Some(UiEvent::Ime(winit_ime_to_iris(ime))),
        _ => None,
    }
}

/// Converts a Winit mouse button into an Iris UI [`IrisMouseButton`].
#[must_use]
#[inline]
pub fn winit_mouse_button_to_iris(button: WinitMouseButton) -> IrisMouseButton {
    match button {
        WinitMouseButton::Left => IrisMouseButton::Left,
        WinitMouseButton::Right => IrisMouseButton::Right,
        WinitMouseButton::Middle => IrisMouseButton::Middle,
        WinitMouseButton::Other(b) => IrisMouseButton::Other(b),
        _ => IrisMouseButton::Left,
    }
}

/// Converts a Winit [`Key`] into an Iris UI [`IrisKeyCode`] and an optional text character representation.
#[must_use]
pub fn winit_key_to_iris(key: &Key) -> (IrisKeyCode, Option<String>) {
    match key {
        Key::Named(named) => {
            let (code, text) = match named {
                NamedKey::Backspace => (IrisKeyCode::Backspace, None),
                NamedKey::Delete => (IrisKeyCode::Delete, None),
                NamedKey::Enter => (IrisKeyCode::Enter, None),
                NamedKey::Escape => (IrisKeyCode::Escape, None),
                NamedKey::ArrowLeft => (IrisKeyCode::ArrowLeft, None),
                NamedKey::ArrowRight => (IrisKeyCode::ArrowRight, None),
                NamedKey::ArrowUp => (IrisKeyCode::ArrowUp, None),
                NamedKey::ArrowDown => (IrisKeyCode::ArrowDown, None),
                NamedKey::Home => (IrisKeyCode::Home, None),
                NamedKey::End => (IrisKeyCode::End, None),
                NamedKey::Tab => (IrisKeyCode::Tab, None),
                NamedKey::Space => (IrisKeyCode::Space, Some(" ".to_string())),
                _ => (IrisKeyCode::Other, None),
            };
            (code, text)
        }
        Key::Character(s) => {
            let mut chars = s.chars();
            if let Some(c) = chars.next() {
                if chars.next().is_none() {
                    (IrisKeyCode::Character(c), Some(s.to_string()))
                } else {
                    (IrisKeyCode::Other, Some(s.to_string()))
                }
            } else {
                (IrisKeyCode::Other, None)
            }
        }
        _ => (IrisKeyCode::Other, None),
    }
}

/// Converts a Winit [`WinitIme`] composition/commit event into an Iris UI [`IrisImeEvent`].
#[must_use]
pub fn winit_ime_to_iris(ime: &WinitIme) -> IrisImeEvent {
    match ime {
        WinitIme::Enabled => IrisImeEvent::Enabled,
        WinitIme::Preedit(str, cursor) => IrisImeEvent::Preedit(str.clone(), *cursor),
        WinitIme::Commit(str) => IrisImeEvent::Commit(str.clone()),
        WinitIme::Disabled => IrisImeEvent::Disabled,
    }
}