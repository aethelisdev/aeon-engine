// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Viewport rectangle bounds and sub-region descriptor types.

/// Screen-space logical boundaries of a rendered 3D viewport panel.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ViewportRect {
    /// Left boundary in logical pixels.
    pub min_x: f32,
    /// Top boundary in logical pixels.
    pub min_y: f32,
    /// Right boundary in logical pixels.
    pub max_x: f32,
    /// Bottom boundary in logical pixels.
    pub max_y: f32,
}

impl ViewportRect {
    /// Returns `true` if logical coordinates `(x, y)` fall inside the viewport boundaries.
    #[inline]
    #[must_use]
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }
}

/// Viewport descriptor for sub-region rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Top-left sub-region offset `[x, y]`.
    pub position: [f32; 2],
    /// Sub-region extent `[width, height]`.
    pub size: [f32; 2],
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            size: [800.0, 600.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_rect_contains_and_inverted_rejection() {
        let rect = ViewportRect {
            min_x: 10.0,
            min_y: 20.0,
            max_x: 110.0,
            max_y: 220.0,
        };
        assert!(rect.contains(10.0, 20.0));
        assert!(rect.contains(60.0, 120.0));
        assert!(!rect.contains(9.9, 120.0));
        assert!(!rect.contains(60.0, 220.1));

        let inverted = ViewportRect {
            min_x: 100.0,
            min_y: 100.0,
            max_x: 10.0,
            max_y: 10.0,
        };
        assert!(!inverted.contains(50.0, 50.0));
    }
}