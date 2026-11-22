// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Modal Descriptor Configuration Type
//!
//! Defines [`ModalDescriptor`] for declarative modal windows rendered via
//! [`UiScope::modal`](super::core::UiScope::modal) and [`UiScope::modal_tagged`](super::core::UiScope::modal_tagged).

use irisui_core::Color;

/// Configuration descriptor for a declarative modal dialog window.
#[derive(Debug, Clone)]
pub struct ModalDescriptor {
    /// Window title caption displayed in the modal header bar.
    pub title: String,
    /// Width constraint in logical pixels.
    pub width: f32,
    /// Height constraint in logical pixels.
    pub height: f32,
    /// Whether clicking the semi-transparent backdrop scrim dismisses the modal.
    pub dismiss_on_scrim: bool,
    /// Whether pressing the Escape key dismisses the modal.
    pub dismiss_on_escape: bool,
    /// Whether to render the standard compact '✕' close button in the header.
    pub show_close_button: bool,
    /// Optional custom backdrop scrim fill color.
    pub scrim_color: Option<Color>,
}

impl ModalDescriptor {
    /// Constructs a new modal dialog descriptor with standard desktop IDE defaults.
    ///
    /// Defaults:
    /// - `dismiss_on_scrim`: `true`
    /// - `dismiss_on_escape`: `true`
    /// - `show_close_button`: `true`
    /// - `scrim_color`: `None` (uses default theme scrim color)
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            dismiss_on_scrim: true,
            dismiss_on_escape: true,
            show_close_button: true,
            scrim_color: None,
        }
    }

    /// Sets whether clicking the backdrop scrim dismisses the dialog.
    #[inline]
    #[must_use]
    pub fn with_dismiss_on_scrim(mut self, enabled: bool) -> Self {
        self.dismiss_on_scrim = enabled;
        self
    }

    /// Sets whether pressing the Escape key dismisses the dialog.
    #[inline]
    #[must_use]
    pub fn with_dismiss_on_escape(mut self, enabled: bool) -> Self {
        self.dismiss_on_escape = enabled;
        self
    }

    /// Sets whether the header '✕' close button is displayed.
    #[inline]
    #[must_use]
    pub fn with_close_button(mut self, enabled: bool) -> Self {
        self.show_close_button = enabled;
        self
    }

    /// Overrides the default semi-transparent backdrop scrim color.
    #[inline]
    #[must_use]
    pub fn with_scrim_color(mut self, color: Color) -> Self {
        self.scrim_color = Some(color);
        self
    }
}

impl From<(&str, f32, f32)> for ModalDescriptor {
    #[inline]
    fn from(val: (&str, f32, f32)) -> Self {
        Self::new(val.0, val.1, val.2)
    }
}

impl From<(String, f32, f32)> for ModalDescriptor {
    #[inline]
    fn from(val: (String, f32, f32)) -> Self {
        Self::new(val.0, val.1, val.2)
    }
}