// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Virtual input action and analog axis mapping with JSON serialization.

use crate::manager::InputManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use winit::keyboard::KeyCode;

/// Binds a physical [`KeyCode`] to an analog axis contribution with a sign/weight multiplier.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AxisBinding {
    /// Physical key contributing to the axis.
    pub key: KeyCode,
    /// Scalar multiplier added to the axis sum while `key` is pressed (typically `+1.0` or `-1.0`).
    pub scale: f32,
}

/// Serializable mapping from logical action and axis names to physical [`KeyCode`] bindings.
///
/// Supports multi-key digital actions, summed/clamped analog axes in `[-1.0, 1.0]`,
/// runtime rebinding, and JSON persistence (`Config/input.json`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ActionMap {
    /// Profile display name (e.g. `"Gameplay"`).
    pub name: String,
    /// Digital action names mapped to physical trigger keys.
    pub actions: HashMap<String, Vec<KeyCode>>,
    /// Analog axis names mapped to weighted key contributions.
    pub axes: HashMap<String, Vec<AxisBinding>>,
}

impl ActionMap {
    /// Builds the default gameplay action and movement axis profile.
    ///
    /// Includes digital actions (`Jump`, `Run`, `Interact`, `Crouch`, `Fire`, `Reload`, `Pause`)
    /// and movement axes (`MoveForward`, `MoveRight`) bound to both WASD and arrow keys.
    #[must_use]
    pub fn default_gameplay() -> Self {
        let mut map = Self {
            name: "Gameplay".to_string(),
            actions: HashMap::new(),
            axes: HashMap::new(),
        };

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
        map.actions.insert(
            "Fire".to_string(),
            vec![KeyCode::KeyF, KeyCode::ControlRight],
        );
        map.actions
            .insert("Reload".to_string(), vec![KeyCode::KeyR]);
        map.actions.insert("Pause".to_string(), vec![KeyCode::KeyP]);

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

    /// Appends a digital key binding to `action_name`, creating the entry if absent.
    pub fn bind_action(&mut self, action_name: &str, key: KeyCode) {
        self.actions
            .entry(action_name.to_string())
            .or_default()
            .push(key);
    }

    /// Appends an axis contribution `(key, scale)` to `axis_name`.
    pub fn bind_axis(&mut self, axis_name: &str, key: KeyCode, scale: f32) {
        self.axes
            .entry(axis_name.to_string())
            .or_default()
            .push(AxisBinding { key, scale });
    }

    /// Replaces the first occurrence of `old_key` in `action_name` with `new_key`.
    ///
    /// Returns `true` if `old_key` was found and updated, or `false` if absent.
    pub fn rebind_action(&mut self, action_name: &str, old_key: KeyCode, new_key: KeyCode) -> bool {
        let Some(slot) = self
            .actions
            .get_mut(action_name)
            .and_then(|keys| keys.iter_mut().find(|k| **k == old_key))
        else {
            return false;
        };
        *slot = new_key;
        true
    }

    /// Returns `true` if any key bound to `action_name` is held in `input`.
    #[must_use]
    pub fn is_action_pressed(&self, input: &InputManager, action_name: &str) -> bool {
        self.actions
            .get(action_name)
            .is_some_and(|keys| keys.iter().any(|&k| input.is_key_pressed(k)))
    }

    /// Returns `true` if any key bound to `action_name` transitioned to pressed this frame.
    #[must_use]
    pub fn is_action_just_pressed(&self, input: &InputManager, action_name: &str) -> bool {
        self.actions
            .get(action_name)
            .is_some_and(|keys| keys.iter().any(|&k| input.is_key_just_pressed(k)))
    }

    /// Returns `true` if any key bound to `action_name` transitioned to released this frame.
    #[must_use]
    pub fn is_action_just_released(&self, input: &InputManager, action_name: &str) -> bool {
        self.actions
            .get(action_name)
            .is_some_and(|keys| keys.iter().any(|&k| input.is_key_just_released(k)))
    }

    /// Sums finite scale weights of all active keys bound to `axis_name`, clamped to `[-1.0, 1.0]`.
    ///
    /// Non-finite scales (e.g. `NaN`) in malformed configs are safely ignored.
    #[must_use]
    pub fn get_axis(&self, input: &InputManager, axis_name: &str) -> f32 {
        let Some(bindings) = self.axes.get(axis_name) else {
            return 0.0;
        };
        let mut value = 0.0f32;
        for b in bindings {
            if input.is_key_pressed(b.key) && b.scale.is_finite() {
                value += b.scale;
            }
        }
        value.clamp(-1.0, 1.0)
    }

    /// Serializes this action map as pretty-printed JSON.
    ///
    /// # Errors
    /// Returns an error string if serialization fails.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    /// Deserializes an [`ActionMap`] from a JSON string.
    ///
    /// # Errors
    /// Returns an error string if JSON parsing or enum decoding fails.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}

impl Default for ActionMap {
    fn default() -> Self {
        Self::default_gameplay()
    }
}