// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Grid-snapped rotation (angle snapping).
pub mod rotate;
/// Grid-snapped scale quantization.
pub mod scale;
/// Grid-snapped position translation.
pub mod translate;

/// Snapping activation mode for transform operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapMode {
    /// Snapping disabled.
    Off,
    /// Snapping active while a modifier key is held (e.g., Ctrl).
    Hold,
    /// Snapping toggled on/off by a key press.
    Toggle,
}

/// Grid-based snapping configuration for transform operations.
#[derive(Debug, Clone, Copy)]
pub struct SnapSettings {
    /// Current snap activation mode.
    pub mode: SnapMode,
    /// Whether snapping is currently active this frame.
    pub current_enabled: bool,
    /// Grid cell size in world units.
    pub grid_size: f32,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            mode: SnapMode::Hold,
            current_enabled: false,
            grid_size: 1.0,
        }
    }
}