// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! ECS lighting component definitions shared across scene serialization, editor tools,
//! gizmo billboards, and rendering extraction.

use serde::{Deserialize, Serialize};

/// Point light ECS component storing world-space position and normalized RGB color.
///
/// Serialized in `.aeworld` archives via `ComponentRegistry`, edited directly through
/// inspector cards with Undo/Redo support, and rendered in Edit mode via light billboards.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Light {
    /// World-space light source position (`[x, y, z]`).
    pub position: [f32; 3],
    /// Linear/sRGB light emission tint (`[r, g, b]`).
    pub color: [f32; 3],
}

impl Default for Light {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0],
        }
    }
}