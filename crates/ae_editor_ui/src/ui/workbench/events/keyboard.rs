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

        // Hierarchy search bar live typing
        let is_hierarchy_search_focused = self
            .iris_overlay
            .focus_manager
            .is_tag_focused(crate::ui::iris_bridge::hierarchy::HIERARCHY_TAG_SEARCH_INPUT);
        if is_hierarchy_search_focused
            && let WindowEvent::KeyboardInput {
                event: key_event, ..
            } = event
            && key_event.state == ElementState::Pressed
        {
            if let winit::keyboard::Key::Named(
                winit::keyboard::NamedKey::Escape | winit::keyboard::NamedKey::Enter,
            ) = key_event.logical_key
            {
                self.iris_overlay.focus_manager.clear_focus();
                return true;
            }
            if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace) =
                key_event.logical_key
            {
                self.iris_overlay.hierarchy.search_query.pop();
                self.iris_overlay.notifier.tag_all();
                return true;
            }
            if let Some(text) = &key_event.text {
                for c in text.chars() {
                    if !c.is_control() {
                        self.iris_overlay.hierarchy.search_query.push(c);
                    }
                }
                self.iris_overlay.notifier.tag_all();
                return true;
            }
        }

        // Assets Content Browser search bar live typing
        let is_assets_search_focused = self
            .iris_overlay
            .focus_manager
            .is_tag_focused(crate::ui::iris_bridge::assets::ASSETS_TAG_SEARCH_INPUT);
        if is_assets_search_focused
            && let WindowEvent::KeyboardInput {
                event: key_event, ..
            } = event
            && key_event.state == ElementState::Pressed
        {
            if let winit::keyboard::Key::Named(
                winit::keyboard::NamedKey::Escape | winit::keyboard::NamedKey::Enter,
            ) = key_event.logical_key
            {
                self.iris_overlay.focus_manager.clear_focus();
                return true;
            }
            if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace) =
                key_event.logical_key
            {
                self.iris_overlay.assets.search_query.pop();
                self.asset_browser.search_query = self.iris_overlay.assets.search_query.clone();
                self.iris_overlay.notifier.tag_all();
                return true;
            }
            if let Some(text) = &key_event.text {
                for c in text.chars() {
                    if !c.is_control() {
                        self.iris_overlay.assets.search_query.push(c);
                    }
                }
                self.asset_browser.search_query = self.iris_overlay.assets.search_query.clone();
                self.iris_overlay.notifier.tag_all();
                return true;
            }
        }

        false
    }
}