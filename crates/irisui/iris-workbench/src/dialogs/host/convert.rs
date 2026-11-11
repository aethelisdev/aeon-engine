// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Conversion Implementations for Active Dialogs
//!
//! Converts strongly typed dialog descriptors into [`ActiveDialog`] instances,
//! initializing state machines, focus cycles, and accessibility metadata.

use crate::dialogs::descriptor::confirm::ConfirmationDialogDescriptor;
use crate::dialogs::descriptor::custom::CustomDialogDescriptor;
use crate::dialogs::descriptor::info::InfoCardDialogDescriptor;
use crate::dialogs::descriptor::input::PromptDialogDescriptor;
use crate::dialogs::descriptor::modal::ModalDescriptor;
use crate::dialogs::descriptor::progress::ProgressModalDescriptor;
use crate::dialogs::descriptor::sheet::{SheetAnchor, SheetDialogDescriptor};
use crate::dialogs::descriptor::unified::DialogDescriptor;
use crate::dialogs::host::active::{ActiveDialog, DialogKind};
use crate::dialogs::host::focus::{DialogFocusCycle, DialogFocusItem};

impl<Action: 'static + Send + Sync> From<DialogDescriptor<Action>> for ActiveDialog<Action> {
    fn from(descriptor: DialogDescriptor<Action>) -> Self {
        match descriptor {
            DialogDescriptor::Modal(modal) => Self::from(modal),
            DialogDescriptor::Sheet(sheet) => Self::from(sheet),
            DialogDescriptor::Confirmation(conf) => Self::from(conf),
            DialogDescriptor::Prompt(prompt) => Self::from(prompt),
            DialogDescriptor::InfoCard(info) => Self::from(info),
            DialogDescriptor::Progress(prog) => Self::from(prog),
            DialogDescriptor::CustomView(custom) => Self::from(custom),
        }
    }
}

impl<Action: 'static + Send + Sync> From<ModalDescriptor> for ActiveDialog<Action> {
    fn from(modal: ModalDescriptor) -> Self {
        let is_cancellable = modal.is_cancellable;
        Self {
            kind: DialogKind::Custom {
                title: modal.title,
                size: modal.size,
                render_fn: Box::new(|_, _| {}),
                on_cancel: None,
            },
            style: modal.style.unwrap_or_default(),
            size: modal.size,
            position: modal.position,
            subtitle: modal.subtitle,
            icon: modal.icon,
            backdrop_color: Some(modal.backdrop_color),
            show_header: modal.show_header,
            dismiss_on_backdrop: is_cancellable && modal.dismiss_on_backdrop,
            dismiss_on_escape: is_cancellable && modal.dismiss_on_escape,
            is_cancellable,
            show_close_button: is_cancellable && modal.show_close_button,
            attention_ticks: 0,
            focus_cycle: DialogFocusCycle::new(),
        }
    }
}

impl<Action: 'static + Send + Sync> From<SheetDialogDescriptor<Action>> for ActiveDialog<Action> {
    fn from(sheet: SheetDialogDescriptor<Action>) -> Self {
        let is_cancellable = sheet.is_cancellable;
        let size = match sheet.anchor {
            SheetAnchor::Bottom | SheetAnchor::Top => (0.0, sheet.extent),
            SheetAnchor::Left | SheetAnchor::Right => (sheet.extent, 0.0),
        };
        Self {
            kind: DialogKind::Sheet {
                title: sheet.title,
                anchor: sheet.anchor,
                extent: sheet.extent,
                render_fn: sheet.render_fn,
                on_cancel: sheet.on_close,
            },
            style: sheet.style,
            size,
            position: None,
            subtitle: None,
            icon: None,
            backdrop_color: None,
            show_header: true,
            dismiss_on_backdrop: is_cancellable && sheet.dismiss_on_backdrop,
            dismiss_on_escape: is_cancellable && sheet.dismiss_on_escape,
            is_cancellable,
            show_close_button: is_cancellable && sheet.show_close_button,
            attention_ticks: 0,
            focus_cycle: DialogFocusCycle::new(),
        }
    }
}

impl<Action: 'static + Send + Sync> From<ConfirmationDialogDescriptor<Action>>
    for ActiveDialog<Action>
{
    fn from(conf: ConfirmationDialogDescriptor<Action>) -> Self {
        let is_cancellable = conf.is_cancellable;
        let targets = if is_cancellable {
            vec![DialogFocusItem::Confirm, DialogFocusItem::Cancel]
        } else {
            vec![DialogFocusItem::Confirm]
        };
        Self {
            kind: DialogKind::Confirm {
                title: conf.title,
                message: conf.message,
                detail: conf.detail,
                confirm_label: conf.confirm_label,
                cancel_label: conf.cancel_label,
                is_danger: conf.is_danger,
                on_confirm: conf.on_confirm,
                on_cancel: conf.on_cancel,
            },
            style: conf.style,
            size: conf.size,
            position: None,
            subtitle: None,
            icon: None,
            backdrop_color: None,
            show_header: true,
            dismiss_on_backdrop: is_cancellable && conf.dismiss_on_backdrop,
            dismiss_on_escape: is_cancellable && conf.dismiss_on_escape,
            is_cancellable,
            show_close_button: is_cancellable && conf.show_close_button,
            attention_ticks: 0,
            focus_cycle: DialogFocusCycle::with_targets(targets),
        }
    }
}

impl<Action: 'static + Send + Sync> From<PromptDialogDescriptor<Action>> for ActiveDialog<Action> {
    fn from(prompt: PromptDialogDescriptor<Action>) -> Self {
        let is_cancellable = prompt.is_cancellable;
        let on_submit_fn = prompt.on_submit.unwrap_or_else(|| {
            Box::new(|_| panic!("PromptDialogDescriptor missing on_submit handler"))
        });

        let targets = if is_cancellable {
            vec![
                DialogFocusItem::Input(0),
                DialogFocusItem::Confirm,
                DialogFocusItem::Cancel,
            ]
        } else {
            vec![DialogFocusItem::Input(0), DialogFocusItem::Confirm]
        };

        Self {
            kind: DialogKind::Input {
                title: prompt.title,
                label: prompt.prompt,
                text: prompt.initial_text,
                placeholder: prompt.placeholder,
                confirm_label: prompt.confirm_label,
                cancel_label: prompt.cancel_label,
                on_submit: on_submit_fn,
                on_cancel: prompt.on_cancel,
            },
            style: prompt.style,
            size: prompt.size,
            position: None,
            subtitle: None,
            icon: None,
            backdrop_color: None,
            show_header: true,
            dismiss_on_backdrop: is_cancellable && prompt.dismiss_on_backdrop,
            dismiss_on_escape: is_cancellable && prompt.dismiss_on_escape,
            is_cancellable,
            show_close_button: is_cancellable && prompt.show_close_button,
            attention_ticks: 0,
            focus_cycle: DialogFocusCycle::with_targets(targets),
        }
    }
}

impl<Action: 'static + Send + Sync> From<InfoCardDialogDescriptor<Action>>
    for ActiveDialog<Action>
{
    fn from(info: InfoCardDialogDescriptor<Action>) -> Self {
        let is_cancellable = info.is_cancellable;
        Self {
            kind: DialogKind::InfoCard {
                title: info.title,
                app_name: info.app_name,
                version: info.version,
                copyright: info.copyright,
                description: info.description,
                entries: info.entries,
                footer_notice: info.footer_notice,
                close_label: info.close_label,
                on_close: info.on_close,
            },
            style: info.style,
            size: info.size,
            position: info.position,
            subtitle: info.subtitle,
            icon: info.icon,
            backdrop_color: info.backdrop_color,
            show_header: true,
            dismiss_on_backdrop: is_cancellable && info.dismiss_on_backdrop,
            dismiss_on_escape: is_cancellable && info.dismiss_on_escape,
            is_cancellable,
            show_close_button: is_cancellable && info.show_close_button,
            attention_ticks: 0,
            focus_cycle: DialogFocusCycle::with_targets(vec![DialogFocusItem::Confirm]),
        }
    }
}

impl<Action: 'static + Send + Sync> From<ProgressModalDescriptor<Action>> for ActiveDialog<Action> {
    fn from(mut progress: ProgressModalDescriptor<Action>) -> Self {
        let is_cancellable = progress.is_cancellable;
        let cancel_label = std::mem::take(&mut progress.cancel_label);
        let on_cancel = progress.on_cancel.take();

        let targets = if is_cancellable {
            vec![DialogFocusItem::Cancel]
        } else {
            Vec::new()
        };

        Self {
            kind: DialogKind::Progress {
                title: progress.title,
                status_message: progress.status_message,
                progress: progress.progress,
                progress_color: progress.progress_color,
                progress_height: progress.progress_height,
                is_cancellable,
                cancel_label,
                on_cancel,
            },
            style: progress.style,
            size: progress.size,
            position: progress.position,
            subtitle: progress.subtitle,
            icon: progress.icon,
            backdrop_color: progress.backdrop_color,
            show_header: true,
            dismiss_on_backdrop: is_cancellable && progress.dismiss_on_backdrop,
            dismiss_on_escape: is_cancellable,
            is_cancellable,
            show_close_button: is_cancellable,
            attention_ticks: 0,
            focus_cycle: DialogFocusCycle::with_targets(targets),
        }
    }
}

impl<Action: 'static + Send + Sync> From<CustomDialogDescriptor<Action>> for ActiveDialog<Action> {
    fn from(custom: CustomDialogDescriptor<Action>) -> Self {
        let is_cancellable = custom.is_cancellable;
        Self {
            kind: DialogKind::Custom {
                title: custom.title,
                size: custom.size,
                render_fn: custom.render_fn,
                on_cancel: custom.on_cancel,
            },
            style: custom.style,
            size: custom.size,
            position: custom.position,
            subtitle: custom.subtitle,
            icon: custom.icon,
            backdrop_color: custom.backdrop_color,
            show_header: custom.show_header,
            dismiss_on_backdrop: is_cancellable && custom.dismiss_on_backdrop,
            dismiss_on_escape: is_cancellable && custom.dismiss_on_escape,
            is_cancellable,
            show_close_button: is_cancellable && custom.show_close_button,
            attention_ticks: 0,
            focus_cycle: DialogFocusCycle::new(),
        }
    }
}