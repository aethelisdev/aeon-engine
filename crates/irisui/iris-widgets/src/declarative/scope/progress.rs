// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Progress Bars & Continuous Loading Slugs
//!
//! Provides determinate and indeterminate animated horizontal progress bar
//! primitives on [`UiScope`].

use super::core::UiScope;
use iris_core::{Color, Style, WidgetRole};

/// Styling and appearance configuration descriptor for customizable progress bars.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressBarConfig {
    /// Foreground fill color of the progress bar slug.
    pub fill_color: Color,
    /// Background fill color for the outer track container.
    pub track_bg: Color,
    /// Outline border color for the outer track container.
    pub track_border: Color,
    /// Corner rounding radius in physical pixels applied to both track and slug.
    pub corner_radius: f32,
}

impl ProgressBarConfig {
    /// Creates a new progress bar configuration with explicit fill and track tokens.
    #[inline]
    #[must_use]
    pub const fn new(
        fill_color: Color,
        track_bg: Color,
        track_border: Color,
        corner_radius: f32,
    ) -> Self {
        Self {
            fill_color,
            track_bg,
            track_border,
            corner_radius,
        }
    }
}

impl<'a> UiScope<'a> {
    /// Emits a determinate progress bar container with a filled slug proportional to `fraction`.
    ///
    /// The outer track container is assigned [`WidgetRole::ProgressBar`] and a default track
    /// height of 4.0 pixels. An inner child node representing the active progress slug is attached
    /// with the clamped fraction stored in its 64-bit tag.
    ///
    /// # Arguments
    /// * `fraction` - Value in range `[0.0, 1.0]` representing the current completion ratio.
    pub fn progress_bar(&mut self, fraction: f32) {
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
    }

    /// Emits a named progress meter with explicit container and fill names, fractional width, and fill color.
    ///
    /// Automatically derives theme-appropriate track background, border, and corner radii
    /// from the active [`Theme`](iris_core::Theme).
    ///
    /// # Arguments
    /// * `track_name` - Descriptive debug identifier for the outer track container.
    /// * `fill_name` - Descriptive debug identifier for the inner filled slug node.
    /// * `width` - Overall track width in physical pixels.
    /// * `height` - Track and fill height in physical pixels.
    /// * `fraction` - Clamped fill ratio in range `[0.0, 1.0]`.
    /// * `fill_color` - Foreground color of the filled progress slug.
    pub fn progress_bar_named(
        &mut self,
        track_name: impl Into<String>,
        fill_name: impl Into<String>,
        width: f32,
        height: f32,
        fraction: f32,
        fill_color: Color,
    ) {
        let config = ProgressBarConfig {
            fill_color,
            track_bg: self.theme().scrollbar_track,
            track_border: self.theme().card_border,
            corner_radius: self.theme().corner_radius_sm,
        };
        self.progress_bar_styled(track_name, fill_name, (width, height), fraction, config);
    }

    /// Emits a fully customizable named progress meter with explicit track colors and corner radius.
    ///
    /// Eliminates manual container styling, nested closure pyramids, and imperative layout plumbing
    /// while giving the caller total control over track colors, border outlines, and corner curvature.
    ///
    /// # Arguments
    /// * `track_name` - Descriptive debug identifier for the outer track container.
    /// * `fill_name` - Descriptive debug identifier for the inner filled slug node.
    /// * `size` - Dimensions `(width, height)` of the track container in physical pixels.
    /// * `fraction` - Clamped fill ratio in range `[0.0, 1.0]`.
    /// * `config` - Visual configuration descriptor governing track/fill colors and curvature.
    pub fn progress_bar_styled(
        &mut self,
        track_name: impl Into<String>,
        fill_name: impl Into<String>,
        size: (f32, f32),
        fraction: f32,
        config: ProgressBarConfig,
    ) {
        let clamped = fraction.clamp(0.0, 1.0);
        let track_style = Style::new()
            .width(size.0)
            .height(size.1)
            .background(config.track_bg)
            .border(1.0, config.track_border)
            .border_radius(config.corner_radius);

        self.container_named(track_name, track_style, |track| {
            if clamped > 0.001 {
                let fill_w = (size.0 * clamped).max(2.0);
                let fill_style = Style::new()
                    .width(fill_w)
                    .height(size.1)
                    .background(config.fill_color)
                    .border_radius(config.corner_radius);
                track.empty_box_passive_named(fill_name, fill_style);
            }
        });
    }

    /// Emits an indeterminate sliding progress bar container with an animated runner slug.
    ///
    /// The outer track container is assigned [`WidgetRole::ProgressBar`] and a default track
    /// height of 4.0 pixels. The child slug animates across the track based on `slide_phase`.
    ///
    /// # Arguments
    /// * `slide_phase` - Normalized phase in range `[0.0, 1.0]` controlling horizontal travel.
    pub fn indeterminate_progress(&mut self, slide_phase: f32) {
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
    }
}