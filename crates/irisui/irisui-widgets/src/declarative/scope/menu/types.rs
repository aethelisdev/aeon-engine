// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Menu & Popup Configuration Types (`irisui-widgets::declarative::scope::menu::types`)
//!
//! Shared descriptor structs, icon wrappers, placement enums, and extension traits
//! used across menu bar, floating dropdown, cascading submenu, and anchored popup builders.

use super::super::containers::InnerResponse;
use super::super::core::UiScope;
use super::super::scoped_popup::PopupCardStyle;
use crate::declarative::types::WidgetResponse;

/// Helper distinguishing string glyphs from GPU texture atlas UVs in dropdown items.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropdownIcon<'b> {
    /// No icon rendered.
    None,
    /// Unicode text or emoji glyph.
    Text(&'b str),
    /// GPU texture atlas UV quad `[u_min, v_min, u_max, v_max]`.
    Texture([f32; 4]),
}

impl<'b> From<Option<crate::cascading_menu::CascadingMenuIcon>> for DropdownIcon<'b> {
    #[inline]
    fn from(opt: Option<crate::cascading_menu::CascadingMenuIcon>) -> Self {
        match opt {
            Some(crate::cascading_menu::CascadingMenuIcon::Text(s)) => Self::Text(s),
            Some(crate::cascading_menu::CascadingMenuIcon::Texture(uv)) => Self::Texture(uv),
            None => Self::None,
        }
    }
}

impl From<irisui_core::UiIcon> for DropdownIcon<'static> {
    #[inline]
    fn from(icon: irisui_core::UiIcon) -> Self {
        match icon {
            irisui_core::UiIcon::Glyph("") => Self::None,
            irisui_core::UiIcon::Glyph(s) => Self::Text(s),
            irisui_core::UiIcon::Atlas(uv) => Self::Texture(uv),
        }
    }
}

impl From<Option<irisui_core::UiIcon>> for DropdownIcon<'static> {
    #[inline]
    fn from(opt: Option<irisui_core::UiIcon>) -> Self {
        match opt {
            Some(icon) => icon.into(),
            None => Self::None,
        }
    }
}

/// Configuration descriptor for elevated cascading popup cards.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CascadingPopupConfig {
    /// Preferred root popup width in physical pixels.
    pub width: f32,
    /// Vertical orientation preference ([`PopupPlacement::Below`] or [`PopupPlacement::Above`]).
    pub placement: PopupPlacement,
    /// Optional semantic tag of the currently active child submenu.
    pub active_submenu_tag: Option<u64>,
    /// Optional pair of `(parent_tag, child_tag)` for nested cascading level-3 flyouts.
    pub active_nested_submenu: Option<(u64, u64)>,
    /// Optional expected height of the popup menu card in physical pixels.
    pub expected_height: Option<f32>,
    /// Fallback anchor bounds used if `anchor_tag` has not been laid out yet.
    pub fallback_rect: irisui_core::Rect,
    /// Customizable card styling overrides resolved against the active [`irisui_core::Theme`].
    pub card_style: PopupCardStyle,
}

impl CascadingPopupConfig {
    /// Creates a new cascading popup configuration with specified width and placement.
    #[inline]
    #[must_use]
    pub const fn new(width: f32, placement: PopupPlacement) -> Self {
        Self {
            width,
            placement,
            active_submenu_tag: None,
            active_nested_submenu: None,
            expected_height: None,
            fallback_rect: irisui_core::Rect::ZERO,
            card_style: PopupCardStyle::new(),
        }
    }

    /// Sets a fallback anchor rectangle used if the tagged anchor widget is not yet laid out.
    #[inline]
    #[must_use]
    pub const fn with_fallback_rect(mut self, fallback_rect: irisui_core::Rect) -> Self {
        self.fallback_rect = fallback_rect;
        self
    }

    /// Sets the active child submenu tag.
    #[inline]
    #[must_use]
    pub const fn with_active_submenu(mut self, tag: Option<u64>) -> Self {
        self.active_submenu_tag = tag;
        self
    }

    /// Sets the active nested (level-3) submenu tags.
    #[inline]
    #[must_use]
    pub const fn with_nested_submenu(mut self, nested: Option<(u64, u64)>) -> Self {
        self.active_nested_submenu = nested;
        self
    }

    /// Specifies an explicit expected initial height for the popup menu card in physical pixels.
    #[inline]
    #[must_use]
    pub const fn with_expected_height(mut self, height: f32) -> Self {
        self.expected_height = Some(height);
        self
    }

    /// Applies custom background, border, corner radius, padding, or shadow styling to the popup card.
    #[inline]
    #[must_use]
    pub const fn with_card_style(mut self, style: PopupCardStyle) -> Self {
        self.card_style = style;
        self
    }

    /// Computes and sets the expected initial height based on the total number of dropdown items.
    #[inline]
    #[must_use]
    pub const fn with_item_count(mut self, count: usize) -> Self {
        self.expected_height = Some(8.0 + (count as f32) * 24.0);
        self
    }
}

/// Placement direction for anchored floating popups relative to their anchor rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopupPlacement {
    /// Positions the popup card directly below the anchor rectangle (default).
    /// If bottom viewport overflow occurs, dynamically flips above the anchor.
    #[default]
    Below,
    /// Positions the popup card directly above the anchor rectangle.
    /// If top viewport overflow occurs, dynamically flips below the anchor.
    Above,
}

/// Extension trait adding autonomous popup capabilities to widget responses.
pub trait PopupExt {
    /// Opens or maintains an autonomous dropdown popup card anchored below this response.
    ///
    /// Returns the inner closure output if open.
    fn popup_below<R, F>(
        self,
        scope: &mut UiScope<'_>,
        width: f32,
        add_contents: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R;
}

impl PopupExt for WidgetResponse {
    #[inline]
    fn popup_below<R, F>(
        self,
        scope: &mut UiScope<'_>,
        width: f32,
        add_contents: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if self.rect.width > 0.0 && self.rect.height > 0.0 {
            Some(scope.popup_anchored_with_placement(
                "PopupBelow",
                self.rect,
                width,
                PopupPlacement::Below,
                add_contents,
            ))
        } else {
            None
        }
    }
}