// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Generic Info and About Dialogue Card Descriptors
//!
//! Declarative configuration structures specifying metadata, entries, licensing,
//! and dismissal policies for generic information, software about, and diagnostic dialogs.

use crate::dialogs::style::ModalStyle;
use iris_core::Color;
use iris_core::geometry::Point;

/// A key-value diagnostic or specification entry displayed inside an [`InfoCardDialogDescriptor`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoEntry {
    /// Semantic key or property label (e.g. "Version", "License", "GPU Backend").
    pub label: String,
    /// Human-readable value or diagnostic state.
    pub value: String,
}

impl InfoEntry {
    /// Creates a new key-value entry.
    #[inline]
    #[must_use]
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// Generic information and software "About" modal dialogue card descriptor.
///
/// Completely unopinionated: all texts, labels, colors, and entries are caller-provided
/// with zero hardcoded strings or brand names, ensuring crates.io desktop independence.
#[derive(Debug, Clone, PartialEq)]
pub struct InfoCardDialogDescriptor<Action = ()>
where
    Action: 'static + Send + Sync,
{
    /// Header title caption (e.g. "About", "System Diagnostics").
    pub title: String,

    /// Optional prominent application or project name.
    pub app_name: Option<String>,

    /// Optional version string (e.g. "0.9.0", "v1.2.3").
    pub version: Option<String>,

    /// Optional copyright attribution notice.
    pub copyright: Option<String>,

    /// Optional descriptive summary or explanatory paragraph.
    pub description: Option<String>,

    /// Optional list of key-value property rows (e.g. Environment, Compiler, Platform).
    pub entries: Vec<InfoEntry>,

    /// Optional footer or disclaimer notice.
    pub footer_notice: Option<String>,

    /// Label displayed on the confirmation/dismissal button (default: "Close").
    pub close_label: String,

    /// Optional action emitted when the dialogue is closed affirmatively.
    pub on_close: Option<Action>,

    /// Dimensions `(width, height)` of the card in logical pixels.
    pub size: (f32, f32),

    /// Explicit top-left coordinate `(x, y)` override. If `None`, centered in viewport.
    pub position: Option<Point>,

    /// Visual chromatic configuration.
    pub style: ModalStyle,

    /// Optional subtitle text displayed in the header.
    pub subtitle: Option<String>,

    /// Optional icon glyph displayed before the header title.
    pub icon: Option<String>,

    /// Optional custom fill color for the full-screen backdrop blocker scrim.
    pub backdrop_color: Option<Color>,

    /// Whether clicking the backdrop dismisses the dialogue.
    pub dismiss_on_backdrop: bool,

    /// Whether pressing the Escape key dismisses the dialogue.
    pub dismiss_on_escape: bool,

    /// Whether the dialogue can be cancelled by the user.
    pub is_cancellable: bool,

    /// Whether the '✕' close button is rendered in the header bar.
    pub show_close_button: bool,
}

impl<Action: 'static + Send + Sync> InfoCardDialogDescriptor<Action> {
    /// Default card size `(480.0, 340.0)` for info dialogues.
    pub const DEFAULT_SIZE: (f32, f32) = (480.0, 340.0);

    /// Default confirmation button label ("Close").
    pub const DEFAULT_CLOSE_LABEL: &'static str = "Close";

    /// Creates a new info card dialogue descriptor with header title.
    ///
    /// By default, dismiss on backdrop, dismiss on Escape, and close button are enabled.
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            app_name: None,
            version: None,
            copyright: None,
            description: None,
            entries: Vec::new(),
            footer_notice: None,
            close_label: Self::DEFAULT_CLOSE_LABEL.to_string(),
            on_close: None,
            size: Self::DEFAULT_SIZE,
            position: None,
            style: ModalStyle::default(),
            subtitle: None,
            icon: None,
            backdrop_color: None,
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            is_cancellable: true,
            show_close_button: true,
        }
    }

    /// Sets the prominent application or software identity name.
    #[inline]
    #[must_use]
    pub fn with_app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    /// Sets the software version string.
    #[inline]
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Sets the copyright attribution string.
    #[inline]
    #[must_use]
    pub fn with_copyright(mut self, copyright: impl Into<String>) -> Self {
        self.copyright = Some(copyright.into());
        self
    }

    /// Sets the descriptive summary or explanatory paragraph text.
    #[inline]
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Appends a key-value diagnostic or specification entry.
    #[inline]
    #[must_use]
    pub fn with_entry(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.entries.push(InfoEntry::new(label, value));
        self
    }

    /// Appends multiple key-value entries.
    #[inline]
    #[must_use]
    pub fn with_entries(mut self, entries: impl IntoIterator<Item = InfoEntry>) -> Self {
        self.entries.extend(entries);
        self
    }

    /// Sets an optional footer disclaimer or legal notice.
    #[inline]
    #[must_use]
    pub fn with_footer_notice(mut self, notice: impl Into<String>) -> Self {
        self.footer_notice = Some(notice.into());
        self
    }

    /// Overrides the confirmation button text (e.g. "OK", "Dismiss", or localized string).
    #[inline]
    #[must_use]
    pub fn with_close_label(mut self, label: impl Into<String>) -> Self {
        self.close_label = label.into();
        self
    }

    /// Attaches an affirmative action emitted when the dialogue is closed.
    #[inline]
    #[must_use]
    pub fn with_on_close(mut self, action: Action) -> Self {
        self.on_close = Some(action);
        self
    }

    /// Overrides card dimensions `(width, height)` in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }

    /// Sets an explicit top-left position override, bypassing automatic screen centering.
    #[inline]
    #[must_use]
    pub const fn with_position(mut self, pos: Point) -> Self {
        self.position = Some(pos);
        self
    }

    /// Overrides visual styling properties.
    #[inline]
    #[must_use]
    pub fn with_style(mut self, style: ModalStyle) -> Self {
        self.style = style;
        self
    }

    /// Attaches an optional descriptive subtitle under the header title.
    #[inline]
    #[must_use]
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Sets an optional icon glyph displayed before the header title.
    #[inline]
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Overrides the semi-transparent backdrop blocker scrim color.
    #[inline]
    #[must_use]
    pub const fn with_backdrop_color(mut self, color: Color) -> Self {
        self.backdrop_color = Some(color);
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

    /// Sets whether the close button is rendered in the header bar.
    #[inline]
    #[must_use]
    pub const fn with_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }

    /// Sets whether the dialogue can be cancelled by the user.
    #[inline]
    #[must_use]
    pub const fn with_cancellable(mut self, cancellable: bool) -> Self {
        self.is_cancellable = cancellable;
        self
    }
}