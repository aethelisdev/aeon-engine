// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Interaction & Event Dispatch Subsystem
//!
//! Evaluates mouse clicks, context menus, search input focus, and submenu cascading
//! for the Scene Hierarchy panel purely via hardware hit-testing ([`UiTree::hit_test_target`])
//! and 64-bit semantic tags without retaining coordinate rectangles.
//!

use crate::ui::iris_bridge::hierarchy::{
    AddSubmenuId, HIERARCHY_TAG_ADD_BUTTON, HIERARCHY_TAG_DELETE_BUTTON, HIERARCHY_TAG_PANEL_ROOT,
    HIERARCHY_TAG_SEARCH_CLEAR, HIERARCHY_TAG_SEARCH_INPUT, HierarchyAction, is_hierarchy_tag,
    parse_eye_tag, parse_foldout_tag, parse_row_tag,
};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::{MouseButton, UiLayer, WidgetRole};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles mouse clicks, context menus, search focus, and submenu cascading for Scene Hierarchy.
    pub(crate) fn handle_hierarchy_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let mut result = IrisOverlayEventResult::default();

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
                && let Some(hit) = self.tree.hit_test_target(click_point)
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
                && let Some(hit) = self.tree.hit_test_target(click_point)
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
            if let Some(hit) = self.tree.hit_test_target(click_point) {
                let effective_tag = resolve_ancestor_tag(&self.tree, hit.id);

                if is_hierarchy_tag(effective_tag) {
                    // Close context menu and add menu on any click inside panel outside menus
                    if self.hierarchy.active_context_menu.is_some() {
                        self.hierarchy.active_context_menu = None;
                        self.notifier.tag_all();
                    }

                    // A. Search Bar Container Focus
                    if effective_tag == HIERARCHY_TAG_SEARCH_INPUT {
                        self.hierarchy.is_search_focused = true;
                        self.notifier.tag_all();
                        result.consumed = true;
                        return Some(result);
                    }

                    // B. Clear Search "✖" Button
                    if effective_tag == HIERARCHY_TAG_SEARCH_CLEAR {
                        self.hierarchy.search_query.clear();
                        self.notifier.tag_all();
                        result.consumed = true;
                        return Some(result);
                    }

                    // C. "➕" Add Entity Button
                    if effective_tag == HIERARCHY_TAG_ADD_BUTTON {
                        self.hierarchy.is_add_menu_open = !self.hierarchy.is_add_menu_open;
                        self.hierarchy.active_submenu = None;
                        self.hierarchy.active_sub_submenu = None;
                        self.notifier.tag_all();
                        result.consumed = true;
                        return Some(result);
                    }

                    // D. "🗑" Delete Selected Button
                    if effective_tag == HIERARCHY_TAG_DELETE_BUTTON {
                        self.hierarchy.actions.push(HierarchyAction::DeleteSelected);
                        self.notifier.tag_all();
                        result.consumed = true;
                        return Some(result);
                    }

                    // E. Eye Visibility Toggle Button
                    if let Some(row_idx) = parse_eye_tag(effective_tag) {
                        if let Some(row) = self.hierarchy.rows_cache.get(row_idx) {
                            self.hierarchy
                                .actions
                                .push(HierarchyAction::ToggleVisibility(row.entity));
                            self.notifier.tag_all();
                            result.consumed = true;
                            return Some(result);
                        }
                    }

                    // E2. Foldout Expand/Collapse Toggle Button
                    if let Some(row_idx) = parse_foldout_tag(effective_tag) {
                        if let Some(row) = self.hierarchy.rows_cache.get(row_idx) {
                            if self.hierarchy.collapsed_entities.contains(&row.entity) {
                                self.hierarchy.collapsed_entities.remove(&row.entity);
                            } else {
                                self.hierarchy.collapsed_entities.insert(row.entity);
                            }
                            self.notifier.tag_all();
                            result.consumed = true;
                            return Some(result);
                        }
                    }

                    // F. Entity Row Selection / Context Menu
                    if let Some(row_idx) = parse_row_tag(effective_tag) {
                        if let Some(row) = self.hierarchy.rows_cache.get(row_idx) {
                            if ui_button == MouseButton::Right {
                                self.hierarchy
                                    .actions
                                    .push(HierarchyAction::SelectEntity(Some(row.entity)));
                                self.hierarchy.active_context_menu =
                                    Some((row.entity, click_point));
                                self.hierarchy.is_add_menu_open = false;
                            } else {
                                self.hierarchy
                                    .actions
                                    .push(HierarchyAction::SelectEntity(Some(row.entity)));
                            }
                            self.notifier.tag_all();
                            result.consumed = true;
                            return Some(result);
                        }
                    }

                    // G. Panel Root or background click
                    if effective_tag == HIERARCHY_TAG_PANEL_ROOT {
                        if *button == WinitMouseButton::Left {
                            self.hierarchy.is_search_focused = false;
                        }
                        if self.hierarchy.is_add_menu_open {
                            self.hierarchy.is_add_menu_open = false;
                            self.notifier.tag_all();
                        }
                        result.consumed = true;
                        return Some(result);
                    }
                }
            }

            // Outside panel click: unfocus search bar and dismiss menus
            if *button == WinitMouseButton::Left {
                if self.hierarchy.is_search_focused {
                    self.hierarchy.is_search_focused = false;
                    self.notifier.tag_all();
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
            if let Some(hit) = self.tree.hit_test_target(cursor)
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

/// Traverses up the widget hierarchy starting from `start_id` to locate the first non-zero semantic tag.
fn resolve_ancestor_tag(
    tree: &irisui::prelude::UiTree,
    start_id: irisui::prelude::WidgetId,
) -> u64 {
    let mut curr = Some(start_id);
    while let Some(id) = curr {
        if let Some(node) = tree.get(id) {
            if node.tag != 0 {
                return node.tag;
            }
            curr = node.parent;
        } else {
            break;
        }
    }
    0
}