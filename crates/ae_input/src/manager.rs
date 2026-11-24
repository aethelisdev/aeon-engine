// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Per-frame keyboard, mouse button, pointer, raw motion, and scroll wheel
//! input tracking paired with [`ActionMap`].

use crate::action_map::ActionMap;
use std::collections::HashSet;
use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

/// Unified input state manager acting as the Single Source of Truth for physical
/// keyboard keys, mouse buttons, cursor coordinates, raw relative mouse motion,
/// scroll wheel deltas, and virtual action/axis bindings.
pub struct InputManager {
    keys_pressed: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    keys_just_released: HashSet<KeyCode>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_just_pressed: HashSet<MouseButton>,
    mouse_buttons_just_released: HashSet<MouseButton>,
    cursor_pos: (f64, f64),
    cursor_delta: (f64, f64),
    has_cursor_pos: bool,
    mouse_delta: (f32, f32),
    scroll_delta: (f32, f32),
    /// High-level virtual action and analog axis mapping table.
    pub action_map: ActionMap,
}

impl InputManager {
    /// Creates an empty `InputManager` initialized with [`ActionMap::default_gameplay`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            keys_just_released: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_just_pressed: HashSet::new(),
            mouse_buttons_just_released: HashSet::new(),
            cursor_pos: (0.0, 0.0),
            cursor_delta: (0.0, 0.0),
            has_cursor_pos: false,
            mouse_delta: (0.0, 0.0),
            scroll_delta: (0.0, 0.0),
            action_map: ActionMap::default(),
        }
    }

    /// Processes a keyboard transition into the three-state key tracking sets.
    ///
    /// OS key-repeat `Pressed` events do not re-trigger `just_pressed` while held.
    pub fn process_key_event(&mut self, keycode: KeyCode, state: ElementState) {
        if state == ElementState::Pressed {
            if self.keys_pressed.insert(keycode) {
                self.keys_just_pressed.insert(keycode);
            }
        } else {
            self.keys_pressed.remove(&keycode);
            self.keys_just_released.insert(keycode);
        }
    }

    /// Processes a mouse button transition into the three-state button tracking sets.
    pub fn process_mouse_button_event(&mut self, button: MouseButton, state: ElementState) {
        if state == ElementState::Pressed {
            if self.mouse_buttons_pressed.insert(button) {
                self.mouse_buttons_just_pressed.insert(button);
            }
        } else {
            self.mouse_buttons_pressed.remove(&button);
            self.mouse_buttons_just_released.insert(button);
        }
    }

    /// Updates cursor coordinates in physical pixels and accumulates per-frame `cursor_delta`.
    ///
    /// The very first sample initializes `cursor_pos` without producing a synthetic delta spike.
    /// Non-finite coordinates are ignored.
    pub fn process_cursor_moved(&mut self, x: f64, y: f64) {
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        if self.has_cursor_pos {
            self.cursor_delta.0 += x - self.cursor_pos.0;
            self.cursor_delta.1 += y - self.cursor_pos.1;
        } else {
            self.has_cursor_pos = true;
        }
        self.cursor_pos = (x, y);
    }

    /// Updates cursor coordinates without modifying `cursor_delta` (used when warping cursor).
    ///
    /// Non-finite coordinates are ignored.
    pub fn set_cursor_pos(&mut self, x: f64, y: f64) {
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        self.cursor_pos = (x, y);
        self.has_cursor_pos = true;
    }

    /// Accumulates un-clamped relative hardware mouse motion deltas for the current frame.
    ///
    /// Non-finite deltas are ignored.
    pub fn add_mouse_delta(&mut self, dx: f32, dy: f32) {
        if !dx.is_finite() || !dy.is_finite() {
            return;
        }
        self.mouse_delta.0 += dx;
        self.mouse_delta.1 += dy;
    }

    /// Accumulates scroll wheel deltas (`x`, `y`) for the current frame.
    ///
    /// Non-finite deltas are ignored.
    pub fn process_mouse_wheel(&mut self, dx: f32, dy: f32) {
        if !dx.is_finite() || !dy.is_finite() {
            return;
        }
        self.scroll_delta.0 += dx;
        self.scroll_delta.1 += dy;
    }

    /// Clears single-frame transitions (`just_pressed`, `just_released`) and per-frame
    /// relative deltas (`cursor_delta`, `mouse_delta`, `scroll_delta`) at frame end.
    pub fn end_frame(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_buttons_just_pressed.clear();
        self.mouse_buttons_just_released.clear();
        self.cursor_delta = (0.0, 0.0);
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = (0.0, 0.0);
    }

    /// Returns current physical cursor coordinates `(x, y)`.
    #[must_use]
    pub fn cursor_pos(&self) -> (f64, f64) {
        self.cursor_pos
    }

    /// Returns accumulated per-frame cursor pixel movement `(dx, dy)`.
    #[must_use]
    pub fn cursor_delta(&self) -> (f64, f64) {
        self.cursor_delta
    }

    /// Returns accumulated relative hardware mouse motion `(dx, dy)` for the current frame.
    #[must_use]
    pub fn mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }

    /// Returns a mutable reference to `mouse_delta` for controllers that consume and reset it.
    pub fn mouse_delta_mut(&mut self) -> &mut (f32, f32) {
        &mut self.mouse_delta
    }

    /// Resets accumulated hardware `mouse_delta` to `(0.0, 0.0)`.
    pub fn clear_mouse_delta(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }

    /// Returns accumulated scroll wheel delta `(dx, dy)` for the current frame.
    #[must_use]
    pub fn scroll_delta(&self) -> (f32, f32) {
        self.scroll_delta
    }

    /// Returns `true` while `keycode` is held down.
    #[must_use]
    pub fn is_key_pressed(&self, keycode: KeyCode) -> bool {
        self.keys_pressed.contains(&keycode)
    }

    /// Returns `true` only on the frame `keycode` transitioned to pressed.
    #[must_use]
    pub fn is_key_just_pressed(&self, keycode: KeyCode) -> bool {
        self.keys_just_pressed.contains(&keycode)
    }

    /// Returns `true` only on the frame `keycode` transitioned to released.
    #[must_use]
    pub fn is_key_just_released(&self, keycode: KeyCode) -> bool {
        self.keys_just_released.contains(&keycode)
    }

    /// Returns `true` if either left or right Control modifier key is held down.
    #[must_use]
    pub fn is_ctrl_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::ControlLeft) || self.is_key_pressed(KeyCode::ControlRight)
    }

    /// Returns `true` if either left or right Shift modifier key is held down.
    #[must_use]
    pub fn is_shift_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::ShiftLeft) || self.is_key_pressed(KeyCode::ShiftRight)
    }

    /// Returns `true` if either left or right Alt modifier key is held down.
    #[must_use]
    pub fn is_alt_pressed(&self) -> bool {
        self.is_key_pressed(KeyCode::AltLeft) || self.is_key_pressed(KeyCode::AltRight)
    }

    /// Returns `true` while `button` is held down.
    #[must_use]
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    /// Returns `true` only on the frame `button` transitioned to pressed.
    #[must_use]
    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_pressed.contains(&button)
    }

    /// Returns `true` only on the frame `button` transitioned to released.
    #[must_use]
    pub fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_released.contains(&button)
    }

    /// Clears all held and transient key/mouse states and motion deltas (used on focus loss).
    pub fn clear_pressed_keys(&mut self) {
        self.keys_pressed.clear();
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_buttons_pressed.clear();
        self.mouse_buttons_just_pressed.clear();
        self.mouse_buttons_just_released.clear();
        self.cursor_delta = (0.0, 0.0);
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = (0.0, 0.0);
    }

    /// Returns `true` if any key bound to logical `action_name` is currently held down.
    #[must_use]
    pub fn is_action_pressed(&self, action_name: &str) -> bool {
        self.action_map.is_action_pressed(self, action_name)
    }

    /// Returns `true` if any key bound to logical `action_name` was pressed this frame.
    #[must_use]
    pub fn is_action_just_pressed(&self, action_name: &str) -> bool {
        self.action_map.is_action_just_pressed(self, action_name)
    }

    /// Returns `true` if any key bound to logical `action_name` was released this frame.
    #[must_use]
    pub fn is_action_just_released(&self, action_name: &str) -> bool {
        self.action_map.is_action_just_released(self, action_name)
    }

    /// Returns the combined clamped analog axis value in `[-1.0, 1.0]` for `axis_name`.
    #[must_use]
    pub fn get_axis(&self, axis_name: &str) -> f32 {
        self.action_map.get_axis(self, axis_name)
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}