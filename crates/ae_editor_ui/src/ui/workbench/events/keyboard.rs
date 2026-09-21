// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Keyboard Input Handling
//!
//! Handles editor hotkeys, UI zoom factor adjustment, and search bar text entry.

use crate::ui::workbench::state::EngineUi;
use winit::event::{ElementState, WindowEvent};

impl EngineUi {
    /// Handles keyboard shortcuts (e.g. Ctrl + / - / 0 for UI zoom factor) and search input.
    pub(crate) fn handle_keyboard_shortcut(&mut self, event: &WindowEvent) -> bool {
        // Keyboard shortcuts: UI scaling (Ctrl + / Ctrl - / Ctrl 0)
        if let WindowEvent::KeyboardInput {
            event: key_event, ..
        } = event
            && key_event.state == ElementState::Pressed
            && self.iris_overlay.chrome.ctrl_held
            && !self.wants_keyboard_input()
        {
            let mut scale_changed = false;
            match key_event.physical_key {
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Equal)
                | winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::NumpadAdd) => {
                    self.step_ui_scale(true);
                    scale_changed = true;
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Minus)
                | winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::NumpadSubtract) => {
                    self.step_ui_scale(false);
                    scale_changed = true;
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit0)
                | winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Numpad0) => {
                    self.reset_ui_scale();
                    scale_changed = true;
                }
                _ => {
                    if let winit::keyboard::Key::Character(ref c) = key_event.logical_key {
                        if c == "+" || c == "=" {
                            self.step_ui_scale(true);
                            scale_changed = true;
                        } else if c == "-" || c == "_" {
                            self.step_ui_scale(false);
                            scale_changed = true;
                        } else if c == "0" {
                            self.reset_ui_scale();
                            scale_changed = true;
                        }
                    }
                }
            }

            if scale_changed {
                self.pending_actions
                    .push(crate::ui::types::EngineUiAction::SetUiScale(
                        self.scale_factor(),
                    ));
                return true;
            }
        }

        // Hierarchy search bar live typing
        if self.iris_overlay.hierarchy.is_search_focused
            && let WindowEvent::KeyboardInput {
                event: key_event, ..
            } = event
            && key_event.state == ElementState::Pressed
        {
            if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) =
                key_event.logical_key
            {
                self.iris_overlay.hierarchy.is_search_focused = false;
                return true;
            }
            if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace) =
                key_event.logical_key
            {
                self.iris_overlay.hierarchy.search_query.pop();
                return true;
            }
            if let Some(text) = &key_event.text {
                for c in text.chars() {
                    if !c.is_control() {
                        self.iris_overlay.hierarchy.search_query.push(c);
                    }
                }
                return true;
            }
        }

        false
    }
}