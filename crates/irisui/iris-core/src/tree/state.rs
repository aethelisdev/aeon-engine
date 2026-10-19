// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Persistent interaction, layout, and event state management for the UI tree.

use crate::dispatch::{DispatchResult, dispatch_ui_event};
use crate::event::UiEvent;
use crate::geometry::{Point, Rect, Size};
use crate::id::WidgetId;

use super::arena::UiTree;

impl UiTree {
    /// Returns whether the collapsing section with the given `tag` is currently expanded.
    ///
    /// If the tag has never been toggled by the user, returns `default_open`.
    #[inline]
    #[must_use]
    pub fn is_collapsing_open(&self, tag: u64, default_open: bool) -> bool {
        self.collapsing_state
            .get(&tag)
            .copied()
            .unwrap_or(default_open)
    }

    /// Sets the persistent expansion state of a collapsing section identified by `tag`.
    #[inline]
    pub fn set_collapsing_open(&mut self, tag: u64, open: bool) {
        self.collapsing_state.insert(tag, open);
    }

    /// Toggles the persistent expansion state of a collapsing section and returns the new state.
    #[inline]
    pub fn toggle_collapsing_open(&mut self, tag: u64, default_open: bool) -> bool {
        let current = self.is_collapsing_open(tag, default_open);
        let new_state = !current;
        self.collapsing_state.insert(tag, new_state);
        new_state
    }

    /// Returns whether the menu bar dropdown identified by semantic `tag` is currently opened.
    #[inline]
    #[must_use]
    pub fn is_menu_open(&self, tag: u64) -> bool {
        self.active_menu_bar_tag == Some(tag)
    }

    /// Sets or clears the active menu bar dropdown identified by semantic `tag`.
    ///
    /// # Arguments
    /// * `tag` - Optional 64-bit semantic tag of the opened menu, or `None` to close all dropdowns.
    #[inline]
    pub fn set_active_menu(&mut self, tag: Option<u64>) {
        self.active_menu_bar_tag = tag;
    }

    /// Closes any currently opened menu bar dropdown popup.
    #[inline]
    pub fn close_menu(&mut self) {
        self.active_menu_bar_tag = None;
    }

    /// Returns whether the combobox identified by semantic `tag` is currently opened.
    #[inline]
    #[must_use]
    pub fn is_combobox_open(&self, tag: u64) -> bool {
        self.active_combobox_tag == Some(tag)
    }

    /// Sets or clears the active combobox dropdown identified by semantic `tag`.
    ///
    /// # Arguments
    /// * `tag` - Optional 64-bit semantic tag of the opened combobox, or `None` to close all dropdowns.
    #[inline]
    pub fn set_active_combobox(&mut self, tag: Option<u64>) {
        self.active_combobox_tag = tag;
    }

    /// Closes any currently opened combobox popup.
    #[inline]
    pub fn close_combobox(&mut self) {
        self.active_combobox_tag = None;
    }

    /// Returns the persistent vertical scroll position for the given semantic `tag`.
    ///
    /// Defaults to `0.0` if this tag has never been scrolled.
    #[inline]
    #[must_use]
    pub fn scroll_offset(&self, tag: u64) -> f32 {
        self.scroll_offsets.get(&tag).copied().unwrap_or(0.0)
    }

    /// Sets the persistent vertical scroll position for the given semantic `tag`.
    #[inline]
    pub fn set_scroll_offset(&mut self, tag: u64, offset: f32) {
        self.scroll_offsets.insert(tag, offset.max(0.0));
    }

    /// Returns the resolved viewport size for the given semantic `tag` computed by layout.
    #[inline]
    #[must_use]
    pub fn viewport_size(&self, tag: u64) -> Option<Size> {
        self.viewport_sizes.get(&tag).copied()
    }

    /// Sets the resolved viewport size for the given semantic `tag`.
    #[inline]
    pub fn set_viewport_size(&mut self, tag: u64, size: Size) {
        self.viewport_sizes.insert(tag, size);
    }

    /// Returns the maximum vertical scroll extent for the given semantic `tag`.
    ///
    /// Defaults to `0.0` if this tag has no measured scrollable overflow content.
    #[inline]
    #[must_use]
    pub fn max_scroll(&self, tag: u64) -> f32 {
        self.max_scrolls.get(&tag).copied().unwrap_or(0.0)
    }

    /// Sets the content size for the node identified by the semantic `tag`.
    ///
    /// Updates the measured content bounds used for scroll normalization and bounds checking.
    #[inline]
    pub fn set_content_size(&mut self, tag: u64, size: Size) {
        if let Some(node) = self.find_node_by_tag(tag).and_then(|id| self.get_mut(id)) {
            node.content_size = size;
        }
    }

    /// Sets the maximum vertical scroll extent for the given semantic `tag`.
    #[inline]
    pub fn set_max_scroll(&mut self, tag: u64, max_scroll: f32) {
        self.max_scrolls.insert(tag, max_scroll.max(0.0));
    }

    /// Dispatches an incoming windowing event hierarchically through the tree.
    ///
    /// Evaluates pointer capture, layered z-order hit-testing, and event bubbling.
    #[inline]
    pub fn dispatch_event(&mut self, event: &UiEvent) -> DispatchResult {
        dispatch_ui_event(self, event)
    }

    /// Sets or releases active pointer capture for a widget node.
    #[inline]
    pub fn set_pointer_capture(&mut self, capture: Option<WidgetId>) {
        self.event_state.pointer_capture = capture;
    }

    /// Returns the widget node currently capturing pointer events, if any.
    #[inline]
    pub fn pointer_capture(&self) -> Option<WidgetId> {
        self.event_state.pointer_capture
    }

    /// Returns the most recently recorded cursor pointer position in physical screen coordinates, if known.
    #[inline]
    #[must_use]
    pub fn cursor_position(&self) -> Option<Point> {
        self.event_state.last_pointer_pos
    }

    /// Sets or updates the active hardware cursor pointer position in physical screen coordinates.
    #[inline]
    pub fn set_cursor_position(&mut self, pos: Point) {
        self.event_state.last_pointer_pos = Some(pos);
    }

    /// Sets the vertical scroll offset in physical pixels on the specified widget node.
    ///
    /// Returns `true` if the node exists and its scroll offset was updated, `false` otherwise.
    #[inline]
    pub fn set_scroll_offset_y(&mut self, id: WidgetId, offset_y: f32) -> bool {
        if let Some(node) = self.nodes.get_mut(id) {
            node.style.scroll_offset_y = offset_y;
            true
        } else {
            false
        }
    }

    /// Sets the computed rectangle Y coordinate on the specified widget node in place.
    ///
    /// Returns `true` if the node exists and was updated, `false` otherwise.
    #[inline]
    pub fn set_computed_rect_y(&mut self, id: WidgetId, y: f32) -> bool {
        if let Some(node) = self.nodes.get_mut(id) {
            node.computed_rect.y = y;
            true
        } else {
            false
        }
    }

    /// Sets the semantic interaction tag on the specified widget node.
    ///
    /// Returns `true` if the node exists and was updated, `false` otherwise.
    #[inline]
    pub fn set_tag(&mut self, id: WidgetId, tag: u64) -> bool {
        if let Some(node) = self.nodes.get_mut(id) {
            node.tag = tag;
            true
        } else {
            false
        }
    }

    /// Sets the resolved computed bounding rectangle on the specified widget node.
    ///
    /// Returns `true` if the node exists and was updated, `false` otherwise.
    #[inline]
    pub fn set_computed_rect(&mut self, id: WidgetId, rect: Rect) -> bool {
        if let Some(node) = self.nodes.get_mut(id) {
            node.computed_rect = rect;
            true
        } else {
            false
        }
    }

    /// Returns the active visual theme tokens configured on this tree.
    #[inline]
    #[must_use]
    pub fn theme(&self) -> crate::theme::Theme {
        self.theme
    }

    /// Sets the active visual theme tokens governing surface colors, text, and metrics on this tree.
    #[inline]
    pub fn set_theme(&mut self, theme: crate::theme::Theme) {
        self.theme = theme;
    }

    /// Returns the cached screen-space bounding rectangle of a tagged node from previous layout passes, if any.
    #[inline]
    #[must_use]
    pub fn tag_rect(&self, tag: u64) -> Option<Rect> {
        self.tag_rects.get(&tag).copied()
    }

    /// Stores the resolved screen-space bounding rectangle for a tagged node.
    #[inline]
    pub fn set_tag_rect(&mut self, tag: u64, rect: Rect) {
        if tag != 0 && rect.width > 0.0 && rect.height > 0.0 {
            self.tag_rects.insert(tag, rect);
        }
    }
}