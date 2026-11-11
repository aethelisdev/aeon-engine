// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Generic Progress Dialogue Descriptor
//!
//! Declarative configuration structure specifying visual parameters, progress ratio,
//! status messages, and cancellation behaviors for long-running task modals.

use crate::dialogs::style::ModalStyle;
use iris_core::Color;
use iris_core::geometry::Point;

/// Declarative descriptor for unopinionated long-running operation progress modals.
///
/// Coordinates progress bars, dynamic status text, optional cancellation actions, and
/// backdrop blocking without hardcoded text or framework-level style locks.
pub struct ProgressModalDescriptor<Action = ()>
where
    Action: 'static + Send + Sync,
{
    /// Dialog title displayed in the header bar.
    pub title: String,
    /// Current operation status message displayed above the progress meter.
    pub status_message: Option<String>,
    /// Progress completion ratio clamped to range `[0.0, 1.0]`.
    pub progress: f32,
    /// Optional foreground fill color override for the progress bar.
    pub progress_color: Option<Color>,
    /// Height of the progress bar in physical pixels.
    pub progress_height: f32,
    /// Whether the long-running operation permits user cancellation.
    pub is_cancellable: bool,
    /// Caller-configurable label for the cancel button.
    pub cancel_label: String,
    /// Action emitted into [`ActionBus`](crate::action_bus::ActionBus) on cancellation.
    pub on_cancel: Option<Action>,
    /// Optional subtitle or auxiliary description.
    pub subtitle: Option<String>,
    /// Optional header icon / glyph.
    pub icon: Option<String>,
    /// Modal card dimensions `(width, height)` in physical pixels.
    pub size: (f32, f32),
    /// Optional explicit viewport position. Centered when `None`.
    pub position: Option<Point>,
    /// Optional background scrim color override.
    pub backdrop_color: Option<Color>,
    /// Visual styling for the modal card, borders, and typography.
    pub style: ModalStyle,
    /// Whether clicking the semi-transparent backdrop triggers cancellation (if cancellable).
    pub dismiss_on_backdrop: bool,
}

impl<Action: 'static + Send + Sync> ProgressModalDescriptor<Action> {
    /// Default modal card dimensions `(width, height)` for progress dialogs.
    pub const DEFAULT_SIZE: (f32, f32) = (460.0, 200.0);

    /// Default progress bar height in physical pixels.
    pub const DEFAULT_PROGRESS_HEIGHT: f32 = 6.0;

    /// Default unopinionated fallback label for the cancel button.
    pub const DEFAULT_CANCEL_LABEL: &'static str = "Cancel";

    /// Creates a new progress modal descriptor with default geometry and styling.
    ///
    /// By default, the dialog is cancellable, uses the default cancel label, and
    /// clamps the initial progress ratio between 0.0 and 1.0.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            status_message: None,
            progress: 0.0,
            progress_color: None,
            progress_height: Self::DEFAULT_PROGRESS_HEIGHT,
            is_cancellable: true,
            cancel_label: String::from(Self::DEFAULT_CANCEL_LABEL),
            on_cancel: None,
            subtitle: None,
            icon: None,
            size: Self::DEFAULT_SIZE,
            position: None,
            backdrop_color: None,
            style: ModalStyle::default(),
            dismiss_on_backdrop: false,
        }
    }

    /// Sets the dynamic operation status message displayed above the progress meter.
    #[must_use]
    pub fn with_status_message(mut self, message: impl Into<String>) -> Self {
        self.status_message = Some(message.into());
        self
    }

    /// Sets the progress completion ratio, clamping it mathematically to `[0.0, 1.0]`.
    #[must_use]
    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Overrides the foreground fill color of the progress bar.
    #[must_use]
    pub fn with_progress_color(mut self, color: Color) -> Self {
        self.progress_color = Some(color);
        self
    }

    /// Sets the height of the progress bar in physical pixels.
    #[must_use]
    pub fn with_progress_height(mut self, height: f32) -> Self {
        self.progress_height = height.max(2.0);
        self
    }

    /// Configures whether user cancellation is allowed for this operation.
    ///
    /// When set to `false`, the cancel button is hidden, Escape key dismissals are disabled,
    /// and the modal completely locks the application until dismissed programmatically.
    #[must_use]
    pub fn with_cancellable(mut self, is_cancellable: bool) -> Self {
        self.is_cancellable = is_cancellable;
        self
    }

    /// Overrides the cancel button text with a custom localized label.
    #[must_use]
    pub fn with_cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_label = label.into();
        self
    }

    /// Specifies the affirmative cancellation action emitted into [`ActionBus`](crate::action_bus::ActionBus).
    #[must_use]
    pub fn with_on_cancel(mut self, action: Action) -> Self {
        self.on_cancel = Some(action);
        self
    }

    /// Sets an optional subtitle or category description in the header bar.
    #[must_use]
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Sets an optional icon or glyph displayed next to the title in the header bar.
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the modal dimensions `(width, height)` in physical pixels.
    #[must_use]
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = (width.max(100.0), height.max(80.0));
        self
    }

    /// Sets an explicit top-left position for the modal card instead of centering it.
    #[must_use]
    pub fn with_position(mut self, position: Point) -> Self {
        self.position = Some(position);
        self
    }

    /// Overrides the semi-transparent backdrop scrim color.
    #[must_use]
    pub fn with_backdrop_color(mut self, color: Color) -> Self {
        self.backdrop_color = Some(color);
        self
    }

    /// Applies custom visual styling for the card surface, borders, and margins.
    #[must_use]
    pub fn with_style(mut self, style: ModalStyle) -> Self {
        self.style = style;
        self
    }

    /// Configures whether clicking the semi-transparent backdrop triggers cancellation.
    ///
    /// Only takes effect if [`is_cancellable`](Self::is_cancellable) is `true`.
    #[must_use]
    pub fn with_dismiss_on_backdrop(mut self, dismiss: bool) -> Self {
        self.dismiss_on_backdrop = dismiss;
        self
    }
}