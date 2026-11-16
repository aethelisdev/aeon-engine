// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Frame-stepped keyboard key release and UTF-8 typing sequencer for live [`crate::McpHost`].

use crate::host::McpHost;
use crate::input::parse_key_spec;
use crate::overlay::{InspectActionKind, InspectVisualOverlay};
use crate::protocol::InspectResponse;
use irisui::core::{KeyCode, UiEvent};
use serde_json::{Value, json};
use std::sync::mpsc::Sender;
use winit::keyboard::KeyCode as WinitKeyCode;

/// Active frame-stepped character typing sequence emitting one keystroke per rendered frame.
pub struct ActiveInspectTyping {
    chars: Vec<char>,
    cursor: usize,
    submit: bool,
    submitted: bool,
    waiting_release: bool,
    responder: Sender<InspectResponse>,
}

/// Pending single-frame key press awaiting release after one full update/render pass.
pub struct PendingInspectKeyRelease {
    iris_key: KeyCode,
    winit_key: Option<WinitKeyCode>,
    ctrl_held: bool,
    shift_held: bool,
    responder: Option<Sender<InspectResponse>>,
    key_label: String,
    ui_consumed: bool,
}

/// Dispatches a keyboard key press and returns the pending release token for the next frame.
pub fn start_key_press<H: McpHost + ?Sized>(
    host: &mut H,
    overlay: &mut InspectVisualOverlay,
    params: &Value,
    responder: Sender<InspectResponse>,
) -> Option<PendingInspectKeyRelease> {
    let Some(key_str) = params.get("key").and_then(Value::as_str) else {
        let _ = responder.send(InspectResponse::err("Missing string param 'key'"));
        return None;
    };
    let ctrl_held = params.get("ctrl").and_then(Value::as_bool).unwrap_or(false);
    let shift_held = params
        .get("shift")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let Some((iris_key, text_char, winit_key)) = parse_key_spec(key_str) else {
        let _ = responder.send(InspectResponse::err(format!(
            "Unsupported key name: '{key_str}'"
        )));
        return None;
    };

    let text = if ctrl_held {
        None
    } else {
        text_char.map(|c| c.to_string())
    };
    let press_consumed = host.dispatch_ui_event(&UiEvent::KeyboardInput {
        key: iris_key,
        text,
        is_pressed: true,
        ctrl_held,
        shift_held,
    });

    if !press_consumed && let Some(wk) = winit_key {
        push_winit_key_state(host, wk, ctrl_held, shift_held, true);
    }

    overlay.record_key_activity(InspectActionKind::Key);
    host.invalidate_ui(true);

    Some(PendingInspectKeyRelease {
        iris_key,
        winit_key: if !press_consumed { winit_key } else { None },
        ctrl_held,
        shift_held,
        responder: Some(responder),
        key_label: key_str.to_string(),
        ui_consumed: press_consumed,
    })
}

/// Completes a previously dispatched single-frame key press by injecting its release event.
pub fn finish_key_release<H: McpHost + ?Sized>(host: &mut H, pending: PendingInspectKeyRelease) {
    let rel_consumed = host.dispatch_ui_event(&UiEvent::KeyboardInput {
        key: pending.iris_key,
        text: None,
        is_pressed: false,
        ctrl_held: pending.ctrl_held,
        shift_held: pending.shift_held,
    });
    if let Some(wk) = pending.winit_key {
        push_winit_key_state(host, wk, pending.ctrl_held, pending.shift_held, false);
    }
    host.invalidate_ui(true);
    if let Some(tx) = pending.responder {
        let _ = tx.send(InspectResponse::ok(json!({
            "key": pending.key_label,
            "ctrl": pending.ctrl_held,
            "shift": pending.shift_held,
            "ui_consumed": pending.ui_consumed || rel_consumed,
        })));
    }
}

/// Initializes a per-frame UTF-8 text typing sequence into the focused UI widget.
pub fn start_type_text<H: McpHost + ?Sized>(
    host: &mut H,
    overlay: &mut InspectVisualOverlay,
    params: &Value,
    responder: Sender<InspectResponse>,
) -> Option<ActiveInspectTyping> {
    let Some(text) = params.get("text").and_then(Value::as_str) else {
        let _ = responder.send(InspectResponse::err("Missing string param 'text'"));
        return None;
    };
    let submit = params
        .get("submit")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() && !submit {
        let focused = host.health_snapshot().wants_keyboard_input;
        let _ = responder.send(InspectResponse::ok(json!({
            "chars_typed": 0,
            "submitted": false,
            "has_text_input_focus": focused,
        })));
        return None;
    }

    let mut state = ActiveInspectTyping {
        chars,
        cursor: 0,
        submit,
        submitted: false,
        waiting_release: false,
        responder,
    };
    dispatch_next_typing_step(host, overlay, &mut state);
    Some(state)
}

/// Advances an active character typing sequence by one frame, returning `None` when completed.
pub fn tick_typing<H: McpHost + ?Sized>(
    host: &mut H,
    overlay: &mut InspectVisualOverlay,
    mut typing: ActiveInspectTyping,
) -> Option<ActiveInspectTyping> {
    if typing.waiting_release {
        let (key, shift_held) = if typing.submitted {
            (KeyCode::Enter, false)
        } else {
            let ch = typing.chars[typing.cursor];
            (KeyCode::Character(ch), ch.is_uppercase())
        };
        let _ = host.dispatch_ui_event(&UiEvent::KeyboardInput {
            key,
            text: None,
            is_pressed: false,
            ctrl_held: false,
            shift_held,
        });
        typing.waiting_release = false;
        if !typing.submitted {
            typing.cursor += 1;
        }
    }

    if typing.cursor < typing.chars.len() || (typing.submit && !typing.submitted) {
        dispatch_next_typing_step(host, overlay, &mut typing);
        Some(typing)
    } else {
        host.invalidate_ui(true);
        let focused = host.health_snapshot().wants_keyboard_input;
        let _ = typing.responder.send(InspectResponse::ok(json!({
            "chars_typed": typing.chars.len(),
            "submitted": typing.submitted,
            "has_text_input_focus": focused,
        })));
        None
    }
}

fn dispatch_next_typing_step<H: McpHost + ?Sized>(
    host: &mut H,
    overlay: &mut InspectVisualOverlay,
    typing: &mut ActiveInspectTyping,
) {
    if typing.cursor < typing.chars.len() {
        let ch = typing.chars[typing.cursor];
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf).to_string();
        let _ = host.dispatch_ui_event(&UiEvent::KeyboardInput {
            key: KeyCode::Character(ch),
            text: Some(s),
            is_pressed: true,
            ctrl_held: false,
            shift_held: ch.is_uppercase(),
        });
        typing.waiting_release = true;
    } else if typing.submit && !typing.submitted {
        let _ = host.dispatch_ui_event(&UiEvent::KeyboardInput {
            key: KeyCode::Enter,
            text: None,
            is_pressed: true,
            ctrl_held: false,
            shift_held: false,
        });
        typing.submitted = true;
        typing.waiting_release = true;
    }

    overlay.record_key_activity(InspectActionKind::TypeText);
    host.invalidate_ui(true);
}

fn push_winit_key_state<H: McpHost + ?Sized>(
    host: &mut H,
    key: WinitKeyCode,
    ctrl: bool,
    shift: bool,
    pressed: bool,
) {
    if ctrl {
        host.inject_shortcut_key(WinitKeyCode::ControlLeft, pressed);
    }
    if shift {
        host.inject_shortcut_key(WinitKeyCode::ShiftLeft, pressed);
    }
    host.inject_shortcut_key(key, pressed);
}