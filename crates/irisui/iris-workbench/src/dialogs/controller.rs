// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Modal Controller and Dialog Renderer
//!
//! Autonomous coordinator handling full-screen scrim rendering, centered or positioned
//! modal cards, header close buttons, confirmation dialogues, and single-line text input dialogues.

use iris_core::geometry::Rect;
use iris_core::style::{AlignItems, JustifyContent, Style};
use iris_widgets::declarative::UiScope;

use super::descriptor::{ConfirmDialogDescriptor, InputDialogDescriptor, ModalDescriptor};
use super::response::{ModalDismissReason, ModalOutcome};
use super::style::ModalStyle;

/// Autonomous coordinator rendering declarative modal dialogues.
pub struct ModalController;

impl ModalController {
    /// Declaratively renders an interactive modal dialogue into the active [`UiScope`].
    ///
    /// Manages the full modal lifecycle:
    /// 1. Intercepts keyboard Escape gestures if `dismiss_on_escape` is enabled.
    /// 2. Emits a full-screen semi-transparent backdrop scrim on [`iris_core::UiLayer::Modal`].
    /// 3. Intercepts backdrop clicks if `dismiss_on_backdrop` is enabled.
    /// 4. Renders an elevated modal card with dark theme styling (centered or explicitly positioned).
    /// 5. Renders a standard header bar with title, optional icon, and optional `✕` close button.
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

        let backdrop_color = descriptor
            .style
            .as_ref()
            .map(|s| s.backdrop_color)
            .unwrap_or(descriptor.backdrop_color);

        let scrim_resp = scope.modal_scrim_sized(viewport, backdrop_color, |scrim| {
            scrim.modal_card(descriptor.size.0, descriptor.size.1, |card| {
                let card_id = card.parent();
                if let Some(card_node) = card.tree_mut().get_mut(card_id) {
                    if let Some(style) = &descriptor.style {
                        card_node.style = card_node
                            .style
                            .background(style.card_background)
                            .border(style.card_border_width, style.card_border_color)
                            .border_radius(style.card_corner_radius)
                            .padding(style.card_padding)
                            .gap(style.content_gap);
                    }

                    if let Some(pos) = descriptor.position {
                        card_node.style =
                            card_node.style.position_absolute().left(pos.x).top(pos.y);
                    }
                }

                // Header Bar: Icon, Title subheading and optional '✕' close button
                card.container(
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::SpaceBetween),
                    |header| {
                        header.container(
                            Style::new()
                                .flex_row()
                                .align_items(AlignItems::Center)
                                .gap(8.0),
                            |title_row| {
                                if let Some(icon) = &descriptor.icon {
                                    title_row.subheading(icon);
                                }
                                title_row.subheading(&descriptor.title);
                            },
                        );

                        if descriptor.show_close_button {
                            let resp = header.modal_close_button();
                            if resp.clicked() {
                                close_button_clicked = true;
                            }
                        }
                    },
                );

                if let Some(subtitle) = &descriptor.subtitle {
                    let text_color = descriptor
                        .style
                        .as_ref()
                        .map(|s| s.subtitle_color)
                        .unwrap_or(ModalStyle::default().subtitle_color);
                    card.label_custom(subtitle, 12.0, text_color, iris_core::TextAlign::Left);
                }

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

    /// Declaratively renders a confirmation dialogue card with message, optional detail, Cancel, and Confirm buttons.
    pub fn render_confirm_dialog(
        scope: &mut UiScope<'_>,
        descriptor: &ConfirmDialogDescriptor,
        viewport: Rect,
        escape_pressed: bool,
    ) -> ModalOutcome<bool> {
        if escape_pressed && descriptor.dismiss_on_escape {
            return ModalOutcome::Dismissed(ModalDismissReason::EscapeKey);
        }

        let mut outcome = ModalOutcome::Active;
        let mut close_button_clicked = false;

        let scrim_resp =
            scope.modal_scrim_sized(viewport, descriptor.style.backdrop_color, |scrim| {
                scrim.modal_card(descriptor.size.0, descriptor.size.1, |card| {
                    let card_id = card.parent();
                    if let Some(card_node) = card.tree_mut().get_mut(card_id) {
                        card_node.style = card_node
                            .style
                            .background(descriptor.style.card_background)
                            .border(
                                descriptor.style.card_border_width,
                                descriptor.style.card_border_color,
                            )
                            .border_radius(descriptor.style.card_corner_radius)
                            .padding(descriptor.style.card_padding)
                            .gap(descriptor.style.content_gap);
                    }

                    // Header Bar
                    card.container(
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::SpaceBetween),
                        |header| {
                            header.container(
                                Style::new()
                                    .flex_row()
                                    .align_items(AlignItems::Center)
                                    .gap(8.0),
                                |title_row| {
                                    if let Some(icon) = &descriptor.icon {
                                        title_row.subheading(icon);
                                    }
                                    title_row.subheading(&descriptor.title);
                                },
                            );
                            if descriptor.show_close_button {
                                let resp = header.modal_close_button();
                                if resp.clicked() {
                                    close_button_clicked = true;
                                }
                            }
                        },
                    );

                    // Body Column
                    card.column(|col| {
                        if descriptor.is_danger {
                            col.colored_label(descriptor.style.danger_color, &descriptor.message);
                        } else {
                            col.label(&descriptor.message);
                        }

                        if let Some(detail) = &descriptor.detail {
                            col.colored_label(descriptor.style.subtitle_color, detail);
                        }
                    });

                    // Footer Bar
                    card.container(
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::FlexEnd)
                            .gap(descriptor.style.footer_gap),
                        |footer| {
                            let cancel_resp = footer.modal_cancel_button(
                                &descriptor.cancel_label,
                                descriptor.style.cancel_button_width,
                            );
                            if cancel_resp.clicked() {
                                outcome = ModalOutcome::Dismissed(ModalDismissReason::CancelButton);
                            }

                            let confirm_clicked = if descriptor.is_danger {
                                footer
                                    .modal_danger_button(
                                        &descriptor.confirm_label,
                                        descriptor.style.confirm_button_width,
                                    )
                                    .clicked()
                            } else {
                                footer
                                    .modal_confirm_button(
                                        &descriptor.confirm_label,
                                        descriptor.style.confirm_button_width,
                                    )
                                    .clicked()
                            };

                            if confirm_clicked {
                                outcome = ModalOutcome::Action(true);
                            }
                        },
                    );
                });
            });

        if close_button_clicked {
            return ModalOutcome::Dismissed(ModalDismissReason::CloseButton);
        }

        if descriptor.dismiss_on_backdrop && scrim_resp.response.clicked() {
            return ModalOutcome::Dismissed(ModalDismissReason::BackdropClick);
        }

        outcome
    }

    /// Declaratively renders a single-line text input dialogue card with prompt, text box, Cancel, and Confirm buttons.
    pub fn render_input_dialog(
        scope: &mut UiScope<'_>,
        descriptor: &InputDialogDescriptor,
        input_buffer: &mut String,
        viewport: Rect,
        escape_pressed: bool,
    ) -> ModalOutcome<String> {
        if escape_pressed && descriptor.dismiss_on_escape {
            return ModalOutcome::Dismissed(ModalDismissReason::EscapeKey);
        }

        let mut outcome = ModalOutcome::Active;
        let mut close_button_clicked = false;
        let mut confirmed = false;

        let scrim_resp =
            scope.modal_scrim_sized(viewport, descriptor.style.backdrop_color, |scrim| {
                scrim.modal_card(descriptor.size.0, descriptor.size.1, |card| {
                    let card_id = card.parent();
                    if let Some(card_node) = card.tree_mut().get_mut(card_id) {
                        card_node.style = card_node
                            .style
                            .background(descriptor.style.card_background)
                            .border(
                                descriptor.style.card_border_width,
                                descriptor.style.card_border_color,
                            )
                            .border_radius(descriptor.style.card_corner_radius)
                            .padding(descriptor.style.card_padding)
                            .gap(descriptor.style.content_gap);
                    }

                    // Header Bar
                    card.container(
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::SpaceBetween),
                        |header| {
                            header.container(
                                Style::new()
                                    .flex_row()
                                    .align_items(AlignItems::Center)
                                    .gap(8.0),
                                |title_row| {
                                    if let Some(icon) = &descriptor.icon {
                                        title_row.subheading(icon);
                                    }
                                    title_row.subheading(&descriptor.title);
                                },
                            );
                            if descriptor.show_close_button {
                                let resp = header.modal_close_button();
                                if resp.clicked() {
                                    close_button_clicked = true;
                                }
                            }
                        },
                    );

                    // Body Column
                    card.column(|col| {
                        if let Some(detail) = &descriptor.detail {
                            col.colored_label(descriptor.style.subtitle_color, detail);
                        }
                        if !descriptor.prompt.is_empty() {
                            col.label(&descriptor.prompt);
                        }
                        col.text_input(&descriptor.placeholder, input_buffer);
                    });

                    // Footer Bar
                    card.container(
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::FlexEnd)
                            .gap(descriptor.style.footer_gap),
                        |footer| {
                            let cancel_resp = footer.modal_cancel_button(
                                &descriptor.cancel_label,
                                descriptor.style.cancel_button_width,
                            );
                            if cancel_resp.clicked() {
                                outcome = ModalOutcome::Dismissed(ModalDismissReason::CancelButton);
                            }

                            let confirm_resp = footer.modal_confirm_button(
                                &descriptor.confirm_label,
                                descriptor.style.confirm_button_width,
                            );
                            if confirm_resp.clicked() {
                                confirmed = true;
                            }
                        },
                    );
                });
            });

        if close_button_clicked {
            return ModalOutcome::Dismissed(ModalDismissReason::CloseButton);
        }

        if descriptor.dismiss_on_backdrop && scrim_resp.response.clicked() {
            return ModalOutcome::Dismissed(ModalDismissReason::BackdropClick);
        }

        if confirmed {
            return ModalOutcome::Action(input_buffer.clone());
        }

        outcome
    }

    /// Renders a standard confirmation dialogue with Confirm and Cancel action buttons.
    pub fn show_confirm_dialog(
        scope: &mut UiScope<'_>,
        descriptor: &ModalDescriptor,
        viewport: Rect,
        escape_pressed: bool,
        message: &str,
        confirm_label: &str,
        is_danger: bool,
    ) -> ModalOutcome<bool> {
        let confirm_desc = ConfirmDialogDescriptor::new(&descriptor.title, message)
            .with_confirm_label(confirm_label)
            .with_danger(is_danger)
            .with_size(descriptor.size.0, descriptor.size.1)
            .with_dismiss_on_backdrop(descriptor.dismiss_on_backdrop)
            .with_dismiss_on_escape(descriptor.dismiss_on_escape)
            .with_close_button(descriptor.show_close_button);

        Self::render_confirm_dialog(scope, &confirm_desc, viewport, escape_pressed)
    }
}