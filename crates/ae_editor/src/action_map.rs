// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::input::InputManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use winit::keyboard::KeyCode;

/// Defines a key or axis binding with scale modifier.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AxisBinding {
    pub key: KeyCode,
    pub scale: f32,
}

/// Input Action Mapping System for virtual action & axis bindings.
/// Maps logical action names (e.g., "Jump", "MoveForward", "MoveRight", "Interact")
/// to physical key codes, allowing dynamic key rebinding, multi-key bindings,
/// analog axis scaling, and JSON serialization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionMap {
    pub name: String,
    pub actions: HashMap<String, Vec<KeyCode>>,
    pub axes: HashMap<String, Vec<AxisBinding>>,
}

impl ActionMap {
    /// Creates a new ActionMap with default gameplay bindings.
    /// Default bindings:
    /// - `"MoveForward"`: W (+1.0), S (-1.0), Up (+1.0), Down (-1.0)
    /// - `"MoveRight"`: D (+1.0), A (-1.0), Right (+1.0), Left (-1.0)
    /// - `"Jump"`: Space
    /// - `"Run"`: ShiftLeft, ShiftRight
    /// - `"Interact"`: KeyE
    /// - `"Crouch"`: ControlLeft, KeyC
    pub fn default_gameplay() -> Self {
        let mut map = Self {
            name: "Gameplay".to_string(),
            actions: HashMap::new(),
            axes: HashMap::new(),
        };

        // Actions
        map.actions.insert("Jump".to_string(), vec![KeyCode::Space]);
        map.actions.insert(
            "Run".to_string(),
            vec![KeyCode::ShiftLeft, KeyCode::ShiftRight],
        );
        map.actions
            .insert("Interact".to_string(), vec![KeyCode::KeyE]);
        map.actions.insert(
            "Crouch".to_string(),
            vec![KeyCode::ControlLeft, KeyCode::KeyC],
        );

        // Axes
        map.axes.insert(
            "MoveForward".to_string(),
            vec![
                AxisBinding {
                    key: KeyCode::KeyW,
                    scale: 1.0,
                },
                AxisBinding {
                    key: KeyCode::KeyS,
                    scale: -1.0,
                },
                AxisBinding {
                    key: KeyCode::ArrowUp,
                    scale: 1.0,
                },
                AxisBinding {
                    key: KeyCode::ArrowDown,
                    scale: -1.0,
                },
            ],
        );

        map.axes.insert(
            "MoveRight".to_string(),
            vec![
                AxisBinding {
                    key: KeyCode::KeyD,
                    scale: 1.0,
                },
                AxisBinding {
                    key: KeyCode::KeyA,
                    scale: -1.0,
                },
                AxisBinding {
                    key: KeyCode::ArrowRight,
                    scale: 1.0,
                },
                AxisBinding {
                    key: KeyCode::ArrowLeft,
                    scale: -1.0,
                },
            ],
        );

        map
    }

    /// Binds a digital action to a key.
    pub fn bind_action(&mut self, action_name: &str, key: KeyCode) {
        self.actions
            .entry(action_name.to_string())
            .or_default()
            .push(key);
    }

    /// Binds an axis to a key with a scale factor (+1.0 or -1.0).
    pub fn bind_axis(&mut self, axis_name: &str, key: KeyCode, scale: f32) {
        self.axes
            .entry(axis_name.to_string())
            .or_default()
            .push(AxisBinding { key, scale });
    }

    /// Rebinds a digital action by replacing an existing key with a new key.
    pub fn rebind_action(&mut self, action_name: &str, old_key: KeyCode, new_key: KeyCode) -> bool {
        if let Some(keys) = self.actions.get_mut(action_name) {
            for k in keys.iter_mut() {
                if *k == old_key {
                    *k = new_key;
                    return true;
                }
            }
        }
        false
    }

    /// Queries whether any key bound to the action is currently pressed down.
    pub fn is_action_pressed(&self, input: &InputManager, action_name: &str) -> bool {
        if let Some(keys) = self.actions.get(action_name) {
            keys.iter().any(|&k| input.is_key_pressed(k))
        } else {
            false
        }
    }

    /// Queries whether any key bound to the action was just pressed down this frame.
    pub fn is_action_just_pressed(&self, input: &InputManager, action_name: &str) -> bool {
        if let Some(keys) = self.actions.get(action_name) {
            keys.iter().any(|&k| input.is_key_just_pressed(k))
        } else {
            false
        }
    }

    /// Queries whether any key bound to the action was just released this frame.
    pub fn is_action_just_released(&self, input: &InputManager, action_name: &str) -> bool {
        if let Some(keys) = self.actions.get(action_name) {
            keys.iter().any(|&k| input.is_key_just_released(k))
        } else {
            false
        }
    }

    /// Calculates the axis value (-1.0 to +1.0) based on currently pressed keys and scale factors.
    pub fn get_axis(&self, input: &InputManager, axis_name: &str) -> f32 {
        let mut value = 0.0f32;
        if let Some(bindings) = self.axes.get(axis_name) {
            for b in bindings {
                if input.is_key_pressed(b.key) {
                    value += b.scale;
                }
            }
        }
        value.clamp(-1.0, 1.0)
    }

    /// Serializes the action map to JSON string.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    /// Deserializes an action map from JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}

impl Default for ActionMap {
    fn default() -> Self {
        Self::default_gameplay()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event::ElementState;

    #[test]
    fn test_action_map_digital_actions_and_axes() {
        let mut input = InputManager::new();
        assert!(!input.is_action_pressed("Jump"));
        assert_eq!(input.get_axis("MoveForward"), 0.0);

        // Press Space -> Jump action active
        input.process_key_event(KeyCode::Space, ElementState::Pressed);
        assert!(input.is_action_pressed("Jump"));
        assert!(input.is_action_just_pressed("Jump"));

        // Press W -> MoveForward axis +1.0
        input.process_key_event(KeyCode::KeyW, ElementState::Pressed);
        assert_eq!(input.get_axis("MoveForward"), 1.0);

        // Press S -> MoveForward axis 0.0 (W: +1.0, S: -1.0 cancel out)
        input.process_key_event(KeyCode::KeyS, ElementState::Pressed);
        assert_eq!(input.get_axis("MoveForward"), 0.0);
    }

    #[test]
    fn test_action_map_json_serialization() {
        let map = ActionMap::default_gameplay();
        let json = map.to_json().expect("Failed to serialize ActionMap");
        let restored = ActionMap::from_json(&json).expect("Failed to deserialize ActionMap");

        assert_eq!(map.name, restored.name);
        assert!(restored.actions.contains_key("Jump"));
        assert!(restored.axes.contains_key("MoveForward"));
    }
}