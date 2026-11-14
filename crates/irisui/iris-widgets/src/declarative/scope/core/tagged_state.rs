// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scope Tagged State & Query Extensions
//!
//! Provides persistent 64-bit semantic tag queries, scroll tracking, inline text edit state,
//! collapsing state persistence, submenu flyout tracking, and geometry introspection for [`UiScope`].

use super::context::UiScope;
use iris_core::{InteractionEvent, Point, Rect, Size, WidgetId, WidgetNode};

impl<'a> UiScope<'a> {
    /// Returns whether a widget tagged with `tag` was clicked during the current frame dispatch pass.
    ///
    /// Evaluates both autonomous event state tracking and explicitly dispatched interaction events.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target widget.
    #[inline]
    #[must_use]
    pub fn is_tag_clicked(&self, tag: u64) -> bool {
        self.tree.event_state.frame_clicked_tag == Some(tag)
            || self
                .tagged_events
                .iter()
                .any(|(t, ev)| *t == tag && matches!(ev, InteractionEvent::Click { .. }))
    }

    /// Returns whether a widget tagged with `tag` is currently pressed down by pointer.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target widget.
    #[inline]
    #[must_use]
    pub fn is_tag_pressed(&self, tag: u64) -> bool {
        self.tree.event_state.pressed_tag == Some(tag)
    }

    /// Returns the active vertical scroll offset in physical pixels for the scrollable viewport tagged with `tag`.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target scroll container.
    #[inline]
    #[must_use]
    pub fn scroll_offset(&self, tag: u64) -> f32 {
        self.tree.scroll_offset(tag)
    }

    /// Updates the active vertical scroll offset for the scrollable viewport tagged with `tag`.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target scroll container.
    /// * `offset` - Target vertical scroll position in physical pixels.
    #[inline]
    pub fn set_scroll_offset(&mut self, tag: u64, offset: f32) {
        self.tree.set_scroll_offset(tag, offset);
    }

    /// Clears any active inline text editing session across all input fields in the UI tree.
    #[inline]
    pub fn clear_text_edit(&mut self) {
        self.tree.event_state.clear_text_edit();
    }

    /// Returns the current text buffer string for an active text input widget tagged with `tag`.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target text input.
    #[inline]
    #[must_use]
    pub fn text_edit_buffer(&self, tag: u64) -> Option<&str> {
        self.tree.event_state.text_edit_buffer(tag)
    }

    /// Returns the active cursor byte index for a text input widget tagged with `tag`.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target text input.
    #[inline]
    #[must_use]
    pub fn text_edit_cursor(&self, tag: u64) -> Option<usize> {
        self.tree.event_state.text_edit_cursor(tag)
    }

    /// Returns whether a collapsing header or tree node tagged with `tag` is currently expanded.
    ///
    /// Falls back to `default_open` when no explicit expansion state has been registered yet.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target collapsing node.
    /// * `default_open` - Default expansion state if not previously interacted with.
    #[inline]
    #[must_use]
    pub fn is_collapsing_open(&self, tag: u64, default_open: bool) -> bool {
        self.tree.is_collapsing_open(tag, default_open)
    }

    /// Updates the persistent expansion state of a collapsing header or tree node tagged with `tag`.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target collapsing node.
    /// * `open` - New expansion flag (`true` for expanded, `false` for collapsed).
    #[inline]
    pub fn set_collapsing_open(&mut self, tag: u64, open: bool) {
        self.tree.set_collapsing_open(tag, open);
    }

    /// Registers an active cascading flyout child submenu under the parent menu card.
    ///
    /// # Arguments
    /// * `parent_tag` - 64-bit semantic tag of the parent menu card.
    /// * `sub_tag` - 64-bit semantic tag of the active child submenu.
    #[inline]
    pub fn set_active_submenu(&mut self, parent_tag: u64, sub_tag: u64) {
        self.tree.set_active_submenu(parent_tag, sub_tag);
    }

    /// Returns whether a child submenu branch tagged with `sub_tag` is currently active.
    ///
    /// # Arguments
    /// * `parent_tag` - 64-bit semantic tag of the parent menu card.
    /// * `sub_tag` - 64-bit semantic tag of the child submenu.
    #[inline]
    #[must_use]
    pub fn is_submenu_open(&self, parent_tag: u64, sub_tag: u64) -> bool {
        self.tree.is_submenu_open(parent_tag, sub_tag)
    }

    /// Sets the total virtual content bounds size for the scrollable container tagged with `tag`.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target scrollable container.
    /// * `width` - Virtual content width in physical pixels.
    /// * `height` - Virtual content height in physical pixels.
    #[inline]
    pub fn set_content_size_tagged(&mut self, tag: u64, width: f32, height: f32) {
        self.tree.set_content_size(tag, Size::new(width, height));
    }

    /// Returns the generational [`WidgetId`] of the widget node tagged with `tag`, if present.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target widget node.
    #[inline]
    #[must_use]
    pub fn node_id_tagged(&self, tag: u64) -> Option<WidgetId> {
        self.tree.find_node_by_tag(tag)
    }

    /// Returns an immutable reference to the [`WidgetNode`] with the specified generational [`WidgetId`].
    ///
    /// # Arguments
    /// * `id` - Generational widget identifier.
    #[inline]
    #[must_use]
    pub fn node_ref(&self, id: WidgetId) -> Option<&WidgetNode> {
        self.tree.get(id)
    }

    /// Returns the computed bounding rectangle of the widget node tagged with `tag`,
    /// translated by an optional root coordinate offset (such as a parent panel position).
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target widget node.
    /// * `offset` - Translation offset vector applied to the local computed bounds.
    #[inline]
    #[must_use]
    pub fn node_rect_tagged(&self, tag: u64, offset: Point) -> Option<Rect> {
        self.tree.node_rect_tagged(tag, offset)
    }

    /// Sets the maximum vertical scroll boundary for the container tagged with `tag`.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target container.
    /// * `max_scroll` - Maximum vertical scroll displacement in physical pixels.
    #[inline]
    pub fn set_max_scroll(&mut self, tag: u64, max_scroll: f32) {
        self.tree.set_max_scroll(tag, max_scroll);
    }

    /// Returns whether the pointer capture is currently held by the widget tagged with `tag`.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target widget.
    #[inline]
    #[must_use]
    pub fn is_tag_captured(&self, tag: u64) -> bool {
        self.tree.event_state.pointer_capture_tag == Some(tag)
            || self
                .tree
                .event_state
                .pointer_capture
                .and_then(|cid| self.tree.get(cid))
                .is_some_and(|n| n.tag == tag)
    }

    /// Returns the resolved viewport size for the container tagged with `tag`.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the container node.
    #[inline]
    #[must_use]
    pub fn viewport_size(&self, tag: u64) -> Option<Size> {
        self.tree.viewport_size(tag)
    }

    /// Returns the physical bounding rectangle of the root window/surface.
    ///
    /// Resolves the root node's `computed_rect`, falling back to its styled dimensions or a
    /// standard `(0, 0, 1920, 1080)` viewport if the root has not yet undergone initial layout computation.
    #[inline]
    #[must_use]
    pub fn screen_bounds(&self) -> Rect {
        self.tree
            .root()
            .and_then(|r| self.tree.get(r))
            .and_then(|n| {
                if n.computed_rect.width > 0.0 && n.computed_rect.height > 0.0 {
                    Some(n.computed_rect)
                } else {
                    match (n.style.width, n.style.height) {
                        (Some(w), Some(h)) if w > 0.0 && h > 0.0 => Some(Rect::new(0.0, 0.0, w, h)),
                        _ => None,
                    }
                }
            })
            .unwrap_or(Rect::new(0.0, 0.0, 1920.0, 1080.0))
    }
}