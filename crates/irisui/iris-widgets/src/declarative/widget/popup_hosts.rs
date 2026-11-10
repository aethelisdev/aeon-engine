// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Dropdown and Context Menu Hosts (`DropdownHost`, `ContextMenuHost`)
//!
//! Provides the generic [`DropdownHost`] and [`ContextMenuHost`] coordinators, encapsulating
//! open/close state transitions, viewport bounds clamping, automatic upward/downward flipping,
//! outside-click dismissal, and Escape key cancellation.
//!
//! Eliminates ad-hoc coordinate calculations, manual scissor bypass hacks, and bespoke
//! popup tracking in client applications.

use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::menu::PopupPlacement;
use iris_core::event::KeyCode;
use iris_core::geometry::{Point, Rect};

/// Magic seed constant used when deriving persistent semantic tags for popup items.
pub const POPUP_HOST_ITEM_SEED: u64 = 0x504F_5055_5048_4F53; // "POPUPOHS"

/// An individual actionable or separator entry within a [`DropdownHost`] or [`ContextMenuHost`].
#[derive(Debug, Clone, PartialEq)]
pub struct DropdownItem<T> {
    /// Action payload emitted when this item is confirmed by the user.
    pub value: T,
    /// Primary text label displayed on the menu row.
    pub label: String,
    /// Optional leading visual icon or emoji text glyph.
    pub icon: Option<String>,
    /// Optional right-aligned keyboard shortcut annotation string.
    pub shortcut: Option<String>,
    /// Whether this row represents a destructive action styled with danger cues.
    pub is_destructive: bool,
    /// Whether this row renders as a visual separator line.
    pub is_separator: bool,
    /// Whether this item is interactive and selectable.
    pub is_enabled: bool,
}

impl<T> DropdownItem<T> {
    /// Constructs an actionable menu item with the specified value and text label.
    #[must_use]
    #[inline]
    pub fn item(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            icon: None,
            shortcut: None,
            is_destructive: false,
            is_separator: false,
            is_enabled: true,
        }
    }

    /// Constructs an actionable menu item with a leading icon glyph, value, and label.
    #[must_use]
    #[inline]
    pub fn item_with_icon(value: T, icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            icon: Some(icon.into()),
            shortcut: None,
            is_destructive: false,
            is_separator: false,
            is_enabled: true,
        }
    }

    /// Constructs a visual horizontal separator row.
    #[must_use]
    #[inline]
    pub fn separator(dummy_value: T) -> Self {
        Self {
            value: dummy_value,
            label: String::new(),
            icon: None,
            shortcut: None,
            is_destructive: false,
            is_separator: true,
            is_enabled: false,
        }
    }

    /// Attaches an optional right-aligned keyboard shortcut annotation.
    #[must_use]
    #[inline]
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Marks this item as representing a dangerous or destructive action.
    #[must_use]
    #[inline]
    pub const fn destructive(mut self) -> Self {
        self.is_destructive = true;
        self
    }

    /// Marks this item as disabled and non-interactive.
    #[must_use]
    #[inline]
    pub const fn disabled(mut self) -> Self {
        self.is_enabled = false;
        self
    }
}

/// Autonomous coordinator managing floating dropdown menus anchored to button widgets.
#[derive(Debug, Clone, PartialEq)]
pub struct DropdownHost<T> {
    is_open: bool,
    anchor_rect: Rect,
    selected: Option<T>,
    preferred_width: Option<f32>,
    placement: PopupPlacement,
}

impl<T> Default for DropdownHost<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DropdownHost<T> {
    /// Creates a new, closed dropdown host coordinator.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            is_open: false,
            anchor_rect: Rect::ZERO,
            selected: None,
            preferred_width: None,
            placement: PopupPlacement::Below,
        }
    }

    /// Opens the dropdown anchored to the specified trigger rectangle.
    #[inline]
    pub fn open(&mut self, anchor: Rect) {
        self.is_open = true;
        self.anchor_rect = anchor;
        self.selected = None;
    }

    /// Toggles the open/closed state of the dropdown for the given anchor rectangle.
    pub fn toggle(&mut self, anchor: Rect) {
        if self.is_open && self.anchor_rect == anchor {
            self.close();
        } else {
            self.open(anchor);
        }
    }

    /// Closes the dropdown menu.
    #[inline]
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Returns `true` if the dropdown menu is currently active and open.
    #[must_use]
    #[inline]
    pub const fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns the active anchor rectangle bounding box.
    #[must_use]
    #[inline]
    pub const fn anchor_rect(&self) -> Rect {
        self.anchor_rect
    }

    /// Overrides the preferred width of the popup menu card in pixels.
    #[must_use]
    #[inline]
    pub const fn with_width(mut self, width: f32) -> Self {
        self.preferred_width = Some(width);
        self
    }

    /// Overrides the vertical placement preference.
    #[must_use]
    #[inline]
    pub const fn with_placement(mut self, placement: PopupPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Queries the most recently confirmed selection without consuming it.
    #[must_use]
    #[inline]
    pub const fn selected(&self) -> Option<&T> {
        self.selected.as_ref()
    }

    /// Consumes and returns the active selection value, resetting it to `None`.
    #[inline]
    pub fn take_selected(&mut self) -> Option<T> {
        self.selected.take()
    }
}

impl<T: Clone + PartialEq> DropdownHost<T> {
    /// Evaluates and renders the dropdown menu if currently open, returning the newly selected item value if clicked.
    ///
    /// Autonomously handles:
    /// 1. Clamping and flipping relative to screen boundaries.
    /// 2. Outside click dismissal.
    /// 3. Escape key cancellation.
    ///
    /// # Arguments
    /// * `scope` - Current UI declarative scope.
    /// * `items` - Slice of selectable dropdown items or separators.
    pub fn show(&mut self, scope: &mut UiScope<'_>, items: &[DropdownItem<T>]) -> Option<T> {
        if !self.is_open {
            return None;
        }

        // 1. Escape key cancellation
        if scope.tree().event_state.frame_key == Some(KeyCode::Escape)
            && scope.tree().event_state.frame_key_pressed
        {
            self.close();
            return None;
        }

        let width = self
            .preferred_width
            .unwrap_or(self.anchor_rect.width.max(160.0));
        let root_id = scope.tree().root().unwrap_or(scope.parent());
        let mut root_scope = scope.child_scope(root_id);

        let mut clicked_value = None;
        let popup_resp = root_scope.popup_anchored_with_placement(
            "DropdownHostPopup",
            self.anchor_rect,
            width,
            self.placement,
            |menu| {
                for (idx, item) in items.iter().enumerate() {
                    if item.is_separator {
                        menu.dropdown_separator();
                    } else {
                        let item_tag = crate::declarative::types::combine_seeds(
                            POPUP_HOST_ITEM_SEED,
                            idx as u64 + 1,
                        );
                        let icon_str = item.icon.as_deref().unwrap_or("");
                        let shortcut_str = item.shortcut.as_deref();
                        let resp = menu.dropdown_item(
                            item_tag,
                            icon_str,
                            &item.label,
                            shortcut_str,
                            item.is_enabled,
                        );
                        if resp.clicked() {
                            clicked_value = Some(item.value.clone());
                        }
                    }
                }
            },
        );

        if let Some(val) = clicked_value {
            self.selected = Some(val.clone());
            self.close();
            return Some(val);
        }

        // 2. Outside click dismissal
        let popup_rect = popup_resp.response.rect;
        if let Some(click_pt) = scope
            .tree()
            .event_state
            .frame_clicked_pos
            .or(scope.tree().event_state.press_origin)
            && !popup_rect.contains_point(click_pt)
            && !self.anchor_rect.contains_point(click_pt)
        {
            self.close();
        }

        self.take_selected()
    }
}

/// Autonomous coordinator managing floating right-click context menus.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenuHost<T> {
    is_open: bool,
    click_pos: Point,
    selected: Option<T>,
    preferred_width: f32,
    header: Option<String>,
}

impl<T> Default for ContextMenuHost<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ContextMenuHost<T> {
    /// Creates a new, closed context menu host coordinator.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            is_open: false,
            click_pos: Point::ZERO,
            selected: None,
            preferred_width: 180.0,
            header: None,
        }
    }

    /// Opens the context menu positioned at the specified pointer coordinate.
    #[inline]
    pub fn open(&mut self, click_pos: Point) {
        self.is_open = true;
        self.click_pos = click_pos;
        self.selected = None;
    }

    /// Closes the context menu.
    #[inline]
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Returns `true` if the context menu is currently open.
    #[must_use]
    #[inline]
    pub const fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns the active click origin position in screen pixels.
    #[must_use]
    #[inline]
    pub const fn click_pos(&self) -> Point {
        self.click_pos
    }

    /// Sets an optional title header string displayed at the top of the menu.
    #[must_use]
    #[inline]
    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Overrides the fixed width of the context menu card in physical pixels.
    #[must_use]
    #[inline]
    pub const fn with_width(mut self, width: f32) -> Self {
        self.preferred_width = width;
        self
    }

    /// Queries the active selection without consuming it.
    #[must_use]
    #[inline]
    pub const fn selected(&self) -> Option<&T> {
        self.selected.as_ref()
    }

    /// Consumes and returns the active selection value, resetting it to `None`.
    #[inline]
    pub fn take_selected(&mut self) -> Option<T> {
        self.selected.take()
    }
}

impl<T: Clone + PartialEq> ContextMenuHost<T> {
    /// Evaluates and renders the context menu card if open, returning the confirmed action payload when clicked.
    ///
    /// # Arguments
    /// * `scope` - Current UI declarative scope.
    /// * `items` - Slice of selectable context menu actions or separators.
    pub fn show(&mut self, scope: &mut UiScope<'_>, items: &[DropdownItem<T>]) -> Option<T> {
        if !self.is_open {
            return None;
        }

        // 1. Escape key cancellation
        if scope.tree().event_state.frame_key == Some(KeyCode::Escape)
            && scope.tree().event_state.frame_key_pressed
        {
            self.close();
            return None;
        }

        let anchor = Rect::new(self.click_pos.x, self.click_pos.y, 1.0, 1.0);
        let root_id = scope.tree().root().unwrap_or(scope.parent());
        let mut root_scope = scope.child_scope(root_id);

        let mut clicked_value = None;
        let popup_resp = root_scope.popup_anchored_named(
            "ContextMenuHostPopup",
            anchor,
            self.preferred_width,
            |menu| {
                if let Some(ref hdr) = self.header {
                    menu.label_custom(
                        hdr,
                        11.0,
                        iris_core::Color::from_u8(140, 145, 160, 255),
                        iris_core::TextAlign::Left,
                    );
                    menu.dropdown_separator();
                }

                for (idx, item) in items.iter().enumerate() {
                    if item.is_separator {
                        menu.dropdown_separator();
                    } else {
                        let item_tag = crate::declarative::types::combine_seeds(
                            POPUP_HOST_ITEM_SEED ^ 0x0F0F,
                            idx as u64 + 1,
                        );
                        let icon_str = item.icon.as_deref().unwrap_or("");
                        let shortcut_str = item.shortcut.as_deref();
                        let resp = menu.dropdown_item(
                            item_tag,
                            icon_str,
                            &item.label,
                            shortcut_str,
                            item.is_enabled,
                        );
                        if resp.clicked() {
                            clicked_value = Some(item.value.clone());
                        }
                    }
                }
            },
        );

        if let Some(val) = clicked_value {
            self.selected = Some(val.clone());
            self.close();
            return Some(val);
        }

        // 2. Outside click dismissal
        let popup_rect = popup_resp.response.rect;
        if let Some(click_pt) = scope
            .tree()
            .event_state
            .frame_clicked_pos
            .or(scope.tree().event_state.press_origin)
            && !popup_rect.contains_point(click_pt)
        {
            self.close();
        }

        self.take_selected()
    }
}