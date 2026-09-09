// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interaction and event handling subsystem for the Scene Hierarchy panel overlay.

use crate::ui::iris_bridge::hierarchy::{self, HierarchyAction};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::MouseButton;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles mouse clicks, context menus, search focus, and submenu cascading for Scene Hierarchy.
    pub(crate) fn handle_hierarchy_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let hier_targets = self.hierarchy_targets.as_ref()?;
        let mut result = IrisOverlayEventResult::default();

        // 1. Mouse Input Handling (Entity clicking, context menus, search bar focus)
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button,
            ..
        } = event
        {
            let click_point = self.cursor_pos;
            let ui_button = match button {
                WinitMouseButton::Left => MouseButton::Left,
                WinitMouseButton::Right => MouseButton::Right,
                WinitMouseButton::Middle => MouseButton::Middle,
                _ => MouseButton::Left,
            };

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
                        self.hierarchy_is_add_menu_open = true;
                        self.hierarchy_active_submenu = None;
                        self.hierarchy_active_sub_submenu = None;
                        self.hierarchy_active_context_menu = None;
                        self.active_menu = None;
                        self.viewport_hud_dropdown = None;
                        self.preferences_dropdown = None;
                    }
                    HierarchyAction::CloseAddMenu => {
                        self.hierarchy_is_add_menu_open = false;
                        self.hierarchy_active_submenu = None;
                        self.hierarchy_active_sub_submenu = None;
                    }
                    HierarchyAction::OpenSubmenu(sub) => {
                        self.hierarchy_active_submenu = Some(sub);
                        self.hierarchy_active_sub_submenu = None;
                    }
                    HierarchyAction::CloseSubmenu => {
                        self.hierarchy_active_submenu = None;
                        self.hierarchy_active_sub_submenu = None;
                    }
                    HierarchyAction::OpenSubSubmenu(sub) => {
                        self.hierarchy_active_sub_submenu = Some(sub);
                    }
                    HierarchyAction::CloseSubSubmenu => {
                        self.hierarchy_active_sub_submenu = None;
                    }
                    HierarchyAction::OpenContextMenu(ent, pos) => {
                        self.hierarchy_active_context_menu = Some((ent, pos));
                        self.hierarchy_is_add_menu_open = false;
                        self.active_menu = None;
                        self.viewport_hud_dropdown = None;
                        self.preferences_dropdown = None;
                    }
                    HierarchyAction::CloseContextMenu => {
                        self.hierarchy_active_context_menu = None;
                    }
                    HierarchyAction::ClearSearchQuery => {
                        self.hierarchy_search_query.clear();
                    }
                    HierarchyAction::SetSearchQuery(q) => {
                        self.hierarchy_search_query = q;
                    }
                    other => {
                        self.hierarchy_actions.push(other);
                    }
                }
            }

            if hier_targets.search_input_rect.contains_point(click_point) {
                self.hierarchy_is_search_focused = true;
            } else if *button == WinitMouseButton::Left {
                self.hierarchy_is_search_focused = false;
            }

            if consumed {
                result.consumed = true;
                return Some(result);
            }
        }

        // 2. Cursor Motion Handling (Add Menu hover and cascading submenus)
        if let WindowEvent::CursorMoved { .. } = event
            && self.hierarchy_is_add_menu_open
        {
            let in_sub_sub = hier_targets
                .active_sub_submenu_rect
                .is_some_and(|r| r.contains_point(self.cursor_pos));
            let in_submenu = hier_targets
                .active_submenu_rect
                .is_some_and(|r| r.contains_point(self.cursor_pos));
            let in_add_menu = hier_targets
                .active_add_menu_rect
                .is_some_and(|r| r.contains_point(self.cursor_pos));

            if in_sub_sub {
                // Inside level-3 sub-submenu (e.g. HUD Presets). Keep both open!
            } else if in_submenu {
                // Inside level-2 submenu (e.g. UI & Canvas).
                let mut hovered_branch = None;
                for (branch_rect, sub_id) in &hier_targets.submenu_branch_items {
                    if branch_rect.contains_point(self.cursor_pos) {
                        hovered_branch = Some(*sub_id);
                        break;
                    }
                }
                if let Some(branch_id) = hovered_branch {
                    self.hierarchy_active_sub_submenu = Some(branch_id);
                } else {
                    let hovering_other_item = hier_targets
                        .submenu_items
                        .iter()
                        .any(|(r, _)| r.contains_point(self.cursor_pos));
                    if hovering_other_item {
                        self.hierarchy_active_sub_submenu = None;
                    }
                }
            } else if in_add_menu {
                // Inside level-1 root Add Menu
                for (item_rect, target_payload) in &hier_targets.add_menu_items {
                    if item_rect.contains_point(self.cursor_pos) {
                        if let Ok(submenu_id) = target_payload {
                            self.hierarchy_active_submenu = Some(*submenu_id);
                            self.hierarchy_active_sub_submenu = None;
                        } else {
                            self.hierarchy_active_submenu = None;
                            self.hierarchy_active_sub_submenu = None;
                        }
                        break;
                    }
                }
            }
        }

        None
    }
}