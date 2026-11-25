// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Window and Platform Input Event Arbitration Subsystem.

use crate::session::EditorSession;
use ae_core::modules::EngineMode;
use ae_editor_ui::ui::EngineUiAction;
use ae_input::{ElementState, KeyCode, MouseButton};
use winit::event::{KeyEvent, MouseScrollDelta, WindowEvent};
use winit::keyboard::PhysicalKey;

/// Represents which subsystem currently holds exclusive pointer capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PointerCapture {
    /// No subsystem currently holds exclusive pointer capture.
    #[default]
    None,
    /// UI overlay holds pointer capture (slider, scrollbar, floating window).
    Ui(MouseButton),
    /// 3D viewport canvas holds pointer capture (camera orbit or gizmo drag).
    Viewport(MouseButton),
}

/// Coordinates platform window event routing between UI panels and 3D viewport interactions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputArbiter {
    pointer_capture: PointerCapture,
}

impl InputArbiter {
    /// Creates a new `InputArbiter` with no active pointer capture.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pointer_capture: PointerCapture::None,
        }
    }

    /// Returns the currently active pointer capture state.
    #[must_use]
    pub fn pointer_capture(&self) -> PointerCapture {
        self.pointer_capture
    }

    /// Explicitly resets pointer capture to [`PointerCapture::None`].
    pub fn reset_pointer_capture(&mut self) {
        self.pointer_capture = PointerCapture::None;
    }

    /// Dispatches an incoming Winit window event to the appropriate editor subsystem.
    pub fn dispatch(&mut self, session: &mut EditorSession, event: &WindowEvent) {
        let consumed = self.determine_ui_consumption(session, event);

        match event {
            WindowEvent::Resized(physical_size) => session.resize(*physical_size),
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                self.handle_keyboard_input(session, key_event, consumed);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(session, delta, consumed);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_input(session, *button, *state, consumed);
            }
            WindowEvent::CursorMoved { position, .. } => {
                session
                    .core
                    .input
                    .process_cursor_moved(position.x, position.y);
                if !consumed || matches!(self.pointer_capture, PointerCapture::Viewport(_)) {
                    session.handle_cursor_moved();
                }
            }
            WindowEvent::Focused(focused) => {
                if !*focused {
                    self.pointer_capture = PointerCapture::None;
                }
                session.handle_focus_change(*focused);
            }
            _ => {}
        }
    }

    fn determine_ui_consumption(&self, session: &mut EditorSession, event: &WindowEvent) -> bool {
        if matches!(self.pointer_capture, PointerCapture::Viewport(_)) {
            return false;
        }

        let zoom = session.ui.scale_factor();
        session.window.winit_state.set_scale_factor(zoom as f64);
        let Some(ui_event) = session.window.winit_state.on_window_event(event) else {
            return false;
        };

        if session.window.mode == EngineMode::Play
            && session.window.is_cursor_grabbed
            && matches!(
                event,
                WindowEvent::MouseInput { .. } | WindowEvent::CursorMoved { .. }
            )
        {
            return false;
        }

        Self::dispatch_ui_event(session, &ui_event)
    }

    /// Dispatches a translated [`irisui::prelude::UiEvent`] to [`EngineUi`](ae_editor_ui::ui::EngineUi).
    pub fn dispatch_ui_event(
        session: &mut EditorSession,
        ui_event: &irisui::prelude::UiEvent,
    ) -> bool {
        session.ui.handle_ui_event(&session.window.window, ui_event)
    }

    fn handle_keyboard_input(
        &mut self,
        session: &mut EditorSession,
        key_event: &KeyEvent,
        consumed: bool,
    ) {
        let PhysicalKey::Code(key) = key_event.physical_key else {
            return;
        };
        let state = key_event.state;

        if let Some(next_mode) =
            Self::mode_transition_for_key(session.window.mode, key, state, key_event.repeat)
        {
            self.pointer_capture = PointerCapture::None;
            session.process_ui_actions(vec![EngineUiAction::ChangeMode(next_mode)]);
        }

        if !consumed
            && (key == KeyCode::Escape
                || session.window.mode == EngineMode::Play
                || !session.ui.wants_keyboard_input())
        {
            session.core.input.process_key_event(key, state);
        }
    }

    fn handle_mouse_wheel(
        &self,
        session: &mut EditorSession,
        delta: &MouseScrollDelta,
        consumed: bool,
    ) {
        if consumed || matches!(self.pointer_capture, PointerCapture::Ui(_)) {
            return;
        }

        let (dx, dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (*x, *y),
            MouseScrollDelta::PixelDelta(pos) => (pos.x as f32 * 0.02, pos.y as f32 * 0.02),
        };
        session.core.input.process_mouse_wheel(dx, dy);

        if session.window.mode == EngineMode::Play || !session.ui.wants_pointer_input() {
            session.handle_mouse_scroll(dy);
        }
    }

    fn handle_mouse_input(
        &mut self,
        session: &mut EditorSession,
        button: MouseButton,
        state: ElementState,
        consumed: bool,
    ) {
        match state {
            ElementState::Pressed => {
                if consumed {
                    self.pointer_capture = PointerCapture::Ui(button);
                    Self::clear_editor_pressed_state(session, button);
                } else {
                    let should_handle = if session.window.mode == EngineMode::Play {
                        session.window.is_cursor_grabbed || !session.ui.wants_pointer_input()
                    } else {
                        !session.ui.wants_pointer_input()
                    };

                    if should_handle {
                        self.pointer_capture = PointerCapture::Viewport(button);
                        session.handle_mouse_click(button, state);
                    } else {
                        self.pointer_capture = PointerCapture::None;
                        Self::clear_editor_pressed_state(session, button);
                    }
                }
            }
            ElementState::Released => match self.pointer_capture {
                PointerCapture::Viewport(captured_button) => {
                    session.handle_mouse_click(button, state);
                    if button == captured_button {
                        self.pointer_capture = PointerCapture::None;
                    }
                }
                PointerCapture::Ui(captured_button) => {
                    Self::clear_editor_pressed_state(session, button);
                    if button == captured_button {
                        self.pointer_capture = PointerCapture::None;
                    }
                }
                PointerCapture::None => {
                    Self::clear_editor_pressed_state(session, button);
                    let should_handle = if session.window.mode == EngineMode::Play {
                        session.window.is_cursor_grabbed || !session.ui.wants_pointer_input()
                    } else {
                        !session.ui.wants_pointer_input()
                    };
                    if should_handle {
                        session.handle_mouse_click(button, state);
                    }
                }
            },
        }
    }

    /// Computes whether a non-repeating key press triggers an Edit ↔ Play mode transition.
    #[must_use]
    pub fn mode_transition_for_key(
        current_mode: EngineMode,
        key: KeyCode,
        state: ElementState,
        is_repeat: bool,
    ) -> Option<EngineMode> {
        if is_repeat || state != ElementState::Pressed {
            return None;
        }
        match key {
            KeyCode::F5 => Some(if current_mode == EngineMode::Play {
                EngineMode::Edit
            } else {
                EngineMode::Play
            }),
            KeyCode::Escape if current_mode == EngineMode::Play => Some(EngineMode::Edit),
            _ => None,
        }
    }

    fn clear_editor_pressed_state(session: &mut EditorSession, button: MouseButton) {
        if button == MouseButton::Right {
            session.editor.viewport_camera_look_active = false;
        }
    }
}

impl EditorSession {
    /// Dispatches and arbitrates incoming window events via [`InputArbiter`].
    pub fn process_window_event(&mut self, event: &WindowEvent) {
        let mut arbiter = std::mem::take(&mut self.window.input_arbiter);
        arbiter.dispatch(self, event);
        self.window.input_arbiter = arbiter;
    }
}