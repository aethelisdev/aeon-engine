// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Modal and Floating Window Descriptors
//!
//! Declarative configuration structures specifying geometry, styling, and dismissal
//! behaviors for centered modal dialogues and detached floating windows.

use iris_core::Color;
use iris_core::geometry::Rect;

/// Declarative specification for an elevated, centered modal dialogue.
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
    pub backdrop_color: Color,

    /// Whether clicking the backdrop outside the card automatically dismisses the modal.
    pub dismiss_on_backdrop: bool,

    /// Whether pressing the keyboard Escape key automatically dismisses the modal.
    pub dismiss_on_escape: bool,

    /// Whether a standard `✕` close button is rendered in the top-right of the header.
    pub show_close_button: bool,
}

impl ModalDescriptor {
    /// Standard semi-transparent dark backdrop color (`rgba(0.0, 0.0, 0.0, 0.60)`).
    pub const DEFAULT_BACKDROP: Color = Color::rgba(0.0, 0.0, 0.0, 0.60);

    /// Creates a new modal descriptor with title and card dimensions.
    ///
    /// By default, dismiss on backdrop click, dismiss on Escape, and close button
    /// are all enabled, with a 60% opacity black backdrop.
    ///
    /// # Parameters
    /// - `title`: Header caption string.
    /// - `width`: Modal card width in logical pixels.
    /// - `height`: Modal card height in logical pixels.
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            title: title.into(),
            size: (width, height),
            backdrop_color: Self::DEFAULT_BACKDROP,
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            show_close_button: true,
        }
    }

    /// Overrides the semi-transparent backdrop blocker color.
    #[inline]
    #[must_use]
    pub const fn with_backdrop_color(mut self, color: Color) -> Self {
        self.backdrop_color = color;
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

    /// Sets whether the top-right `✕` close button is visible.
    #[inline]
    #[must_use]
    pub const fn with_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }
}

/// Declarative descriptor for a detached floating tool window.
///
/// Governs window positioning, title, minimum dimensional constraints, and
/// viewport boundary clamping so floating windows never escape the visible desktop screen.
///
/// # Generics
/// - `TabId`: The application's unique dock panel identifier type.
#[derive(Debug, Clone, PartialEq)]
pub struct FloatingWindowDescriptor<TabId> {
    /// Unique tab / panel identifier hosted inside this floating window.
    pub id: TabId,

    /// Title string presented on the floating window title bar.
    pub title: String,

    /// Bounding rectangle `(x, y, width, height)` of the floating window.
    pub rect: Rect,

    /// Whether this floating window currently holds keyboard / input focus.
    pub is_focused: bool,

    /// Whether the user is permitted to interactively resize this floating window.
    pub is_resizable: bool,

    /// Minimum allowable width and height `(min_width, min_height)`.
    pub min_size: (f32, f32),
}

impl<TabId> FloatingWindowDescriptor<TabId> {
    /// Default minimum allowable dimensions `(180.0, 120.0)` for floating windows.
    pub const DEFAULT_MIN_SIZE: (f32, f32) = (180.0, 120.0);

    /// Creates a new floating window descriptor.
    ///
    /// # Parameters
    /// - `id`: The hosted panel identifier.
    /// - `title`: Window title string.
    /// - `rect`: Position and initial dimensions.
    #[inline]
    #[must_use]
    pub fn new(id: TabId, title: impl Into<String>, rect: Rect) -> Self {
        Self {
            id,
            title: title.into(),
            rect,
            is_focused: false,
            is_resizable: true,
            min_size: Self::DEFAULT_MIN_SIZE,
        }
    }

    /// Sets whether this floating window has active focus.
    #[inline]
    #[must_use]
    pub const fn with_focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    /// Sets whether interactive border resizing is enabled.
    #[inline]
    #[must_use]
    pub const fn with_resizable(mut self, resizable: bool) -> Self {
        self.is_resizable = resizable;
        self
    }

    /// Sets minimum allowable width and height constraints.
    #[inline]
    #[must_use]
    pub const fn with_min_size(mut self, min_width: f32, min_height: f32) -> Self {
        self.min_size = (min_width, min_height);
        self
    }

    /// Clamps the floating window's bounds to remain fully visible within a viewport boundary.
    ///
    /// Guarantees that the window title bar and content are not pushed outside the screen.
    #[must_use]
    pub fn clamp_to_viewport(mut self, viewport: Rect) -> Self {
        let min_w = self.min_size.0;
        let min_h = self.min_size.1;

        let w = self.rect.width.max(min_w).min(viewport.width);
        let h = self.rect.height.max(min_h).min(viewport.height);

        let max_x = (viewport.x + viewport.width - w).max(viewport.x);
        let max_y = (viewport.y + viewport.height - h).max(viewport.y);

        let x = self.rect.x.clamp(viewport.x, max_x);
        let y = self.rect.y.clamp(viewport.y, max_y);

        self.rect = Rect::new(x, y, w, h);
        self
    }
}