// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use serde::{Deserialize, Serialize};

/// Active dimension mode governing the Aeon Engine runtime and memory budget allocation.
/// Ensures strict mutual exclusion:
/// - In `Mode2D`, 3D pipeline resources (Cascaded Shadow Maps, Skybox Cubemaps, PBR IBL textures,
///   and 3D mesh buffers) are completely bypassed and never allocated in GPU/CPU memory.
/// - In `Mode3D`, 2D sprite batching and quad instancing buffers are never allocated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ActiveDimensionMode {
    /// 2D Sprite mode: Lightweight, orthographic, tile/sprite batching without 3D lighting or shadow passes.
    #[default]
    Mode2D,

    /// 3D Spatial mode: Full PBR, deferred/forward passes, cascade shadows, skeletal mesh animation.
    Mode3D,
}

impl ActiveDimensionMode {
    /// Returns whether 2D sprite rendering and 2D physics are currently active.
    pub fn is_2d(&self) -> bool {
        matches!(self, Self::Mode2D)
    }

    /// Returns whether 3D rendering and 3D spatial pipelines are currently active.
    pub fn is_3d(&self) -> bool {
        matches!(self, Self::Mode3D)
    }
}