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
                self.cursor_pos = Point::new(position.x as f32, position.y as f32);

                // Desktop-standard behavior: when any dropdown menu is active,
                // hovering over other menu headers automatically switches the open menu.
                if self.active_menu.is_some() && self.cursor_pos.y <= Self::MENUBAR_HEIGHT {
                    if self.cursor_pos.x >= 6.0 && self.cursor_pos.x < 44.0 {
                        self.active_menu = Some(ActiveMenu::File);
                    } else if self.cursor_pos.x >= 44.0 && self.cursor_pos.x < 84.0 {
                        self.active_menu = Some(ActiveMenu::Edit);
                    } else if self.cursor_pos.x >= 84.0 && self.cursor_pos.x < 126.0 {
                        self.active_menu = Some(ActiveMenu::View);
                    } else if self.cursor_pos.x >= 126.0 && self.cursor_pos.x < 186.0 {
                        self.active_menu = Some(ActiveMenu::Window);
                    } else if self.cursor_pos.x >= 186.0 && self.cursor_pos.x < 226.0 {
                        self.active_menu = Some(ActiveMenu::Help);
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } => {
                let click_point = self.cursor_pos;

                // 1. Check if clicking on menubar buttons
                if click_point.y <= Self::MENUBAR_HEIGHT {
                    result.consumed = true;
                    self.hierarchy_is_add_menu_open = false;
                    self.hierarchy_active_submenu = None;
                    self.hierarchy_active_context_menu = None;
                    self.viewport_hud_dropdown = None;
                    self.preferences_dropdown = None;

                    if click_point.x >= 6.0 && click_point.x < 44.0 {
                        self.active_menu = if self.active_menu == Some(ActiveMenu::File) {
                            None
                        } else {
                            Some(ActiveMenu::File)
                        };
                        return Some(result);
                    }

                    if click_point.x >= 44.0 && click_point.x < 84.0 {
                        self.active_menu = if self.active_menu == Some(ActiveMenu::Edit) {
                            None
                        } else {
                            Some(ActiveMenu::Edit)
                        };
                        return Some(result);
                    }

                    if click_point.x >= 84.0 && click_point.x < 126.0 {
                        self.active_menu = if self.active_menu == Some(ActiveMenu::View) {
                            None
                        } else {
                            Some(ActiveMenu::View)
                        };
                        return Some(result);
                    }

                    if click_point.x >= 126.0 && click_point.x < 186.0 {
                        self.active_menu = if self.active_menu == Some(ActiveMenu::Window) {
                            None
                        } else {
                            Some(ActiveMenu::Window)
                        };
                        return Some(result);
                    }

                    if click_point.x >= 186.0 && click_point.x < 226.0 {
                        self.active_menu = if self.active_menu == Some(ActiveMenu::Help) {
                            None
                        } else {
                            Some(ActiveMenu::Help)
                        };
                        return Some(result);
                    }

                    if click_point.x >= (self.screen_width - 90.0) {
                        self.active_menu = None;
                        result.ui_action = Some(crate::ui::EngineUiAction::ChangeMode(
                            ae_core::modules::EngineMode::Play,
                        ));
                        return Some(result);
                    }

                    // Clicked menubar empty area -> close dropdown
                    self.active_menu = None;
                    return Some(result);
                }

                // 2. Check if clicking on an item inside the active dropdown popup
                if self.active_menu.is_some() {
                    let mut clicked_item = None;

                    for (item_rect, action) in &self.dropdown_items {
                        if item_rect.contains_point(click_point) {
                            clicked_item = Some(action.clone());
                            break;
                        }
                    }

                    if let Some(action) = clicked_item {
                        match action {
                            DropdownAction::UiAction(act) => result.ui_action = Some(act),
                            DropdownAction::TogglePanel(p) => result.toggle_panel = Some(p),
                            DropdownAction::ResetLayout => result.reset_layout = true,
                            DropdownAction::OpenPreferences => result.open_preferences = true,
                            DropdownAction::OpenAbout => result.open_about = true,
                        }
                        self.active_menu = None;
                        result.consumed = true;
                        return Some(result);
                    }

                    // Clicked outside dropdown -> dismiss popup
                    self.active_menu = None;
                    result.consumed = true;
                    return Some(result);
                }
            }
            _ => {}
        }

        None
    }
}