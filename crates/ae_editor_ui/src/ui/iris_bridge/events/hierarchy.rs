// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Interaction & Event Dispatch Subsystem
//!
//! Evaluates mouse clicks, context menus, search input focus, and submenu cascading
//! for the Scene Hierarchy panel purely via hardware hit-testing ([`UiTree::hit_test_target`])
//! and 64-bit semantic tags without retaining coordinate rectangles.
//!

use crate::ui::iris_bridge::hierarchy::{
    AddSubmenuId, HIERARCHY_TAG_SEARCH_INPUT, HierarchyAction, HierarchyTagTarget,
    resolve_hierarchy_tag,
};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::{MouseButton, Rect, ScrollAreaStyle, ScrollBarGeometry, UiLayer, WidgetRole};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles mouse clicks, context menus, search focus, and submenu cascading for Scene Hierarchy.
    pub(crate) fn handle_hierarchy_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let mut result = IrisOverlayEventResult::default();

        // 0a. Scrollbar drag release
        if let WindowEvent::MouseInput {
            state: ElementState::Released,
            button: WinitMouseButton::Left,
            ..
        } = event
            && self.hierarchy.active_scrollbar_drag.is_some()
        {
            self.hierarchy.active_scrollbar_drag = None;
            self.release_pointer_capture();
            self.chrome.needs_scroll_sync = true;
            result.consumed = true;
            return Some(result);
        }

        // 0b. Scrollbar continuous mouse drag
        if let WindowEvent::CursorMoved { .. } = event
            && let Some((start_y, start_scroll)) = self.hierarchy.active_scrollbar_drag
        {
            let cursor = self.cursor_pos();
            let delta_y = cursor.y - start_y;
            let panel_rect = self
                .hierarchy
                .last_rect
                .unwrap_or(Rect::new(0.0, 0.0, 260.0, 400.0));
            let vp_h = (panel_rect.height - 63.0).max(40.0);
            let total_h = vp_h + self.hierarchy.max_scroll;
            let vp_rect = Rect::new(0.0, 31.0, panel_rect.width, vp_h);
            let scroll_style = ScrollAreaStyle::dark_default();
            if let Some(geom) =
                ScrollBarGeometry::compute_vertical(vp_rect, total_h, start_scroll, &scroll_style)
            {
                let delta_scroll = ScrollBarGeometry::scroll_from_thumb_drag(
                    delta_y,
                    geom.track_rect.height,
                    geom.thumb_rect.height,
                    self.hierarchy.max_scroll,
                );
                self.hierarchy.scroll_y =
                    (start_scroll + delta_scroll).clamp(0.0, self.hierarchy.max_scroll);
                self.chrome.needs_scroll_sync = true;
                result.consumed = true;
                return Some(result);
            }
        }

        // 1. Mouse Input Handling (Entity clicking, context menus, search bar focus, add menu)
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button,
            ..
        } = event
        {
            let click_point = self.cursor_pos();
            let ui_button = match button {
                WinitMouseButton::Left => MouseButton::Left,
                WinitMouseButton::Right => MouseButton::Right,
                WinitMouseButton::Middle => MouseButton::Middle,
                _ => MouseButton::Left,
            };

            // 1.1 Direct Hit-Testing on Add Menu Items (Zero-Allocation O(1) Dispatch)
            if self.hierarchy.is_add_menu_open
                && ui_button == MouseButton::Left
                && let Some(hit) = self
                    .overlay_tree
                    .tree()
                    .hit_test_target(click_point)
                    .or_else(|| self.tree.hit_test_target(click_point))
                && hit.layer == UiLayer::Popup
            {
                if hit.role == WidgetRole::DropdownItem {
                    if let Some(sub_id) = AddSubmenuId::from_tag(hit.tag) {
                        if sub_id == AddSubmenuId::HudPresets {
                            self.hierarchy.active_sub_submenu = Some(sub_id);
                        } else {
                            self.hierarchy.active_submenu = Some(sub_id);
                            self.hierarchy.active_sub_submenu = None;
                        }
                        self.notifier.tag_all();
                        result.consumed = true;
                        return Some(result);
                    } else if let Some(action) =
                        crate::ui::iris_bridge::hierarchy::add_menu::get_hierarchy_add_menu_action(
                            hit.tag,
                        )
                    {
                        self.hierarchy.actions.push(action);
                        self.hierarchy.is_add_menu_open = false;
                        self.hierarchy.active_submenu = None;
                        self.hierarchy.active_sub_submenu = None;
                        self.notifier.tag_all();
                        result.consumed = true;
                        return Some(result);
                    }
                }
                // Clicked inside popup container background
                result.consumed = true;
                return Some(result);
            }

            // 1.2 Direct Hit-Testing on Right-Click Entity Context Menu (Zero-Allocation O(1) Dispatch)
            if let Some((target_ent, _)) = self.hierarchy.active_context_menu
                && ui_button == MouseButton::Left
                && let Some(hit) = self
                    .overlay_tree
                    .tree()
                    .hit_test_target(click_point)
                    .or_else(|| self.tree.hit_test_target(click_point))
                && hit.layer == UiLayer::Popup
            {
                if hit.role == WidgetRole::DropdownItem {
                    match hit.tag {
                        crate::ui::iris_bridge::hierarchy::types::HIERARCHY_CTX_DELETE => {
                            self.hierarchy
                                .actions
                                .push(HierarchyAction::SelectEntity(Some(target_ent)));
                            self.hierarchy.actions.push(HierarchyAction::DeleteSelected);
                            self.hierarchy.active_context_menu = None;
                            self.notifier.tag_all();
                            result.consumed = true;
                            return Some(result);
                        }
                        crate::ui::iris_bridge::hierarchy::types::HIERARCHY_CTX_VISIBILITY => {
                            self.hierarchy
                                .actions
                                .push(HierarchyAction::ToggleVisibility(target_ent));
                            self.hierarchy.active_context_menu = None;
                            self.notifier.tag_all();
                            result.consumed = true;
                            return Some(result);
                        }
                        _ => {}
                    }
                }
                // Clicked inside context menu container background
                result.consumed = true;
                return Some(result);
            }

            // 1.3 Direct Hardware Hit-Testing on Scene Hierarchy Panel Widgets
            let hit_opt = self
                .hit_test_panel(crate::ui::panel_layout::PanelId::Hierarchy, click_point)
                .or_else(|| self.tree.hit_test_target(click_point));

            if let Some(hit) = hit_opt {
                let effective_tag = if hit.tag != 0 {
                    hit.tag
                } else {
                    self.resolve_panel_tag(crate::ui::panel_layout::PanelId::Hierarchy, hit.id)
                };

                if let Some(target) = resolve_hierarchy_tag(effective_tag) {
                    // Close context menu and add menu on any click inside panel outside menus
                    if self.hierarchy.active_context_menu.is_some() {
                        self.hierarchy.active_context_menu = None;
                        self.notifier.tag_all();
                    }

                    match target {
                        HierarchyTagTarget::SearchInput => {
                            self.focus_manager.set_focus_tag(HIERARCHY_TAG_SEARCH_INPUT);
                            result.consumed = true;
                            return Some(result);
                        }
                        HierarchyTagTarget::SearchClear => {
                            self.hierarchy.search_input.clear();
                            self.notifier.tag_all();
                            result.consumed = true;
                            return Some(result);
                        }
                        HierarchyTagTarget::AddButton => {
                            self.hierarchy.is_add_menu_open = !self.hierarchy.is_add_menu_open;
                            self.hierarchy.active_submenu = None;
                            self.hierarchy.active_sub_submenu = None;
                            self.notifier.tag_all();
                            result.consumed = true;
                            return Some(result);
                        }
                        HierarchyTagTarget::DeleteButton => {
                            self.hierarchy.actions.push(HierarchyAction::DeleteSelected);
                            self.notifier.tag_all();
                            result.consumed = true;
                            return Some(result);
                        }
                        HierarchyTagTarget::Eye(row_idx) => {
                            if let Some(entity) =
                                self.hierarchy.rows_cache.get(row_idx).map(|r| r.entity)
                            {
                                self.hierarchy
                                    .actions
                                    .push(HierarchyAction::ToggleVisibility(entity));
                                self.notifier.tag_all();
                                result.consumed = true;
                                return Some(result);
                            }
                        }
                        HierarchyTagTarget::Foldout(row_idx) => {
                            if let Some(entity) =
                                self.hierarchy.rows_cache.get(row_idx).map(|r| r.entity)
                            {
                                if self.hierarchy.collapsed_entities.contains(&entity) {
                                    self.hierarchy.collapsed_entities.remove(&entity);
                                } else {
                                    self.hierarchy.collapsed_entities.insert(entity);
                                }
                                self.notifier.tag_all();
                                result.consumed = true;
                                return Some(result);
                            }
                        }
                        HierarchyTagTarget::Row(row_idx) => {
                            if let Some(entity) =
                                self.hierarchy.rows_cache.get(row_idx).map(|r| r.entity)
                            {
                                if ui_button == MouseButton::Right {
                                    self.hierarchy
                                        .actions
                                        .push(HierarchyAction::SelectEntity(Some(entity)));
                                    self.hierarchy.active_context_menu =
                                        Some((entity, click_point));
                                    self.hierarchy.is_add_menu_open = false;
                                } else {
                                    self.hierarchy
                                        .actions
                                        .push(HierarchyAction::SelectEntity(Some(entity)));
                                }
                                self.notifier.tag_all();
                                result.consumed = true;
                                return Some(result);
                            }
                        }
                        HierarchyTagTarget::PanelRoot => {
                            if *button == WinitMouseButton::Left
                                && self
                                    .focus_manager
                                    .is_tag_focused(HIERARCHY_TAG_SEARCH_INPUT)
                            {
                                self.focus_manager.clear_focus();
                            }
                            if self.hierarchy.is_add_menu_open {
                                self.hierarchy.is_add_menu_open = false;
                                self.notifier.tag_all();
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                        HierarchyTagTarget::Viewport => {}
                        HierarchyTagTarget::ScrollbarThumb => {
                            self.hierarchy.active_scrollbar_drag =
                                Some((click_point.y, self.hierarchy.scroll_y));
                            self.set_pointer_capture(crate::ui::panel_layout::PanelId::Hierarchy);
                            result.consumed = true;
                            return Some(result);
                        }
                        HierarchyTagTarget::ScrollbarTrack => {
                            let panel_rect = self
                                .hierarchy
                                .last_rect
                                .unwrap_or(Rect::new(0.0, 0.0, 260.0, 400.0));
                            let vp_h = (panel_rect.height - 63.0).max(40.0);
                            let total_h = vp_h + self.hierarchy.max_scroll;
                            let vp_rect = Rect::new(0.0, 31.0, panel_rect.width, vp_h);
                            let style = ScrollAreaStyle::dark_default();
                            if let Some(geom) = ScrollBarGeometry::compute_vertical(
                                vp_rect,
                                total_h,
                                self.hierarchy.scroll_y,
                                &style,
                            ) {
                                let local_y = if let Some(bounds) = self.hierarchy.last_rect {
                                    crate::ui::iris_bridge::compositor::screen_to_panel_local(
                                        click_point,
                                        bounds,
                                    )
                                    .y
                                } else {
                                    click_point.y
                                };
                                let target_scroll = ScrollBarGeometry::scroll_from_track_click(
                                    local_y,
                                    geom.track_rect.y,
                                    geom.track_rect.height,
                                    geom.thumb_rect.height,
                                    self.hierarchy.max_scroll,
                                );
                                self.hierarchy.scroll_y =
                                    target_scroll.clamp(0.0, self.hierarchy.max_scroll);
                                self.hierarchy.active_scrollbar_drag =
                                    Some((click_point.y, self.hierarchy.scroll_y));
                                self.set_pointer_capture(
                                    crate::ui::panel_layout::PanelId::Hierarchy,
                                );
                                self.chrome.needs_scroll_sync = true;
                                result.consumed = true;
                                return Some(result);
                            }
                        }
                    }
                }
            }

            // Outside panel click: unfocus search bar and dismiss menus
            if *button == WinitMouseButton::Left {
                if self
                    .focus_manager
                    .is_tag_focused(HIERARCHY_TAG_SEARCH_INPUT)
                {
                    self.focus_manager.clear_focus();
                }
                if self.hierarchy.is_add_menu_open {
                    self.hierarchy.is_add_menu_open = false;
                    self.notifier.tag_all();
                }
                if self.hierarchy.active_context_menu.is_some() {
                    self.hierarchy.active_context_menu = None;
                    self.notifier.tag_all();
                }
            }
        }

        // 2. Cursor Motion Handling (Cascading Submenu Hover via UiTree Hit-Testing)
        if let WindowEvent::CursorMoved { .. } = event
            && self.hierarchy.is_add_menu_open
        {
            let cursor = self.cursor_pos();
            if let Some(hit) = self
                .overlay_tree
                .tree()
                .hit_test_target(cursor)
                .or_else(|| self.tree.hit_test_target(cursor))
                && hit.layer == UiLayer::Popup
            {
                if hit.role == WidgetRole::DropdownItem {
                    if let Some(sub_id) = AddSubmenuId::from_tag(hit.tag) {
                        if self.hierarchy.activate_submenu(sub_id) {
                            self.notifier.tag_all();
                        }
                    } else if self.hierarchy.active_sub_submenu.is_some() {
                        // Hovering non-branch item in Level 2: close Level 3 sub-submenu
                        self.hierarchy.active_sub_submenu = None;
                        self.notifier.tag_all();
                    }
                }
                result.consumed = true;
                return Some(result);
            }
        }

        None
    }
}