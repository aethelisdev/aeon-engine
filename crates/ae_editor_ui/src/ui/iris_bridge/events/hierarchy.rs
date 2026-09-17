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
        let mut result = IrisOverlayEventResult::default();

        // 1. Mouse Input Handling (Entity clicking, context menus, search bar focus)
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

        // 2. Cursor Motion Handling (Add Menu hover and cascading submenus)
        if let WindowEvent::CursorMoved { .. } = event
            && self.hierarchy.is_add_menu_open
            && let Some(hier_targets) = self.hierarchy.interactions.targets.as_ref()
        {
            let cursor = self.cursor_pos();
            let in_sub_sub = hier_targets
                .active_sub_submenu_rect
                .is_some_and(|r| r.contains_point(cursor));
            let in_submenu = hier_targets
                .active_submenu_rect
                .is_some_and(|r| r.contains_point(cursor));
            let in_add_menu = hier_targets
                .active_add_menu_rect
                .is_some_and(|r| r.contains_point(cursor));

            if in_sub_sub {
                // Inside level-3 sub-submenu (e.g. HUD Presets). Keep both open!
                result.consumed = true;
                self.notifier.tag_all();
                return Some(result);
            } else if in_submenu {
                // Inside level-2 submenu (e.g. UI & Canvas).
                let mut hovered_branch = None;
                for (branch_rect, sub_id) in &hier_targets.submenu_branch_items {
                    if branch_rect.contains_point(cursor) {
                        hovered_branch = Some(*sub_id);
                        break;
                    }
                }
                if let Some(branch_id) = hovered_branch {
                    if self.hierarchy.active_sub_submenu != Some(branch_id) {
                        self.hierarchy.active_sub_submenu = Some(branch_id);
                        self.notifier.tag_all();
                    }
                } else {
                    let hovering_other_item = hier_targets
                        .submenu_items
                        .iter()
                        .any(|(r, _)| r.contains_point(cursor));
                    if hovering_other_item && self.hierarchy.active_sub_submenu.is_some() {
                        self.hierarchy.active_sub_submenu = None;
                        self.notifier.tag_all();
                    }
                }
                result.consumed = true;
                return Some(result);
            } else if in_add_menu {
                // Inside level-1 root Add Menu
                for (item_rect, target_payload) in &hier_targets.add_menu_items {
                    if item_rect.contains_point(cursor) {
                        if let Ok(submenu_id) = target_payload {
                            if self.hierarchy.active_submenu != Some(*submenu_id) {
                                self.hierarchy.active_submenu = Some(*submenu_id);
                                self.hierarchy.active_sub_submenu = None;
                                self.notifier.tag_all();
                            }
                        } else if self.hierarchy.active_submenu.is_some() {
                            self.hierarchy.active_submenu = None;
                            self.hierarchy.active_sub_submenu = None;
                            self.notifier.tag_all();
                        }
                        break;
                    }
                }
                result.consumed = true;
                return Some(result);
            }
        }

        None
    }
}