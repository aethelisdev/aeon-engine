// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use std::collections::HashSet;
use winit::event::ElementState;
pub use winit::keyboard::KeyCode;

/// Keyboard input state manager with three-state key tracking.
/// Tracks three distinct sets per key:
/// - `keys_pressed`: currently held down (continuous, e.g., movement)
/// - `keys_just_pressed`: first pressed this frame (one-shot, e.g., jump)
/// - `keys_just_released`: released this frame (one-shot, e.g., trigger release)
pub struct InputManager {
    keys_pressed: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    keys_just_released: HashSet<KeyCode>,
    /// High-level Input Action Mapping system.
    pub action_map: crate::action_map::ActionMap,
}

impl InputManager {
    /// Creates a new InputManager with all key sets empty and default gameplay ActionMap.
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            keys_just_released: HashSet::new(),
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

    /// Clears one-shot key sets at the end of each frame.
    pub fn end_frame(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
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

    /// Clears all pressed key states. Useful when the window loses focus to prevent keys from getting "stuck".
    pub fn clear_pressed_keys(&mut self) {
        self.keys_pressed.clear();
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
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