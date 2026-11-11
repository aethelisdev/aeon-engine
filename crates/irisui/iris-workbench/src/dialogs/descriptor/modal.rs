// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Centered Modal Dialogue Descriptor
//!
//! Provides declarative specification structures for centered or positioned modal cards,
//! controlling backdrop occlusion, dimensions, and dismissal policies.

use iris_core::Color;
use iris_core::geometry::Point;

use crate::dialogs::style::ModalStyle;

/// Declarative alias for workbench-level modal window configurations.
pub type WorkbenchModal = ModalDescriptor;

/// Declarative specification for an elevated, centered or positioned modal dialogue.
///
/// Encapsulates all dimensional, chromatic, and behavioral properties required
/// to present a modal card over an application workspace, including automatic
/// backdrop blocking and dismissal policies.
#[derive(Debug, Clone, PartialEq)]
pub struct ModalDescriptor {
    /// Title string presented in the modal card header bar.
    pub title: String,

    /// Width and height `(width, height)` of the centered modal card in logical pixels.
    pub size: (f32, f32),

    /// Semi-transparent fill color for the full-screen backdrop blocker scrim.
    /// When `None`, derives dynamically from [`Theme::modal_scrim`].
    pub backdrop_color: Option<Color>,

    /// Explicit top-left coordinate `(x, y)` override. If `None`, the card is automatically centered.
    pub position: Option<Point>,

    /// Optional visual style overrides for surface colors, borders, and button geometries.
    pub style: Option<ModalStyle>,

    /// Optional subtitle or helper text displayed in the header area.
    pub subtitle: Option<String>,

    /// Optional icon glyph displayed before the title caption.
    pub icon: Option<String>,

    /// Whether clicking the backdrop outside the card automatically dismisses the modal.
    pub dismiss_on_backdrop: bool,

    /// Whether pressing the keyboard Escape key automatically dismisses the modal.
    pub dismiss_on_escape: bool,

    /// Whether the modal dialogue can be cancelled by the user.
    pub is_cancellable: bool,

    /// Whether a standard `✕` close button is rendered in the top-right of the header.
    pub show_close_button: bool,

    /// Whether the standard top header title bar is rendered.
    ///
    /// When set to `false`, the header row is omitted completely, allowing child content
    /// or custom tab bars to occupy the entire modal card area.
    pub show_header: bool,
}

impl ModalDescriptor {
    /// Standard semi-transparent dark backdrop color (`rgba(0.0, 0.0, 0.0, 0.60)`).
    pub const DEFAULT_BACKDROP: Color = Color::rgba(0.0, 0.0, 0.0, 0.60);

    /// Creates a new modal descriptor with title and card dimensions.
    ///
    /// By default, dismiss on backdrop click, dismiss on Escape, header bar, and close button
    /// are all enabled, with a 60% opacity black backdrop.
    ///
    /// # Arguments
    /// * `title` - Header caption string.
    /// * `width` - Modal card width in logical pixels.
    /// * `height` - Modal card height in logical pixels.
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            title: title.into(),
            size: (width, height),
            backdrop_color: None,
            position: None,
            style: None,
            subtitle: None,
            icon: None,
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            is_cancellable: true,
            show_close_button: true,
            show_header: true,
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

    /// Overrides the visual styling configuration of the modal card.
    #[inline]
    #[must_use]
    pub fn with_style(mut self, style: ModalStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Attaches an optional descriptive subtitle or path under the header title.
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

    /// Overrides the semi-transparent backdrop blocker color.
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

    /// Sets whether pressing the Escape key dismisses the dialogue.
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

    /// Sets whether the modal dialogue can be cancelled by the user.
    #[inline]
    #[must_use]
    pub const fn with_cancellable(mut self, cancellable: bool) -> Self {
        self.is_cancellable = cancellable;
        self
    }

    /// Sets whether the standard top header title bar is rendered.
    #[inline]
    #[must_use]
    pub const fn with_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }
}