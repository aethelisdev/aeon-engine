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

        // Keyboard shortcut: Preferences toggle (Ctrl + ,)
        if let WindowEvent::KeyboardInput {
            event: key_event, ..
        } = event
            && key_event.state == ElementState::Pressed
            && self.iris_overlay.chrome.ctrl_held
            && !self.wants_keyboard_input()
        {
            let is_comma = match key_event.physical_key {
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Comma) => true,
                _ => {
                    if let winit::keyboard::Key::Character(ref c) = key_event.logical_key {
                        c == ","
                    } else {
                        false
                    }
                }
            };
            if is_comma {
                self.show_preferences = !self.show_preferences;
                self.iris_overlay.notifier.tag_all();
                return true;
            }
        }

        // Hierarchy search bar live typing via TextInputState
        let ctrl_held = self.iris_overlay.chrome.ctrl_held;
        let hier_res = crate::ui::iris_bridge::events::text_input::handle_focused_text_input(
            &mut self.iris_overlay.hierarchy.search_input,
            &mut self.iris_overlay.focus_manager,
            crate::ui::iris_bridge::hierarchy::HIERARCHY_TAG_SEARCH_INPUT,
            event,
            ctrl_held,
        );
        if hier_res.is_consumed() {
            if hier_res.is_changed() {
                self.iris_overlay.notifier.tag_all();
            }
            return true;
        }

        // Assets Content Browser search bar live typing via TextInputState
        let assets_res = crate::ui::iris_bridge::events::text_input::handle_focused_text_input(
            &mut self.iris_overlay.assets.search_input,
            &mut self.iris_overlay.focus_manager,
            crate::ui::iris_bridge::assets::ASSETS_TAG_SEARCH_INPUT,
            event,
            ctrl_held,
        );
        if assets_res.is_consumed() {
            if assets_res.is_changed() {
                self.asset_browser.search_query =
                    self.iris_overlay.assets.search_input.text().to_string();
                self.iris_overlay.notifier.tag_all();
            }
            return true;
        }

        // Developer Console search bar live typing via TextInputState
        let console_res = crate::ui::iris_bridge::events::text_input::handle_focused_text_input(
            &mut self.iris_overlay.console.search_input,
            &mut self.iris_overlay.focus_manager,
            crate::ui::iris_bridge::console::CONSOLE_TAG_SEARCH_INPUT,
            event,
            ctrl_held,
        );
        if console_res.is_consumed() {
            if console_res.is_changed() {
                self.iris_overlay.notifier.tag_all();
            }
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::iris_bridge::events::text_input::{
        TextInputActionResult, handle_focused_text_input,
    };
    use irisui::prelude::{FocusManager, TextInputState};
    use winit::event::{Ime, WindowEvent};

    #[test]
    fn test_assets_search_input_single_dispatch_ime() {
        let mut search_input = TextInputState::new("asset_");
        let mut focus_manager = FocusManager::new();
        let target_tag = crate::ui::iris_bridge::assets::ASSETS_TAG_SEARCH_INPUT;
        focus_manager.set_focus_tag(target_tag);

        let event = WindowEvent::Ime(Ime::Commit("model".into()));
        let res = handle_focused_text_input(
            &mut search_input,
            &mut focus_manager,
            target_tag,
            &event,
            false,
        );

        assert_eq!(res, TextInputActionResult::Changed);
        assert_eq!(search_input.text(), "asset_model");
        assert!(focus_manager.is_tag_focused(target_tag));
    }

    #[test]
    fn test_console_search_input_centralized_dispatch_ime() {
        let mut search_input = TextInputState::new("log_");
        let mut focus_manager = FocusManager::new();
        let target_tag = crate::ui::iris_bridge::console::CONSOLE_TAG_SEARCH_INPUT;
        focus_manager.set_focus_tag(target_tag);

        let event = WindowEvent::Ime(Ime::Commit("filter".into()));
        let res = handle_focused_text_input(
            &mut search_input,
            &mut focus_manager,
            target_tag,
            &event,
            false,
        );

        assert_eq!(res, TextInputActionResult::Changed);
        assert_eq!(search_input.text(), "log_filter");
        assert!(focus_manager.is_tag_focused(target_tag));
    }
}