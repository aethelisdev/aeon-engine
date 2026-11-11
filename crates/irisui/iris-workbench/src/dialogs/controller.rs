// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Modal Controller and Dialog Renderer
//!
//! Autonomous coordinator handling full-screen scrim rendering, centered or positioned
//! modal cards, header close buttons, confirmation dialogues, and single-line text input dialogues.

use iris_core::geometry::Rect;
use iris_core::style::{AlignItems, JustifyContent, Style};
use iris_widgets::declarative::UiScope;

use super::descriptor::{
    ConfirmDialogDescriptor, InfoCardDialogDescriptor, InputDialogDescriptor, ModalDescriptor,
    ProgressModalDescriptor,
};
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

        let theme = scope.theme();
        let backdrop_color = descriptor
            .backdrop_color
            .or_else(|| descriptor.style.as_ref().and_then(|s| s.backdrop_color))
            .unwrap_or(theme.modal_scrim);

        let scrim_resp = scope.modal_scrim_sized(viewport, backdrop_color, |scrim| {
            scrim.modal_card(descriptor.size.0, descriptor.size.1, |card| {
                let card_id = card.parent();
                if let Some(card_node) = card.tree_mut().get_mut(card_id) {
                    let card_bg = descriptor
                        .style
                        .as_ref()
                        .map(|s| s.resolve_card_background(&theme))
                        .unwrap_or(theme.modal_bg);
                    let border_col = descriptor
                        .style
                        .as_ref()
                        .map(|s| s.resolve_card_border_color(&theme))
                        .unwrap_or(theme.modal_border);
                    let border_w = descriptor
                        .style
                        .as_ref()
                        .map(|s| s.card_border_width)
                        .unwrap_or(1.0);
                    let corner_r = descriptor
                        .style
                        .as_ref()
                        .map(|s| s.card_corner_radius)
                        .unwrap_or(8.0);
                    let pad = descriptor
                        .style
                        .as_ref()
                        .map(|s| s.card_padding)
                        .unwrap_or(16.0);
                    let gap = descriptor
                        .style
                        .as_ref()
                        .map(|s| s.content_gap)
                        .unwrap_or(12.0);

                    card_node.style = card_node
                        .style
                        .background(card_bg)
                        .border(border_w, border_col)
                        .border_radius(corner_r)
                        .padding(pad)
                        .gap(gap);

                    if let Some(pos) = descriptor.position {
                        card_node.style =
                            card_node.style.position_absolute().left(pos.x).top(pos.y);
                    }
                }

                // Header Bar: Icon, Title subheading and optional '✕' close button
                if descriptor.show_header {
                    let header_col = descriptor
                        .style
                        .as_ref()
                        .map(|s| s.resolve_header_text_color(&theme))
                        .unwrap_or(theme.text_primary);

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
                                        title_row.label_custom(
                                            icon,
                                            13.5,
                                            header_col,
                                            iris_core::TextAlign::Left,
                                        );
                                    }
                                    title_row.label_custom(
                                        &descriptor.title,
                                        13.5,
                                        header_col,
                                        iris_core::TextAlign::Left,
                                    );
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
                            .map(|s| s.resolve_subtitle_color(&theme))
                            .unwrap_or(theme.text_secondary);
                        card.label_custom(subtitle, 12.0, text_color, iris_core::TextAlign::Left);
                    }
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
        if scrim_resp.response.clicked() {
            if descriptor.dismiss_on_backdrop {
                return ModalOutcome::Dismissed(ModalDismissReason::BackdropClick);
            }
            return ModalOutcome::Attention;
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

        let theme = scope.theme();
        let backdrop_color = descriptor.style.resolve_backdrop_color(&theme);
        let card_bg = descriptor.style.resolve_card_background(&theme);
        let card_border_col = descriptor.style.resolve_card_border_color(&theme);
        let header_col = descriptor.style.resolve_header_text_color(&theme);
        let subtitle_col = descriptor.style.resolve_subtitle_color(&theme);
        let danger_col = descriptor.style.resolve_danger_color(&theme);

        let scrim_resp = scope.modal_scrim_sized(viewport, backdrop_color, |scrim| {
            scrim.modal_card(descriptor.size.0, descriptor.size.1, |card| {
                let card_id = card.parent();
                if let Some(card_node) = card.tree_mut().get_mut(card_id) {
                    card_node.style = card_node
                        .style
                        .background(card_bg)
                        .border(descriptor.style.card_border_width, card_border_col)
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
                                    title_row.label_custom(
                                        icon,
                                        13.5,
                                        header_col,
                                        iris_core::TextAlign::Left,
                                    );
                                }
                                title_row.label_custom(
                                    &descriptor.title,
                                    13.5,
                                    header_col,
                                    iris_core::TextAlign::Left,
                                );
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
                        col.colored_label(danger_col, &descriptor.message);
                    } else {
                        col.colored_label(header_col, &descriptor.message);
                    }

                    if let Some(detail) = &descriptor.detail {
                        col.colored_label(subtitle_col, detail);
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

        if scrim_resp.response.clicked() {
            if descriptor.dismiss_on_backdrop {
                return ModalOutcome::Dismissed(ModalDismissReason::BackdropClick);
            }
            return ModalOutcome::Attention;
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

        let theme = scope.theme();
        let backdrop_color = descriptor.style.resolve_backdrop_color(&theme);
        let card_bg = descriptor.style.resolve_card_background(&theme);
        let card_border_col = descriptor.style.resolve_card_border_color(&theme);
        let header_col = descriptor.style.resolve_header_text_color(&theme);
        let subtitle_col = descriptor.style.resolve_subtitle_color(&theme);

        let scrim_resp = scope.modal_scrim_sized(viewport, backdrop_color, |scrim| {
            scrim.modal_card(descriptor.size.0, descriptor.size.1, |card| {
                let card_id = card.parent();
                if let Some(card_node) = card.tree_mut().get_mut(card_id) {
                    card_node.style = card_node
                        .style
                        .background(card_bg)
                        .border(descriptor.style.card_border_width, card_border_col)
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
                                    title_row.label_custom(
                                        icon,
                                        13.5,
                                        header_col,
                                        iris_core::TextAlign::Left,
                                    );
                                }
                                title_row.label_custom(
                                    &descriptor.title,
                                    13.5,
                                    header_col,
                                    iris_core::TextAlign::Left,
                                );
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
                        col.colored_label(subtitle_col, detail);
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

        if scrim_resp.response.clicked() {
            if descriptor.dismiss_on_backdrop {
                return ModalOutcome::Dismissed(ModalDismissReason::BackdropClick);
            }
            return ModalOutcome::Attention;
        }

        if confirmed {
            return ModalOutcome::Action(input_buffer.clone());
        }

        outcome
    }

    /// Declaratively renders an unopinionated information / about dialogue card.
    ///
    /// Presents structured application metadata, diagnostic properties, and copyright notes
    /// in an unopinionated declarative layout with an optional affirmative close button.
    pub fn render_info_dialog<Action: 'static + Send + Sync>(
        scope: &mut UiScope<'_>,
        descriptor: &InfoCardDialogDescriptor<Action>,
        viewport: Rect,
        escape_pressed: bool,
    ) -> ModalOutcome<bool> {
        if escape_pressed && descriptor.dismiss_on_escape {
            return ModalOutcome::Dismissed(ModalDismissReason::EscapeKey);
        }

        let mut close_button_clicked = false;
        let mut confirmed = false;

        let theme = scope.theme();
        let backdrop_color = descriptor
            .backdrop_color
            .or(descriptor.style.backdrop_color)
            .unwrap_or(theme.modal_scrim);
        let card_bg = descriptor.style.resolve_card_background(&theme);
        let card_border_col = descriptor.style.resolve_card_border_color(&theme);
        let header_col = descriptor.style.resolve_header_text_color(&theme);
        let subtitle_col = descriptor.style.resolve_subtitle_color(&theme);

        let scrim_resp = scope.modal_scrim_sized(viewport, backdrop_color, |scrim| {
            scrim.modal_card(descriptor.size.0, descriptor.size.1, |card| {
                let card_id = card.parent();
                if let Some(card_node) = card.tree_mut().get_mut(card_id) {
                    card_node.style = card_node
                        .style
                        .background(card_bg)
                        .border(descriptor.style.card_border_width, card_border_col)
                        .border_radius(descriptor.style.card_corner_radius)
                        .padding(descriptor.style.card_padding)
                        .gap(descriptor.style.content_gap);

                    if let Some(pos) = descriptor.position {
                        card_node.style =
                            card_node.style.position_absolute().left(pos.x).top(pos.y);
                    }
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
                                    title_row.label_custom(
                                        icon,
                                        13.5,
                                        header_col,
                                        iris_core::TextAlign::Left,
                                    );
                                }
                                title_row.label_custom(
                                    &descriptor.title,
                                    13.5,
                                    header_col,
                                    iris_core::TextAlign::Left,
                                );
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
                    card.label_custom(subtitle, 12.0, subtitle_col, iris_core::TextAlign::Left);
                }

                // Body Column
                card.column(|col| {
                    if let Some(app) = &descriptor.app_name {
                        if let Some(color) = descriptor.app_name_color {
                            col.label_custom(app, 20.0, color, iris_core::TextAlign::Center);
                        } else {
                            col.heading(app);
                        }
                    }
                    if let Some(ver) = &descriptor.version {
                        col.colored_label(subtitle_col, ver);
                    }
                    if let Some(copy) = &descriptor.copyright {
                        col.colored_label(subtitle_col, copy);
                    }
                    if let Some(desc) = &descriptor.description {
                        col.separator();
                        col.colored_label(subtitle_col, desc);
                    }
                    if let Some(link) = &descriptor.link
                        && col.link(&link.label, 11.0).clicked()
                    {
                        iris_core::platform::open_url(&link.url);
                    }
                    if !descriptor.entries.is_empty() {
                        col.separator();
                        for entry in &descriptor.entries {
                            col.row(|row| {
                                row.colored_label(subtitle_col, &entry.label);
                                row.label(&entry.value);
                            });
                        }
                    }
                    if let Some(notice) = &descriptor.footer_notice {
                        let color = descriptor.footer_notice_color.unwrap_or(subtitle_col);
                        col.colored_label(color, notice);
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
                        let confirm_resp = footer.modal_confirm_button(
                            &descriptor.close_label,
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

        if scrim_resp.response.clicked() {
            if descriptor.dismiss_on_backdrop {
                return ModalOutcome::Dismissed(ModalDismissReason::BackdropClick);
            }
            return ModalOutcome::Attention;
        }

        if confirmed {
            return ModalOutcome::Action(true);
        }

        ModalOutcome::Active
    }

    /// Declaratively renders an unopinionated progress modal dialogue card.
    ///
    /// Displays an operation title, optional subtitle, dynamic status message,
    /// fractional progress meter, and an optional cancellation button.
    pub fn render_progress_dialog<Action: 'static + Send + Sync>(
        scope: &mut UiScope<'_>,
        descriptor: &ProgressModalDescriptor<Action>,
        viewport: Rect,
        escape_pressed: bool,
    ) -> ModalOutcome<bool> {
        if escape_pressed && descriptor.is_cancellable {
            return ModalOutcome::Action(true);
        }

        let mut close_button_clicked = false;
        let mut cancel_button_clicked = false;

        let theme = scope.theme();
        let backdrop_color = descriptor
            .backdrop_color
            .or(descriptor.style.backdrop_color)
            .unwrap_or(theme.modal_scrim);
        let card_bg = descriptor.style.resolve_card_background(&theme);
        let card_border_col = descriptor.style.resolve_card_border_color(&theme);
        let header_col = descriptor.style.resolve_header_text_color(&theme);
        let subtitle_col = descriptor.style.resolve_subtitle_color(&theme);

        let scrim_resp = scope.modal_scrim_sized(viewport, backdrop_color, |scrim| {
            scrim.modal_card(descriptor.size.0, descriptor.size.1, |card| {
                let card_id = card.parent();
                if let Some(card_node) = card.tree_mut().get_mut(card_id) {
                    card_node.style = card_node
                        .style
                        .background(card_bg)
                        .border(descriptor.style.card_border_width, card_border_col)
                        .border_radius(descriptor.style.card_corner_radius)
                        .padding(descriptor.style.card_padding)
                        .gap(descriptor.style.content_gap);

                    if let Some(pos) = descriptor.position {
                        card_node.style =
                            card_node.style.position_absolute().left(pos.x).top(pos.y);
                    }
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
                                    title_row.label_custom(
                                        icon,
                                        13.5,
                                        header_col,
                                        iris_core::TextAlign::Left,
                                    );
                                }
                                title_row.label_custom(
                                    &descriptor.title,
                                    13.5,
                                    header_col,
                                    iris_core::TextAlign::Left,
                                );
                            },
                        );
                        if descriptor.is_cancellable {
                            let resp = header.modal_close_button();
                            if resp.clicked() {
                                close_button_clicked = true;
                            }
                        }
                    },
                );

                if let Some(subtitle) = &descriptor.subtitle {
                    card.label_custom(subtitle, 12.0, subtitle_col, iris_core::TextAlign::Left);
                }

                // Body: Status message & Progress Bar
                card.column(|col| {
                    if let Some(status) = &descriptor.status_message {
                        col.label(status);
                    }
                    if let Some(fill_color) = descriptor.progress_color {
                        col.progress_bar_named(
                            "ProgressTrack",
                            "ProgressFill",
                            descriptor.size.0 - (descriptor.style.card_padding * 2.0),
                            descriptor.progress_height,
                            descriptor.progress,
                            fill_color,
                        );
                    } else {
                        col.progress_bar(descriptor.progress);
                    }
                });

                // Footer Bar (Cancel button if cancellable)
                if descriptor.is_cancellable {
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
                                cancel_button_clicked = true;
                            }
                        },
                    );
                }
            });
        });

        if close_button_clicked || cancel_button_clicked {
            return ModalOutcome::Action(true);
        }

        if scrim_resp.response.clicked() {
            if descriptor.is_cancellable && descriptor.dismiss_on_backdrop {
                return ModalOutcome::Action(true);
            }
            return ModalOutcome::Attention;
        }

        ModalOutcome::Active
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