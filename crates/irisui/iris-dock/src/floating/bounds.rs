// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Screen bounding constraints, viewport margin clamping, and docking coordinate validation.
//!
//! Enforces that floating surfaces remain interactable, within user-visible screen extents,
//! and never collapse below ergonomically usable minimum dimensions.

use crate::floating::model::{FloatingWindow, FloatingWindowStyle};

/// Configuration parameters for constraining floating windows within visible screen boundaries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingWindowClampBounds {
    /// Total screen or viewport width.
    pub screen_width: f32,
    /// Total screen or viewport height.
    pub screen_height: f32,
    /// Minimum allowed Y coordinate (e.g. below a top menu bar). Defaults to 0.0.
    pub min_y: f32,
    /// Bottom inset height reserved for status bars or docking bars. Defaults to 0.0.
    pub bottom_inset: f32,
    /// Minimum allowed window width. Defaults to 220.0.
    pub min_width: f32,
    /// Minimum allowed window height. Defaults to 140.0.
    pub min_height: f32,
    /// Minimum visible grab width kept on screen so the window cannot be lost horizontally. Defaults to 60.0.
    pub min_visible_margin: f32,
    /// Title bar height used to ensure the title bar remains interactable. Defaults to 26.0.
    pub title_bar_height: f32,
}

impl Default for FloatingWindowClampBounds {
    fn default() -> Self {
        Self {
            screen_width: 1920.0,
            screen_height: 1080.0,
            min_y: 0.0,
            bottom_inset: 0.0,
            min_width: 220.0,
            min_height: 140.0,
            min_visible_margin: 60.0,
            title_bar_height: FloatingWindowStyle::DEFAULT_TITLE_BAR_HEIGHT,
        }
    }
}

impl FloatingWindowClampBounds {
    /// Creates a clamp bounds specification with the given screen dimensions.
    #[inline]
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
            ..Default::default()
        }
    }

    /// Sets the minimum Y boundary (e.g. below a top menu bar).
    #[inline]
    pub fn with_min_y(mut self, min_y: f32) -> Self {
        self.min_y = min_y;
        self
    }

    /// Sets the bottom reserved inset height (e.g. status bar).
    #[inline]
    pub fn with_bottom_inset(mut self, bottom_inset: f32) -> Self {
        self.bottom_inset = bottom_inset;
        self
    }

    /// Sets custom minimum window dimensions.
    #[inline]
    pub fn with_min_size(mut self, min_width: f32, min_height: f32) -> Self {
        self.min_width = min_width;
        self.min_height = min_height;
        self
    }

    /// Sets the minimum horizontal grab margin kept on screen.
    #[inline]
    pub fn with_min_visible_margin(mut self, margin: f32) -> Self {
        self.min_visible_margin = margin;
        self
    }
}

impl<T> FloatingWindow<T> {
    /// Clamps this window's geometry and coordinates to remain within the specified screen bounds.
    ///
    /// Ensures minimum width and height constraints are met, keeps the title bar accessible
    /// between `min_y` and screen bottom, and guarantees a grab handle margin stays horizontally visible.
    pub fn clamp_to_bounds(&mut self, bounds: &FloatingWindowClampBounds) {
        let available_h = (bounds.screen_height - bounds.min_y - bounds.bottom_inset)
            .max(bounds.title_bar_height);
        let max_y = (bounds.screen_height - bounds.bottom_inset - bounds.title_bar_height)
            .max(bounds.min_y);

        if bounds.screen_width > 100.0 {
            self.rect.width = self.rect.width.clamp(bounds.min_width, bounds.screen_width);
        }
        if available_h > bounds.title_bar_height {
            self.rect.height = self.rect.height.clamp(bounds.min_height, available_h);
        }

        self.rect.y = self.rect.y.clamp(bounds.min_y, max_y);

        let max_x = (bounds.screen_width - bounds.min_visible_margin).max(0.0);
        let min_x = (bounds.min_visible_margin - self.rect.width).min(0.0);
        self.rect.x = self.rect.x.clamp(min_x, max_x);
    }
}

/// Clamps a slice of floating windows to remain accessible and within visible viewport boundaries.
#[inline]
pub fn clamp_floating_windows<T>(
    windows: &mut [FloatingWindow<T>],
    bounds: &FloatingWindowClampBounds,
) {
    for win in windows {
        win.clamp_to_bounds(bounds);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::geometry::Rect;

    #[test]
    fn test_floating_window_clamping() {
        let mut window = FloatingWindow::new(
            1,
            "Clamp Test",
            Rect::new(-500.0, -100.0, 50.0, 50.0),
            vec!["TabA"],
        );

        let bounds = FloatingWindowClampBounds::new(1920.0, 1080.0)
            .with_min_y(32.0)
            .with_bottom_inset(24.0);

        window.clamp_to_bounds(&bounds);

        // Min dimensions enforced
        assert_eq!(window.rect.width, 220.0);
        assert_eq!(window.rect.height, 140.0);

        // Y clamped to min_y (32.0)
        assert_eq!(window.rect.y, 32.0);

        // X clamped so at least 60px remains visible on left: min_x = 60.0 - 220.0 = -160.0
        assert_eq!(window.rect.x, -160.0);

        // Push window far right and bottom
        window.rect.x = 2500.0;
        window.rect.y = 2000.0;
        window.clamp_to_bounds(&bounds);

        // X clamped so at least 60px remains visible: max_x = 1920.0 - 60.0 = 1860.0
        assert_eq!(window.rect.x, 1860.0);

        // Y clamped to max_y = 1080 - 24 - 26 = 1030.0
        assert_eq!(window.rect.y, 1030.0);
    }

    #[test]
    fn test_batch_clamp_floating_windows() {
        let mut windows = vec![
            FloatingWindow::new(1, "W1", Rect::new(0.0, 0.0, 100.0, 100.0), vec!["A"]),
            FloatingWindow::new(2, "W2", Rect::new(5000.0, 5000.0, 100.0, 100.0), vec!["B"]),
        ];

        let bounds = FloatingWindowClampBounds::new(800.0, 600.0).with_min_y(30.0);
        clamp_floating_windows(&mut windows, &bounds);

        assert_eq!(windows[0].rect.y, 30.0);
        assert_eq!(windows[0].rect.width, 220.0);
        assert_eq!(windows[1].rect.x, 800.0 - 60.0);
    }
}