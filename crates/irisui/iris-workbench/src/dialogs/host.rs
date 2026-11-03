// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Dialog and Modal Host State Machine
//!
//! Autonomous coordinator managing declarative modal dialog state transitions,
//! semi-transparent backdrop scrim occlusion, keyboard shortcut intercepts (Escape to dismiss,
//! Enter to confirm), and automatic routing of confirm/submit payloads into the [`ActionBus`].

#[cfg(test)]
mod tests;

use iris_core::event::KeyCode;
use iris_core::geometry::Rect;
use iris_widgets::declarative::UiScope;

use super::controller::ModalController;
use super::descriptor::{ConfirmDialogDescriptor, InputDialogDescriptor, ModalDescriptor};
use super::response::ModalOutcome;
use super::style::ModalStyle;
use crate::action_bus::ActionBus;

/// Type alias for user-supplied custom modal rendering closures.
pub type CustomModalRenderFn<Action> =
    Box<dyn FnMut(&mut UiScope<'_>, &mut ActionBus<Action>) + Send + Sync>;

/// Strongly typed modal dialogue variant tracked by [`DialogHost`].
pub enum DialogKind<Action: 'static + Send + Sync> {
    /// Two-button confirmation dialogue (Confirm / Cancel).
    Confirm {
        /// Modal card title text.
        title: String,
        /// Informative body message.
        message: String,
        /// Optional target detail path or contextual hint.
        detail: Option<String>,
        /// Caption presented on the confirm action button.
        confirm_label: String,
        /// Caption presented on the cancel button.
        cancel_label: String,
        /// Whether the confirm button applies high-visibility danger styling.
        is_danger: bool,
        /// Strongly-typed action dispatched to the [`ActionBus`] on confirmation.
        on_confirm: Action,
    },
    /// Single-line text input dialogue with prompt, text buffer, and submit callback.
    Input {
        /// Modal card title text.
        title: String,
        /// Input field prompt label.
        label: String,
        /// Current editable text buffer.
        text: String,
        /// Ghost placeholder text shown when buffer is empty.
        placeholder: String,
        /// Caption presented on the submit action button.
        confirm_label: String,
        /// Caption presented on the cancel button.
        cancel_label: String,
        /// Callback converting the submitted text into an application action.
        on_submit: Box<dyn Fn(String) -> Action + Send + Sync>,
    },
    /// Custom user-defined modal card with immediate-mode UI rendering closure.
    Custom {
        /// Modal card title text.
        title: String,
        /// Explicit dimensions `(width, height)` in logical pixels.
        size: (f32, f32),
        /// Custom render closure evaluated inside the modal card.
        render_fn: CustomModalRenderFn<Action>,
    },
}

impl<Action: 'static + Send + Sync + std::fmt::Debug> std::fmt::Debug for DialogKind<Action> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Confirm {
                title,
                message,
                detail,
                confirm_label,
                cancel_label,
                is_danger,
                on_confirm,
            } => f
                .debug_struct("Confirm")
                .field("title", title)
                .field("message", message)
                .field("detail", detail)
                .field("confirm_label", confirm_label)
                .field("cancel_label", cancel_label)
                .field("is_danger", is_danger)
                .field("on_confirm", on_confirm)
                .finish(),
            Self::Input {
                title,
                label,
                text,
                placeholder,
                confirm_label,
                cancel_label,
                ..
            } => f
                .debug_struct("Input")
                .field("title", title)
                .field("label", label)
                .field("text", text)
                .field("placeholder", placeholder)
                .field("confirm_label", confirm_label)
                .field("cancel_label", cancel_label)
                .finish(),
            Self::Custom { title, size, .. } => f
                .debug_struct("Custom")
                .field("title", title)
                .field("size", size)
                .finish(),
        }
    }
}

/// Active modal dialog representation tracked by [`DialogHost`].
pub struct ActiveDialog<Action: 'static + Send + Sync> {
    /// Kind and specific configuration of this active dialog.
    pub kind: DialogKind<Action>,
    /// Visual style parameters (backdrop, card background, border, corner radius).
    pub style: ModalStyle,
    /// Explicit width and height of the dialog card in logical pixels.
    pub size: (f32, f32),
    /// Whether clicking the semi-transparent backdrop dismisses the dialog.
    pub dismiss_on_backdrop: bool,
    /// Whether pressing the Escape key dismisses the dialog.
    pub dismiss_on_escape: bool,
    /// Whether to render the '✕' close button in the header bar.
    pub show_close_button: bool,
}

/// Autonomous modal and dialogue state machine for workbench applications.
///
/// Manages modal dialogue presentation, semi-transparent backdrop occlusion,
/// keyboard shortcut interceptors (Escape to dismiss, Enter to confirm), and
/// automatically routes confirmation payloads into the centralized [`ActionBus`].
pub struct DialogHost<Action: 'static + Send + Sync> {
    active_dialog: Option<ActiveDialog<Action>>,
    default_style: ModalStyle,
}

impl<Action: 'static + Send + Sync> DialogHost<Action> {
    /// Creates a new, idle dialog host state machine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_dialog: None,
            default_style: ModalStyle::default(),
        }
    }

    /// Returns `true` if a modal dialogue is currently active and occluding the workbench.
    #[must_use]
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active_dialog.is_some()
    }

    /// Returns a shared reference to the active dialog, if one is currently presented.
    #[must_use]
    #[inline]
    pub fn active_dialog(&self) -> Option<&ActiveDialog<Action>> {
        self.active_dialog.as_ref()
    }

    /// Returns a mutable reference to the active dialog, if one is currently presented.
    #[must_use]
    #[inline]
    pub fn active_dialog_mut(&mut self) -> Option<&mut ActiveDialog<Action>> {
        self.active_dialog.as_mut()
    }

    /// Dismisses the currently active dialogue immediately without emitting any action.
    pub fn close(&mut self) {
        self.active_dialog = None;
    }

    /// Overrides the default visual styling applied to subsequently opened dialogues.
    pub fn set_default_style(&mut self, style: ModalStyle) {
        self.default_style = style;
    }

    /// Opens a two-button confirmation dialogue card.
    ///
    /// # Arguments
    /// * `title` - Title text displayed in the modal card header.
    /// * `message` - Informative description prompt.
    /// * `on_confirm` - Strongly-typed action dispatched to the [`ActionBus`] on confirm.
    pub fn open_confirm(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        on_confirm: Action,
    ) {
        self.active_dialog = Some(ActiveDialog {
            kind: DialogKind::Confirm {
                title: title.into(),
                message: message.into(),
                detail: None,
                confirm_label: "Confirm".to_string(),
                cancel_label: "Cancel".to_string(),
                is_danger: false,
                on_confirm,
            },
            style: self.default_style.clone(),
            size: ConfirmDialogDescriptor::DEFAULT_SIZE,
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            show_close_button: true,
        });
    }

    /// Opens a high-visibility danger confirmation dialogue (e.g. permanent deletion).
    ///
    /// # Arguments
    /// * `title` - Title text displayed in the modal card header.
    /// * `message` - Informative description prompt.
    /// * `on_confirm` - Strongly-typed action dispatched to the [`ActionBus`] on confirm.
    pub fn open_confirm_danger(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        on_confirm: Action,
    ) {
        self.active_dialog = Some(ActiveDialog {
            kind: DialogKind::Confirm {
                title: title.into(),
                message: message.into(),
                detail: None,
                confirm_label: "Delete".to_string(),
                cancel_label: "Cancel".to_string(),
                is_danger: true,
                on_confirm,
            },
            style: self.default_style.clone(),
            size: ConfirmDialogDescriptor::DEFAULT_SIZE,
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            show_close_button: true,
        });
    }

    /// Opens a single-line text input dialogue card.
    ///
    /// # Arguments
    /// * `title` - Title text displayed in the modal card header.
    /// * `label` - Prompt label displayed above the input field.
    /// * `initial_text` - Initial buffer content for editing.
    /// * `on_submit` - Callback converting submitted text into an application action.
    pub fn open_input(
        &mut self,
        title: impl Into<String>,
        label: impl Into<String>,
        initial_text: impl Into<String>,
        on_submit: impl Fn(String) -> Action + Send + Sync + 'static,
    ) {
        self.active_dialog = Some(ActiveDialog {
            kind: DialogKind::Input {
                title: title.into(),
                label: label.into(),
                text: initial_text.into(),
                placeholder: String::new(),
                confirm_label: "OK".to_string(),
                cancel_label: "Cancel".to_string(),
                on_submit: Box::new(on_submit),
            },
            style: self.default_style.clone(),
            size: InputDialogDescriptor::DEFAULT_SIZE,
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            show_close_button: true,
        });
    }

    /// Opens a custom modal dialogue card with user-supplied rendering logic.
    ///
    /// # Arguments
    /// * `title` - Title text displayed in the modal card header.
    /// * `size` - Dimensions `(width, height)` in logical pixels.
    /// * `render_fn` - Closure building declarative UI nodes within the modal card.
    pub fn open_custom<F>(&mut self, title: impl Into<String>, size: (f32, f32), render_fn: F)
    where
        F: FnMut(&mut UiScope<'_>, &mut ActionBus<Action>) + Send + Sync + 'static,
    {
        self.active_dialog = Some(ActiveDialog {
            kind: DialogKind::Custom {
                title: title.into(),
                size,
                render_fn: Box::new(render_fn),
            },
            style: self.default_style.clone(),
            size,
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            show_close_button: true,
        });
    }

    /// Intercepts hardware-independent keyboard events when a modal is active.
    ///
    /// Returns `true` if the event was consumed by the modal dialogue state machine,
    /// isolating underlying workbench views and 3D scenes from receiving input.
    ///
    /// # Keyboard Behavior
    /// * `Escape`: If `dismiss_on_escape` is true, closes the dialog without emitting an action.
    /// * `Enter`: Confirms the dialog, dispatches the payload to `action_bus`, and closes it.
    /// * All other keys: Consumed while the modal is open to prevent key leakage.
    pub fn handle_keyboard_input(
        &mut self,
        key: &KeyCode,
        is_pressed: bool,
        action_bus: &mut ActionBus<Action>,
    ) -> bool {
        let Some(dialog) = &self.active_dialog else {
            return false;
        };

        if !is_pressed {
            return true;
        }

        if *key == KeyCode::Escape && dialog.dismiss_on_escape {
            self.close();
            return true;
        }

        if *key == KeyCode::Enter {
            match self.active_dialog.take() {
                Some(ActiveDialog {
                    kind: DialogKind::Confirm { on_confirm, .. },
                    ..
                }) => {
                    action_bus.emit(on_confirm);
                    return true;
                }
                Some(ActiveDialog {
                    kind:
                        DialogKind::Input {
                            text, on_submit, ..
                        },
                    ..
                }) => {
                    let action = on_submit(text);
                    action_bus.emit(action);
                    return true;
                }
                Some(other) => {
                    self.active_dialog = Some(other);
                }
                None => {}
            }
        }

        true
    }

    /// Declaratively renders the active modal dialogue and backdrop scrim into the provided [`UiScope`].
    ///
    /// Automatically manages:
    /// - Fullscreen semi-transparent backdrop blocker.
    /// - Modal card centering, header bar with title and close button.
    /// - Body message, detail subtext, or input field.
    /// - Confirm / Cancel action footer buttons.
    /// - Dispatching confirmed actions into [`ActionBus`] and closing the dialog state.
    pub fn render(
        &mut self,
        scope: &mut UiScope<'_>,
        viewport: Rect,
        escape_pressed: bool,
        action_bus: &mut ActionBus<Action>,
    ) {
        let Some(active) = &self.active_dialog else {
            return;
        };

        if escape_pressed && active.dismiss_on_escape {
            self.close();
            return;
        }

        match &mut self.active_dialog {
            Some(ActiveDialog {
                kind:
                    DialogKind::Confirm {
                        title,
                        message,
                        detail,
                        confirm_label,
                        cancel_label,
                        is_danger,
                        ..
                    },
                style,
                size,
                dismiss_on_backdrop,
                dismiss_on_escape,
                show_close_button,
            }) => {
                let mut desc = ConfirmDialogDescriptor::new(&*title, &*message)
                    .with_confirm_label(&*confirm_label)
                    .with_cancel_label(&*cancel_label)
                    .with_danger(*is_danger)
                    .with_size(size.0, size.1)
                    .with_dismiss_on_backdrop(*dismiss_on_backdrop)
                    .with_dismiss_on_escape(*dismiss_on_escape)
                    .with_close_button(*show_close_button)
                    .with_style(style.clone());

                if let Some(det) = detail {
                    desc = desc.with_detail(&*det);
                }

                let outcome =
                    ModalController::render_confirm_dialog(scope, &desc, viewport, escape_pressed);
                match outcome {
                    ModalOutcome::Action(true) => {
                        if let Some(dialog) = self.active_dialog.take()
                            && let DialogKind::Confirm { on_confirm, .. } = dialog.kind
                        {
                            action_bus.emit(on_confirm);
                        }
                    }
                    ModalOutcome::Dismissed(_) => {
                        self.active_dialog = None;
                    }
                    _ => {}
                }
            }
            Some(ActiveDialog {
                kind:
                    DialogKind::Input {
                        title,
                        label,
                        text,
                        placeholder,
                        confirm_label,
                        cancel_label,
                        ..
                    },
                style,
                size,
                dismiss_on_backdrop,
                dismiss_on_escape,
                show_close_button,
            }) => {
                let desc = InputDialogDescriptor::new(&*title, &*label)
                    .with_confirm_label(&*confirm_label)
                    .with_cancel_label(&*cancel_label)
                    .with_placeholder(&*placeholder)
                    .with_size(size.0, size.1)
                    .with_dismiss_on_backdrop(*dismiss_on_backdrop)
                    .with_dismiss_on_escape(*dismiss_on_escape)
                    .with_close_button(*show_close_button)
                    .with_style(style.clone());

                let outcome = ModalController::render_input_dialog(
                    scope,
                    &desc,
                    text,
                    viewport,
                    escape_pressed,
                );
                match outcome {
                    ModalOutcome::Action(submitted_text) => {
                        if let Some(dialog) = self.active_dialog.take()
                            && let DialogKind::Input { on_submit, .. } = dialog.kind
                        {
                            let action = on_submit(submitted_text);
                            action_bus.emit(action);
                        }
                    }
                    ModalOutcome::Dismissed(_) => {
                        self.active_dialog = None;
                    }
                    _ => {}
                }
            }
            Some(ActiveDialog {
                kind:
                    DialogKind::Custom {
                        title,
                        size,
                        render_fn,
                    },
                style,
                dismiss_on_backdrop,
                dismiss_on_escape,
                show_close_button,
                ..
            }) => {
                let desc = ModalDescriptor::new(&*title, size.0, size.1)
                    .with_dismiss_on_backdrop(*dismiss_on_backdrop)
                    .with_dismiss_on_escape(*dismiss_on_escape)
                    .with_close_button(*show_close_button)
                    .with_style(style.clone());

                let outcome = ModalController::render_modal(
                    scope,
                    &desc,
                    viewport,
                    escape_pressed,
                    |card_scope| {
                        render_fn(card_scope, action_bus);
                        ModalOutcome::<()>::Active
                    },
                );
                if matches!(outcome, ModalOutcome::Dismissed(_)) {
                    self.active_dialog = None;
                }
            }
            None => {}
        }
    }
}

impl<Action: 'static + Send + Sync> Default for DialogHost<Action> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Action: 'static + Send + Sync + std::fmt::Debug> std::fmt::Debug for DialogHost<Action> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogHost")
            .field("is_active", &self.is_active())
            .field(
                "active_dialog_kind",
                &self.active_dialog.as_ref().map(|d| &d.kind),
            )
            .finish()
    }
}