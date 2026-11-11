// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Sliding Sheet Dialogue Descriptors
//!
//! Provides declarative specifications for sliding edge drawer dialogues anchored
//! to bottom, top, left, or right workbench window boundaries.

use crate::action_bus::ActionBus;
use crate::dialogs::style::ModalStyle;
use iris_widgets::declarative::UiScope;

/// Type alias for user-supplied custom sheet rendering closures.
pub type CustomSheetRenderFn<Action> =
    Box<dyn FnMut(&mut UiScope<'_>, &mut ActionBus<Action>) + Send + Sync + 'static>;

/// Screen or window edge anchor placement for sliding sheet dialogues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SheetAnchor {
    /// Sheet slides up from the bottom edge of the workbench window.
    #[default]
    Bottom,
    /// Sheet slides down from the top edge (below the menubar/titlebar).
    Top,
    /// Sheet slides in from the left edge.
    Left,
    /// Sheet slides in from the right edge.
    Right,
}

impl SheetAnchor {
    /// Returns `true` if this anchor placement is horizontal (Left or Right).
    #[inline]
    #[must_use]
    pub const fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// Returns `true` if this anchor placement is vertical (Bottom or Top).
    #[inline]
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::Bottom | Self::Top)
    }
}

/// Declarative descriptor for a sliding drawer sheet dialogue anchored to a window boundary.
pub struct SheetDialogDescriptor<Action = ()>
where
    Action: 'static + Send + Sync,
{
    /// Window title caption displayed in the sheet header bar.
    pub title: String,

    /// Target anchor edge the sheet slides in from.
    pub anchor: SheetAnchor,

    /// Dimension along the slide axis in logical pixels (height for Top/Bottom, width for Left/Right).
    pub extent: f32,

    /// Visual chromatic configuration.
    pub style: ModalStyle,

    /// Whether clicking the semi-transparent backdrop dismisses the sheet.
    pub dismiss_on_backdrop: bool,

    /// Whether pressing the Escape key dismisses the sheet.
    pub dismiss_on_escape: bool,

    /// Whether the sheet can be cancelled by the user.
    pub is_cancellable: bool,

    /// Whether a standard close button is rendered in the header.
    pub show_close_button: bool,

    /// Optional action emitted when the sheet is closed.
    pub on_close: Option<Action>,

    /// Custom render closure evaluated inside the sheet body.
    pub render_fn: Option<CustomSheetRenderFn<Action>>,
}

impl<Action: 'static + Send + Sync> SheetDialogDescriptor<Action> {
    /// Default extent for vertical sheets (320.0 px height).
    pub const DEFAULT_VERTICAL_EXTENT: f32 = 320.0;

    /// Default extent for horizontal sheets (360.0 px width).
    pub const DEFAULT_HORIZONTAL_EXTENT: f32 = 360.0;

    /// Creates a bottom-anchored sliding sheet dialogue specification.
    #[inline]
    #[must_use]
    pub fn bottom(title: impl Into<String>, height: f32) -> Self {
        Self::new(title, SheetAnchor::Bottom, height)
    }

    /// Creates a top-anchored sliding sheet dialogue specification.
    #[inline]
    #[must_use]
    pub fn top(title: impl Into<String>, height: f32) -> Self {
        Self::new(title, SheetAnchor::Top, height)
    }

    /// Creates a left-anchored sliding sheet dialogue specification.
    #[inline]
    #[must_use]
    pub fn left(title: impl Into<String>, width: f32) -> Self {
        Self::new(title, SheetAnchor::Left, width)
    }

    /// Creates a right-anchored sliding sheet dialogue specification.
    #[inline]
    #[must_use]
    pub fn right(title: impl Into<String>, width: f32) -> Self {
        Self::new(title, SheetAnchor::Right, width)
    }

    /// Constructs a sliding sheet dialogue descriptor with specified anchor and dimension.
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, anchor: SheetAnchor, extent: f32) -> Self {
        Self {
            title: title.into(),
            anchor,
            extent,
            style: ModalStyle::default(),
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            is_cancellable: true,
            show_close_button: true,
            on_close: None,
            render_fn: None,
        }
    }

    /// Sets the anchor edge placement for this sheet.
    #[inline]
    #[must_use]
    pub const fn with_anchor(mut self, anchor: SheetAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets the sheet extent dimension (height for vertical, width for horizontal).
    #[inline]
    #[must_use]
    pub const fn with_extent(mut self, extent: f32) -> Self {
        self.extent = extent;
        self
    }

    /// Sets an action payload emitted on sheet closure.
    #[inline]
    #[must_use]
    pub fn with_on_close(mut self, action: Action) -> Self {
        self.on_close = Some(action);
        self
    }

    /// Sets a custom declarative rendering closure for the sheet content.
    #[inline]
    #[must_use]
    pub fn with_render_fn<F>(mut self, render_fn: F) -> Self
    where
        F: FnMut(&mut UiScope<'_>, &mut ActionBus<Action>) + Send + Sync + 'static,
    {
        self.render_fn = Some(Box::new(render_fn));
        self
    }

    /// Overrides visual styling properties.
    #[inline]
    #[must_use]
    pub fn with_style(mut self, style: ModalStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets whether clicking the backdrop dismisses the sheet.
    #[inline]
    #[must_use]
    pub const fn with_dismiss_on_backdrop(mut self, allow: bool) -> Self {
        self.dismiss_on_backdrop = allow;
        self
    }

    /// Sets whether pressing Escape dismisses the sheet.
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

    /// Sets whether the sheet can be cancelled by the user.
    #[inline]
    #[must_use]
    pub const fn with_cancellable(mut self, cancellable: bool) -> Self {
        self.is_cancellable = cancellable;
        self
    }

    /// Sets an optional cancellation action dispatched when the sheet is cancelled or dismissed.
    #[inline]
    #[must_use]
    pub fn with_cancel_action(mut self, action: Action) -> Self {
        self.on_close = Some(action);
        self
    }
}