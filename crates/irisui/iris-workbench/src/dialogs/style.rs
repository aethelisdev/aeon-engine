// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Modal Dialogue Styling and Chromatic Configuration
//!
//! Provides customizable visual attributes for modal dialog cards, backdrop blockers,
//! header bars, and action buttons, pre-configured with dark studio defaults.

use iris_core::Color;

/// Visual and chromatic configuration for modal dialogues and confirmation cards.
///
/// Encapsulates full-screen scrim opacity, modal card elevation styling, border thickness,
/// corner rounding, internal padding, and standard action button geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct ModalStyle {
    /// Semi-transparent fill color of the full-screen backdrop scrim blocker.
    pub backdrop_color: Color,

    /// Fill color of the elevated modal card surface.
    pub card_background: Color,

    /// Stroke color of the modal card border perimeter.
    pub card_border_color: Color,

    /// Stroke thickness of the modal card perimeter border in logical pixels.
    pub card_border_width: f32,

    /// Radius applied to all four corners of the modal card in logical pixels.
    pub card_corner_radius: f32,

    /// Internal padding applied within the modal card perimeter.
    pub card_padding: f32,

    /// Vertical gap between header, body column, and footer button row.
    pub content_gap: f32,

    /// Horizontal gap between footer action buttons in logical pixels.
    pub footer_gap: f32,

    /// Primary header text color for the title subheading.
    pub header_text_color: Color,

    /// Secondary subtitle text color for descriptive prompt text.
    pub subtitle_color: Color,

    /// Chromatic tone used for high-visibility destructive/danger warning text.
    pub danger_color: Color,

    /// Standard width allocated for cancel action buttons in logical pixels.
    pub cancel_button_width: f32,

    /// Standard width allocated for confirm / danger action buttons in logical pixels.
    pub confirm_button_width: f32,
}

impl Default for ModalStyle {
    /// Creates a dark studio default modal styling matching professional creative suites.
    fn default() -> Self {
        Self {
            backdrop_color: Color::rgba(0.0, 0.0, 0.0, 0.60),
            card_background: Color::rgba(0.12, 0.13, 0.16, 0.98),
            card_border_color: Color::rgba(0.24, 0.26, 0.32, 1.0),
            card_border_width: 1.0,
            card_corner_radius: 8.0,
            card_padding: 16.0,
            content_gap: 12.0,
            footer_gap: 8.0,
            header_text_color: Color::rgba(0.92, 0.94, 0.98, 1.0),
            subtitle_color: Color::rgba(0.60, 0.64, 0.72, 1.0),
            danger_color: Color::rgba(1.0, 0.45, 0.45, 1.0),
            cancel_button_width: 80.0,
            confirm_button_width: 120.0,
        }
    }
}

impl ModalStyle {
    /// Creates a new modal style with dark studio defaults.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new modal style with dark studio theme colors and spacing.
    #[inline]
    #[must_use]
    pub fn dark_studio() -> Self {
        Self::default()
    }

    /// Creates a clean light studio modal styling preset.
    #[inline]
    #[must_use]
    pub fn light_studio() -> Self {
        Self {
            backdrop_color: Color::rgba(0.0, 0.0, 0.0, 0.40),
            card_background: Color::rgba(0.96, 0.97, 0.98, 0.98),
            card_border_color: Color::rgba(0.80, 0.82, 0.86, 1.0),
            card_border_width: 1.0,
            card_corner_radius: 8.0,
            card_padding: 16.0,
            content_gap: 12.0,
            footer_gap: 8.0,
            header_text_color: Color::rgba(0.12, 0.14, 0.18, 1.0),
            subtitle_color: Color::rgba(0.35, 0.38, 0.45, 1.0),
            danger_color: Color::rgba(0.85, 0.15, 0.15, 1.0),
            cancel_button_width: 80.0,
            confirm_button_width: 120.0,
        }
    }

    /// Sets the semi-transparent backdrop blocker fill color.
    #[inline]
    #[must_use]
    pub const fn with_backdrop_color(mut self, color: Color) -> Self {
        self.backdrop_color = color;
        self
    }

    /// Sets the fill color of the elevated modal card surface.
    #[inline]
    #[must_use]
    pub const fn with_card_background(mut self, color: Color) -> Self {
        self.card_background = color;
        self
    }

    /// Sets the border stroke color and width of the modal card.
    #[inline]
    #[must_use]
    pub const fn with_card_border(mut self, color: Color, width: f32) -> Self {
        self.card_border_color = color;
        self.card_border_width = width;
        self
    }

    /// Sets the corner rounding radius of the modal card in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_corner_radius(mut self, radius: f32) -> Self {
        self.card_corner_radius = radius;
        self
    }

    /// Sets the internal perimeter padding of the modal card in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_card_padding(mut self, padding: f32) -> Self {
        self.card_padding = padding;
        self
    }

    /// Sets the vertical spacing gap between modal sections in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_content_gap(mut self, gap: f32) -> Self {
        self.content_gap = gap;
        self
    }

    /// Sets the horizontal spacing gap between footer buttons in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_footer_gap(mut self, gap: f32) -> Self {
        self.footer_gap = gap;
        self
    }

    /// Sets the primary header text color.
    #[inline]
    #[must_use]
    pub const fn with_header_text_color(mut self, color: Color) -> Self {
        self.header_text_color = color;
        self
    }

    /// Sets the secondary subtitle text color.
    #[inline]
    #[must_use]
    pub const fn with_subtitle_color(mut self, color: Color) -> Self {
        self.subtitle_color = color;
        self
    }

    /// Sets the high-visibility destructive / danger warning text color.
    #[inline]
    #[must_use]
    pub const fn with_danger_color(mut self, color: Color) -> Self {
        self.danger_color = color;
        self
    }

    /// Sets the action button widths for cancel and confirm operations.
    #[inline]
    #[must_use]
    pub const fn with_button_widths(mut self, cancel: f32, confirm: f32) -> Self {
        self.cancel_button_width = cancel;
        self.confirm_button_width = confirm;
        self
    }

    /// Sets the confirmation button width in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_confirm_button_width(mut self, width: f32) -> Self {
        self.confirm_button_width = width;
        self
    }

    /// Sets the cancel button width in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_cancel_button_width(mut self, width: f32) -> Self {
        self.cancel_button_width = width;
        self
    }
}