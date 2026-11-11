// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Confirmation Dialogue Descriptors
//!
//! Provides declarative specifications for standard, alert, and destructive warning
//! modal dialogues, supporting both untyped and strongly typed action dispatches.

use crate::dialogs::style::ModalStyle;

/// Declarative specification for confirmation, alert, and destructive warning modal dialogues.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmDialogDescriptor {
    /// Title string presented in the modal header bar.
    pub title: String,

    /// Primary confirmation prompt or description text.
    pub message: String,

    /// Optional target detail path or warning subtext.
    pub detail: Option<String>,

    /// Optional icon glyph or emoji shown alongside the title.
    pub icon: Option<String>,

    /// Caption presented on the primary action button (e.g. "Confirm", "Save", "Delete").
    pub confirm_label: String,

    /// Caption presented on the secondary cancel button.
    pub cancel_label: String,

    /// Whether the confirm button applies high-visibility danger styling.
    pub is_danger: bool,

    /// Card dimensions `(width, height)` in logical pixels.
    pub size: (f32, f32),

    /// Visual chromatic configuration.
    pub style: ModalStyle,

    /// Whether clicking the backdrop dismisses the dialogue.
    pub dismiss_on_backdrop: bool,

    /// Whether pressing the keyboard Escape key dismisses the dialogue.
    pub dismiss_on_escape: bool,

    /// Whether the dialogue can be cancelled by the user.
    pub is_cancellable: bool,

    /// Whether the top-right `✕` close button is visible.
    pub show_close_button: bool,
}

impl ConfirmDialogDescriptor {
    /// Default dimensions `(420.0, 180.0)` for standard confirmation cards.
    pub const DEFAULT_SIZE: (f32, f32) = (420.0, 180.0);

    /// Creates a standard confirmation dialogue specification.
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            detail: None,
            icon: None,
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            is_danger: false,
            size: Self::DEFAULT_SIZE,
            style: ModalStyle::default(),
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            is_cancellable: true,
            show_close_button: true,
        }
    }

    /// Creates a high-visibility danger confirmation dialogue specification (e.g., permanent deletion).
    #[inline]
    #[must_use]
    pub fn danger(
        title: impl Into<String>,
        message: impl Into<String>,
        confirm_label: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            detail: None,
            icon: None,
            confirm_label: confirm_label.into(),
            cancel_label: "Cancel".to_string(),
            is_danger: true,
            size: Self::DEFAULT_SIZE,
            style: ModalStyle::default(),
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            is_cancellable: true,
            show_close_button: true,
        }
    }

    /// Sets an optional target detail string (e.g. file path or asset URI).
    #[inline]
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Sets an optional icon glyph or emoji shown alongside the title.
    #[inline]
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the label string on the confirmation button.
    #[inline]
    #[must_use]
    pub fn with_confirm_label(mut self, label: impl Into<String>) -> Self {
        self.confirm_label = label.into();
        self
    }

    /// Sets whether the confirmation button applies high-visibility danger styling.
    #[inline]
    #[must_use]
    pub const fn with_danger(mut self, is_danger: bool) -> Self {
        self.is_danger = is_danger;
        self
    }

    /// Sets the label string on the cancel button.
    #[inline]
    #[must_use]
    pub fn with_cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_label = label.into();
        self
    }

    /// Overrides modal card dimensions `(width, height)` in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }

    /// Overrides visual styling properties.
    #[inline]
    #[must_use]
    pub fn with_style(mut self, style: ModalStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets whether clicking the backdrop dismisses the dialogue.
    #[inline]
    #[must_use]
    pub const fn with_dismiss_on_backdrop(mut self, allow: bool) -> Self {
        self.dismiss_on_backdrop = allow;
        self
    }

    /// Sets whether pressing Escape dismisses the dialogue.
    #[inline]
    #[must_use]
    pub const fn with_dismiss_on_escape(mut self, allow: bool) -> Self {
        self.dismiss_on_escape = allow;
        self
    }

    /// Sets whether the dialogue is cancellable by the user.
    #[inline]
    #[must_use]
    pub const fn with_cancellable(mut self, cancellable: bool) -> Self {
        self.is_cancellable = cancellable;
        self
    }

    /// Sets whether the close button is rendered in the header bar.
    #[inline]
    #[must_use]
    pub const fn with_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }
}

/// Strongly typed affirmative or destructive confirmation dialogue specification with an action payload.
///
/// Dispatches `on_confirm` to the application event loop or action bus upon positive user interaction.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmationDialogDescriptor<Action = ()> {
    /// Title string presented in the modal header bar.
    pub title: String,

    /// Primary confirmation prompt or description text.
    pub message: String,

    /// Optional target detail path or warning subtext.
    pub detail: Option<String>,

    /// Optional icon glyph or emoji shown alongside the title.
    pub icon: Option<String>,

    /// Caption presented on the primary action button.
    pub confirm_label: String,

    /// Caption presented on the secondary cancel button.
    pub cancel_label: String,

    /// Whether the confirm button applies high-visibility danger styling.
    pub is_danger: bool,

    /// Card dimensions `(width, height)` in logical pixels.
    pub size: (f32, f32),

    /// Visual chromatic configuration.
    pub style: ModalStyle,

    /// Whether clicking the backdrop dismisses the dialogue.
    pub dismiss_on_backdrop: bool,

    /// Whether pressing the keyboard Escape key dismisses the dialogue.
    pub dismiss_on_escape: bool,

    /// Whether the dialogue can be cancelled by the user.
    pub is_cancellable: bool,

    /// Whether the top-right `✕` close button is visible.
    pub show_close_button: bool,

    /// Action dispatched when confirmation is confirmed.
    pub on_confirm: Action,

    /// Optional action dispatched when the dialogue is cancelled or dismissed.
    pub on_cancel: Option<Action>,
}

impl<Action> ConfirmationDialogDescriptor<Action> {
    /// Creates a standard confirmation dialogue specification with an action payload.
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, message: impl Into<String>, on_confirm: Action) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            detail: None,
            icon: None,
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            is_danger: false,
            size: ConfirmDialogDescriptor::DEFAULT_SIZE,
            style: ModalStyle::default(),
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            is_cancellable: true,
            show_close_button: true,
            on_confirm,
            on_cancel: None,
        }
    }

    /// Creates a high-visibility danger confirmation dialogue specification with an action payload.
    #[inline]
    #[must_use]
    pub fn danger(
        title: impl Into<String>,
        message: impl Into<String>,
        confirm_label: impl Into<String>,
        on_confirm: Action,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            detail: None,
            icon: None,
            confirm_label: confirm_label.into(),
            cancel_label: "Cancel".to_string(),
            is_danger: true,
            size: ConfirmDialogDescriptor::DEFAULT_SIZE,
            style: ModalStyle::default(),
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            is_cancellable: true,
            show_close_button: true,
            on_confirm,
            on_cancel: None,
        }
    }

    /// Sets an optional target detail string (e.g. file path or asset URI).
    #[inline]
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Sets an optional icon glyph or emoji shown alongside the title.
    #[inline]
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the label string on the confirmation button.
    #[inline]
    #[must_use]
    pub fn with_confirm_label(mut self, label: impl Into<String>) -> Self {
        self.confirm_label = label.into();
        self
    }

    /// Sets the label string on the cancel button.
    #[inline]
    #[must_use]
    pub fn with_cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_label = label.into();
        self
    }

    /// Sets whether the confirmation button applies danger styling.
    #[inline]
    #[must_use]
    pub const fn with_danger(mut self, is_danger: bool) -> Self {
        self.is_danger = is_danger;
        self
    }

    /// Overrides modal card dimensions `(width, height)` in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }

    /// Overrides visual styling properties.
    #[inline]
    #[must_use]
    pub fn with_style(mut self, style: ModalStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets whether clicking the backdrop dismisses the dialogue.
    #[inline]
    #[must_use]
    pub const fn with_dismiss_on_backdrop(mut self, allow: bool) -> Self {
        self.dismiss_on_backdrop = allow;
        self
    }

    /// Sets whether pressing Escape dismisses the dialogue.
    #[inline]
    #[must_use]
    pub const fn with_dismiss_on_escape(mut self, allow: bool) -> Self {
        self.dismiss_on_escape = allow;
        self
    }

    /// Sets whether the dialogue is cancellable by the user.
    #[inline]
    #[must_use]
    pub const fn with_cancellable(mut self, cancellable: bool) -> Self {
        self.is_cancellable = cancellable;
        self
    }

    /// Sets whether the close button is rendered in the header bar.
    #[inline]
    #[must_use]
    pub const fn with_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }

    /// Sets an optional cancellation action dispatched when the dialogue is cancelled or dismissed.
    #[inline]
    #[must_use]
    pub fn with_cancel_action(mut self, action: Action) -> Self {
        self.on_cancel = Some(action);
        self
    }
}