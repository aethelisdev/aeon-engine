// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scope Tagged State & Query Extensions
//!
//! Provides persistent 64-bit semantic tag queries, scroll tracking, inline text edit state,
//! collapsing state persistence, submenu flyout tracking, and geometry introspection for [`UiScope`].

use super::context::UiScope;
use irisui_core::{InteractionEvent, Point, Rect, Size, WidgetId, WidgetNode};

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

    /// Synchronizes internal caret blink timing inside `text_system` with this scope's [`irisui_core::UiTree`]
    /// using the monotonic system clock.
    #[inline]
    pub fn sync_caret_blink(
        &self,
        text_system: &mut irisui_text::TextSystem,
    ) -> irisui_text::CaretBlinkUpdate {
        text_system.sync_caret_blink(self.tree)
    }

    /// Synchronizes internal caret blink timing inside `text_system` with this scope's [`irisui_core::UiTree`]
    /// using an explicit elapsed frame `delta`.
    #[inline]
    pub fn sync_caret_blink_with_delta(
        &self,
        text_system: &mut irisui_text::TextSystem,
        delta: std::time::Duration,
    ) -> irisui_text::CaretBlinkUpdate {
        text_system.sync_caret_blink_with_delta(self.tree, delta)
    }

    /// Returns `true` when any text input in this scope holds keyboard focus and `text_system`
    /// is currently in the visible phase of its internal caret blink cycle.
    #[inline]
    #[must_use]
    pub fn is_caret_visible(&self, text_system: &irisui_text::TextSystem) -> bool {
        self.tree.is_text_input_focused() && text_system.is_caret_visible()
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

    /// Evaluates whether the scroll container tagged with `tag` was scrolled during the current frame.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target scroll container.
    #[inline]
    #[must_use]
    pub fn is_tag_scrolled(&self, tag: u64) -> bool {
        self.tree.is_tag_scrolled(tag)
    }

    /// Returns the mouse wheel or scroll displacement delta `(delta_x, delta_y)` for the container tagged with `tag`.
    ///
    /// Returns `None` if the container was not scrolled during the current dispatch cycle.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target scroll container.
    #[inline]
    #[must_use]
    pub fn tag_scroll_delta(&self, tag: u64) -> Option<(f32, f32)> {
        self.tree.tag_scroll_delta(tag)
    }

    /// Evaluates whether an upward scroll gesture (`delta_y > 0.0`) occurred on the container tagged with `tag`.
    ///
    /// In typical mouse wheel scrolling, an upward wheel roll produces positive `delta_y`,
    /// moving viewport content towards the top (earlier entries) and decreasing scroll offset.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target scroll container.
    #[inline]
    #[must_use]
    pub fn is_tag_scrolled_up(&self, tag: u64) -> bool {
        self.tree.is_tag_scrolled_up(tag)
    }

    /// Evaluates whether a downward scroll gesture (`delta_y < 0.0`) occurred on the container tagged with `tag`.
    ///
    /// In typical mouse wheel scrolling, a downward wheel roll produces negative `delta_y`,
    /// moving viewport content towards the bottom (later entries) and increasing scroll offset.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target scroll container.
    #[inline]
    #[must_use]
    pub fn is_tag_scrolled_down(&self, tag: u64) -> bool {
        self.tree.is_tag_scrolled_down(tag)
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

    /// Returns the most recently observed logical cursor pointer position in screen coordinates, if known.
    ///
    /// Directly introspects the backing [`irisui_core::UiTree`]'s tracked pointer event state
    /// without heap allocation or imperative state traversal.
    ///
    /// # Returns
    /// * `Some(Point)` containing the active physical/logical screen cursor coordinates.
    /// * `None` if no pointer interactions have yet occurred or if the cursor exited the surface.
    #[inline]
    #[must_use]
    pub fn cursor_pos(&self) -> Option<Point> {
        self.tree.cursor_position()
    }

    /// Evaluates active cursor pointer coordinates or returns `fallback` when unavailable.
    #[inline]
    #[must_use]
    pub fn resolved_cursor_pos(&self, fallback: Point) -> Point {
        self.cursor_pos().unwrap_or(fallback)
    }

    /// Sets or updates the active cursor pointer position on the backing [`irisui_core::UiTree`].
    ///
    /// Updates the autonomous event state on the widget tree to synchronize programmatic
    /// or synthetic pointer interactions with subsequent layout and widget queries.
    ///
    /// # Arguments
    /// * `pos` - Target screen coordinates for the cursor pointer.
    #[inline]
    pub fn set_cursor_pos(&mut self, pos: Point) {
        self.tree.set_cursor_position(pos);
    }

    /// Returns whether the specified semantic `tag` is registered as an active popup menu.
    ///
    /// Directly introspects the backing [`irisui_core::UiTree`] without leaking mutable state.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target popup or dropdown widget.
    #[inline]
    #[must_use]
    pub fn is_popup_tag_active(&self, tag: u64) -> bool {
        self.tree.is_popup_tag_active(tag)
    }

    /// Registers or unregisters a 64-bit semantic `tag` as an active open popup menu or dropdown.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the popup or dropdown widget (`0` is ignored when opening).
    /// * `active` - `true` to mark the popup as open; `false` to close it.
    #[inline]
    pub fn set_popup_tag_active(&mut self, tag: u64, active: bool) {
        self.tree.set_popup_tag_active(tag, active);
    }

    /// Marks the scoped popup identified by semantic `tag` as open in the backing [`irisui_core::UiTree`].
    #[inline]
    pub fn open_popup(&mut self, tag: u64) {
        self.tree.open_popup(tag);
    }

    /// Closes the scoped popup identified by semantic `tag`.
    #[inline]
    pub fn close_popup(&mut self, tag: u64) {
        self.tree.close_popup(tag);
    }

    /// Toggles the open state of the scoped popup identified by semantic `tag`, returning the new state.
    #[inline]
    pub fn toggle_popup(&mut self, tag: u64) -> bool {
        self.tree.toggle_popup(tag)
    }

    /// Toggles a mutually exclusive scoped popup identified by `tag`, clearing other active dropdown/submenu states.
    #[inline]
    pub fn toggle_exclusive_popup(&mut self, tag: u64) -> bool {
        self.tree.toggle_exclusive_popup(tag)
    }

    /// Declaratively executes `add_contents` only if the popup identified by `tag` is currently active.
    ///
    /// Eliminates intermediate boolean fields and conditional boilerplate in consumer UI panels.
    #[inline]
    pub fn when_popup_active<R, F>(&mut self, tag: u64, add_contents: F) -> Option<R>
    where
        F: FnOnce(&mut Self) -> R,
    {
        if self.tree.is_popup_tag_active(tag) {
            Some(add_contents(self))
        } else {
            None
        }
    }

    /// Clears all registered active popup tags on the backing [`irisui_core::UiTree`].
    #[inline]
    pub fn clear_active_popup_tags(&mut self) {
        self.tree.clear_active_popup_tags();
    }

    /// Dismisses active popups and context menus if `point` falls outside their interactive bounds.
    ///
    /// Delegates directly to [`irisui_core::UiTree::dismiss_popups_on_outside_click`].
    ///
    /// # Arguments
    /// * `point` - Pointer coordinate where the mouse press occurred.
    ///
    /// # Returns
    /// `true` if an outside click was detected and popups were dismissed; `false` otherwise.
    #[inline]
    pub fn dismiss_popups_on_outside_click(&mut self, point: Point) -> bool {
        self.tree.dismiss_popups_on_outside_click(point)
    }

    /// Evaluates whether any floating popup layer or dropdown menu is actively open in the tree.
    ///
    /// Delegates directly to [`irisui_core::UiTree::has_open_popup`].
    #[inline]
    #[must_use]
    pub fn has_open_popup(&self) -> bool {
        self.tree.has_open_popup()
    }

    /// Dismisses and closes all active floating popup layers and dropdown menus across the tree.
    ///
    /// Returns `true` if at least one popup layer was actively open and closed.
    #[inline]
    pub fn close_all_popups(&mut self) -> bool {
        self.tree.close_all_popups()
    }

    /// Returns `true` if an outside-click or dismiss gesture closed popups or context menus during the current frame.
    #[inline]
    #[must_use]
    pub fn popups_dismissed(&self) -> bool {
        self.tree.popups_dismissed()
    }

    /// Returns whether the widget tagged with `tag` currently holds keyboard focus or an active text edit session.
    ///
    /// Delegates directly to [`irisui_core::UiTree::is_tag_focused`].
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the target widget.
    #[inline]
    #[must_use]
    pub fn is_tag_focused(&self, tag: u64) -> bool {
        self.tree.is_tag_focused(tag)
    }

    /// Returns whether an active inline text or numeric editing buffer currently holds keyboard input focus.
    #[inline]
    #[must_use]
    pub fn has_text_input_focus(&self) -> bool {
        self.tree.event_state.has_text_input_focus()
    }

    /// Clears any active keyboard input focus, pressed state, and text editing sessions across the tree.
    #[inline]
    pub fn clear_focus(&mut self) {
        self.tree.event_state.clear_focus();
    }

    /// Automatically synchronizes the provided [`crate::controls::focus::FocusState`] with the active text edit state in the tree.
    ///
    /// Drops focus if text editing has concluded. Returns `true` if active focus was cleared.
    #[inline]
    pub fn auto_sync_focus_state(
        &mut self,
        focus_state: &mut crate::controls::focus::FocusState<crate::controls::focus::FocusTarget>,
    ) -> bool {
        focus_state.auto_sync_focus_state(self.tree)
    }

    /// Automatically synchronizes the provided core [`irisui_core::event::FocusManager`] with the active text edit state in the tree.
    ///
    /// Drops focus if text editing has concluded. Returns `true` if active focus was cleared.
    #[inline]
    pub fn auto_sync_core_focus(
        &mut self,
        focus_manager: &mut irisui_core::event::FocusManager,
    ) -> bool {
        focus_manager.auto_sync_focus_state(self.tree)
    }

    /// Automatically synchronizes a [`crate::input::TextInputState`] with live text editing buffer events
    /// and handles optional clear button clicks.
    ///
    /// If text editing is active on `input_tag`, updates `state.text` and `state.cursor_byte_idx`.
    /// If `clear_tag` is clicked, clears `state`, clears keyboard focus, clears active text edit,
    /// and returns `true`.
    ///
    /// # Arguments
    /// * `input_tag` - 64-bit semantic tag of the target text input field.
    /// * `clear_tag` - Optional 64-bit semantic tag of the input's clear/reset button.
    /// * `state` - Mutable reference to the persistent text input state.
    pub fn auto_sync_text_input(
        &mut self,
        input_tag: u64,
        clear_tag: Option<u64>,
        state: &mut crate::input::TextInputState,
    ) -> bool {
        if let Some(buf) = self.text_edit_buffer(input_tag) {
            state.set_text(buf);
            if let Some(cur) = self.text_edit_cursor(input_tag) {
                state.cursor_byte_idx = cur;
            }
            return true;
        }
        if let Some(c_tag) = clear_tag
            && self.is_tag_clicked(c_tag)
        {
            state.clear();
            self.clear_focus();
            self.clear_text_edit();
            return true;
        }
        false
    }

    /// Assigns keyboard focus to `tag` across both core [`FocusManager`] and widget [`FocusState`].
    #[inline]
    pub fn assign_focus_with(
        &mut self,
        tag: u64,
        focus_manager: &mut irisui_core::event::FocusManager,
        focus_state: &mut crate::controls::focus::FocusState<crate::controls::focus::FocusTarget>,
    ) {
        focus_manager.set_focus_tag(tag);
        focus_state.sync_from_core(focus_manager);
    }

    /// Clears keyboard focus on core [`FocusManager`], widget [`FocusState`], and this tree scope.
    #[inline]
    pub fn clear_focus_with(
        &mut self,
        focus_manager: &mut irisui_core::event::FocusManager,
        focus_state: &mut crate::controls::focus::FocusState<crate::controls::focus::FocusTarget>,
    ) -> bool {
        let had_core = focus_manager.has_focus();
        focus_manager.clear_focus();
        let cleared_state = focus_state.clear_focus();
        self.clear_focus();
        self.clear_text_edit();
        had_core || cleared_state
    }

    /// Synchronizes keyboard focus managers alongside live search text buffer and clear triggers.
    pub fn auto_sync_search_input(
        &mut self,
        input_tag: u64,
        clear_tag: Option<u64>,
        state: &mut crate::input::TextInputState,
        focus_manager: &mut irisui_core::event::FocusManager,
        focus_state: &mut crate::controls::focus::FocusState<crate::controls::focus::FocusTarget>,
    ) -> bool {
        self.auto_sync_core_focus(focus_manager);
        self.auto_sync_focus_state(focus_state);
        if clear_tag.is_some_and(|c_tag| self.is_tag_clicked(c_tag)) {
            state.clear();
            self.clear_focus_with(focus_manager, focus_state);
            return true;
        }
        self.auto_sync_text_input(input_tag, None, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::geometry::{Point, Rect};
    use irisui_core::node::{UiLayer, WidgetRole};
    use irisui_core::tree::UiTree;

    #[test]
    fn test_ui_scope_popup_introspection_and_dismissal() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");

        let mut scope = UiScope::new(&mut tree, root);
        assert!(!scope.has_open_popup());
        assert!(!scope.is_popup_tag_active(42));

        scope.set_popup_tag_active(42, true);
        assert!(scope.is_popup_tag_active(42));

        // Create a popup node
        let popup_node = scope.tree.create_node();
        if let Some(node) = scope.tree.get_mut(popup_node) {
            node.set_layer(UiLayer::Popup);
            node.set_role(WidgetRole::DropdownPopup);
            node.computed_rect = Rect::new(100.0, 100.0, 200.0, 150.0);
            node.interactive = true;
        }
        let _ = scope.tree.add_child(root, popup_node);

        assert!(scope.has_open_popup());

        // Inside click should not dismiss
        let inside_click = Point::new(150.0, 150.0);
        assert!(!scope.dismiss_popups_on_outside_click(inside_click));
        assert!(scope.has_open_popup());
        assert!(scope.is_popup_tag_active(42));

        // Outside click should dismiss popups and clear active tags
        let outside_click = Point::new(500.0, 500.0);
        assert!(scope.dismiss_popups_on_outside_click(outside_click));
        assert!(!scope.has_open_popup());
        assert!(!scope.is_popup_tag_active(42));
        assert!(scope.popups_dismissed());
    }

    #[test]
    fn test_ui_scope_close_all_popups() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");

        let mut scope = UiScope::new(&mut tree, root);
        assert!(!scope.has_open_popup());

        scope.set_popup_tag_active(99, true);
        assert!(scope.has_open_popup());
        assert!(scope.close_all_popups());
        assert!(!scope.has_open_popup());
        assert!(!scope.close_all_popups());
    }

    #[test]
    fn test_ui_scope_focus_queries_and_auto_sync() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");

        let mut scope = UiScope::new(&mut tree, root);
        assert!(!scope.is_tag_focused(55));
        assert!(!scope.has_text_input_focus());

        // 1. Start text edit
        scope
            .tree
            .event_state
            .start_text_edit(None, 55, "abc", 3, None);
        assert!(scope.is_tag_focused(55));
        assert!(scope.has_text_input_focus());

        // 2. Auto-sync focus state and core focus
        let mut focus_state = crate::controls::focus::FocusState::new();
        let mut focus_manager = irisui_core::event::FocusManager::new();

        let cleared_state = scope.auto_sync_focus_state(&mut focus_state);
        let cleared_core = scope.auto_sync_core_focus(&mut focus_manager);
        assert!(!cleared_state);
        assert!(!cleared_core);
        assert!(focus_state.is_tag_focused(55));
        assert_eq!(focus_manager.focused_tag(), Some(55));

        // 3. Clear text edit session
        scope.clear_text_edit();
        assert!(!scope.has_text_input_focus());

        // 4. Auto-sync drops focus automatically
        let cleared_state = scope.auto_sync_focus_state(&mut focus_state);
        let cleared_core = scope.auto_sync_core_focus(&mut focus_manager);
        assert!(cleared_state);
        assert!(cleared_core);
        assert!(!focus_state.has_focus());
        assert!(!focus_manager.has_focus());
    }

    #[test]
    fn test_ui_scope_auto_sync_text_input() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");

        let mut scope = UiScope::new(&mut tree, root);
        let mut input_state = crate::input::TextInputState::new("initial");

        // 1. Inactive text edit does nothing
        assert!(!scope.auto_sync_text_input(100, Some(101), &mut input_state));
        assert_eq!(input_state.text(), "initial");

        // 2. Active text edit syncs text and cursor
        scope
            .tree
            .event_state
            .start_text_edit(None, 100, "updated query", 13, None);
        assert!(scope.auto_sync_text_input(100, Some(101), &mut input_state));
        assert_eq!(input_state.text(), "updated query");
        assert_eq!(input_state.cursor_byte_idx, 13);

        // 3. Clear button click clears state, focus, and text edit
        scope.clear_text_edit();
        scope.tree.event_state.frame_clicked_tag = Some(101);
        assert!(scope.auto_sync_text_input(100, Some(101), &mut input_state));
        assert_eq!(input_state.text(), "");
        assert_eq!(input_state.cursor_byte_idx, 0);
    }
}