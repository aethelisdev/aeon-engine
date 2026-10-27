// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Modal and Floating Window Descriptors
//!
//! Declarative configuration structures specifying geometry, styling, and dismissal
//! behaviors for centered modal dialogues and detached floating windows.

use iris_core::Color;
use iris_core::geometry::{Point, Rect};

use super::style::ModalStyle;

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
    pub backdrop_color: Color,

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
            position: None,
            style: None,
            subtitle: None,
            icon: None,
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            show_close_button: true,
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

/// Declarative specification for confirmation, alert, and destructive warning modal dialogues.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmDialogDescriptor {
    /// Title string presented in the modal header bar.
    pub title: String,

    /// Primary confirmation prompt or description text.
    pub message: String,

    /// Optional target detail path or warning subtext.
    pub detail: Option<String>,

    /// Optional icon glyph or emoji shown alongside the title.
    pub icon: Option<String>,

    /// Caption presented on the primary action button (e.g. "Confirm", "Save", "Delete").
    pub confirm_label: String,

    /// Caption presented on the secondary cancel button.
    pub cancel_label: String,

    /// Whether the confirm button applies high-visibility danger styling.
    pub is_danger: bool,

    /// Card dimensions `(width, height)` in logical pixels.
    pub size: (f32, f32),

    /// Visual chromatic configuration.
    pub style: ModalStyle,

    /// Whether clicking the backdrop dismisses the dialogue.
    pub dismiss_on_backdrop: bool,

    /// Whether pressing the keyboard Escape key dismisses the dialogue.
    pub dismiss_on_escape: bool,

    /// Whether the top-right `✕` close button is visible.
    pub show_close_button: bool,
}

impl ConfirmDialogDescriptor {
    /// Default dimensions `(420.0, 180.0)` for standard confirmation cards.
    pub const DEFAULT_SIZE: (f32, f32) = (420.0, 180.0);

    /// Creates a standard confirmation dialogue specification.
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            detail: None,
            icon: None,
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            is_danger: false,
            size: Self::DEFAULT_SIZE,
            style: ModalStyle::default(),
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            show_close_button: true,
        }
    }

    /// Creates a high-visibility danger confirmation dialogue specification (e.g., permanent deletion).
    #[inline]
    #[must_use]
    pub fn danger(
        title: impl Into<String>,
        message: impl Into<String>,
        confirm_label: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            detail: None,
            icon: None,
            confirm_label: confirm_label.into(),
            cancel_label: "Cancel".to_string(),
            is_danger: true,
            size: Self::DEFAULT_SIZE,
            style: ModalStyle::default(),
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            show_close_button: true,
        }
    }

    /// Sets an optional target detail string (e.g. file path or asset URI).
    #[inline]
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Sets an optional icon glyph or emoji shown alongside the title.
    #[inline]
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the label string on the confirmation button.
    #[inline]
    #[must_use]
    pub fn with_confirm_label(mut self, label: impl Into<String>) -> Self {
        self.confirm_label = label.into();
        self
    }

    /// Sets whether the confirmation button applies high-visibility danger styling.
    #[inline]
    #[must_use]
    pub const fn with_danger(mut self, is_danger: bool) -> Self {
        self.is_danger = is_danger;
        self
    }

    /// Sets the label string on the cancel button.
    #[inline]
    #[must_use]
    pub fn with_cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_label = label.into();
        self
    }

    /// Overrides modal card dimensions `(width, height)` in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }

    /// Overrides visual styling properties.
    #[inline]
    #[must_use]
    pub fn with_style(mut self, style: ModalStyle) -> Self {
        self.style = style;
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
}

/// Declarative specification for single-line text input modal dialogues.
#[derive(Debug, Clone, PartialEq)]
pub struct InputDialogDescriptor {
    /// Title string presented in the modal header bar.
    pub title: String,

    /// Descriptive prompt or label text shown above the text input field.
    pub prompt: String,

    /// Ghost placeholder text shown when the input buffer is empty.
    pub placeholder: String,

    /// Optional target detail path or contextual hint.
    pub detail: Option<String>,

    /// Optional icon glyph or emoji shown alongside the title.
    pub icon: Option<String>,

    /// Caption presented on the primary action button (e.g. "Create", "Apply").
    pub confirm_label: String,

    /// Caption presented on the secondary cancel button.
    pub cancel_label: String,

    /// Card dimensions `(width, height)` in logical pixels.
    pub size: (f32, f32),

    /// Visual chromatic configuration.
    pub style: ModalStyle,

    /// Whether clicking the backdrop dismisses the dialogue.
    pub dismiss_on_backdrop: bool,

    /// Whether pressing the keyboard Escape key dismisses the dialogue.
    pub dismiss_on_escape: bool,

    /// Whether the top-right `✕` close button is visible.
    pub show_close_button: bool,
}

impl InputDialogDescriptor {
    /// Default dimensions `(400.0, 190.0)` for text input cards.
    pub const DEFAULT_SIZE: (f32, f32) = (400.0, 190.0);

    /// Creates a new text input dialogue specification.
    #[inline]
    #[must_use]
    pub fn new(title: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            prompt: prompt.into(),
            placeholder: String::new(),
            detail: None,
            icon: None,
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            size: Self::DEFAULT_SIZE,
            style: ModalStyle::default(),
            dismiss_on_backdrop: true,
            dismiss_on_escape: true,
            show_close_button: true,
        }
    }

    /// Sets the placeholder text for the text input box.
    #[inline]
    #[must_use]
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets an optional detail text string (e.g. target path or parent folder).
    #[inline]
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Sets an optional icon glyph or emoji shown alongside the title.
    #[inline]
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the caption for the primary confirmation button.
    #[inline]
    #[must_use]
    pub fn with_confirm_label(mut self, label: impl Into<String>) -> Self {
        self.confirm_label = label.into();
        self
    }

    /// Sets the caption for the secondary cancel button.
    #[inline]
    #[must_use]
    pub fn with_cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_label = label.into();
        self
    }

    /// Overrides modal card dimensions `(width, height)` in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }

    /// Overrides visual styling properties.
    #[inline]
    #[must_use]
    pub fn with_style(mut self, style: ModalStyle) -> Self {
        self.style = style;
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