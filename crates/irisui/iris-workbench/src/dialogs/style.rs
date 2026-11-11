// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Modal Dialogue Styling and Chromatic Configuration
//!
//! Provides customizable visual attributes for modal dialog cards, backdrop blockers,
//! header bars, and action buttons, dynamically resolving colors against the active [`iris_core::Theme`].

use iris_core::{Color, Theme};

/// Visual and chromatic configuration for modal dialogues and confirmation cards.
///
/// Encapsulates full-screen scrim opacity, modal card elevation styling, border thickness,
/// corner rounding, internal padding, and standard action button geometry.
///
/// Color fields are optional overrides: when `None`, colors are dynamically resolved
/// from the application's active [`Theme`].
#[derive(Debug, Clone, PartialEq)]
pub struct ModalStyle {
    /// Semi-transparent fill color of the full-screen backdrop scrim blocker.
    /// When `None`, dynamically resolves from [`Theme::modal_scrim`].
    pub backdrop_color: Option<Color>,

    /// Fill color of the elevated modal card surface.
    /// When `None`, dynamically resolves from [`Theme::modal_bg`].
    pub card_background: Option<Color>,

    /// Stroke color of the modal card border perimeter.
    /// When `None`, dynamically resolves from [`Theme::modal_border`].
    pub card_border_color: Option<Color>,

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
    /// When `None`, dynamically resolves from [`Theme::text_primary`].
    pub header_text_color: Option<Color>,

    /// Secondary subtitle text color for descriptive prompt text.
    /// When `None`, dynamically resolves from [`Theme::text_secondary`].
    pub subtitle_color: Option<Color>,

    /// Chromatic tone used for high-visibility destructive/danger warning text.
    /// When `None`, dynamically resolves from [`Theme::danger`].
    pub danger_color: Option<Color>,

    /// Standard width allocated for cancel action buttons in logical pixels.
    pub cancel_button_width: f32,

    /// Standard width allocated for confirm / danger action buttons in logical pixels.
    pub confirm_button_width: f32,

    /// Highlight border color pulsed when user clicks the backdrop of a non-dismissible modal.
    /// When `None`, dynamically resolves from [`Theme::accent`].
    pub attention_border_color: Option<Color>,

    /// Peak lateral shake displacement amplitude in logical pixels for attention feedback.
    pub attention_shake_amplitude: f32,
}

impl Default for ModalStyle {
    /// Creates a default modal style with zero hardcoded colors, fully delegating to the active theme.
    fn default() -> Self {
        Self {
            backdrop_color: None,
            card_background: None,
            card_border_color: None,
            card_border_width: 1.0,
            card_corner_radius: 8.0,
            card_padding: 16.0,
            content_gap: 12.0,
            footer_gap: 8.0,
            header_text_color: None,
            subtitle_color: None,
            danger_color: None,
            cancel_button_width: 80.0,
            confirm_button_width: 120.0,
            attention_border_color: None,
            attention_shake_amplitude: 6.0,
        }
    }
}

impl ModalStyle {
    /// Creates a new modal style with theme-driven defaults.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new modal style with theme-driven defaults.
    #[inline]
    #[must_use]
    pub fn dark_studio() -> Self {
        Self::default()
    }

    /// Constructs a modal style explicitly bound to the visual chromas of a [`Theme`].
    #[must_use]
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            backdrop_color: Some(theme.modal_scrim),
            card_background: Some(theme.modal_bg),
            card_border_color: Some(theme.modal_border),
            card_border_width: 1.0,
            card_corner_radius: 8.0,
            card_padding: 16.0,
            content_gap: 12.0,
            footer_gap: 8.0,
            header_text_color: Some(theme.text_primary),
            subtitle_color: Some(theme.text_secondary),
            danger_color: Some(theme.danger),
            cancel_button_width: 80.0,
            confirm_button_width: 120.0,
            attention_border_color: Some(theme.accent),
            attention_shake_amplitude: 6.0,
        }
    }

    /// Creates a clean light studio modal styling preset with explicit color overrides.
    #[inline]
    #[must_use]
    pub fn light_studio() -> Self {
        Self {
            backdrop_color: Some(Color::rgba(0.0, 0.0, 0.0, 0.40)),
            card_background: Some(Color::rgba(0.96, 0.97, 0.98, 0.98)),
            card_border_color: Some(Color::rgba(0.80, 0.82, 0.86, 1.0)),
            card_border_width: 1.0,
            card_corner_radius: 8.0,
            card_padding: 16.0,
            content_gap: 12.0,
            footer_gap: 8.0,
            header_text_color: Some(Color::rgba(0.12, 0.14, 0.18, 1.0)),
            subtitle_color: Some(Color::rgba(0.35, 0.38, 0.45, 1.0)),
            danger_color: Some(Color::rgba(0.85, 0.15, 0.15, 1.0)),
            cancel_button_width: 80.0,
            confirm_button_width: 120.0,
            attention_border_color: Some(Color::rgba(0.20, 0.50, 0.90, 1.0)),
            attention_shake_amplitude: 6.0,
        }
    }

    /// Resolves the effective backdrop scrim color against the provided active [`Theme`].
    #[inline]
    #[must_use]
    pub fn resolve_backdrop_color(&self, theme: &Theme) -> Color {
        self.backdrop_color.unwrap_or(theme.modal_scrim)
    }

    /// Resolves the effective card surface background color against the provided active [`Theme`].
    #[inline]
    #[must_use]
    pub fn resolve_card_background(&self, theme: &Theme) -> Color {
        self.card_background.unwrap_or(theme.modal_bg)
    }

    /// Resolves the effective card border stroke color against the provided active [`Theme`].
    #[inline]
    #[must_use]
    pub fn resolve_card_border_color(&self, theme: &Theme) -> Color {
        self.card_border_color.unwrap_or(theme.modal_border)
    }

    /// Resolves the primary header text color against the provided active [`Theme`].
    #[inline]
    #[must_use]
    pub fn resolve_header_text_color(&self, theme: &Theme) -> Color {
        self.header_text_color.unwrap_or(theme.text_primary)
    }

    /// Resolves the secondary subtitle/description text color against the provided active [`Theme`].
    #[inline]
    #[must_use]
    pub fn resolve_subtitle_color(&self, theme: &Theme) -> Color {
        self.subtitle_color.unwrap_or(theme.text_secondary)
    }

    /// Resolves the destructive / danger warning text color against the provided active [`Theme`].
    #[inline]
    #[must_use]
    pub fn resolve_danger_color(&self, theme: &Theme) -> Color {
        self.danger_color.unwrap_or(theme.danger)
    }

    /// Resolves the attention highlight border color against the provided active [`Theme`].
    #[inline]
    #[must_use]
    pub fn resolve_attention_border_color(&self, theme: &Theme) -> Color {
        self.attention_border_color.unwrap_or(theme.accent)
    }

    /// Sets the semi-transparent backdrop blocker fill color.
    #[inline]
    #[must_use]
    pub const fn with_backdrop_color(mut self, color: Color) -> Self {
        self.backdrop_color = Some(color);
        self
    }

    /// Sets the fill color of the elevated modal card surface.
    #[inline]
    #[must_use]
    pub const fn with_card_background(mut self, color: Color) -> Self {
        self.card_background = Some(color);
        self
    }

    /// Sets the border stroke color and width of the modal card.
    #[inline]
    #[must_use]
    pub const fn with_card_border(mut self, color: Color, width: f32) -> Self {
        self.card_border_color = Some(color);
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
        self.header_text_color = Some(color);
        self
    }

    /// Sets the secondary subtitle text color.
    #[inline]
    #[must_use]
    pub const fn with_subtitle_color(mut self, color: Color) -> Self {
        self.subtitle_color = Some(color);
        self
    }

    /// Sets the high-visibility destructive / danger warning text color.
    #[inline]
    #[must_use]
    pub const fn with_danger_color(mut self, color: Color) -> Self {
        self.danger_color = Some(color);
        self
    }

    /// Sets the border stroke color of the modal card.
    #[inline]
    #[must_use]
    pub const fn with_card_border_color(mut self, color: Color) -> Self {
        self.card_border_color = Some(color);
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

    /// Sets the border glow color pulsed when attention is requested on backdrop clicks.
    #[inline]
    #[must_use]
    pub const fn with_attention_border_color(mut self, color: Color) -> Self {
        self.attention_border_color = Some(color);
        self
    }

    /// Sets the peak lateral shake displacement amplitude in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_attention_shake_amplitude(mut self, amplitude: f32) -> Self {
        self.attention_shake_amplitude = amplitude;
        self
    }
}