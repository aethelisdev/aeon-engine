// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Text Input and Prompt Dialogue Descriptors
//!
//! Provides declarative specifications for text entry dialogues, user prompts,
//! and contextual value editors.

use crate::dialogs::style::ModalStyle;

/// Declarative specification for single-line text input modal dialogues.
#[derive(Debug, Clone, PartialEq)]
pub struct InputDialogDescriptor {
    /// Title string presented in the modal header bar.
    pub title: String,

    /// Descriptive prompt or label text shown above the text input field.
    pub prompt: String,

    /// Ghost placeholder text shown when the input buffer is empty.
    pub placeholder: String,

    /// Optional target detail path or contextual hint.
    pub detail: Option<String>,

    /// Optional icon glyph or emoji shown alongside the title.
    pub icon: Option<String>,

    /// Caption presented on the primary action button (e.g. "Create", "Apply").
    pub confirm_label: String,

    /// Caption presented on the secondary cancel button.
    pub cancel_label: String,

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

impl InputDialogDescriptor {
    /// Default dimensions `(400.0, 190.0)` for text input cards.
    pub const DEFAULT_SIZE: (f32, f32) = (400.0, 190.0);

    /// Creates a new text input dialogue specification.
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            prompt: prompt.into(),
            placeholder: String::new(),
            detail: None,
            icon: None,
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            size: Self::DEFAULT_SIZE,
            style: ModalStyle::default(),
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            is_cancellable: true,
            show_close_button: true,
        }
    }

    /// Sets the placeholder text for the text input box.
    #[inline]
    #[must_use]
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets an optional detail text string (e.g. target path or parent folder).
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

    /// Sets the caption for the primary confirmation button.
    #[inline]
    #[must_use]
    pub fn with_confirm_label(mut self, label: impl Into<String>) -> Self {
        self.confirm_label = label.into();
        self
    }

    /// Sets the caption for the secondary cancel button.
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

    /// Sets whether the dialogue can be cancelled by the user.
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

/// Type alias for text submission action converter closures.
pub type PromptSubmitFn<Action> = Box<dyn Fn(String) -> Action + Send + Sync>;

/// Strongly typed text input prompt dialogue specification with an action constructor callback.
///
/// Converts submitted text into an application action dispatched to the event loop.
pub struct PromptDialogDescriptor<Action = ()> {
    /// Title string presented in the modal header bar.
    pub title: String,

    /// Descriptive prompt or label text shown above the text input field.
    pub prompt: String,

    /// Ghost placeholder text shown when the input buffer is empty.
    pub placeholder: String,

    /// Initial buffer content for editing.
    pub initial_text: String,

    /// Optional target detail path or contextual hint.
    pub detail: Option<String>,

    /// Optional icon glyph or emoji shown alongside the title.
    pub icon: Option<String>,

    /// Caption presented on the primary action button.
    pub confirm_label: String,

    /// Caption presented on the secondary cancel button.
    pub cancel_label: String,

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

    /// Callback converting the submitted text into an application action.
    pub on_submit: Option<PromptSubmitFn<Action>>,

    /// Optional action dispatched when the dialogue is cancelled or dismissed.
    pub on_cancel: Option<Action>,
}

impl<Action> PromptDialogDescriptor<Action> {
    /// Creates a new text prompt dialogue specification with a submit action callback.
    #[inline]
    #[must_use]
    pub fn new(
        title: impl Into<String>,
        prompt: impl Into<String>,
        on_submit: impl Fn(String) -> Action + Send + Sync + 'static,
    ) -> Self {
        Self {
            title: title.into(),
            prompt: prompt.into(),
            placeholder: String::new(),
            initial_text: String::new(),
            detail: None,
            icon: None,
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            size: InputDialogDescriptor::DEFAULT_SIZE,
            style: ModalStyle::default(),
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            is_cancellable: true,
            show_close_button: true,
            on_submit: Some(Box::new(on_submit)),
            on_cancel: None,
        }
    }

    /// Sets the initial text content for editing.
    #[inline]
    #[must_use]
    pub fn with_initial_text(mut self, text: impl Into<String>) -> Self {
        self.initial_text = text.into();
        self
    }

    /// Sets placeholder ghost text shown when the buffer is empty.
    #[inline]
    #[must_use]
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets an optional detail text string.
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

    /// Sets the caption for the primary confirmation button.
    #[inline]
    #[must_use]
    pub fn with_confirm_label(mut self, label: impl Into<String>) -> Self {
        self.confirm_label = label.into();
        self
    }

    /// Sets the caption for the secondary cancel button.
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

    /// Sets whether the dialogue can be cancelled by the user.
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