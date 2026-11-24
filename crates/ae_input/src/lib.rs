// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Three-state keyboard/mouse input manager and serializable virtual action/axis mapping.

/// Logical action and analog axis binding table with JSON persistence.
pub mod action_map;
/// Per-frame continuous and one-shot keyboard/mouse input state tracker.
pub mod manager;

pub use action_map::{ActionMap, AxisBinding};
pub use manager::InputManager;
pub use winit::event::{ElementState, MouseButton};
pub use winit::keyboard::KeyCode;

#[cfg(test)]
mod tests;