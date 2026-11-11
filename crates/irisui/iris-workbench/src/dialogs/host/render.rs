// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Dialog Stack Renderer
//!
//! Orchestrates visual scrim rendering, centered/sheet dialog geometry, action dispatch,
//! and stack-aware dismissal transitions.

use crate::action_bus::ActionBus;
use crate::dialogs::controller::ModalController;
use crate::dialogs::descriptor::confirm::ConfirmDialogDescriptor;
use crate::dialogs::descriptor::input::InputDialogDescriptor;
use crate::dialogs::descriptor::modal::ModalDescriptor;
use crate::dialogs::descriptor::sheet::SheetAnchor;
use crate::dialogs::host::active::{ActiveDialog, DialogKind};
use crate::dialogs::host::stack::DialogHost;
use crate::dialogs::response::ModalOutcome;
use iris_core::geometry::{Point, Rect};
use iris_widgets::declarative::UiScope;

/// Declaratively renders the active dialog stack into the provided [`UiScope`].
pub fn render_dialog_host<Action: 'static + Send + Sync>(
    host: &mut DialogHost<Action>,
    scope: &mut UiScope<'_>,
    viewport: Rect,
    escape_pressed: bool,
    action_bus: &mut ActionBus<Action>,
) {
    if host.dialog_stack.is_empty() {
        return;
    }

    if escape_pressed {
        if !should_dismiss_top_on_escape(host) {
            host.trigger_attention();
        } else if let Some(mut dialog) = host.dialog_stack.pop()
            && let Some(action) = dialog.take_cancel_action()
        {
            action_bus.emit(action);
        }
        if host.dialog_stack.is_empty() {
            return;
        }
    }

    render_stacked_dialogs(host, scope, viewport, action_bus);
}

/// Evaluates whether the topmost dialog allows dismissal via the Escape key.
fn should_dismiss_top_on_escape<Action: 'static + Send + Sync>(host: &DialogHost<Action>) -> bool {
    host.dialog_stack
        .last()
        .map(|d| d.is_cancellable && d.dismiss_on_escape)
        .unwrap_or(false)
}

/// Renders all dialogues in bottom-to-top stacking order, granting exclusive interaction to the top.
fn render_stacked_dialogs<Action: 'static + Send + Sync>(
    host: &mut DialogHost<Action>,
    scope: &mut UiScope<'_>,
    viewport: Rect,
    action_bus: &mut ActionBus<Action>,
) {
    let stack_len = host.dialog_stack.len();
    if stack_len == 0 {
        return;
    }

    // Render underlying background dialogs if any (non-interactive display only)
    if stack_len > 1 {
        for idx in 0..(stack_len - 1) {
            render_background_dialog(&mut host.dialog_stack[idx], scope, viewport);
        }
    }

    // Render the active topmost dialog with full interaction
    render_top_dialog(host, scope, viewport, action_bus);
}

/// Renders an underlying inactive modal dialog in the background without event handling.
fn render_background_dialog<Action: 'static + Send + Sync>(
    dialog: &mut ActiveDialog<Action>,
    scope: &mut UiScope<'_>,
    viewport: Rect,
) {
    let mut style = dialog.style.clone();
    // Dim background dialog scrims slightly so they don't over-darken the viewport
    style.backdrop_color.a *= 0.5;

    match &dialog.kind {
        DialogKind::Confirm {
            title,
            message,
            detail,
            confirm_label,
            cancel_label,
            is_danger,
            ..
        } => {
            let mut desc = ConfirmDialogDescriptor::new(title.as_str(), message.as_str())
                .with_confirm_label(confirm_label.as_str())
                .with_cancel_label(cancel_label.as_str())
                .with_danger(*is_danger)
                .with_size(dialog.size.0, dialog.size.1)
                .with_dismiss_on_backdrop(false)
                .with_dismiss_on_escape(false)
                .with_close_button(dialog.show_close_button)
                .with_style(style);

            if let Some(det) = detail {
                desc = desc.with_detail(det.as_str());
            }

            let _ = ModalController::render_confirm_dialog(scope, &desc, viewport, false);
        }
        DialogKind::Input {
            title,
            label,
            text,
            placeholder,
            confirm_label,
            cancel_label,
            ..
        } => {
            let desc = InputDialogDescriptor::new(title.as_str(), label.as_str())
                .with_confirm_label(confirm_label.as_str())
                .with_cancel_label(cancel_label.as_str())
                .with_placeholder(placeholder.as_str())
                .with_size(dialog.size.0, dialog.size.1)
                .with_dismiss_on_backdrop(false)
                .with_dismiss_on_escape(false)
                .with_close_button(dialog.show_close_button)
                .with_style(style);

            let mut text_clone = text.clone();
            let _ = ModalController::render_input_dialog(
                scope,
                &desc,
                &mut text_clone,
                viewport,
                false,
            );
        }
        DialogKind::Custom { .. } => {
            let desc = dialog.build_modal_desc(false);

            let _ = ModalController::render_modal(scope, &desc, viewport, false, |_| {
                ModalOutcome::<()>::Active
            });
        }
        DialogKind::InfoCard { .. } => {
            if let Some(desc) = dialog.build_info_desc(false) {
                let _ = ModalController::render_info_dialog(scope, &desc, viewport, false);
            }
        }
        DialogKind::Progress { .. } => {
            if let Some(desc) = dialog.build_progress_desc(false) {
                let _ = ModalController::render_progress_dialog(scope, &desc, viewport, false);
            }
        }
        DialogKind::Sheet {
            title,
            anchor,
            extent,
            ..
        } => {
            let (size, pos) = compute_sheet_geometry(*anchor, *extent, viewport);
            let desc = ModalDescriptor::new(title.as_str(), size.0, size.1)
                .with_position(pos)
                .with_dismiss_on_backdrop(false)
                .with_dismiss_on_escape(false)
                .with_close_button(dialog.show_close_button)
                .with_style(style);

            let _ = ModalController::render_modal(scope, &desc, viewport, false, |_| {
                ModalOutcome::<()>::Active
            });
        }
    }
}

/// Renders the topmost dialog and handles its affirmative/dismissal outcomes.
fn render_top_dialog<Action: 'static + Send + Sync>(
    host: &mut DialogHost<Action>,
    scope: &mut UiScope<'_>,
    viewport: Rect,
    action_bus: &mut ActionBus<Action>,
) {
    let top_idx = host.dialog_stack.len().saturating_sub(1);
    let top = &mut host.dialog_stack[top_idx];
    top.tick_attention();
    let custom_desc = top.build_modal_desc(true);

    match &mut top.kind {
        DialogKind::Confirm {
            title,
            message,
            detail,
            confirm_label,
            cancel_label,
            is_danger,
            ..
        } => {
            let mut style = top.style.clone();
            if top.attention_ticks > 0 {
                style.card_border_color = top.style.attention_border_color;
            }

            let mut desc = ConfirmDialogDescriptor::new(title.as_str(), message.as_str())
                .with_confirm_label(confirm_label.as_str())
                .with_cancel_label(cancel_label.as_str())
                .with_danger(*is_danger)
                .with_size(top.size.0, top.size.1)
                .with_dismiss_on_backdrop(top.dismiss_on_backdrop)
                .with_dismiss_on_escape(top.dismiss_on_escape)
                .with_cancellable(top.is_cancellable)
                .with_close_button(top.show_close_button)
                .with_style(style);

            if let Some(det) = detail {
                desc = desc.with_detail(det.as_str());
            }

            let outcome = ModalController::render_confirm_dialog(scope, &desc, viewport, false);
            handle_confirm_outcome(host, outcome, action_bus);
        }
        DialogKind::Input {
            title,
            label,
            text,
            placeholder,
            confirm_label,
            cancel_label,
            ..
        } => {
            let mut style = top.style.clone();
            if top.attention_ticks > 0 {
                style.card_border_color = top.style.attention_border_color;
            }

            let desc = InputDialogDescriptor::new(title.as_str(), label.as_str())
                .with_confirm_label(confirm_label.as_str())
                .with_cancel_label(cancel_label.as_str())
                .with_placeholder(placeholder.as_str())
                .with_size(top.size.0, top.size.1)
                .with_dismiss_on_backdrop(top.dismiss_on_backdrop)
                .with_dismiss_on_escape(top.dismiss_on_escape)
                .with_cancellable(top.is_cancellable)
                .with_close_button(top.show_close_button)
                .with_style(style);

            let outcome = ModalController::render_input_dialog(scope, &desc, text, viewport, false);
            handle_input_outcome(host, outcome, action_bus);
        }
        DialogKind::Custom { render_fn, .. } => {
            let outcome =
                ModalController::render_modal(scope, &custom_desc, viewport, false, |card_scope| {
                    render_fn(card_scope, action_bus);
                    ModalOutcome::<()>::Active
                });

            if matches!(outcome, ModalOutcome::Dismissed(_)) {
                if let Some(mut dialog) = host.dialog_stack.pop()
                    && let Some(action) = dialog.take_cancel_action()
                {
                    action_bus.emit(action);
                }
            } else if outcome.is_attention() {
                host.trigger_attention();
            }
        }
        DialogKind::InfoCard { .. } => {
            if let Some(desc) = top.build_info_desc(true) {
                let outcome = ModalController::render_info_dialog(scope, &desc, viewport, false);
                handle_info_outcome(host, outcome, action_bus);
            }
        }
        DialogKind::Progress { .. } => {
            if let Some(desc) = top.build_progress_desc(true) {
                let outcome =
                    ModalController::render_progress_dialog(scope, &desc, viewport, false);
                handle_progress_outcome(host, outcome, action_bus);
            }
        }
        DialogKind::Sheet {
            title,
            anchor,
            extent,
            render_fn,
            ..
        } => {
            let (size, pos) = compute_sheet_geometry(*anchor, *extent, viewport);
            let desc = ModalDescriptor::new(title.as_str(), size.0, size.1)
                .with_position(pos)
                .with_dismiss_on_backdrop(top.dismiss_on_backdrop)
                .with_dismiss_on_escape(top.dismiss_on_escape)
                .with_cancellable(top.is_cancellable)
                .with_close_button(top.show_close_button)
                .with_style(top.style.clone());

            let outcome =
                ModalController::render_modal(scope, &desc, viewport, false, |card_scope| {
                    if let Some(render) = render_fn {
                        render(card_scope, action_bus);
                    }
                    ModalOutcome::<()>::Active
                });

            if matches!(outcome, ModalOutcome::Dismissed(_)) {
                if let Some(mut dialog) = host.dialog_stack.pop()
                    && let Some(action) = dialog.take_cancel_action()
                {
                    action_bus.emit(action);
                }
            } else if outcome.is_attention() {
                host.trigger_attention();
            }
        }
    }
}

/// Processes the result of a confirmation dialogue.
fn handle_confirm_outcome<Action: 'static + Send + Sync>(
    host: &mut DialogHost<Action>,
    outcome: ModalOutcome<bool>,
    action_bus: &mut ActionBus<Action>,
) {
    match outcome {
        ModalOutcome::Action(true) => {
            if let Some(dialog) = host.dialog_stack.pop()
                && let DialogKind::Confirm { on_confirm, .. } = dialog.kind
            {
                action_bus.emit(on_confirm);
            }
        }
        ModalOutcome::Action(false) | ModalOutcome::Dismissed(_) => {
            if let Some(top) = host.dialog_stack.last()
                && !top.is_cancellable
            {
                host.trigger_attention();
            } else if let Some(mut dialog) = host.dialog_stack.pop()
                && let Some(action) = dialog.take_cancel_action()
            {
                action_bus.emit(action);
            }
        }
        ModalOutcome::Attention => {
            host.trigger_attention();
        }
        _ => {}
    }
}

/// Processes the result of an information / about dialogue.
fn handle_info_outcome<Action: 'static + Send + Sync>(
    host: &mut DialogHost<Action>,
    outcome: ModalOutcome<bool>,
    action_bus: &mut ActionBus<Action>,
) {
    match outcome {
        ModalOutcome::Action(true) => {
            if let Some(dialog) = host.dialog_stack.pop()
                && let DialogKind::InfoCard { on_close, .. } = dialog.kind
                && let Some(action) = on_close
            {
                action_bus.emit(action);
            }
        }
        ModalOutcome::Action(false) | ModalOutcome::Dismissed(_) => {
            if let Some(top) = host.dialog_stack.last()
                && !top.is_cancellable
            {
                host.trigger_attention();
            } else {
                host.dialog_stack.pop();
            }
        }
        ModalOutcome::Attention => {
            host.trigger_attention();
        }
        _ => {}
    }
}

/// Processes the result of a progress modal dialogue.
fn handle_progress_outcome<Action: 'static + Send + Sync>(
    host: &mut DialogHost<Action>,
    outcome: ModalOutcome<bool>,
    action_bus: &mut ActionBus<Action>,
) {
    match outcome {
        ModalOutcome::Action(true) | ModalOutcome::Dismissed(_) => {
            if let Some(top) = host.dialog_stack.last()
                && !top.is_cancellable
            {
                host.trigger_attention();
            } else if let Some(mut dialog) = host.dialog_stack.pop()
                && let Some(action) = dialog.take_cancel_action()
            {
                action_bus.emit(action);
            }
        }
        ModalOutcome::Attention => {
            host.trigger_attention();
        }
        _ => {}
    }
}

/// Processes the result of a single-line text input dialogue.
fn handle_input_outcome<Action: 'static + Send + Sync>(
    host: &mut DialogHost<Action>,
    outcome: ModalOutcome<String>,
    action_bus: &mut ActionBus<Action>,
) {
    match outcome {
        ModalOutcome::Action(submitted_text) => {
            if let Some(dialog) = host.dialog_stack.pop()
                && let DialogKind::Input { on_submit, .. } = dialog.kind
            {
                let action = on_submit(submitted_text);
                action_bus.emit(action);
            }
        }
        ModalOutcome::Dismissed(_) => {
            if let Some(top) = host.dialog_stack.last()
                && !top.is_cancellable
            {
                host.trigger_attention();
            } else if let Some(mut dialog) = host.dialog_stack.pop()
                && let Some(action) = dialog.take_cancel_action()
            {
                action_bus.emit(action);
            }
        }
        ModalOutcome::Attention => {
            host.trigger_attention();
        }
        _ => {}
    }
}

/// Computes logical dimensions and position coordinates for sliding sheet drawers.
fn compute_sheet_geometry(anchor: SheetAnchor, extent: f32, viewport: Rect) -> ((f32, f32), Point) {
    match anchor {
        SheetAnchor::Bottom => (
            (viewport.width, extent.min(viewport.height)),
            Point::new(viewport.x, (viewport.bottom() - extent).max(viewport.y)),
        ),
        SheetAnchor::Top => (
            (viewport.width, extent.min(viewport.height)),
            Point::new(viewport.x, viewport.y),
        ),
        SheetAnchor::Left => (
            (extent.min(viewport.width), viewport.height),
            Point::new(viewport.x, viewport.y),
        ),
        SheetAnchor::Right => (
            (extent.min(viewport.width), viewport.height),
            Point::new((viewport.right() - extent).max(viewport.x), viewport.y),
        ),
    }
}