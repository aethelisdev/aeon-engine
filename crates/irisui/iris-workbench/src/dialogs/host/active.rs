// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Active Dialog Representations and Kinds
//!
//! Strongly-typed dialog kind variants, custom render closures, and the [`ActiveDialog`]
//! struct representing instantiated dialog cards within the modal stack.

use crate::action_bus::ActionBus;
use crate::dialogs::descriptor::info::{InfoCardDialogDescriptor, InfoEntry};
use crate::dialogs::descriptor::modal::ModalDescriptor;
use crate::dialogs::descriptor::progress::ProgressModalDescriptor;
use crate::dialogs::descriptor::sheet::SheetAnchor;
use crate::dialogs::host::focus::{DialogFocusCycle, DialogFocusItem};
use crate::dialogs::style::ModalStyle;
use iris_core::Color;
use iris_core::geometry::Point;
use iris_widgets::declarative::UiScope;

/// Type alias for user-supplied custom modal rendering closures.
pub type CustomModalRenderFn<Action> =
    Box<dyn FnMut(&mut UiScope<'_>, &mut ActionBus<Action>) + Send + Sync + 'static>;

/// Type alias for user-supplied custom sheet drawer rendering closures.
pub type CustomSheetRenderFn<Action> =
    Box<dyn FnMut(&mut UiScope<'_>, &mut ActionBus<Action>) + Send + Sync + 'static>;

/// Strongly typed modal dialogue variant tracked by [`ActiveDialog`].
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
        /// Optional action dispatched on cancellation or dismissal.
        on_cancel: Option<Action>,
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
        on_submit: Box<dyn Fn(String) -> Action + Send + Sync + 'static>,
        /// Optional action dispatched on cancellation or dismissal.
        on_cancel: Option<Action>,
    },

    /// Custom user-defined modal card with immediate-mode UI rendering closure.
    Custom {
        /// Modal card title text.
        title: String,
        /// Explicit dimensions `(width, height)` in logical pixels.
        size: (f32, f32),
        /// Custom render closure evaluated inside the modal card.
        render_fn: CustomModalRenderFn<Action>,
        /// Optional action dispatched on cancellation or dismissal.
        on_cancel: Option<Action>,
    },

    /// Sliding sheet drawer dialogue anchored to a window edge.
    Sheet {
        /// Modal card title text.
        title: String,
        /// Window edge anchor determining slide origin (`Bottom`, `Top`, `Left`, `Right`).
        anchor: SheetAnchor,
        /// Fixed thickness dimension in logical pixels.
        extent: f32,
        /// Optional custom render closure evaluated inside the sheet drawer.
        render_fn: Option<CustomSheetRenderFn<Action>>,
        /// Optional action dispatched on cancellation or dismissal.
        on_cancel: Option<Action>,
    },

    /// Generic information, software "About", or diagnostic modal dialogue card.
    InfoCard {
        /// Modal card title text.
        title: String,
        /// Prominent application or project name.
        app_name: Option<String>,
        /// Software version string.
        version: Option<String>,
        /// Copyright attribution notice.
        copyright: Option<String>,
        /// Descriptive summary or explanatory paragraph.
        description: Option<String>,
        /// Key-value diagnostic or specification entries.
        entries: Vec<InfoEntry>,
        /// Footer or disclaimer notice.
        footer_notice: Option<String>,
        /// Caption presented on the confirmation button.
        close_label: String,
        /// Optional action dispatched to the [`ActionBus`] on closure.
        on_close: Option<Action>,
    },

    /// Long-running operation progress modal dialogue card with optional cancellation.
    Progress {
        /// Modal card title text.
        title: String,
        /// Current operation status message displayed above the progress meter.
        status_message: Option<String>,
        /// Progress completion ratio clamped to range `[0.0, 1.0]`.
        progress: f32,
        /// Optional foreground fill color override for the progress bar.
        progress_color: Option<Color>,
        /// Height of the progress bar in physical pixels.
        progress_height: f32,
        /// Whether user cancellation is permitted.
        is_cancellable: bool,
        /// Caption presented on the cancel button.
        cancel_label: String,
        /// Optional action dispatched to the [`ActionBus`] on cancellation.
        on_cancel: Option<Action>,
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
                on_cancel,
            } => f
                .debug_struct("Confirm")
                .field("title", title)
                .field("message", message)
                .field("detail", detail)
                .field("confirm_label", confirm_label)
                .field("cancel_label", cancel_label)
                .field("is_danger", is_danger)
                .field("on_confirm", on_confirm)
                .field("on_cancel", on_cancel)
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
            Self::Sheet {
                title,
                anchor,
                extent,
                ..
            } => f
                .debug_struct("Sheet")
                .field("title", title)
                .field("anchor", anchor)
                .field("extent", extent)
                .finish(),
            Self::InfoCard {
                title,
                app_name,
                version,
                copyright,
                description,
                entries,
                footer_notice,
                close_label,
                on_close,
            } => f
                .debug_struct("InfoCard")
                .field("title", title)
                .field("app_name", app_name)
                .field("version", version)
                .field("copyright", copyright)
                .field("description", description)
                .field("entries", entries)
                .field("footer_notice", footer_notice)
                .field("close_label", close_label)
                .field("on_close", on_close)
                .finish(),
            Self::Progress {
                title,
                status_message,
                progress,
                progress_color,
                progress_height,
                is_cancellable,
                cancel_label,
                on_cancel,
            } => f
                .debug_struct("Progress")
                .field("title", title)
                .field("status_message", status_message)
                .field("progress", progress)
                .field("progress_color", progress_color)
                .field("progress_height", progress_height)
                .field("is_cancellable", is_cancellable)
                .field("cancel_label", cancel_label)
                .field("on_cancel", on_cancel)
                .finish(),
        }
    }
}

/// Active modal dialog representation tracked by [`super::DialogHost`].
pub struct ActiveDialog<Action: 'static + Send + Sync> {
    /// Kind and specific configuration of this active dialog.
    pub kind: DialogKind<Action>,
    /// Visual style parameters (backdrop, card background, border, corner radius).
    pub style: ModalStyle,
    /// Explicit width and height of the dialog card in logical pixels.
    pub size: (f32, f32),
    /// Explicit top-left coordinate `(x, y)` override. If `None`, centered in viewport.
    pub position: Option<Point>,
    /// Optional subtitle text displayed in the header.
    pub subtitle: Option<String>,
    /// Optional icon glyph displayed before the title.
    pub icon: Option<String>,
    /// Optional custom fill color for the full-screen backdrop blocker scrim.
    pub backdrop_color: Option<Color>,
    /// Whether the standard top header title bar is rendered.
    pub show_header: bool,
    /// Whether clicking the semi-transparent backdrop dismisses the dialog.
    pub dismiss_on_backdrop: bool,
    /// Whether pressing the Escape key dismisses the dialog.
    pub dismiss_on_escape: bool,
    /// Whether the dialog can be cancelled by user interaction.
    pub is_cancellable: bool,
    /// Whether to render the '✕' close button in the header bar.
    pub show_close_button: bool,
    /// Remaining frame ticks for the attention shake and glow highlight animation.
    pub attention_ticks: u8,
    /// Cyclic keyboard focus state machine for Tab/Shift+Tab control traversal.
    pub focus_cycle: DialogFocusCycle,
}

impl<Action: 'static + Send + Sync> ActiveDialog<Action> {
    /// Returns the title text of the active dialog.
    #[must_use]
    pub fn title(&self) -> &str {
        match &self.kind {
            DialogKind::Confirm { title, .. } => title.as_str(),
            DialogKind::Input { title, .. } => title.as_str(),
            DialogKind::Custom { title, .. } => title.as_str(),
            DialogKind::Sheet { title, .. } => title.as_str(),
            DialogKind::InfoCard { title, .. } => title.as_str(),
            DialogKind::Progress { title, .. } => title.as_str(),
        }
    }

    /// Returns `true` if the dialog is a destructive/danger confirmation.
    #[must_use]
    pub fn is_danger(&self) -> bool {
        match &self.kind {
            DialogKind::Confirm { is_danger, .. } => *is_danger,
            _ => false,
        }
    }

    /// Triggers the attention shake and border glow highlight animation.
    ///
    /// Called when the user clicks the backdrop scrim of a non-dismissible modal.
    pub fn trigger_attention(&mut self) {
        self.attention_ticks = 8;
    }

    /// Returns whether the active dialog can be cancelled by user interaction.
    #[must_use]
    #[inline]
    pub const fn is_cancellable(&self) -> bool {
        self.is_cancellable
    }

    /// Takes and returns the cancellation action payload, if configured.
    pub fn take_cancel_action(&mut self) -> Option<Action> {
        match &mut self.kind {
            DialogKind::Confirm { on_cancel, .. } => on_cancel.take(),
            DialogKind::Input { on_cancel, .. } => on_cancel.take(),
            DialogKind::Custom { on_cancel, .. } => on_cancel.take(),
            DialogKind::Sheet { on_cancel, .. } => on_cancel.take(),
            DialogKind::InfoCard { .. } => None,
            DialogKind::Progress { on_cancel, .. } => on_cancel.take(),
        }
    }

    /// Returns a shared reference to the cyclic focus state machine.
    #[must_use]
    #[inline]
    pub const fn focus_cycle(&self) -> &DialogFocusCycle {
        &self.focus_cycle
    }

    /// Returns a mutable reference to the cyclic focus state machine.
    #[inline]
    pub fn focus_cycle_mut(&mut self) -> &mut DialogFocusCycle {
        &mut self.focus_cycle
    }

    /// Returns the currently focused interactive control item, if any.
    #[must_use]
    #[inline]
    pub const fn focused_item(&self) -> Option<DialogFocusItem> {
        self.focus_cycle.current()
    }

    /// Advances keyboard focus forward (`Tab`) or backward (`Shift+Tab`).
    pub fn cycle_focus(&mut self, reverse: bool) -> Option<DialogFocusItem> {
        self.focus_cycle.cycle(reverse)
    }

    /// Evaluates whether this dialog is currently animating an attention feedback sequence.
    #[must_use]
    #[inline]
    pub const fn is_demanding_attention(&self) -> bool {
        self.attention_ticks > 0
    }

    /// Computes the current lateral shake offset in logical pixels without advancing ticks.
    #[must_use]
    pub fn attention_offset(&self) -> f32 {
        if self.attention_ticks == 0 {
            return 0.0;
        }

        let base_amplitude = self.style.attention_shake_amplitude;
        let factor = match self.attention_ticks {
            8 => 1.0,
            7 => -1.0,
            6 => 0.75,
            5 => -0.75,
            4 => 0.50,
            3 => -0.50,
            2 => 0.25,
            1 => -0.25,
            _ => 0.0,
        };

        base_amplitude * factor
    }

    /// Advances the attention animation sequence by one tick.
    pub fn tick_attention(&mut self) {
        self.attention_ticks = self.attention_ticks.saturating_sub(1);
    }

    /// Constructs a fully populated [`ModalDescriptor`] representing this active dialog.
    #[must_use]
    pub fn build_modal_desc(&self, is_top: bool) -> ModalDescriptor {
        let mut style = self.style.clone();
        let mut position = self.position;
        if is_top && self.attention_ticks > 0 {
            style.card_border_color = self.style.attention_border_color;
            let offset = self.attention_offset();
            if let Some(pos) = position {
                position = Some(Point::new(pos.x + offset, pos.y));
            }
        }

        let mut desc = ModalDescriptor::new(self.title(), self.size.0, self.size.1)
            .with_dismiss_on_backdrop(if is_top {
                self.dismiss_on_backdrop
            } else {
                false
            })
            .with_dismiss_on_escape(if is_top {
                self.dismiss_on_escape
            } else {
                false
            })
            .with_close_button(self.show_close_button)
            .with_header(self.show_header)
            .with_style(style);

        if let Some(pos) = position {
            desc = desc.with_position(pos);
        }
        if let Some(ref sub) = self.subtitle {
            desc = desc.with_subtitle(sub.as_str());
        }
        if let Some(ref ic) = self.icon {
            desc = desc.with_icon(ic.as_str());
        }
        if let Some(bg) = self.backdrop_color {
            desc = desc.with_backdrop_color(bg);
        }

        desc
    }

    /// Constructs a fully populated [`InfoCardDialogDescriptor`] if this dialog is an [`DialogKind::InfoCard`].
    #[must_use]
    pub fn build_info_desc(&self, is_top: bool) -> Option<InfoCardDialogDescriptor<Action>> {
        let DialogKind::InfoCard {
            title,
            app_name,
            version,
            copyright,
            description,
            entries,
            footer_notice,
            close_label,
            ..
        } = &self.kind
        else {
            return None;
        };

        let mut style = self.style.clone();
        let mut position = self.position;
        if is_top && self.attention_ticks > 0 {
            style.card_border_color = self.style.attention_border_color;
            let offset = self.attention_offset();
            if let Some(pos) = position {
                position = Some(Point::new(pos.x + offset, pos.y));
            }
        }

        let mut desc = InfoCardDialogDescriptor::new(title.as_str())
            .with_close_label(close_label.as_str())
            .with_size(self.size.0, self.size.1)
            .with_dismiss_on_backdrop(if is_top {
                self.dismiss_on_backdrop
            } else {
                false
            })
            .with_dismiss_on_escape(if is_top {
                self.dismiss_on_escape
            } else {
                false
            })
            .with_close_button(self.show_close_button)
            .with_style(style);

        if let Some(pos) = position {
            desc = desc.with_position(pos);
        }
        if let Some(ref sub) = self.subtitle {
            desc = desc.with_subtitle(sub.as_str());
        }
        if let Some(ref ic) = self.icon {
            desc = desc.with_icon(ic.as_str());
        }
        if let Some(bg) = self.backdrop_color {
            desc = desc.with_backdrop_color(bg);
        }
        if let Some(app) = app_name {
            desc = desc.with_app_name(app.as_str());
        }
        if let Some(ver) = version {
            desc = desc.with_version(ver.as_str());
        }
        if let Some(copy) = copyright {
            desc = desc.with_copyright(copy.as_str());
        }
        if let Some(d) = description {
            desc = desc.with_description(d.as_str());
        }
        desc = desc.with_entries(entries.clone());
        if let Some(notice) = footer_notice {
            desc = desc.with_footer_notice(notice.as_str());
        }

        Some(desc)
    }

    /// Reconstructs a transient [`ProgressModalDescriptor`] representing this active dialog.
    ///
    /// # Arguments
    /// * `is_top` - Whether this dialog is the topmost active card in the modal stack.
    #[must_use]
    pub fn build_progress_desc(&self, is_top: bool) -> Option<ProgressModalDescriptor<Action>> {
        let DialogKind::Progress {
            title,
            status_message,
            progress,
            progress_color,
            progress_height,
            is_cancellable,
            cancel_label,
            ..
        } = &self.kind
        else {
            return None;
        };

        let mut style = self.style.clone();
        let mut position = self.position;
        if is_top && self.attention_ticks > 0 {
            style.card_border_color = self.style.attention_border_color;
            let offset = self.attention_offset();
            if let Some(pos) = position {
                position = Some(Point::new(pos.x + offset, pos.y));
            }
        }

        let mut desc = ProgressModalDescriptor::new(title.as_str())
            .with_progress(*progress)
            .with_progress_height(*progress_height)
            .with_cancellable(*is_cancellable)
            .with_cancel_label(cancel_label.as_str())
            .with_size(self.size.0, self.size.1)
            .with_dismiss_on_backdrop(if is_top && *is_cancellable {
                self.dismiss_on_backdrop
            } else {
                false
            })
            .with_style(style);

        if let Some(pos) = position {
            desc = desc.with_position(pos);
        }
        if let Some(ref sub) = self.subtitle {
            desc = desc.with_subtitle(sub.as_str());
        }
        if let Some(ref ic) = self.icon {
            desc = desc.with_icon(ic.as_str());
        }
        if let Some(bg) = self.backdrop_color {
            desc = desc.with_backdrop_color(bg);
        }
        if let Some(status) = status_message {
            desc = desc.with_status_message(status.as_str());
        }
        if let Some(color) = *progress_color {
            desc = desc.with_progress_color(color);
        }

        Some(desc)
    }
}