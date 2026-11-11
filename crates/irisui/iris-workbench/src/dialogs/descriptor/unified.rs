// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unified Declarative Dialog Descriptor
//!
//! Provides the primary [`DialogDescriptor`] enumeration encapsulating all varieties of
//! workbench dialogue cards: centered modals, edge sliding sheets, confirmations,
//! text prompts, and arbitrary custom declarative views.

use crate::action_bus::ActionBus;
use crate::dialogs::descriptor::confirm::ConfirmationDialogDescriptor;
use crate::dialogs::descriptor::custom::CustomDialogDescriptor;
use crate::dialogs::descriptor::info::InfoCardDialogDescriptor;
use crate::dialogs::descriptor::input::PromptDialogDescriptor;
use crate::dialogs::descriptor::modal::ModalDescriptor;
use crate::dialogs::descriptor::progress::ProgressModalDescriptor;
use crate::dialogs::descriptor::sheet::{SheetAnchor, SheetDialogDescriptor};
use iris_widgets::declarative::UiScope;

/// Unified declarative dialog descriptor capturing all modal varieties in the desktop workbench.
///
/// Encapsulates centered modal dialogues, sliding edge sheets, two-button confirmation prompts,
/// text input prompt dialogues, progress monitoring cards, generic info cards, and custom view cards.
///
/// # Generics
/// - `Action`: Application-specific action payload emitted on positive user confirmation.
pub enum DialogDescriptor<Action = ()>
where
    Action: 'static + Send + Sync,
{
    /// Standard centered modal dialogue card.
    Modal(ModalDescriptor),

    /// Sliding sheet dialogue anchored to a window boundary (Bottom, Top, Left, Right).
    Sheet(SheetDialogDescriptor<Action>),

    /// Two-button affirmative or destructive confirmation dialogue.
    Confirmation(ConfirmationDialogDescriptor<Action>),

    /// Single-line or multi-line text input prompt dialogue.
    Prompt(PromptDialogDescriptor<Action>),

    /// Generic information, software "About", or diagnostic modal dialogue card.
    InfoCard(InfoCardDialogDescriptor<Action>),

    /// Long-running operation progress modal dialogue card with optional cancellation.
    Progress(ProgressModalDescriptor<Action>),

    /// Custom declarative closure evaluated inside an elevated modal card.
    CustomView(CustomDialogDescriptor<Action>),
}

impl<Action: 'static + Send + Sync> DialogDescriptor<Action> {
    /// Constructs a standard centered modal dialogue card.
    #[inline]
    #[must_use]
    pub fn modal(title: impl Into<String>, width: f32, height: f32) -> Self {
        Self::Modal(ModalDescriptor::new(title, width, height))
    }

    /// Constructs a sliding drawer sheet dialogue anchored to a window boundary.
    #[inline]
    #[must_use]
    pub fn sheet(title: impl Into<String>, anchor: SheetAnchor, extent: f32) -> Self {
        Self::Sheet(SheetDialogDescriptor::new(title, anchor, extent))
    }

    /// Constructs a standard affirmative confirmation dialogue with an action payload.
    #[inline]
    #[must_use]
    pub fn confirmation(
        title: impl Into<String>,
        message: impl Into<String>,
        on_confirm: Action,
    ) -> Self {
        Self::Confirmation(ConfirmationDialogDescriptor::new(
            title, message, on_confirm,
        ))
    }

    /// Constructs a high-visibility danger confirmation dialogue with an action payload.
    #[inline]
    #[must_use]
    pub fn danger(
        title: impl Into<String>,
        message: impl Into<String>,
        confirm_label: impl Into<String>,
        on_confirm: Action,
    ) -> Self {
        Self::Confirmation(ConfirmationDialogDescriptor::danger(
            title,
            message,
            confirm_label,
            on_confirm,
        ))
    }

    /// Constructs a text input prompt dialogue with a submit action callback.
    #[inline]
    #[must_use]
    pub fn prompt(
        title: impl Into<String>,
        prompt: impl Into<String>,
        on_submit: impl Fn(String) -> Action + Send + Sync + 'static,
    ) -> Self {
        Self::Prompt(PromptDialogDescriptor::new(title, prompt, on_submit))
    }

    /// Constructs a generic information or software "About" dialogue descriptor.
    #[inline]
    #[must_use]
    pub fn info_card(title: impl Into<String>) -> Self {
        Self::InfoCard(InfoCardDialogDescriptor::new(title))
    }

    /// Constructs a progress modal dialogue descriptor for long-running operations.
    #[inline]
    #[must_use]
    pub fn progress(title: impl Into<String>) -> Self {
        Self::Progress(ProgressModalDescriptor::new(title))
    }

    /// Constructs a custom modal dialogue card evaluated via a declarative builder closure.
    #[inline]
    #[must_use]
    pub fn custom_view<F>(title: impl Into<String>, size: (f32, f32), render_fn: F) -> Self
    where
        F: FnMut(&mut UiScope<'_>, &mut ActionBus<Action>) + Send + Sync + 'static,
    {
        Self::CustomView(CustomDialogDescriptor::new(title, size, render_fn))
    }

    /// Returns the window title caption displayed in the dialogue header bar.
    #[must_use]
    pub fn title(&self) -> &str {
        match self {
            Self::Modal(d) => &d.title,
            Self::Sheet(d) => &d.title,
            Self::Confirmation(d) => &d.title,
            Self::Prompt(d) => &d.title,
            Self::InfoCard(d) => &d.title,
            Self::Progress(d) => &d.title,
            Self::CustomView(d) => &d.title,
        }
    }

    /// Returns the dimensions `(width, height)` of the dialogue card in logical pixels.
    ///
    /// For sheets, returns `(extent, extent)` as the constrained primary axis.
    #[must_use]
    pub fn size(&self) -> (f32, f32) {
        match self {
            Self::Modal(d) => d.size,
            Self::Sheet(d) => {
                if d.anchor.is_horizontal() {
                    (d.extent, 600.0)
                } else {
                    (800.0, d.extent)
                }
            }
            Self::Confirmation(d) => d.size,
            Self::Prompt(d) => d.size,
            Self::InfoCard(d) => d.size,
            Self::Progress(d) => d.size,
            Self::CustomView(d) => d.size,
        }
    }

    /// Returns whether the dialogue can be cancelled by user interaction (Escape key, cancel button, or backdrop).
    #[must_use]
    pub const fn is_cancellable(&self) -> bool {
        match self {
            Self::Modal(d) => d.is_cancellable,
            Self::Sheet(d) => d.is_cancellable,
            Self::Confirmation(d) => d.is_cancellable,
            Self::Prompt(d) => d.is_cancellable,
            Self::InfoCard(d) => d.is_cancellable,
            Self::Progress(d) => d.is_cancellable,
            Self::CustomView(d) => d.is_cancellable,
        }
    }

    /// Returns whether clicking the semi-transparent backdrop dismisses the dialogue.
    #[must_use]
    pub const fn dismiss_on_backdrop(&self) -> bool {
        if !self.is_cancellable() {
            return false;
        }
        match self {
            Self::Modal(d) => d.dismiss_on_backdrop,
            Self::Sheet(d) => d.dismiss_on_backdrop,
            Self::Confirmation(d) => d.dismiss_on_backdrop,
            Self::Prompt(d) => d.dismiss_on_backdrop,
            Self::InfoCard(d) => d.dismiss_on_backdrop,
            Self::Progress(d) => d.dismiss_on_backdrop,
            Self::CustomView(d) => d.dismiss_on_backdrop,
        }
    }

    /// Returns whether pressing the keyboard Escape key dismisses the dialogue.
    #[must_use]
    pub const fn dismiss_on_escape(&self) -> bool {
        if !self.is_cancellable() {
            return false;
        }
        match self {
            Self::Modal(d) => d.dismiss_on_escape,
            Self::Sheet(d) => d.dismiss_on_escape,
            Self::Confirmation(d) => d.dismiss_on_escape,
            Self::Prompt(d) => d.dismiss_on_escape,
            Self::InfoCard(d) => d.dismiss_on_escape,
            Self::Progress(_) => true,
            Self::CustomView(d) => d.dismiss_on_escape,
        }
    }

    /// Returns whether a standard close button is rendered in the header bar.
    #[must_use]
    pub const fn show_close_button(&self) -> bool {
        if !self.is_cancellable() {
            return false;
        }
        match self {
            Self::Modal(d) => d.show_close_button,
            Self::Sheet(d) => d.show_close_button,
            Self::Confirmation(d) => d.show_close_button,
            Self::Prompt(d) => d.show_close_button,
            Self::InfoCard(d) => d.show_close_button,
            Self::Progress(_) => true,
            Self::CustomView(d) => d.show_close_button,
        }
    }
}

impl<Action: 'static + Send + Sync> From<ModalDescriptor> for DialogDescriptor<Action> {
    #[inline]
    fn from(val: ModalDescriptor) -> Self {
        Self::Modal(val)
    }
}

impl<Action: 'static + Send + Sync> From<SheetDialogDescriptor<Action>>
    for DialogDescriptor<Action>
{
    #[inline]
    fn from(val: SheetDialogDescriptor<Action>) -> Self {
        Self::Sheet(val)
    }
}

impl<Action: 'static + Send + Sync> From<ConfirmationDialogDescriptor<Action>>
    for DialogDescriptor<Action>
{
    #[inline]
    fn from(val: ConfirmationDialogDescriptor<Action>) -> Self {
        Self::Confirmation(val)
    }
}

impl<Action: 'static + Send + Sync> From<PromptDialogDescriptor<Action>>
    for DialogDescriptor<Action>
{
    #[inline]
    fn from(val: PromptDialogDescriptor<Action>) -> Self {
        Self::Prompt(val)
    }
}

impl<Action: 'static + Send + Sync> From<InfoCardDialogDescriptor<Action>>
    for DialogDescriptor<Action>
{
    #[inline]
    fn from(val: InfoCardDialogDescriptor<Action>) -> Self {
        Self::InfoCard(val)
    }
}

impl<Action: 'static + Send + Sync> From<ProgressModalDescriptor<Action>>
    for DialogDescriptor<Action>
{
    #[inline]
    fn from(val: ProgressModalDescriptor<Action>) -> Self {
        Self::Progress(val)
    }
}

impl<Action: 'static + Send + Sync> From<CustomDialogDescriptor<Action>>
    for DialogDescriptor<Action>
{
    #[inline]
    fn from(val: CustomDialogDescriptor<Action>) -> Self {
        Self::CustomView(val)
    }
}