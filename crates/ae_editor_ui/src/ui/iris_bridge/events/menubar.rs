// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event routing logic for the top Menubar and floating dropdown menus.

use super::super::types::{ActiveMenu, DropdownAction, IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles cursor hovering and mouse clicks over the top Menubar and open dropdown popups.
    pub(crate) fn handle_menubar_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let mut result = IrisOverlayEventResult::default();

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.chrome.cursor_pos = Point::new(position.x as f32, position.y as f32);

                // Desktop-standard behavior: when any dropdown menu is active,
                // hovering over other menu headers automatically switches the open menu.
                if self.menubar.active_menu.is_some()
                    && self.cursor_pos().y <= Self::MENUBAR_HEIGHT
                    && let Some(hit) = self.tree.hit_test_target(self.cursor_pos())
                    && hit.role == WidgetRole::MenuBarItem
                    && let Some(new_menu) = ActiveMenu::from_tag(hit.tag)
                    && self.menubar.active_menu != Some(new_menu)
                {
                    self.menubar.active_menu = Some(new_menu);
                    self.chrome.needs_layout_rebuild = true;
                    self.notifier.tag_all();
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } => {
                let click_point = self.cursor_pos();

                if let Some(hit) = self.tree.hit_test_target(click_point) {
                    // 1. Menu item headers (File, Edit, View, Window, Help)
                    if hit.role == WidgetRole::MenuBarItem {
                        result.consumed = true;
                        self.hierarchy.is_add_menu_open = false;
                        self.hierarchy.active_submenu = None;
                        self.hierarchy.active_sub_submenu = None;
                        self.hierarchy.active_context_menu = None;
                        self.viewport_hud.dropdown = None;
                        self.preferences.dropdown = None;

                        if let Some(clicked_menu) = ActiveMenu::from_tag(hit.tag) {
                            self.menubar.active_menu =
                                if self.menubar.active_menu == Some(clicked_menu) {
                                    None
                                } else {
                                    Some(clicked_menu)
                                };
                            self.chrome.needs_layout_rebuild = true;
                            self.notifier.tag_all();
                            return Some(result);
                        }
                    }

                    // 2. Action buttons (Play/Stop toggle)
                    if hit.role == WidgetRole::Button
                        && hit.tag == super::super::menubar::TAG_ACTION_PLAY_PAUSE
                    {
                        result.consumed = true;
                        self.menubar.active_menu = None;
                        result.ui_action = Some(crate::ui::EngineUiAction::ChangeMode(
                            ae_core::modules::EngineMode::Play,
                        ));
                        self.chrome.needs_layout_rebuild = true;
                        self.notifier.tag_all();
                        return Some(result);
                    }

                    // 3. Dropdown items inside open popup (strictly when menubar has an active menu open)
                    if self.menubar.active_menu.is_some()
                        && hit.role == WidgetRole::DropdownItem
                        && hit.layer == UiLayer::Popup
                    {
                        result.consumed = true;
                        self.menubar.active_menu = None;
                        let action_idx = hit.tag as usize;
                        if let Some(action) = self.menubar.actions.get(action_idx) {
                            match action {
                                DropdownAction::UiAction(act) => {
                                    result.ui_action = Some(act.clone())
                                }
                                DropdownAction::TogglePanel(p) => result.toggle_panel = Some(*p),
                                DropdownAction::ResetLayout => result.reset_layout = true,
                                DropdownAction::OpenPreferences => result.open_preferences = true,
                                DropdownAction::OpenAbout => result.open_about = true,
                            }
                        }
                        self.chrome.needs_layout_rebuild = true;
                        self.notifier.tag_all();
                        return Some(result);
                    }
                }

                // If clicking outside dropdown when dropdown is open, dismiss it
                if self.menubar.active_menu.is_some() {
                    self.menubar.active_menu = None;
                    self.chrome.needs_layout_rebuild = true;
                    self.notifier.tag_all();
                    result.consumed = true;
                    return Some(result);
                }
            }
            _ => {}
        }

        None
    }
}