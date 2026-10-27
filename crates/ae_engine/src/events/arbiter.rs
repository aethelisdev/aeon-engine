// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Window and Platform Input Event Arbitration Subsystem.
//!
//! Arbitrates raw Winit window events between editor UI overlays, 3D viewport canvas
//! interactions, gizmo transformations, and active gameplay subsystems using an explicit
//! pointer capture state machine.

use crate::engine::AeEngine;
use ae_core::modules::EngineMode;
use ae_editor_ui::ui::EngineUiAction;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::PhysicalKey;

/// Represents which subsystem currently holds exclusive pointer capture.
///
/// Implements deterministic pointer capture to prevent stuck camera states
/// and accidental cross-system interaction leaks during mouse drag operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PointerCapture {
    /// No subsystem currently holds exclusive pointer capture.
    #[default]
    None,
    /// User interface overlay currently holds pointer capture (e.g. dragging a slider, scrollbar, or window).
    Ui(MouseButton),
    /// 3D Viewport currently holds pointer capture (e.g. camera orbit navigation or gizmo transformation).
    Viewport(MouseButton),
}

/// Coordinates platform window event routing and input arbitration.
///
/// Prevents cross-system input leaks between 2D UI panels and the 3D scene,
/// manages pointer capture states during mode transitions, and prevents
/// stuck camera/gizmo states when release events occur over UI elements.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputArbiter {
    /// Active pointer capture state machine.
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

    /// Dispatches an incoming Winit window event to the appropriate engine subsystem.
    ///
    /// Evaluates UI consumption first, then routes to keyboard hotkeys, mouse clicks,
    /// viewport scrolling, or OS lifecycle handlers based on active engine modes.
    pub fn dispatch(&mut self, engine: &mut AeEngine, event: &WindowEvent) {
        let consumed = self.determine_ui_consumption(engine, event);

        match event {
            WindowEvent::Resized(physical_size) => {
                engine.resize(*physical_size);
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                self.handle_keyboard_input(engine, key_event);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(engine, delta, consumed);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_input(engine, *button, *state, consumed);
            }
            WindowEvent::CursorMoved { position, .. } => {
                if !consumed || matches!(self.pointer_capture, PointerCapture::Viewport(_)) {
                    engine.handle_cursor_moved(position.x, position.y);
                }
            }
            WindowEvent::Focused(focused) => {
                if !*focused {
                    self.pointer_capture = PointerCapture::None;
                }
                engine.handle_focus_change(*focused);
            }
            WindowEvent::DroppedFile(path) if !consumed => {
                crate::importer::handle_dropped_file(engine, path.clone());
            }
            _ => {}
        }
    }

    /// Evaluates whether the UI overlay consumes the incoming window event.
    ///
    /// Translates raw Winit [`WindowEvent`] into declarative [`irisui::prelude::UiEvent`]
    /// representations at the engine root before dispatching to editor UI panels.
    ///
    /// If 3D Viewport holds pointer capture, UI is completely bypassed to prevent
    /// camera orbit or gizmo release events from being swallowed by background panels.
    fn determine_ui_consumption(&self, engine: &mut AeEngine, event: &WindowEvent) -> bool {
        if matches!(self.pointer_capture, PointerCapture::Viewport(_)) {
            return false;
        }

        let zoom = engine.ui.scale_factor();
        engine.winit_state.set_scale_factor(zoom as f64);
        let ui_event = engine.winit_state.on_window_event(event);

        // Synchronize keyboard modifier flags to editor UI
        engine.ui.iris_overlay.chrome.ctrl_held = engine.winit_state.modifiers().ctrl();
        engine.ui.iris_overlay.chrome.shift_held = engine.winit_state.modifiers().shift();
        engine.ui.iris_overlay.chrome.alt_held = engine.winit_state.modifiers().alt();

        let Some(ui_event) = ui_event else {
            return false;
        };

        let window = &engine.render_state.window;

        if engine.mode == EngineMode::Play && engine.is_cursor_grabbed {
            match event {
                WindowEvent::MouseInput { .. } | WindowEvent::CursorMoved { .. } => false,
                _ => engine
                    .ui
                    .handle_event(window, &ui_event, Some(&engine.ecs.world)),
            }
        } else {
            engine
                .ui
                .handle_event(window, &ui_event, Some(&engine.ecs.world))
        }
    }

    /// Routes keyboard input, handling global editor mode hotkeys and gameplay forwarding.
    fn handle_keyboard_input(&mut self, engine: &mut AeEngine, key_event: &KeyEvent) {
        let PhysicalKey::Code(key) = key_event.physical_key else {
            return;
        };
        let state = key_event.state;

        // F5: Toggle between Edit and Play mode
        if key == winit::keyboard::KeyCode::F5 && state == ElementState::Pressed {
            self.pointer_capture = PointerCapture::None;
            let next_mode = if engine.mode == EngineMode::Play {
                EngineMode::Edit
            } else {
                EngineMode::Play
            };
            engine.process_ui_actions(vec![EngineUiAction::ChangeMode(next_mode)]);
        }

        // Escape: In Play mode, cleanly return to Edit mode
        if key == winit::keyboard::KeyCode::Escape
            && state == ElementState::Pressed
            && engine.mode == EngineMode::Play
        {
            self.pointer_capture = PointerCapture::None;
            engine.process_ui_actions(vec![EngineUiAction::ChangeMode(EngineMode::Edit)]);
        }

        // Forward keys to gameplay if Escape, in Play mode, or UI is not capturing keyboard input
        if key == winit::keyboard::KeyCode::Escape
            || engine.mode == EngineMode::Play
            || !engine.ui.wants_keyboard_input()
        {
            engine.input.process_key_event(key, state);
        }
    }

    /// Routes mouse wheel scroll events to 3D camera zoom if cursor is outside UI rects.
    fn handle_mouse_wheel(
        &self,
        engine: &mut AeEngine,
        delta: &winit::event::MouseScrollDelta,
        consumed: bool,
    ) {
        if consumed || matches!(self.pointer_capture, PointerCapture::Ui(_)) {
            return;
        }

        if engine.mode == EngineMode::Play || !engine.ui.wants_pointer_input() {
            let (cursor_x, cursor_y) = engine.editor.last_cursor_pos;
            engine.handle_mouse_scroll(delta, cursor_x, cursor_y);
        }
    }

    /// Arbitrates mouse press and release events using pointer capture state machine.
    fn handle_mouse_input(
        &mut self,
        engine: &mut AeEngine,
        button: MouseButton,
        state: ElementState,
        consumed: bool,
    ) {
        match state {
            ElementState::Pressed => {
                if consumed {
                    self.pointer_capture = PointerCapture::Ui(button);
                    Self::clear_editor_pressed_state(engine, button);
                } else {
                    let should_handle = if engine.mode == EngineMode::Play {
                        engine.is_cursor_grabbed || !engine.ui.wants_pointer_input()
                    } else {
                        !engine.ui.wants_pointer_input()
                    };

                    if should_handle {
                        self.pointer_capture = PointerCapture::Viewport(button);
                        engine.handle_mouse_click(button, state);
                    } else {
                        self.pointer_capture = PointerCapture::None;
                        Self::clear_editor_pressed_state(engine, button);
                    }
                }
            }
            ElementState::Released => {
                match self.pointer_capture {
                    PointerCapture::Viewport(captured_button) => {
                        // Guarantee release is delivered to 3D camera / gizmo even if cursor drifted over UI
                        engine.handle_mouse_click(button, state);
                        if button == captured_button {
                            self.pointer_capture = PointerCapture::None;
                        }
                    }
                    PointerCapture::Ui(captured_button) => {
                        Self::clear_editor_pressed_state(engine, button);
                        if button == captured_button {
                            self.pointer_capture = PointerCapture::None;
                        }
                    }
                    PointerCapture::None => {
                        Self::clear_editor_pressed_state(engine, button);

                        let should_handle = if engine.mode == EngineMode::Play {
                            engine.is_cursor_grabbed || !engine.ui.wants_pointer_input()
                        } else {
                            !engine.ui.wants_pointer_input()
                        };

                        if should_handle {
                            engine.handle_mouse_click(button, state);
                        }
                    }
                }
            }
        }
    }

    /// Resets corresponding editor button pressed flags to prevent lingering held state.
    fn clear_editor_pressed_state(engine: &mut AeEngine, button: MouseButton) {
        match button {
            MouseButton::Left => engine.editor.left_mouse_pressed = false,
            MouseButton::Right => engine.editor.right_mouse_pressed = false,
            _ => {}
        }
    }
}

impl AeEngine {
    /// Dispatches and arbitrates incoming window events via [`InputArbiter`].
    ///
    /// Routes interactions between UI panels, the 3D viewport canvas, and active
    /// gameplay systems according to engine mode and pointer capture state.
    pub fn process_window_event(&mut self, event: &WindowEvent) {
        let mut arbiter = std::mem::take(&mut self.input_arbiter);
        arbiter.dispatch(self, event);
        self.input_arbiter = arbiter;
    }
}