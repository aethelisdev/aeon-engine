// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Custom Modal Dialogue Descriptors
//!
//! Provides declarative specifications for custom immediate-mode UI dialogue cards
//! evaluated using arbitrary builder closures.

use crate::action_bus::ActionBus;
use crate::dialogs::style::ModalStyle;
use iris_core::Color;
use iris_core::geometry::Point;
use iris_widgets::declarative::UiScope;

/// Type alias for user-supplied custom modal rendering closures.
pub type CustomModalRenderFn<Action> =
    Box<dyn FnMut(&mut UiScope<'_>, &mut ActionBus<Action>) + Send + Sync + 'static>;

/// Custom modal dialog descriptor evaluated via an immediate-mode declarative closure.
pub struct CustomDialogDescriptor<Action = ()>
where
    Action: 'static + Send + Sync,
{
    /// Modal card title text displayed in the header bar.
    pub title: String,

    /// Explicit dimensions `(width, height)` in logical pixels.
    pub size: (f32, f32),

    /// Explicit top-left coordinate `(x, y)` override. If `None`, the card is automatically centered.
    pub position: Option<Point>,

    /// Visual chromatic configuration.
    pub style: ModalStyle,

    /// Optional subtitle or helper text displayed in the header area.
    pub subtitle: Option<String>,

    /// Optional icon glyph displayed before the title caption.
    pub icon: Option<String>,

    /// Optional custom fill color for the full-screen backdrop blocker scrim.
    pub backdrop_color: Option<Color>,

    /// Whether the standard header bar is rendered. If `false`, child content has full card area.
    pub show_header: bool,

    /// Whether clicking the backdrop dismisses the dialogue.
    pub dismiss_on_backdrop: bool,

    /// Whether pressing the Escape key dismisses the dialogue.
    pub dismiss_on_escape: bool,

    /// Whether the dialogue can be cancelled by the user.
    pub is_cancellable: bool,

    /// Whether the close button is rendered in the header bar.
    pub show_close_button: bool,

    /// Optional action dispatched when the dialogue is cancelled or dismissed.
    pub on_cancel: Option<Action>,

    /// Custom render closure evaluated inside the modal card.
    pub render_fn: CustomModalRenderFn<Action>,
}

impl<Action: 'static + Send + Sync> CustomDialogDescriptor<Action> {
    /// Creates a new custom dialogue descriptor with title, dimensions, and render closure.
    #[inline]
    #[must_use]
    pub fn new<F>(title: impl Into<String>, size: (f32, f32), render_fn: F) -> Self
    where
        F: FnMut(&mut UiScope<'_>, &mut ActionBus<Action>) + Send + Sync + 'static,
    {
        Self {
            title: title.into(),
            size,
            position: None,
            style: ModalStyle::default(),
            subtitle: None,
            icon: None,
            backdrop_color: None,
            show_header: true,
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            is_cancellable: true,
            show_close_button: true,
            on_cancel: None,
            render_fn: Box::new(render_fn),
        }
    }

    /// Overrides modal card dimensions `(width, height)` in logical pixels.
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

    /// Attaches an optional descriptive subtitle or helper text under the header title.
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

    /// Sets whether the standard header bar is rendered.
    ///
    /// When `false`, the standard header title bar is omitted, allowing child content
    /// to occupy 100% of the modal card geometry.
    #[inline]
    #[must_use]
    pub const fn with_header(mut self, show: bool) -> Self {
        self.show_header = show;
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

    /// Sets an optional cancellation action dispatched when the dialogue is cancelled or dismissed.
    #[inline]
    #[must_use]
    pub fn with_cancel_action(mut self, action: Action) -> Self {
        self.on_cancel = Some(action);
        self
    }
}