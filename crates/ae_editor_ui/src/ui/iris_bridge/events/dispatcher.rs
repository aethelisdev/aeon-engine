// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Top-level event orchestration and interaction routing for Iris UI editor overlays.

use crate::ui::iris_bridge::assets::{
    ASSETS_TAG_CONTENT_VIEWPORT, ASSETS_TAG_SEARCH_CLEAR, ASSETS_TAG_SEARCH_INPUT,
    ASSETS_TAG_TREE_VIEWPORT,
};
use crate::ui::iris_bridge::compositor;
use crate::ui::iris_bridge::console::{
    CONSOLE_TAG_SEARCH_CLEAR, CONSOLE_TAG_SEARCH_INPUT, CONSOLE_TAG_VIEWPORT,
};
use crate::ui::iris_bridge::hierarchy::{HIERARCHY_TAG_SEARCH_CLEAR, HIERARCHY_TAG_SEARCH_INPUT};
use crate::ui::iris_bridge::stats::STATS_TAG_VIEWPORT;
use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use crate::ui::iris_bridge::{ActiveOverlay, ModalKind};
use crate::ui::panel_layout::PanelId;
use irisui::prelude::*;
use winit::event::WindowEvent;

impl IrisEditorOverlay {
    /// Intercepts and processes window mouse input and cursor movement events across all active Iris UI subsystems.
    ///
    /// Verifies both returned result flags and pending panel actions enqueued in [`PanelInteractionState`]
    /// to trigger reactive UI tree invalidation with zero lost frames.
    pub fn handle_event(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
    ) -> IrisOverlayEventResult {
        // Track real-time cursor position for accurate dispatch hit-testing
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.chrome.cursor_pos = Point::new(position.x as f32, position.y as f32);
        }
        let current_cursor = self.chrome.cursor_pos;

        // 0. Autonomous Iris UI hierarchical tree dispatch (Phase 4 Foundation)
        // Dispatches events through the retained tree hierarchy (hover, reactive clicks, scroll bubbling).
        let tree_consumed =
            if let Some(ui_event) = irisui::winit::winit_to_ui_event(event, current_cursor) {
                let tree_dispatch = self.tree.dispatch_event(&ui_event);
                if tree_dispatch.is_consumed() {
                    if let Some(target_id) = tree_dispatch.target {
                        let scroll_y = self
                            .tree
                            .get(target_id)
                            .map_or(0.0, |n| n.style.scroll_offset_y);
                        let tag = tree_dispatch.target_tag;
                        if tag == CONSOLE_TAG_VIEWPORT {
                            self.console.scroll_y = scroll_y;
                            self.console.interactions.scroll_y = scroll_y;
                        } else if tag == ASSETS_TAG_CONTENT_VIEWPORT {
                            self.assets.scroll_y = scroll_y;
                            self.assets.interactions.scroll_y = scroll_y;
                        } else if tag == ASSETS_TAG_TREE_VIEWPORT {
                            self.assets.tree_scroll_y = scroll_y;
                        } else if tag == STATS_TAG_VIEWPORT {
                            self.stats.scroll_y = scroll_y;
                        }
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            };

        let mut result = self.dispatch_window_event_internal(event, world);
        if tree_consumed {
            result.consumed = true;
        }

        // Reactive Event Invalidation: If any UI element consumed this event, produced
        // an action result, or queued typed actions in its PanelInteractionState, immediately
        // flag the UI tree as dirty so the next frame reflects the change instantly, even
        // if the mouse cursor does not move a single pixel.
        if result.consumed
            || self.has_pending_panel_actions()
            || result.preferences_action.is_some()
            || result.ui_action.is_some()
            || result.toggle_panel.is_some()
            || result.open_preferences
            || result.close_preferences
            || result.open_about
            || result.close_about
            || result.confirm_delete
            || result.cancel_delete
            || result.create_folder.is_some()
            || result.apply_rename.is_some()
            || result.reset_layout
        {
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
        }
        result
    }

    /// Returns `true` if any docked or modal panel has pending queued typed actions in its [`PanelInteractionState`].
    ///
    /// Checks all standardized panel action queues in O(1) without heap allocation or collection draining.
    #[inline]
    #[must_use]
    pub fn has_pending_panel_actions(&self) -> bool {
        self.hierarchy.interactions.has_pending_actions()
            || self.stats.interactions.has_pending_actions()
            || self.console.interactions.has_pending_actions()
            || self.material.interactions.has_pending_actions()
            || self.assets.interactions.has_pending_actions()
            || self.preferences.interactions.has_pending_actions()
            || self.inspector.interactions.has_pending_actions()
            || self.timeline.interactions.has_pending_actions()
            || self.ui_designer.interactions.has_pending_actions()
            || !self.viewport_hud.actions.is_empty()
    }

    /// Internal routing pipeline that tests UI layers in strict Z-order.
    fn dispatch_window_event_internal(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
    ) -> IrisOverlayEventResult {
        let mut result = IrisOverlayEventResult::default();

        // 1. Track modifier keys for accelerated / fine-tune dragging
        if let WindowEvent::ModifiersChanged(modifiers) = event {
            self.chrome.shift_held = modifiers.state().shift_key();
            self.chrome.alt_held = modifiers.state().alt_key();
            self.chrome.ctrl_held = modifiers.state().control_key();
        }

        // 2. Real-time cursor position tracking
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.chrome.cursor_pos = Point::new(position.x as f32, position.y as f32);
        }

        // 3. Continuous global dragging interactions (Inspector scrubbers, Preferences dialog/sliders, Asset drags)
        if let Some(drag_res) = self.dispatch_continuous_drag_events(event) {
            return drag_res;
        }

        // 4. Loading Splash Screen (blocks all underlying interactions)
        if self.modals.is_loading_active {
            result.consumed = true;
            return result;
        }

        // 5. LAYER 1: Top-Level Overlay / Portal Layer (OverlayTree)
        // Menubar dropdowns, context menus, modals, color picker popups.
        if self.overlay_tree.is_open() {
            let cursor = self.cursor_pos();

            // Modal dialogs capture all keyboard and IME input regardless of cursor coordinates
            if matches!(
                event,
                WindowEvent::KeyboardInput { .. } | WindowEvent::Ime(_)
            ) {
                match self.overlay_tree.active_overlay() {
                    Some(ActiveOverlay::Modal(ModalKind::Preferences)) => {
                        if let Some(pref_res) = self.handle_preferences_event(event) {
                            return pref_res;
                        }
                    }
                    Some(ActiveOverlay::Modal(_)) => {
                        if let Some(modal_res) = self.handle_modal_events(event) {
                            return modal_res;
                        }
                    }
                    _ => {}
                }
            }

            if self.overlay_tree.contains_point(cursor) {
                // Event point is inside active overlay -> dispatch specifically to active overlay handler
                match self.overlay_tree.active_overlay() {
                    Some(ActiveOverlay::MenubarDropdown(_)) => {
                        if let Some(mb_res) = self.handle_menubar_event(event) {
                            return mb_res;
                        }
                    }
                    Some(ActiveOverlay::Modal(ModalKind::Preferences)) => {
                        if let Some(pref_res) = self.handle_preferences_event(event) {
                            return pref_res;
                        }
                    }
                    Some(ActiveOverlay::Modal(_)) => {
                        if let Some(modal_res) = self.handle_modal_events(event) {
                            return modal_res;
                        }
                    }
                    Some(ActiveOverlay::HierarchyContextMenu) => {
                        if let Some(hier_res) = self.handle_hierarchy_window_event(event) {
                            return hier_res;
                        }
                    }
                    Some(
                        ActiveOverlay::InspectorAddComponent
                        | ActiveOverlay::InspectorDropdown
                        | ActiveOverlay::InspectorColorPicker,
                    ) => {
                        if let Some(insp_res) = self.handle_inspector_window_event(event, world) {
                            return insp_res;
                        }
                    }
                    Some(
                        ActiveOverlay::UiDesignerAddElement | ActiveOverlay::UiDesignerAspectRatio,
                    ) => {
                        if let Some(ui_res) = self.handle_ui_designer_window_event(event) {
                            return ui_res;
                        }
                    }
                    _ => {
                        if let Some(popup_res) = self.dispatch_floating_popup_events(event, world) {
                            return popup_res;
                        }
                        if let Some(modal_res) = self.handle_modal_events(event) {
                            return modal_res;
                        }
                        if let Some(pref_res) = self.handle_preferences_event(event) {
                            return pref_res;
                        }
                        if let Some(mb_res) = self.handle_menubar_event(event) {
                            return mb_res;
                        }
                    }
                }

                // If pointer input was inside overlay body, consume to prevent click-through
                if matches!(event, WindowEvent::MouseInput { .. }) {
                    result.consumed = true;
                    return result;
                }
            } else if let WindowEvent::CursorMoved { .. } = event {
                // Desktop standard: hovering over menubar buttons while a dropdown is open switches active menu
                if cursor.y <= Self::MENUBAR_HEIGHT
                    && self.menubar.active_menu.is_some()
                    && let Some(mb_res) = self.handle_menubar_event(event)
                {
                    return mb_res;
                }
            } else if let WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                ..
            } = event
            {
                // Pointer click is OUTSIDE the active overlay -> evaluate dismiss policy
                let dismiss_res = self.overlay_tree.handle_pointer_down(cursor);
                if dismiss_res == crate::ui::iris_bridge::OverlayDismissResult::Dismissed {
                    self.dismiss_active_overlays();
                    self.chrome.needs_layout_rebuild = true;
                    self.notifier.tag_all();
                    // Clicking on the menubar strip opens the newly clicked menu directly
                    if cursor.y <= Self::MENUBAR_HEIGHT
                        && let Some(mb_res) = self.handle_menubar_event(event)
                    {
                        return mb_res;
                    }
                    result.consumed = true;
                    return result; // Consumed: click-outside closes overlay without triggering click-through!
                } else if dismiss_res == crate::ui::iris_bridge::OverlayDismissResult::Consumed
                    || matches!(
                        self.overlay_tree.active_overlay(),
                        Some(ActiveOverlay::Modal(m)) if *m != ModalKind::Preferences
                    )
                {
                    result.consumed = true;
                    return result; // Modal dialog explicitly blocks click-outside interactions!
                }
            }
        } else if self.assets.context_menu.is_some()
            && let WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                ..
            } = event
        {
            self.assets.context_menu = None;
            self.chrome.needs_layout_rebuild = true;
            self.notifier.tag_all();
            result.consumed = true;
            return result;
        }

        // Overlay & context menu cancellation via Escape key
        if let WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } = event
            && (self.overlay_tree.is_open() || self.assets.context_menu.is_some())
        {
            self.dismiss_active_overlays();
            self.chrome.needs_layout_rebuild = true;
            self.notifier.tag_all();
            return IrisOverlayEventResult {
                consumed: true,
                ..Default::default()
            };
        }

        // 5b. Search input focus cancellation via Escape key (Single Source of Truth)
        if let WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } = event
            && self.focus_state.has_focus()
        {
            self.clear_focus();
            self.chrome.needs_layout_rebuild = true;
            self.notifier.tag_all();
            return IrisOverlayEventResult {
                consumed: true,
                ..Default::default()
            };
        }

        // 5c. Autonomous focus clearing on outside click (clear_on_outside_click)
        if let WindowEvent::MouseInput {
            state: winit::event::ElementState::Pressed,
            button: winit::event::MouseButton::Left,
            ..
        } = event
            && self.focus_state.has_focus()
        {
            let cursor = self.cursor_pos();
            let hit = self.hit_test_any(cursor);
            let hit_tag = hit.as_ref().map(|h| h.tag);
            let hit_id = hit.as_ref().map(|h| h.id);
            let is_hit_focused = match (self.focus_state.focused, hit_tag) {
                (Some(FocusTarget::Tag(focused_tag)), Some(tag)) => {
                    tag == focused_tag
                        || (focused_tag == HIERARCHY_TAG_SEARCH_INPUT
                            && tag == HIERARCHY_TAG_SEARCH_CLEAR)
                        || (focused_tag == ASSETS_TAG_SEARCH_INPUT
                            && tag == ASSETS_TAG_SEARCH_CLEAR)
                        || (focused_tag == CONSOLE_TAG_SEARCH_INPUT
                            && tag == CONSOLE_TAG_SEARCH_CLEAR)
                }
                (Some(FocusTarget::Widget(focused_id)), _) => hit_id == Some(focused_id),
                _ => false,
            };
            if !is_hit_focused {
                self.clear_focus();
                self.chrome.needs_layout_rebuild = true;
                self.notifier.tag_all();
            }
        }

        // 6. Top Menubar header bar (when cursor is over the header strip)
        if self.cursor_pos().y <= Self::MENUBAR_HEIGHT
            && let Some(mb_res) = self.handle_menubar_event(event)
        {
            return mb_res;
        }

        // 7. LAYER 2: Floating Windows Layer
        // If cursor is over any floating window, dispatch using local coordinates (cursor - float_rect.min)
        // and block mouse clicks/hovers from falling through to docked panels underneath.
        let cursor = self.cursor_pos();
        const FLOAT_GRAB_OUTSET: f32 = 4.0;
        let floating_rect_opt = self
            .chrome
            .floating_window_rects
            .iter()
            .rev()
            .find(|r| {
                cursor.x >= r.x - FLOAT_GRAB_OUTSET
                    && cursor.x <= r.right() + FLOAT_GRAB_OUTSET
                    && cursor.y >= r.y - FLOAT_GRAB_OUTSET
                    && cursor.y <= r.bottom() + FLOAT_GRAB_OUTSET
            })
            .copied();

        if let Some(float_rect) = floating_rect_opt {
            let is_in_title_bar = cursor.y < float_rect.y + 26.0;
            let is_on_resize_edge = irisui::dock::detect_resize_edge(
                float_rect,
                cursor,
                irisui::dock::DEFAULT_RESIZE_MARGIN,
            )
            .is_some();

            if is_in_title_bar || is_on_resize_edge {
                // Event targets the floating window title or resize boundary.
                // Prevent falling through to docked panels or HUD, delegating directly to autonomous dock handling.
                return result;
            }

            if let Some(floating_res) =
                self.dispatch_floating_layer_events(event, world, float_rect)
            {
                return floating_res;
            }
            // Pointer press over floating window content background: consumed to prevent click-through
            if matches!(
                event,
                WindowEvent::MouseInput {
                    state: winit::event::ElementState::Pressed,
                    ..
                }
            ) {
                result.consumed = true;
                return result;
            }
            // Pointer is over floating window interior body: block fall-through to underlying docked panels
            return result;
        }

        // 8. Viewport HUD Controls (Interactive on docked or floating viewports)
        if let Some(hud_res) = self.handle_viewport_hud_window_event(event) {
            return hud_res;
        }

        // 9. LAYER 3: Docked Panels Layer
        // Dispatches to docked panels using local coordinates (cursor - panel_rect.min).
        if let Some(panel_res) = self.dispatch_docked_panel_events(event, world) {
            return panel_res;
        }

        result
    }

    /// Dispatches continuous window-wide dragging and release events (Inspector scrubbers, Preferences, Assets).
    fn dispatch_continuous_drag_events(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        // Inspector active dragging / scrubber motion & release (runs globally)
        if let Some(insp_drag_res) = self.handle_inspector_drag_events(event) {
            return Some(insp_drag_res);
        }

        // Active Preferences Drag Interaction (Window dragging or slider dragging)
        if let Some(pref_drag_res) = self.handle_preferences_drag_events(event) {
            return Some(pref_drag_res);
        }

        None
    }

    /// Dispatches events to active floating popups in [`UiLayer::Popup`].
    fn dispatch_floating_popup_events(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
    ) -> Option<IrisOverlayEventResult> {
        if let Some(hud_res) = self.handle_viewport_hud_window_event(event) {
            return Some(hud_res);
        }
        if let Some(hier_res) = self.handle_hierarchy_window_event(event) {
            return Some(hier_res);
        }
        if let Some(insp_res) = self.handle_inspector_window_event(event, world) {
            return Some(insp_res);
        }
        if let Some(ui_res) = self.handle_ui_designer_window_event(event) {
            return Some(ui_res);
        }
        None
    }

    /// Dispatches events targeted at active floating windows and their contained panels,
    /// converting the global mouse cursor into local panel coordinates.
    fn dispatch_floating_layer_events(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
        float_rect: Rect,
    ) -> Option<IrisOverlayEventResult> {
        let cursor = self.cursor_pos();
        let _local_cursor = compositor::screen_to_panel_local(cursor, float_rect);

        // Try panel handlers that might be hosted inside this floating container
        if let Some(insp_res) = self.handle_inspector_window_event(event, world) {
            return Some(insp_res);
        }
        if let Some(hier_res) = self.handle_hierarchy_window_event(event) {
            return Some(hier_res);
        }
        if let Some(mat_res) = self.handle_material_window_event(event) {
            return Some(mat_res);
        }
        if let Some(ui_res) = self.handle_ui_designer_window_event(event) {
            return Some(ui_res);
        }

        None
    }

    /// Dispatches events across docked panels adhering to their typed action models.
    fn dispatch_docked_panel_events(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
    ) -> Option<IrisOverlayEventResult> {
        let cursor = self.cursor_pos();
        let is_pointer = matches!(
            event,
            WindowEvent::MouseInput { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseWheel { .. }
        );
        let captured = self.chrome.pointer_capture;

        let is_in = |panels: &PanelRegistry, id: PanelId| -> bool {
            if let Some(cap) = captured {
                return cap == id;
            }
            panels
                .get(id.id_str())
                .and_then(|p| p.bounds())
                .is_some_and(|r| compositor::is_point_in_panel(cursor, r))
        };

        let mut res = None;

        if (!is_pointer || is_in(&self.panels, PanelId::Hierarchy))
            && let Some(hier_res) = self.handle_hierarchy_window_event(event)
        {
            res = Some(hier_res);
        } else if (!is_pointer || is_in(&self.panels, PanelId::AnimationTimeline))
            && let Some(timeline_res) = self.handle_timeline_window_event(event)
        {
            res = Some(timeline_res);
        } else if (!is_pointer || is_in(&self.panels, PanelId::MaterialEditor))
            && let Some(mat_res) = self.handle_material_window_event(event)
        {
            res = Some(mat_res);
        } else if (!is_pointer || is_in(&self.panels, PanelId::UiDesigner))
            && let Some(ui_res) = self.handle_ui_designer_window_event(event)
        {
            res = Some(ui_res);
        } else if (!is_pointer || is_in(&self.panels, PanelId::Inspector))
            && let Some(insp_res) = self.handle_inspector_window_event(event, world)
        {
            res = Some(insp_res);
        }

        // On pointer release, clear active pointer capture if the operation finished
        if matches!(
            event,
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Released,
                ..
            }
        ) && self.chrome.pointer_capture.is_some()
        {
            let still_dragging = match self.chrome.pointer_capture {
                Some(PanelId::AnimationTimeline) => self.timeline.is_dragging,
                Some(PanelId::Assets) => self.assets.click_tracker.is_dragging_asset,
                Some(PanelId::Inspector) => self.inspector.active_number_input.is_some(),
                Some(PanelId::MaterialEditor) => self.material.scroll_state.is_dragging(),
                Some(PanelId::Hierarchy) => self.hierarchy.scroll_state.is_dragging(),
                _ => false,
            };
            if !still_dragging {
                self.chrome.pointer_capture = None;
            }
        }

        res
    }

    /// Dismisses all transient floating menus, dropdowns, and context popups on click-outside or escape.
    pub(crate) fn dismiss_active_overlays(&mut self) {
        self.menubar.active_menu = None;
        self.menubar.actions.clear();
        self.menubar.dropdown_rect = None;
        self.assets.context_menu = None;
        self.hierarchy.is_add_menu_open = false;
        self.hierarchy.active_context_menu = None;
        self.inspector.is_color_picker_open = false;
        self.inspector.is_add_menu_open = false;
        self.inspector.active_submenu = None;
        self.inspector.active_dropdown = None;
        self.ui_designer.is_add_menu_open = false;
        self.ui_designer.is_aspect_open = false;
        self.overlay_tree.close();
    }
}