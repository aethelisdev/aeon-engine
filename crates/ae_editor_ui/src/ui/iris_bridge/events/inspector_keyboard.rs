// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Keyboard and IME text input dispatch subsystem for Inspector text and numeric fields.

use crate::ui::iris_bridge::inspector::{self, InspectorAction};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::*;
use winit::event::{ElementState, WindowEvent};

impl IrisEditorOverlay {
    /// Handles keyboard input and IME composition for active text fields in Inspector.
    pub(crate) fn handle_inspector_keyboard_input(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        if self.inspector_active_number_input.is_none()
            && self.inspector_active_text_input.is_none()
            && self.inspector_rename_buffer.is_none()
            && self.inspector_hex_buffer.is_none()
        {
            return None;
        }

        let mut result = IrisOverlayEventResult::default();
        match event {
            WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                if let Some(ref mut session) = self.inspector_active_number_input {
                    if session.is_all_selected {
                        session.buffer.clear();
                        session.cursor_idx = 0;
                        session.is_all_selected = false;
                    }
                    for c in text.chars() {
                        if is_allowed_math_char(c) {
                            session.buffer.insert(session.cursor_idx, c);
                            session.cursor_idx += c.len_utf8();
                        }
                    }
                    result.consumed = true;
                    return Some(result);
                }
                if let Some((_, _, ref mut buf)) = self.inspector_active_text_input {
                    buf.push_str(text);
                    result.consumed = true;
                    return Some(result);
                }
                if let Some((_, ref mut buf)) = self.inspector_rename_buffer {
                    buf.push_str(text);
                    result.consumed = true;
                    return Some(result);
                }
                if let Some((entity, ref mut buf)) = self.inspector_hex_buffer {
                    for c in text.chars() {
                        if (c.is_ascii_hexdigit() || c == '#') && buf.len() < 7 {
                            buf.push(c);
                        }
                    }
                    let clean_hex = buf.trim_start_matches('#');
                    if clean_hex.len() == 6
                        && let Ok(rgb) = u32::from_str_radix(clean_hex, 16)
                    {
                        let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
                        let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
                        let b = (rgb & 0xFF) as f32 / 255.0;
                        self.inspector_actions.push(InspectorAction::SetObjectColor(
                            entity,
                            Color::rgba(r, g, b, 1.0),
                        ));
                    }
                    result.consumed = true;
                    return Some(result);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(key),
                        text,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if self.inspector_active_text_input.is_some() {
                    match *key {
                        winit::keyboard::KeyCode::Escape => {
                            self.inspector_active_text_input = None;
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                            if let Some((entity, id, buf)) = self.inspector_active_text_input.take()
                            {
                                self.inspector_actions
                                    .push(InspectorAction::SetTextValue(entity, id, buf));
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::Backspace => {
                            if let Some((_, _, ref mut buf)) = self.inspector_active_text_input {
                                buf.pop();
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        _ => {
                            if let Some(t) = text
                                && let Some((_, _, ref mut buf)) = self.inspector_active_text_input
                                && !t.chars().any(|c| c.is_control())
                            {
                                buf.push_str(t);
                                result.consumed = true;
                                return Some(result);
                            }
                        }
                    }
                }
                if self.inspector_active_number_input.is_some() {
                    match *key {
                        winit::keyboard::KeyCode::Escape => {
                            self.inspector_active_number_input = None;
                            self.inspector_edit_start_snapshot = None;
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                            if let Some(session) = self.inspector_active_number_input.take() {
                                if let Ok(v) = inspector::evaluate_inspector_math(
                                    &session.buffer,
                                    session.initial_val,
                                ) {
                                    let clamped_v = session
                                        .id
                                        .clamp_value(v.clamp(session.min_val, session.max_val));
                                    self.inspector_actions.push(InspectorAction::SetNumberValue(
                                        session.entity,
                                        session.id,
                                        clamped_v,
                                    ));
                                    self.inspector_actions
                                        .push(InspectorAction::CommitNumberEdit(
                                            session.entity,
                                            session.id,
                                        ));
                                } else {
                                    self.inspector_edit_start_snapshot = None;
                                }
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::ArrowLeft => {
                            if let Some(ref mut session) = self.inspector_active_number_input {
                                session.is_all_selected = false;
                                if session.cursor_idx > 0 {
                                    session.cursor_idx = session.buffer[..session.cursor_idx]
                                        .char_indices()
                                        .next_back()
                                        .map(|(i, _)| i)
                                        .unwrap_or(0);
                                }
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::ArrowRight => {
                            if let Some(ref mut session) = self.inspector_active_number_input {
                                session.is_all_selected = false;
                                if session.cursor_idx < session.buffer.len() {
                                    session.cursor_idx = session.buffer[session.cursor_idx..]
                                        .char_indices()
                                        .nth(1)
                                        .map(|(offset, _)| session.cursor_idx + offset)
                                        .unwrap_or(session.buffer.len());
                                }
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::Home => {
                            if let Some(ref mut session) = self.inspector_active_number_input {
                                session.is_all_selected = false;
                                session.cursor_idx = 0;
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::End => {
                            if let Some(ref mut session) = self.inspector_active_number_input {
                                session.is_all_selected = false;
                                session.cursor_idx = session.buffer.len();
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::Backspace => {
                            if let Some(ref mut session) = self.inspector_active_number_input {
                                if session.is_all_selected {
                                    session.buffer.clear();
                                    session.cursor_idx = 0;
                                    session.is_all_selected = false;
                                } else if session.cursor_idx > 0 {
                                    let prev_idx = session.buffer[..session.cursor_idx]
                                        .char_indices()
                                        .next_back()
                                        .map(|(i, _)| i)
                                        .unwrap_or(0);
                                    session.buffer.drain(prev_idx..session.cursor_idx);
                                    session.cursor_idx = prev_idx;
                                }
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::Delete => {
                            if let Some(ref mut session) = self.inspector_active_number_input {
                                if session.is_all_selected {
                                    session.buffer.clear();
                                    session.cursor_idx = 0;
                                    session.is_all_selected = false;
                                } else if session.cursor_idx < session.buffer.len() {
                                    let next_idx = session.buffer[session.cursor_idx..]
                                        .char_indices()
                                        .nth(1)
                                        .map(|(offset, _)| session.cursor_idx + offset)
                                        .unwrap_or(session.buffer.len());
                                    session.buffer.drain(session.cursor_idx..next_idx);
                                }
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        _ => {
                            if let Some(t) = text
                                && let Some(ref mut session) = self.inspector_active_number_input
                            {
                                if session.is_all_selected {
                                    session.buffer.clear();
                                    session.cursor_idx = 0;
                                    session.is_all_selected = false;
                                }
                                for c in t.chars() {
                                    if is_allowed_math_char(c) {
                                        session.buffer.insert(session.cursor_idx, c);
                                        session.cursor_idx += c.len_utf8();
                                    }
                                }
                                result.consumed = true;
                                return Some(result);
                            }
                        }
                    }
                }
                if self.inspector_rename_buffer.is_some() {
                    match *key {
                        winit::keyboard::KeyCode::Escape => {
                            self.inspector_rename_buffer = None;
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                            if let Some((entity, buf)) = self.inspector_rename_buffer.take()
                                && !buf.trim().is_empty()
                            {
                                self.inspector_actions
                                    .push(InspectorAction::RenameEntity(entity, buf));
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::Backspace => {
                            if let Some((_, ref mut buf)) = self.inspector_rename_buffer {
                                buf.pop();
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        _ => {
                            if let Some(t) = text
                                && let Some((_, ref mut buf)) = self.inspector_rename_buffer
                                && !t.chars().any(|c| c.is_control())
                            {
                                buf.push_str(t);
                                result.consumed = true;
                                return Some(result);
                            }
                        }
                    }
                }
                if self.inspector_hex_buffer.is_some() {
                    match *key {
                        winit::keyboard::KeyCode::Escape => {
                            self.inspector_hex_buffer = None;
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                            if let Some((entity, buf)) = self.inspector_hex_buffer.take() {
                                let clean_hex = buf.trim_start_matches('#');
                                if (clean_hex.len() == 6 || clean_hex.len() == 3)
                                    && let Ok(rgb) = u32::from_str_radix(clean_hex, 16)
                                {
                                    let (r, g, b) = if clean_hex.len() == 6 {
                                        (
                                            ((rgb >> 16) & 0xFF) as f32 / 255.0,
                                            ((rgb >> 8) & 0xFF) as f32 / 255.0,
                                            (rgb & 0xFF) as f32 / 255.0,
                                        )
                                    } else {
                                        (
                                            (((rgb >> 8) & 0xF) * 17) as f32 / 255.0,
                                            (((rgb >> 4) & 0xF) * 17) as f32 / 255.0,
                                            ((rgb & 0xF) * 17) as f32 / 255.0,
                                        )
                                    };
                                    self.inspector_actions.push(InspectorAction::SetObjectColor(
                                        entity,
                                        Color::rgba(r, g, b, 1.0),
                                    ));
                                }
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::Backspace => {
                            if let Some((_, ref mut buf)) = self.inspector_hex_buffer {
                                buf.pop();
                                if buf.is_empty() {
                                    buf.push('#');
                                }
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        _ => {
                            if let Some(t) = text
                                && let Some((entity, ref mut buf)) = self.inspector_hex_buffer
                            {
                                for c in t.chars() {
                                    if (c.is_ascii_hexdigit() || c == '#') && buf.len() < 7 {
                                        buf.push(c);
                                    }
                                }
                                let clean_hex = buf.trim_start_matches('#');
                                if clean_hex.len() == 6
                                    && let Ok(rgb) = u32::from_str_radix(clean_hex, 16)
                                {
                                    let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
                                    let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
                                    let b = (rgb & 0xFF) as f32 / 255.0;
                                    self.inspector_actions.push(InspectorAction::SetObjectColor(
                                        entity,
                                        Color::rgba(r, g, b, 1.0),
                                    ));
                                }
                                result.consumed = true;
                                return Some(result);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        None
    }
}

/// Checks whether a character is permitted in Inspector mathematical numeric inputs.
#[inline]
fn is_allowed_math_char(c: char) -> bool {
    c.is_ascii_digit()
        || c == '.'
        || c == '+'
        || c == '-'
        || c == '*'
        || c == '/'
        || c == '('
        || c == ')'
        || c == '='
        || c == ' '
}