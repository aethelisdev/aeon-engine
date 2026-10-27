// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Modal Controller and Dialog Renderer
//!
//! Autonomous coordinator handling full-screen scrim rendering, centered modal card
//! composition, header close buttons, and Escape/backdrop click dismissals.

use iris_core::geometry::Rect;
use iris_core::style::{AlignItems, JustifyContent, Style};
use iris_widgets::declarative::UiScope;

use super::descriptor::ModalDescriptor;
use super::response::{ModalDismissReason, ModalOutcome};

/// Autonomous coordinator rendering declarative modal dialogues.
pub struct ModalController;

impl ModalController {
    /// Declaratively renders an interactive modal dialogue into the active [`UiScope`].
    ///
    /// Manages the full modal lifecycle:
    /// 1. Intercepts keyboard Escape gestures if `dismiss_on_escape` is enabled.
    /// 2. Emits a full-screen semi-transparent backdrop scrim on [`iris_core::UiLayer::Modal`].
    /// 3. Intercepts backdrop clicks if `dismiss_on_backdrop` is enabled.
    /// 4. Renders an elevated, centered modal card with dark theme styling.
    /// 5. Renders a standard header bar with the title caption and an optional `✕` close button.
    /// 6. Evaluates user-supplied child content via `content`.
    ///
    /// # Generics
    /// - `Action`: Application-specific action payload produced when interacting with the modal.
    /// - `F`: Closure providing child widget content inside the modal card.
    pub fn render_modal<Action, F>(
        scope: &mut UiScope<'_>,
        descriptor: &ModalDescriptor,
        viewport: Rect,
        escape_pressed: bool,
        content: F,
    ) -> ModalOutcome<Action>
    where
        F: FnOnce(&mut UiScope<'_>) -> ModalOutcome<Action>,
    {
        // 1. Immediate Escape key dismissal check
        if escape_pressed && descriptor.dismiss_on_escape {
            return ModalOutcome::Dismissed(ModalDismissReason::EscapeKey);
        }

        let mut outcome = ModalOutcome::Active;
        let mut close_button_clicked = false;

        let scrim_resp = scope.modal_scrim_sized(viewport, descriptor.backdrop_color, |scrim| {
            scrim.modal_card(descriptor.size.0, descriptor.size.1, |card| {
                // Header Bar: Title subheading and optional '✕' close button
                card.container(
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::SpaceBetween),
                    |header| {
                        header.subheading(&descriptor.title);
                        if descriptor.show_close_button {
                            let resp = header.modal_close_button();
                            if resp.clicked() {
                                close_button_clicked = true;
                            }
                        }
                    },
                );

                // Child content invocation
                outcome = content(card);
            });
        });

        // 2. Evaluate close button click
        if close_button_clicked {
            return ModalOutcome::Dismissed(ModalDismissReason::CloseButton);
        }

        // 3. Evaluate backdrop click (clicked outside modal card)
        if descriptor.dismiss_on_backdrop && scrim_resp.response.clicked() {
            return ModalOutcome::Dismissed(ModalDismissReason::BackdropClick);
        }

        outcome
    }

    /// Renders a standard confirmation dialogue with Confirm and Cancel action buttons.
    ///
    /// # Parameters
    /// - `scope`: Active declarative UI scope.
    /// - `descriptor`: Modal card configuration.
    /// - `viewport`: Screen bounds for backdrop coverage.
    /// - `escape_pressed`: Whether the Escape key was pressed this frame.
    /// - `message`: Body description or warning text.
    /// - `confirm_label`: Text for the confirmation button (e.g. "Save", "Delete", "Confirm").
    /// - `is_danger`: Whether the confirm button should apply high-visibility danger styling.
    pub fn show_confirm_dialog(
        scope: &mut UiScope<'_>,
        descriptor: &ModalDescriptor,
        viewport: Rect,
        escape_pressed: bool,
        message: &str,
        confirm_label: &str,
        is_danger: bool,
    ) -> ModalOutcome<bool> {
        Self::render_modal(scope, descriptor, viewport, escape_pressed, |card| {
            let mut child_action = ModalOutcome::Active;

            // Body text column
            card.column(|col| {
                col.label(message);
            });

            // Footer Bar with Cancel and Confirm buttons
            card.container(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::FlexEnd)
                    .gap(8.0),
                |footer| {
                    let cancel_resp = footer.modal_cancel_button("Cancel", 75.0);
                    if cancel_resp.clicked() {
                        child_action = ModalOutcome::Dismissed(ModalDismissReason::CancelButton);
                    }

                    let confirm_clicked = if is_danger {
                        footer.modal_danger_button(confirm_label, 120.0).clicked()
                    } else {
                        footer.modal_confirm_button(confirm_label, 100.0).clicked()
                    };

                    if confirm_clicked {
                        child_action = ModalOutcome::Action(true);
                    }
                },
            );

            child_action
        })
    }
}