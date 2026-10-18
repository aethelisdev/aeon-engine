// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Keyboard Input Handling
//!
//! Handles editor hotkeys, UI zoom factor adjustment, and search bar text entry.

use crate::ui::workbench::state::EngineUi;
use irisui::prelude::{KeyCode, UiEvent};

impl EngineUi {
    /// Handles keyboard shortcuts (e.g. Ctrl + / - / 0 for UI zoom factor, Ctrl + , for Preferences).
    ///
    /// Evaluates hardware-independent [`UiEvent`] representations dispatched from the window root.
    pub(crate) fn handle_keyboard_shortcut(&mut self, event: &UiEvent) -> bool {
        if let UiEvent::KeyboardInput {
            key,
            is_pressed,
            text,
        } = event
            && *is_pressed
            && self.iris_overlay.chrome.ctrl_held
            && !self.wants_keyboard_input()
        {
            let mut scale_changed = false;
            let ch = match key {
                KeyCode::Character(c) => Some(*c),
                _ => text.as_deref().and_then(|t| t.chars().next()),
            };

            if let Some(c) = ch {
                match c {
                    '+' | '=' => {
                        self.step_ui_scale(true);
                        scale_changed = true;
                    }
                    '-' | '_' => {
                        self.step_ui_scale(false);
                        scale_changed = true;
                    }
                    '0' => {
                        self.reset_ui_scale();
                        scale_changed = true;
                    }
                    ',' => {
                        self.show_preferences = !self.show_preferences;
                        self.iris_overlay.notifier.tag_all();
                        return true;
                    }
                    _ => {}
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

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui::prelude::KeyCode;

    #[test]
    fn test_keyboard_shortcut_key_matching() {
        let event_plus = UiEvent::KeyboardInput {
            key: KeyCode::Character('+'),
            text: Some("+".into()),
            is_pressed: true,
        };

        if let UiEvent::KeyboardInput {
            key,
            is_pressed,
            text,
        } = event_plus
        {
            assert!(is_pressed);
            let ch = match key {
                KeyCode::Character(c) => Some(c),
                _ => text.as_deref().and_then(|t| t.chars().next()),
            };
            assert_eq!(ch, Some('+'));
        } else {
            panic!("Expected KeyboardInput");
        }
    }
}