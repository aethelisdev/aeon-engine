// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Detached Floating Window Descriptors
//!
//! Provides configuration structures for detached dock tabs and auxiliary floating panels.

use iris_core::geometry::Rect;

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
    /// # Arguments
    /// * `id` - The hosted panel identifier.
    /// * `title` - Window title string.
    /// * `rect` - Position and initial dimensions.
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
        let candidate = Rect::new(
            self.rect.x,
            self.rect.y,
            self.rect.width.max(min_w),
            self.rect.height.max(min_h),
        );
        self.rect = candidate.clamp_to_bounds(viewport);
        self
    }
}