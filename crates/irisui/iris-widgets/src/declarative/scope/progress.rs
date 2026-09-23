// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Progress Bars & Continuous Loading Slugs
//!
//! Provides determinate and indeterminate animated horizontal progress bar
//! primitives on [`UiScope`].
//!

use super::core::UiScope;
use iris_core::{Color, Style, WidgetId, WidgetRole};

impl<'a> UiScope<'a> {
    /// Emits a determinate progress bar container with a filled slug proportional to `fraction`.
    ///
    /// The outer track container is assigned [`WidgetRole::ProgressBar`] and a default track
    /// height of 4.0 pixels. An inner child node representing the active progress slug is attached
    /// with the clamped fraction stored in its 64-bit tag.
    ///
    /// # Arguments
    /// * `fraction` - Value in range `[0.0, 1.0]` representing the current completion ratio.
    ///
    /// Returns the allocated [`WidgetId`] of the track container.
    pub fn progress_bar(&mut self, fraction: f32) -> WidgetId {
        let track_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(track_id) {
            node.role = WidgetRole::ProgressBar;
            node.set_style(
                Style::new()
                    .height(4.0)
                    .border_radius(2.0)
                    .background(Color::rgba(0.08, 0.09, 0.12, 0.95))
                    .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.80)),
            );
        }
        let _ = self.tree.add_child(self.parent, track_id);

        let slug_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(slug_id) {
            node.set_name("ProgressSlug");
            node.tag = fraction.clamp(0.0, 1.0).to_bits() as u64;
            node.set_style(
                Style::new()
                    .border_radius(2.0)
                    .background(Color::rgba(0.0, 0.88, 1.0, 0.95)),
            );
        }
        let _ = self.tree.add_child(track_id, slug_id);

        track_id
    }

    /// Emits an indeterminate sliding progress bar container with an animated runner slug.
    ///
    /// The outer track container is assigned [`WidgetRole::ProgressBar`] and a default track
    /// height of 4.0 pixels. The child slug animates across the track based on `slide_phase`.
    ///
    /// # Arguments
    /// * `slide_phase` - Normalized phase in range `[0.0, 1.0]` controlling horizontal travel.
    ///
    /// Returns the allocated [`WidgetId`] of the track container.
    pub fn indeterminate_progress(&mut self, slide_phase: f32) -> WidgetId {
        let track_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(track_id) {
            node.role = WidgetRole::ProgressBar;
            node.set_style(
                Style::new()
                    .height(4.0)
                    .border_radius(2.0)
                    .background(Color::rgba(0.08, 0.09, 0.12, 0.95))
                    .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.80)),
            );
        }
        let _ = self.tree.add_child(self.parent, track_id);

        let slug_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(slug_id) {
            node.set_name("IndeterminateProgressSlug");
            node.tag = slide_phase.clamp(0.0, 1.0).to_bits() as u64;
            node.set_style(
                Style::new()
                    .border_radius(2.0)
                    .background(Color::rgba(0.0, 0.88, 1.0, 0.95)),
            );
        }
        let _ = self.tree.add_child(track_id, slug_id);

        track_id
    }
}