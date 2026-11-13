// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Dialog Host State Machine & Stack Management
//!
//! Autonomous coordinator managing modal dialog presentation, hierarchical stacking,
//! focus trapping, and keyboard/pointer event isolation.

use crate::action_bus::ActionBus;
use crate::dialogs::descriptor::confirm::ConfirmationDialogDescriptor;
use crate::dialogs::descriptor::custom::CustomDialogDescriptor;
use crate::dialogs::descriptor::info::InfoCardDialogDescriptor;
use crate::dialogs::descriptor::input::PromptDialogDescriptor;
use crate::dialogs::descriptor::progress::ProgressModalDescriptor;
use crate::dialogs::descriptor::sheet::{SheetAnchor, SheetDialogDescriptor};
use crate::dialogs::descriptor::unified::DialogDescriptor;
use crate::dialogs::host::active::{ActiveDialog, DialogKind};
use crate::dialogs::host::focus::DialogFocusItem;
use crate::dialogs::host::trap::DialogFocusTrap;
use crate::dialogs::style::ModalStyle;
use iris_core::event::KeyCode;
use iris_core::geometry::{Point, Rect};
use iris_widgets::declarative::UiScope;

/// Autonomous modal and dialogue state machine for workbench applications.
///
/// Coordinates stacked modal presentations, edge drawer sheets, confirmation dialogs,
/// semi-transparent backdrop scrim occlusion, focus traps, and action bus dispatching.
pub struct DialogHost<Action: 'static + Send + Sync> {
    pub(crate) dialog_stack: Vec<ActiveDialog<Action>>,
    pub(crate) default_style: ModalStyle,
}

impl<Action: 'static + Send + Sync> DialogHost<Action> {
    /// Creates a new, idle dialog host state machine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            dialog_stack: Vec::new(),
            default_style: ModalStyle::default(),
        }
    }

    /// Returns `true` if one or more modal dialogues are currently presented and active.
    #[must_use]
    #[inline]
    pub fn is_active(&self) -> bool {
        !self.dialog_stack.is_empty()
    }

    /// Returns the number of stacked modal dialogues currently open.
    #[must_use]
    #[inline]
    pub fn stack_depth(&self) -> usize {
        self.dialog_stack.len()
    }

    /// Returns an immutable slice of all active dialogues in bottom-to-top stacking order.
    #[must_use]
    #[inline]
    pub fn dialog_stack(&self) -> &[ActiveDialog<Action>] {
        &self.dialog_stack
    }

    /// Returns a shared reference to the topmost active dialog, if one is currently presented.
    #[must_use]
    #[inline]
    pub fn active_dialog(&self) -> Option<&ActiveDialog<Action>> {
        self.dialog_stack.last()
    }

    /// Returns a mutable reference to the topmost active dialog, if one is currently presented.
    #[must_use]
    #[inline]
    pub fn active_dialog_mut(&mut self) -> Option<&mut ActiveDialog<Action>> {
        self.dialog_stack.last_mut()
    }

    /// Triggers an attention shake and border glow highlight on the topmost active dialogue.
    ///
    /// Called automatically when the user clicks the backdrop scrim of a non-dismissible modal.
    /// Returns `true` if a modal was active and attention was triggered, or `false` otherwise.
    pub fn trigger_attention(&mut self) -> bool {
        let Some(top) = self.dialog_stack.last_mut() else {
            return false;
        };
        top.trigger_attention();
        true
    }

    /// Evaluates whether the topmost active dialogue is currently animating an attention feedback shake.
    #[must_use]
    #[inline]
    pub fn is_demanding_attention(&self) -> bool {
        self.dialog_stack
            .last()
            .map(|d| d.is_demanding_attention())
            .unwrap_or(false)
    }

    /// Returns whether the topmost active dialogue is cancellable by the user.
    #[must_use]
    #[inline]
    pub fn is_top_cancellable(&self) -> bool {
        self.dialog_stack
            .last()
            .map(|d| d.is_cancellable)
            .unwrap_or(false)
    }

    /// Attempts to cancel the topmost active dialogue.
    ///
    /// If the dialogue is cancellable, pops it from the stack and dispatches any configured
    /// cancellation action to the [`ActionBus`], returning `true`. If the dialogue is non-cancellable,
    /// triggers an attention shake animation and returns `false`.
    pub fn cancel_top(&mut self, action_bus: &mut ActionBus<Action>) -> bool {
        let Some(top) = self.dialog_stack.last() else {
            return false;
        };

        if !top.is_cancellable {
            self.trigger_attention();
            return false;
        }

        if let Some(mut dialog) = self.dialog_stack.pop()
            && let Some(action) = dialog.take_cancel_action()
        {
            action_bus.emit(action);
        }

        true
    }

    /// Pushes a pre-constructed [`ActiveDialog`] onto the top of the modal stack.
    pub fn push(&mut self, dialog: ActiveDialog<Action>) {
        self.dialog_stack.push(dialog);
    }

    /// Pops and returns the topmost dialogue from the modal stack, if any.
    pub fn pop(&mut self) -> Option<ActiveDialog<Action>> {
        self.dialog_stack.pop()
    }

    /// Dismisses the topmost dialogue immediately without emitting any affirmative action.
    ///
    /// Revealing the underlying dialogue if multiple modals were stacked.
    pub fn close(&mut self) {
        self.dialog_stack.pop();
    }

    /// Dismisses all stacked dialogues immediately, clearing the modal state completely.
    pub fn close_all(&mut self) {
        self.dialog_stack.clear();
    }

    /// Overrides the default visual styling applied to subsequently opened dialogues.
    pub fn set_default_style(&mut self, style: ModalStyle) {
        self.default_style = style;
    }

    /// Returns a reference to the default visual styling for dialogues.
    #[must_use]
    #[inline]
    pub fn default_style(&self) -> &ModalStyle {
        &self.default_style
    }

    /// Opens any generic [`DialogDescriptor`] (Modal, Sheet, Confirmation, Prompt, CustomView),
    /// pushing it onto the top of the modal dialogue stack.
    pub fn open(&mut self, descriptor: impl Into<DialogDescriptor<Action>>) {
        let mut active = ActiveDialog::from(descriptor.into());
        if active.style == ModalStyle::default() {
            active.style = self.default_style.clone();
        }
        self.dialog_stack.push(active);
    }

    /// Opens a two-button confirmation dialogue card on top of the modal stack.
    pub fn open_confirm(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        on_confirm: Action,
    ) {
        self.open(ConfirmationDialogDescriptor::new(
            title, message, on_confirm,
        ));
    }

    /// Opens a high-visibility danger confirmation dialogue (e.g. permanent deletion).
    pub fn open_confirm_danger(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        on_confirm: Action,
    ) {
        self.open(
            ConfirmationDialogDescriptor::new(title, message, on_confirm)
                .with_confirm_label("Delete")
                .with_danger(true),
        );
    }

    /// Opens a two-button confirmation dialogue with caller-specified button labels and danger styling.
    ///
    /// Completely unopinionated: allows arbitrary localized or customized confirm and cancel labels.
    pub fn open_confirm_custom(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        confirm_label: impl Into<String>,
        cancel_label: impl Into<String>,
        is_danger: bool,
        on_confirm: Action,
    ) {
        self.open(
            ConfirmationDialogDescriptor::new(title, message, on_confirm)
                .with_confirm_label(confirm_label)
                .with_cancel_label(cancel_label)
                .with_danger(is_danger),
        );
    }

    /// Opens a single-line text input dialogue card.
    pub fn open_input(
        &mut self,
        title: impl Into<String>,
        label: impl Into<String>,
        initial_text: impl Into<String>,
        on_submit: impl Fn(String) -> Action + Send + Sync + 'static,
    ) {
        self.open(
            PromptDialogDescriptor::new(title, label, on_submit).with_initial_text(initial_text),
        );
    }

    /// Opens a custom modal dialogue card with user-supplied rendering logic.
    pub fn open_custom<F>(&mut self, title: impl Into<String>, size: (f32, f32), render_fn: F)
    where
        F: FnMut(&mut UiScope<'_>, &mut ActionBus<Action>) + Send + Sync + 'static,
    {
        self.open(CustomDialogDescriptor::new(title, size, render_fn));
    }

    /// Opens an informational / about / diagnostic card dialogue on top of the modal stack.
    ///
    /// The info card displays structured application metadata, diagnostic properties, and
    /// copyright notes in an unopinionated declarative layout.
    pub fn open_info(&mut self, descriptor: InfoCardDialogDescriptor<Action>) {
        self.open(descriptor);
    }

    /// Opens a progress monitoring dialogue for long-running operations on top of the modal stack.
    ///
    /// Pushes the progress card onto the top of the modal stack and engages
    /// the backdrop scrim and input focus trap.
    pub fn open_progress(&mut self, descriptor: ProgressModalDescriptor<Action>) {
        self.open(descriptor);
    }

    /// Dynamically mutates the progress completion ratio and status message of the active progress dialogue.
    ///
    /// Performs an in-place $O(1)$ mutation without reallocating or shifting the modal stack.
    /// Returns `true` if the topmost dialogue was an active [`ProgressModalDescriptor`], or `false` otherwise.
    ///
    /// # Arguments
    /// * `progress` - Clamped ratio in range `[0.0, 1.0]`.
    /// * `status_message` - Optional updated descriptive text. If `None`, existing status message is preserved.
    pub fn update_progress(
        &mut self,
        progress: f32,
        status_message: Option<impl Into<String>>,
    ) -> bool {
        let Some(top) = self.dialog_stack.last_mut() else {
            return false;
        };

        let DialogKind::Progress {
            progress: ref mut prog,
            status_message: ref mut msg,
            ..
        } = top.kind
        else {
            return false;
        };

        *prog = progress.clamp(0.0, 1.0);
        if let Some(new_msg) = status_message {
            *msg = Some(new_msg.into());
        }

        true
    }

    /// Returns `true` if any progress monitoring dialogue is currently active on the modal stack.
    ///
    /// Iterates across active stacked dialogues and returns `true` as soon as a [`DialogKind::Progress`]
    /// variant is encountered.
    #[must_use]
    pub fn has_progress(&self) -> bool {
        self.dialog_stack
            .iter()
            .any(|d| matches!(d.kind, DialogKind::Progress { .. }))
    }

    /// Closes and removes the topmost progress dialogue from the modal stack, if active.
    ///
    /// Returns `true` if an active progress dialogue was found and dismissed, or `false` otherwise.
    pub fn close_progress(&mut self) -> bool {
        if let Some(pos) = self
            .dialog_stack
            .iter()
            .rposition(|d| matches!(d.kind, DialogKind::Progress { .. }))
        {
            self.dialog_stack.remove(pos);
            true
        } else {
            false
        }
    }

    /// Opens a sliding sheet drawer dialogue anchored to a window boundary.
    pub fn open_sheet(&mut self, title: impl Into<String>, anchor: SheetAnchor, extent: f32) {
        self.open(SheetDialogDescriptor::new(title, anchor, extent));
    }

    /// Intercepts hardware-independent keyboard events when one or more modals are active.
    ///
    /// Returns `true` if the event was consumed by the modal dialogue state machine,
    /// isolating underlying workbench views and 3D scenes from receiving input.
    pub fn handle_keyboard_input(
        &mut self,
        key: &KeyCode,
        is_pressed: bool,
        action_bus: &mut ActionBus<Action>,
    ) -> bool {
        self.handle_keyboard_input_with_modifiers(key, is_pressed, false, action_bus)
    }

    /// Intercepts hardware-independent keyboard events with modifier key support (`Shift`).
    ///
    /// Handles `KeyCode::Tab` for cyclic focus cycling forward or backward (`Shift+Tab`),
    /// `KeyCode::Escape` for dismissal or attention triggers, and `KeyCode::Enter` for affirmative
    /// actions or cancellation if the cancel button is focused.
    pub fn handle_keyboard_input_with_modifiers(
        &mut self,
        key: &KeyCode,
        is_pressed: bool,
        shift_held: bool,
        action_bus: &mut ActionBus<Action>,
    ) -> bool {
        DialogFocusTrap::handle_key_with_modifiers(
            &mut self.dialog_stack,
            key,
            is_pressed,
            shift_held,
            action_bus,
        )
    }

    /// Returns the currently focused interactive item on the topmost active modal dialog.
    #[must_use]
    pub fn focused_item(&self) -> Option<DialogFocusItem> {
        self.dialog_stack.last().and_then(|d| d.focused_item())
    }

    /// Cycles keyboard focus across interactive controls in the topmost active modal dialog.
    ///
    /// If `reverse` is `false`, navigates forward (`Tab`).
    /// If `reverse` is `true`, navigates backward (`Shift+Tab`).
    pub fn cycle_focus(&mut self, reverse: bool) -> Option<DialogFocusItem> {
        self.dialog_stack
            .last_mut()
            .and_then(|d| d.cycle_focus(reverse))
    }

    /// Evaluates whether keyboard focus is currently trapped by an active modal stack.
    #[must_use]
    #[inline]
    pub fn is_focus_trapped(&self) -> bool {
        DialogFocusTrap::is_trapped(self.dialog_stack.len())
    }

    /// Evaluates whether the workbench currently requires active text keyboard focus.
    #[must_use]
    #[inline]
    pub fn wants_keyboard_input(&self) -> bool {
        self.is_active()
    }

    /// Evaluates whether the workbench currently requires exclusive pointer capture for modal dialogs.
    #[must_use]
    #[inline]
    pub fn wants_pointer_input(&self) -> bool {
        self.is_active()
    }

    /// Evaluates whether a screen-space cursor coordinate falls within the modal occlusion zone.
    #[must_use]
    #[inline]
    pub fn contains_point(&self, point: Point, viewport: Rect) -> bool {
        DialogFocusTrap::occludes_point(self.is_active(), point, viewport)
    }

    /// Declaratively renders all stacked dialogues into the provided [`UiScope`].
    ///
    /// Manages scrim backdrop rendering, card geometry, dismissals, and action routing.
    pub fn render(
        &mut self,
        scope: &mut UiScope<'_>,
        viewport: Rect,
        escape_pressed: bool,
        action_bus: &mut ActionBus<Action>,
    ) {
        super::render::render_dialog_host(self, scope, viewport, escape_pressed, action_bus);
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
            .field("stack_depth", &self.stack_depth())
            .field("is_active", &self.is_active())
            .field(
                "active_dialog_title",
                &self.active_dialog().map(|d| d.title()),
            )
            .finish()
    }
}