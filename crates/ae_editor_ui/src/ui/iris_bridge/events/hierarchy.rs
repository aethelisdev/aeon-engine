// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interaction and event handling subsystem for the Scene Hierarchy panel overlay.

use crate::ui::iris_bridge::hierarchy::{self, AddSubmenuId, HierarchyAction};
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
                        hierarchy::add_menu::get_hierarchy_add_menu_action(hit.tag)
                    {
                        self.hierarchy.interactions.actions.push(action);
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
                        hierarchy::types::HIERARCHY_CTX_DELETE => {
                            self.hierarchy
                                .interactions
                                .actions
                                .push(HierarchyAction::SelectEntity(Some(target_ent)));
                            self.hierarchy
                                .interactions
                                .actions
                                .push(HierarchyAction::DeleteSelected);
                            self.hierarchy.active_context_menu = None;
                            self.notifier.tag_all();
                            result.consumed = true;
                            return Some(result);
                        }
                        hierarchy::types::HIERARCHY_CTX_VISIBILITY => {
                            self.hierarchy
                                .interactions
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

            let hier_targets = self.hierarchy.interactions.targets.as_ref()?;
            let search_input_rect = hier_targets.search_input_rect;

            let mut actions = Vec::new();
            let consumed = hierarchy::handle_hierarchy_click(
                click_point,
                ui_button,
                hier_targets,
                &mut actions,
            );

            for action in actions {
                match action {
                    HierarchyAction::OpenAddMenu(_pos) => {
                        self.hierarchy.is_add_menu_open = true;
                        self.hierarchy.active_submenu = None;
                        self.hierarchy.active_sub_submenu = None;
                        self.hierarchy.active_context_menu = None;
                        self.menubar.active_menu = None;
                        self.viewport_hud.dropdown = None;
                        self.preferences.dropdown = None;
                        self.notifier.tag_all();
                    }
                    HierarchyAction::CloseAddMenu => {
                        self.hierarchy.is_add_menu_open = false;
                        self.hierarchy.active_submenu = None;
                        self.hierarchy.active_sub_submenu = None;
                        self.notifier.tag_all();
                    }
                    HierarchyAction::OpenSubmenu(sub) => {
                        self.hierarchy.active_submenu = Some(sub);
                        self.hierarchy.active_sub_submenu = None;
                        self.notifier.tag_all();
                    }
                    HierarchyAction::CloseSubmenu => {
                        self.hierarchy.active_submenu = None;
                        self.hierarchy.active_sub_submenu = None;
                        self.notifier.tag_all();
                    }
                    HierarchyAction::OpenSubSubmenu(sub) => {
                        self.hierarchy.active_sub_submenu = Some(sub);
                        self.notifier.tag_all();
                    }
                    HierarchyAction::CloseSubSubmenu => {
                        self.hierarchy.active_sub_submenu = None;
                        self.notifier.tag_all();
                    }
                    HierarchyAction::OpenContextMenu(ent, pos) => {
                        self.hierarchy.active_context_menu = Some((ent, pos));
                        self.hierarchy.is_add_menu_open = false;
                        self.menubar.active_menu = None;
                        self.viewport_hud.dropdown = None;
                        self.preferences.dropdown = None;
                        self.notifier.tag_all();
                    }
                    HierarchyAction::CloseContextMenu => {
                        self.hierarchy.active_context_menu = None;
                        self.notifier.tag_all();
                    }
                    HierarchyAction::ClearSearchQuery => {
                        self.hierarchy.search_query.clear();
                        self.notifier.tag_all();
                    }
                    HierarchyAction::SetSearchQuery(q) => {
                        self.hierarchy.search_query = q;
                        self.notifier.tag_all();
                    }
                    other => {
                        self.hierarchy.interactions.actions.push(other);
                        self.notifier.tag_all();
                        self.chrome.needs_layout_rebuild = true;
                    }
                }
            }

            if search_input_rect.contains_point(click_point) {
                self.hierarchy.is_search_focused = true;
            } else if *button == WinitMouseButton::Left {
                self.hierarchy.is_search_focused = false;
            }

            if consumed {
                result.consumed = true;
                return Some(result);
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
                        if sub_id == AddSubmenuId::HudPresets {
                            if self.hierarchy.active_sub_submenu != Some(sub_id) {
                                self.hierarchy.active_sub_submenu = Some(sub_id);
                                self.notifier.tag_all();
                            }
                        } else if self.hierarchy.active_submenu != Some(sub_id) {
                            self.hierarchy.active_submenu = Some(sub_id);
                            self.hierarchy.active_sub_submenu = None;
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