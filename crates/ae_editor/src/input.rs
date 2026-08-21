// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use std::collections::HashSet;
use winit::event::{ElementState, MouseButton};
pub use winit::keyboard::KeyCode;

/// Input state manager with three-state key and mouse button tracking.
/// Tracks three distinct sets per key and mouse button:
/// - `pressed`: currently held down (continuous, e.g., movement)
/// - `just_pressed`: first pressed this frame (one-shot, e.g., jump, fire)
/// - `just_released`: released this frame (one-shot, e.g., trigger release)
pub struct InputManager {
    keys_pressed: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    keys_just_released: HashSet<KeyCode>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_just_pressed: HashSet<MouseButton>,
    mouse_buttons_just_released: HashSet<MouseButton>,
    /// High-level Input Action Mapping system.
    pub action_map: crate::action_map::ActionMap,
}

impl InputManager {
    /// Creates a new InputManager with all key and mouse sets empty and default gameplay ActionMap.
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            keys_just_released: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_just_pressed: HashSet::new(),
            mouse_buttons_just_released: HashSet::new(),
            action_map: crate::action_map::ActionMap::default(),
        }
    }

    /// Processes a keyboard event from winit into the three-state tracking sets.
    pub fn process_key_event(&mut self, keycode: KeyCode, state: ElementState) {
        let is_pressed = state == ElementState::Pressed;

        if is_pressed {
            if self.keys_pressed.insert(keycode) {
                self.keys_just_pressed.insert(keycode);
            }
        } else {
            self.keys_pressed.remove(&keycode);
            self.keys_just_released.insert(keycode);
        }
    }

    /// Processes a mouse button event from winit into the three-state tracking sets.
    pub fn process_mouse_button_event(&mut self, button: MouseButton, state: ElementState) {
        let is_pressed = state == ElementState::Pressed;

        if is_pressed {
            if self.mouse_buttons_pressed.insert(button) {
                self.mouse_buttons_just_pressed.insert(button);
            }
        } else {
            self.mouse_buttons_pressed.remove(&button);
            self.mouse_buttons_just_released.insert(button);
        }
    }

    /// Clears one-shot key and mouse sets at the end of each frame.
    pub fn end_frame(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_buttons_just_pressed.clear();
        self.mouse_buttons_just_released.clear();
    }

    /// Returns `true` if the key is currently held down (continuous query).
    pub fn is_key_pressed(&self, keycode: KeyCode) -> bool {
        self.keys_pressed.contains(&keycode)
    }

    /// Returns `true` only on the first frame the key was pressed (one-shot query).
    pub fn is_key_just_pressed(&self, keycode: KeyCode) -> bool {
        self.keys_just_pressed.contains(&keycode)
    }

    /// Returns `true` only on the frame the key was released (one-shot query).
    pub fn is_key_just_released(&self, keycode: KeyCode) -> bool {
        self.keys_just_released.contains(&keycode)
    }

    /// Returns `true` if the mouse button is currently held down.
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    /// Returns `true` only on the first frame the mouse button was pressed.
    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_pressed.contains(&button)
    }

    /// Returns `true` only on the frame the mouse button was released.
    pub fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_released.contains(&button)
    }

    /// Clears all pressed key and mouse states. Useful when the window loses focus to prevent keys from getting "stuck".
    pub fn clear_pressed_keys(&mut self) {
        self.keys_pressed.clear();
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_buttons_pressed.clear();
        self.mouse_buttons_just_pressed.clear();
        self.mouse_buttons_just_released.clear();
    }

    /// Queries whether any key bound to a logical action (e.g. "Jump", "Run") is currently held down.
    pub fn is_action_pressed(&self, action_name: &str) -> bool {
        self.action_map.is_action_pressed(self, action_name)
    }

    /// Queries whether any key bound to a logical action was just pressed this frame.
    pub fn is_action_just_pressed(&self, action_name: &str) -> bool {
        self.action_map.is_action_just_pressed(self, action_name)
    }

    /// Queries whether any key bound to a logical action was just released this frame.
    pub fn is_action_just_released(&self, action_name: &str) -> bool {
        self.action_map.is_action_just_released(self, action_name)
    }

    /// Returns the continuous axis value (-1.0 to +1.0) for a logical axis name (e.g. "MoveForward", "MoveRight").
    pub fn get_axis(&self, axis_name: &str) -> f32 {
        self.action_map.get_axis(self, axis_name)
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}