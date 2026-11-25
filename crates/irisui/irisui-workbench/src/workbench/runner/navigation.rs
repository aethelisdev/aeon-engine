// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench UI Zoom and Scale Navigation Controls
//!
//! Provides scale factor calculation, step-based zooming across predefined presets,
//! and zoom reset utilities on [`Workbench`].

use super::coordinator::Workbench;
use std::fmt::Debug;
use std::hash::Hash;

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
    Action: Send + Sync + 'static,
{
    /// Returns the active UI scale factor clamped to the valid range `[0.6, 2.0]`.
    #[inline]
    #[must_use]
    pub fn scale_factor(&self) -> f32 {
        if self.zoom_factor.is_finite() && self.zoom_factor > 0.1 {
            self.zoom_factor.clamp(0.6, 2.0)
        } else {
            1.0
        }
    }

    /// Steps the active UI zoom factor up or down across the predefined scale presets.
    ///
    /// Presets: `[0.75, 0.80, 0.90, 1.00, 1.10, 1.25, 1.50]`.
    pub fn step_zoom(&mut self, increase: bool) -> f32 {
        const PRESETS: [f32; 7] = [0.75, 0.80, 0.90, 1.00, 1.10, 1.25, 1.50];
        let current = self.scale_factor();
        let target = if increase {
            PRESETS
                .iter()
                .copied()
                .find(|&s| s > current + 0.01)
                .unwrap_or(*PRESETS.last().unwrap_or(&1.50))
        } else {
            PRESETS
                .iter()
                .copied()
                .rfind(|&s| s < current - 0.01)
                .unwrap_or(*PRESETS.first().unwrap_or(&0.75))
        };
        self.zoom_factor = target;
        target
    }

    /// Sets the active UI zoom scale factor clamped to `[0.6, 2.0]` (defaulting to `1.0` for
    /// non-finite or non-positive values) and forces shell/layout invalidation on change.
    #[inline]
    pub fn set_scale_factor(&mut self, scale_factor: f32) -> f32 {
        let clamped = if scale_factor.is_finite() && scale_factor > 0.1 {
            scale_factor.clamp(0.6, 2.0)
        } else {
            1.0
        };
        if (self.zoom_factor - clamped).abs() > f32::EPSILON {
            self.zoom_factor = clamped;
            self.lifecycle.mark_shell_dirty();
            self.notifier.tag_all();
        }
        self.zoom_factor
    }

    /// Resets the UI zoom factor to default 100% (1.0).
    #[inline]
    pub fn reset_zoom(&mut self) -> f32 {
        self.zoom_factor = 1.0;
        1.0
    }
}