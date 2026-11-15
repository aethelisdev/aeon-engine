// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Persistent interaction, layout, and event state management for the UI tree.

use crate::dispatch::{DispatchResult, dispatch_ui_event};
use crate::event::{FocusManager, UiEvent};
use crate::geometry::{Point, Rect, Size};
use crate::id::WidgetId;
use crate::node::{UiLayer, WidgetRole};

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

    /// Returns whether the modal dialog identified by semantic `tag` is currently opened.
    #[inline]
    #[must_use]
    pub fn is_modal_open(&self, tag: u64) -> bool {
        self.modal_state.get(&tag).copied().unwrap_or(false)
    }

    /// Sets the persistent visibility state of a modal dialog identified by semantic `tag`.
    #[inline]
    pub fn set_modal_open(&mut self, tag: u64, open: bool) {
        if open {
            self.modal_state.insert(tag, true);
        } else {
            self.modal_state.remove(&tag);
        }
    }

    /// Opens a modal dialog identified by semantic `tag`.
    #[inline]
    pub fn open_modal(&mut self, tag: u64) {
        self.set_modal_open(tag, true);
    }

    /// Closes a modal dialog identified by semantic `tag`.
    #[inline]
    pub fn close_modal(&mut self, tag: u64) {
        self.set_modal_open(tag, false);
    }

    /// Closes all active modal dialogs in the UI tree.
    #[inline]
    pub fn close_all_modals(&mut self) {
        self.modal_state.clear();
    }

    /// Returns whether any modal dialog is currently opened in the UI tree.
    #[inline]
    #[must_use]
    pub fn is_any_modal_open(&self) -> bool {
        !self.modal_state.is_empty()
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

    /// Closes all active popups, context menus, combobox dropdowns, submenus, tooltips, and popup-layer widgets.
    ///
    /// Returns `true` if any open popup, tooltip, or menu state was dismissed.
    #[inline]
    pub fn close_all_popups(&mut self) -> bool {
        let had_popup =
            self.has_open_popup() || self.has_open_tooltip() || self.has_active_popup_nodes();
        self.active_menu_bar_tag = None;
        self.active_combobox_tag = None;
        self.active_popup_tags.clear();
        self.event_state.active_context_menu = None;
        self.active_submenus.clear();
        self.event_state.close_tooltip();

        let root_id = self.root;
        for node in self.nodes.values_mut() {
            if node.layer == UiLayer::Popup {
                let is_anonymous_root_portal = node.role == WidgetRole::Default
                    && node.tag == 0
                    && node.name.is_none()
                    && node.parent == root_id;
                if !is_anonymous_root_portal {
                    node.visible = false;
                }
            }
        }

        if had_popup {
            self.event_state.frame_popups_dismissed = true;
        }

        had_popup
    }

    /// Returns `true` if an outside-click or dismiss gesture closed popups or context menus during the current frame.
    #[inline]
    #[must_use]
    pub fn popups_dismissed(&self) -> bool {
        self.event_state.frame_popups_dismissed
    }

    /// Returns whether any popup, context menu, combobox dropdown, menu bar dropdown,
    /// active semantic popup tag, or visible popup-layer card is currently open.
    #[inline]
    #[must_use]
    pub fn has_open_popup(&self) -> bool {
        self.event_state.active_context_menu.is_some()
            || self.active_menu_bar_tag.is_some()
            || self.active_combobox_tag.is_some()
            || !self.active_submenus.is_empty()
            || !self.active_popup_tags.is_empty()
    }

    /// Returns whether the specified semantic `tag` is registered as an active popup menu or dropdown.
    #[inline]
    #[must_use]
    pub fn is_popup_tag_active(&self, tag: u64) -> bool {
        tag != 0 && self.active_popup_tags.contains(&tag)
    }

    /// Registers or unregisters a 64-bit semantic `tag` as an active open popup menu or dropdown.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag of the popup or dropdown widget (`0` is ignored when opening).
    /// * `active` - `true` to mark the popup as open; `false` to remove it.
    #[inline]
    pub fn set_popup_tag_active(&mut self, tag: u64, active: bool) {
        if active {
            if tag != 0 {
                self.active_popup_tags.insert(tag);
            }
        } else {
            self.active_popup_tags.remove(&tag);
        }
    }

    /// Marks the semantic `tag` as an open scoped popup in [`UiTree::active_popup_tags`].
    #[inline]
    pub fn open_popup(&mut self, tag: u64) {
        self.set_popup_tag_active(tag, true);
    }

    /// Closes the scoped popup identified by `tag`, removing it from [`UiTree::active_popup_tags`].
    #[inline]
    pub fn close_popup(&mut self, tag: u64) {
        self.set_popup_tag_active(tag, false);
    }

    /// Toggles the open state of the scoped popup identified by `tag` and returns the resulting active state.
    #[inline]
    pub fn toggle_popup(&mut self, tag: u64) -> bool {
        if tag == 0 {
            return false;
        }
        if self.is_popup_tag_active(tag) {
            self.active_popup_tags.remove(&tag);
            false
        } else {
            self.active_popup_tags.insert(tag);
            true
        }
    }

    /// Toggles a mutually exclusive scoped popup identified by `tag` and returns its new state.
    ///
    /// If `tag` is already active, closes all open popups and returns `false`. Otherwise clears
    /// other active dropdown/submenu/context states and opens `tag`, returning `true`.
    #[inline]
    pub fn toggle_exclusive_popup(&mut self, tag: u64) -> bool {
        if tag == 0 {
            return false;
        }
        if self.is_popup_tag_active(tag) {
            self.close_all_popups();
            false
        } else {
            self.active_menu_bar_tag = None;
            self.active_combobox_tag = None;
            self.active_popup_tags.clear();
            self.event_state.active_context_menu = None;
            self.active_submenus.clear();
            self.active_popup_tags.insert(tag);
            true
        }
    }

    /// Clears all registered active popup tags.
    #[inline]
    pub fn clear_active_popup_tags(&mut self) {
        self.active_popup_tags.clear();
    }

    /// Returns `true` if any visible widget or non-empty portal child currently exists on [`UiLayer::Popup`].
    #[inline]
    #[must_use]
    pub fn has_active_popup_nodes(&self) -> bool {
        self.nodes.values().any(|n| {
            n.visible
                && n.layer == UiLayer::Popup
                && (n.role != WidgetRole::Default
                    || n.children
                        .iter()
                        .any(|cid| self.nodes.get(*cid).is_some_and(|c| c.visible)))
        })
    }

    /// Checks whether the specified screen-space coordinate lies within the boundary
    /// of an open popup, context menu, combobox dropdown, or elevated layer widget.
    ///
    /// Resolves semantic tags across the ancestor hierarchy so clicks on child labels,
    /// glyph icons, or padding wrappers inside the popup trigger or popup card boundary
    /// are accurately recognized as inside the popup.
    ///
    /// # Arguments
    /// * `point` - Screen coordinate in logical pixels.
    ///
    /// # Returns
    /// `true` if `point` hits an interactive element inside an open popup hierarchy; `false` otherwise.
    #[must_use]
    pub fn is_point_inside_popup(&self, point: Point) -> bool {
        let Some(hid) = self.hit_test_layered(point) else {
            return false;
        };

        if let Some(active) = self.event_state.active_context_menu
            && let Some(menu_cid) = active.container_id
            && (hid == menu_cid || self.is_descendant_of(hid, menu_cid))
        {
            return true;
        }

        if self.effective_layer(hid) >= UiLayer::Popup {
            return true;
        }

        if self
            .find_ancestor_with_role(hid, WidgetRole::DropdownPopup)
            .is_some()
        {
            return true;
        }

        let Some(node) = self.get(hid) else {
            return false;
        };

        if matches!(
            node.role,
            WidgetRole::DropdownPopup | WidgetRole::DropdownItem | WidgetRole::Tooltip
        ) {
            return true;
        }

        let hit_tag = if node.tag != 0 {
            node.tag
        } else {
            self.resolve_ancestor_tag(hid)
        };

        let is_active_menu = self.active_menu_bar_tag.is_some_and(|t| t == hit_tag);
        let is_active_combo = self.active_combobox_tag.is_some_and(|t| t == hit_tag);
        let is_active_submenu = hit_tag != 0
            && (self.active_submenus.contains_key(&hit_tag)
                || self.active_submenus.values().any(|&v| v == hit_tag));
        let is_active_popup_tag = hit_tag != 0 && self.active_popup_tags.contains(&hit_tag);

        is_active_menu || is_active_combo || is_active_submenu || is_active_popup_tag
    }

    /// Evaluates whether a screen coordinate falls outside currently active popups or context menus.
    ///
    /// When any popup, context menu, or dropdown is currently open and `point` is outside
    /// its interactive boundaries, all open popups are dismissed, pending press state is cleared,
    /// and this method returns `true` (signaling that the outside click event should be swallowed).
    ///
    /// If no popup is open or if `point` falls inside an open popup, returns `false`.
    ///
    /// # Arguments
    /// * `point` - Pointer coordinate where the mouse press occurred.
    ///
    /// # Returns
    /// `true` if an outside click was detected and popups were dismissed; `false` otherwise.
    pub fn outside_click_detector(&mut self, point: Point) -> bool {
        if !self.has_open_popup() && !self.has_active_popup_nodes() {
            return false;
        }

        if self.is_point_inside_popup(point) {
            return false;
        }

        self.close_all_popups();
        self.event_state.clear_pressed();
        true
    }

    /// Evaluates whether a screen coordinate falls outside currently active popups or context menus,
    /// dismissing all open popups if an outside click occurred.
    ///
    /// This method enforces declarative popup layer and semantic tag protection. When any popup,
    /// elevated context menu, combobox dropdown, cascading menu, or registered popup tag is active
    /// and `point` is strictly outside its interactive boundaries, all open popups are dismissed,
    /// pending press state is cleared, and this method returns `true` (signaling that the click was consumed by popup dismissal).
    ///
    /// If no popup is open, or if `point` lies inside an active popup or its anchor trigger tag, returns `false`.
    ///
    /// # Arguments
    /// * `point` - Pointer coordinate where the mouse press occurred.
    ///
    /// # Returns
    /// `true` if an outside click was detected and popups were dismissed; `false` otherwise.
    #[inline]
    pub fn dismiss_popups_on_outside_click(&mut self, point: Point) -> bool {
        self.outside_click_detector(point)
    }

    /// Returns `true` if an autonomous tooltip is currently open and actively visible.
    #[inline]
    #[must_use]
    pub fn has_open_tooltip(&self) -> bool {
        self.event_state.has_open_tooltip()
    }

    /// Closes any active or pending tooltip display.
    #[inline]
    pub fn close_tooltip(&mut self) {
        self.event_state.close_tooltip();
    }

    /// Checks if a tooltip is currently open for the given widget identifier or semantic tag,
    /// promoting pending hover states to active if the delay threshold has elapsed.
    #[inline]
    pub fn is_tooltip_open_for(&mut self, target_id: WidgetId, target_tag: u64) -> bool {
        if let Some(ref mut tooltip) = self.event_state.active_tooltip {
            let is_match = tooltip.target_id == target_id
                || (target_tag != 0 && tooltip.target_tag == target_tag);
            if is_match {
                if !tooltip.is_open
                    && std::time::Instant::now().duration_since(tooltip.hover_start)
                        >= tooltip.delay
                {
                    tooltip.is_open = true;
                }
                return tooltip.is_open;
            }
        }
        false
    }

    /// Checks active hover timers against the current monotonic timestamp, promoting
    /// pending tooltips to open state if their delay duration has elapsed.
    ///
    /// Returns `true` if a pending tooltip transitioned from closed to open, indicating
    /// that an immediate visual redraw should occur.
    #[inline]
    pub fn check_hover_timers(&mut self, now: std::time::Instant) -> bool {
        if let Some(ref mut tooltip) = self.event_state.active_tooltip
            && !tooltip.is_open
            && now.duration_since(tooltip.hover_start) >= tooltip.delay
        {
            tooltip.is_open = true;
            return true;
        }
        false
    }

    /// Returns whether the submenu identified by semantic `submenu_tag` branching from `parent_tag` is currently opened.
    #[inline]
    #[must_use]
    pub fn is_submenu_open(&self, parent_tag: u64, submenu_tag: u64) -> bool {
        self.active_submenus.get(&parent_tag) == Some(&submenu_tag)
    }

    /// Sets the active submenu identified by `submenu_tag` branching from the given `parent_tag`.
    #[inline]
    pub fn set_active_submenu(&mut self, parent_tag: u64, submenu_tag: u64) {
        self.active_submenus.insert(parent_tag, submenu_tag);
    }

    /// Closes any opened submenu branching from `parent_tag`.
    #[inline]
    pub fn close_submenu(&mut self, parent_tag: u64) {
        self.active_submenus.remove(&parent_tag);
    }

    /// Closes all active cascading submenus across the entire tree.
    #[inline]
    pub fn clear_all_submenus(&mut self) {
        self.active_submenus.clear();
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

    /// Evaluates whether an active text or numeric input widget currently possesses keyboard focus.
    ///
    /// Checks autonomous editing buffers ([`crate::dispatch::EventState::has_text_input_focus`])
    /// and whether the currently focused node possesses a [`WidgetRole::TextInput`] role.
    /// Note: [`WidgetRole::NumericInput`] scrubbers only capture keyboard focus when actively
    /// running an inline text editing session ([`crate::dispatch::EventState::has_text_input_focus`]);
    /// pointer dragging or hovering on numeric fields must never swallow global shortcuts.
    #[must_use]
    pub fn is_text_input_focused(&self) -> bool {
        if self.event_state.has_text_input_focus() {
            return true;
        }
        if let Some(id) = self.event_state.focused
            && let Some(node) = self.nodes.get(id)
        {
            return node.role == WidgetRole::TextInput;
        }
        if let Some(tag) = self.event_state.focused_tag
            && let Some(id) = self.find_node_by_tag(tag)
            && let Some(node) = self.nodes.get(id)
        {
            return node.role == WidgetRole::TextInput;
        }
        false
    }

    /// Checks if the specified 64-bit semantic tag currently holds keyboard focus or an active text edit session.
    ///
    /// Evaluates active inline text/numeric buffers, explicit focused tags, and tagged node IDs.
    #[must_use]
    #[inline]
    pub fn is_tag_focused(&self, tag: u64) -> bool {
        if tag == 0 {
            return false;
        }
        if self
            .event_state
            .active_text_edit
            .as_ref()
            .is_some_and(|s| s.tag == tag)
        {
            return true;
        }
        if self.event_state.focused_tag == Some(tag) {
            return true;
        }
        self.event_state
            .focused
            .and_then(|id| self.get(id))
            .is_some_and(|n| n.tag == tag)
    }

    /// Automatically synchronizes keyboard focus with the active text editing state,
    /// dropping focus from the provided [`FocusManager`] when text editing concludes.
    ///
    /// Returns `true` if active focus was cleared during synchronization.
    #[inline]
    pub fn auto_sync_focus_state(&self, focus: &mut FocusManager) -> bool {
        focus.auto_sync_focus_state(self)
    }

    /// Dispatches an incoming [`UiEvent`] windowing input through the UI graph and
    /// automatically synchronizes the provided [`FocusManager`] with the resulting text edit state.
    #[inline]
    pub fn dispatch_event_with_focus(
        &mut self,
        event: &UiEvent,
        focus: &mut FocusManager,
    ) -> DispatchResult {
        let result = self.dispatch_event(event);
        focus.auto_sync_focus_state(self);
        result
    }

    /// Evaluates whether the scroll container associated with the specified semantic tag was scrolled in the current frame.
    ///
    /// Checks [`crate::dispatch::EventState::frame_scrolled_tag`] matching `tag`.
    #[inline]
    #[must_use]
    pub fn is_tag_scrolled(&self, tag: u64) -> bool {
        tag != 0 && self.event_state.frame_scrolled_tag == Some(tag)
    }

    /// Returns the mouse wheel or scrollbar displacement delta `(delta_x, delta_y)` for the specified container tag, if scrolled.
    ///
    /// Returns `None` if the container was not scrolled during the current dispatch cycle.
    #[inline]
    #[must_use]
    pub fn tag_scroll_delta(&self, tag: u64) -> Option<(f32, f32)> {
        if self.is_tag_scrolled(tag) {
            self.event_state.frame_scroll_delta
        } else {
            None
        }
    }

    /// Evaluates whether an upward scroll gesture (`delta_y > 0.0`) occurred on the specified tag during the current frame.
    ///
    /// In typical mouse wheel scrolling, an upward wheel roll produces positive `delta_y`,
    /// moving viewport content towards the top (earlier entries) and decreasing scroll offset.
    #[inline]
    #[must_use]
    pub fn is_tag_scrolled_up(&self, tag: u64) -> bool {
        self.tag_scroll_delta(tag).is_some_and(|(_, dy)| dy > 0.0)
    }

    /// Evaluates whether a downward scroll gesture (`delta_y < 0.0`) occurred on the specified tag during the current frame.
    ///
    /// In typical mouse wheel scrolling, a downward wheel roll produces negative `delta_y`,
    /// moving viewport content towards the bottom (later entries) and increasing scroll offset.
    #[inline]
    #[must_use]
    pub fn is_tag_scrolled_down(&self, tag: u64) -> bool {
        self.tag_scroll_delta(tag).is_some_and(|(_, dy)| dy < 0.0)
    }
}