// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::workbench::state::EngineUi;
use winit::{event::WindowEvent, window::Window};

impl EngineUi {
    /// Forwards winit window events to egui and Iris UI for input processing.
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let iris_res = self.iris_overlay.handle_event(event);
        if let Some(act) = iris_res.ui_action {
            self.pending_actions.push(act);
        }
        if let Some(panel) = iris_res.toggle_panel {
            self.layout_state.activate_or_open(panel);
        }
        if iris_res.reset_layout {
            self.layout_state.reset_to_default();
        }
        if iris_res.open_preferences {
            self.show_preferences = true;
        }
        if iris_res.close_preferences {
            self.show_preferences = false;
        }
        if let Some(pref_act) = iris_res.preferences_action {
            self.pending_preferences_actions.push(pref_act);
        }
        if iris_res.open_about {
            self.show_about = true;
        }
        if iris_res.close_about {
            self.show_about = false;
        }
        if iris_res.confirm_delete
            && let Some(target) = self.asset_browser.delete_confirmation.take()
        {
            let _ = crate::ui::panels::assets::file_ops::delete_asset_or_folder(&target);
            if self.asset_browser.selected_asset.as_ref() == Some(&target) {
                self.asset_browser.selected_asset = None;
            }
        }
        if iris_res.cancel_delete {
            self.asset_browser.delete_confirmation = None;
        }

        if let Some(folder_name) = iris_res.create_folder
            && let Some(parent) = self.asset_browser.new_folder_parent.take()
        {
            let _ = crate::ui::panels::assets::file_ops::create_subfolder(&parent, &folder_name);
            self.iris_overlay.new_folder_buffer.clear();
            self.asset_browser.new_folder_name.clear();
        }
        if iris_res.cancel_new_folder {
            self.asset_browser.new_folder_parent = None;
            self.iris_overlay.new_folder_buffer.clear();
            self.asset_browser.new_folder_name.clear();
        }

        if let Some(new_name) = iris_res.apply_rename
            && let Some(ren) = self.asset_browser.rename_state.take()
        {
            let _ = crate::ui::panels::assets::file_ops::rename_asset_or_folder(
                &ren.target_path,
                &new_name,
            );
            self.iris_overlay.rename_buffer.clear();
        }
        if iris_res.cancel_rename {
            self.asset_browser.rename_state = None;
            self.iris_overlay.rename_buffer.clear();
        }

        // Hierarchy search bar live typing
        if self.iris_overlay.hierarchy_is_search_focused
            && let WindowEvent::KeyboardInput {
                event: key_event, ..
            } = event
            && key_event.state == winit::event::ElementState::Pressed
        {
            if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) =
                key_event.logical_key
            {
                self.iris_overlay.hierarchy_is_search_focused = false;
                return true;
            }
            if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace) =
                key_event.logical_key
            {
                self.iris_overlay.hierarchy_search_query.pop();
                return true;
            }
            if let Some(text) = &key_event.text {
                for c in text.chars() {
                    if !c.is_control() {
                        self.iris_overlay.hierarchy_search_query.push(c);
                    }
                }
                return true;
            }
        }

        // Always pass event to egui state so pointer and drag states never get starved or desynchronized
        let response = self.state.on_window_event(window, event);

        if iris_res.consumed {
            let mut is_hovering_interactive = false;
            let p = self.iris_overlay.cursor_pos;

            if let Some(ref targets) = self.iris_overlay.hierarchy_targets
                && (targets.add_btn_rect.contains_point(p)
                    || targets.delete_btn_rect.is_some_and(|r| r.contains_point(p))
                    || targets
                        .search_clear_btn_rect
                        .is_some_and(|r| r.contains_point(p))
                    || targets
                        .entity_rows
                        .iter()
                        .any(|(_, r, eye, _)| r.contains_point(p) || eye.contains_point(p))
                    || targets
                        .add_menu_items
                        .iter()
                        .any(|(r, _)| r.contains_point(p))
                    || targets
                        .submenu_items
                        .iter()
                        .any(|(r, _)| r.contains_point(p))
                    || targets.active_context_menu.is_some_and(|(_, _, del, vis)| {
                        del.contains_point(p) || vis.contains_point(p)
                    }))
            {
                is_hovering_interactive = true;
            }

            if let Some(ref targets) = self.iris_overlay.about_targets
                && (targets.header_close_rect.contains_point(p)
                    || targets.bottom_close_rect.contains_point(p)
                    || targets.link_rect.contains_point(p))
            {
                is_hovering_interactive = true;
            }
            if let Some(ref targets) = self.iris_overlay.preferences_targets
                && (targets.close_button.contains_point(p)
                    || targets.tabs.iter().any(|(_, r)| r.contains_point(p))
                    || targets.toggles.iter().any(|(_, r)| r.contains_point(p))
                    || targets
                        .sliders
                        .iter()
                        .any(|(_, r, _, _, _)| r.contains_point(p))
                    || targets.dropdowns.iter().any(|(_, r)| r.contains_point(p))
                    || targets
                        .active_dropdown_items
                        .iter()
                        .any(|(_, r, _)| r.contains_point(p)))
            {
                is_hovering_interactive = true;
            }
            if let Some(ref targets) = self.iris_overlay.delete_targets
                && (targets.header_close_rect.contains_point(p)
                    || targets.confirm_btn_rect.contains_point(p)
                    || targets.cancel_btn_rect.contains_point(p))
            {
                is_hovering_interactive = true;
            }
            if let Some(ref targets) = self.iris_overlay.new_folder_targets
                && (targets.header_close_rect.contains_point(p)
                    || targets.confirm_btn_rect.contains_point(p)
                    || targets.cancel_btn_rect.contains_point(p))
            {
                is_hovering_interactive = true;
            }
            if let Some(ref targets) = self.iris_overlay.rename_targets
                && (targets.header_close_rect.contains_point(p)
                    || targets.confirm_btn_rect.contains_point(p)
                    || targets.cancel_btn_rect.contains_point(p))
            {
                is_hovering_interactive = true;
            }

            let requested_cursor = self.iris_overlay.requested_cursor_icon();
            if requested_cursor != winit::window::CursorIcon::Default {
                window.set_cursor(requested_cursor);
            } else if is_hovering_interactive {
                window.set_cursor(winit::window::CursorIcon::Pointer);
            } else {
                window.set_cursor(winit::window::CursorIcon::Default);
            }
            return true;
        }

        response.consumed
    }

    /// Returns true if the point is over any UI panel, floating modal dialog, or outside the 3D viewport.
    pub fn is_point_over_ui_rects(&self, pos: egui::Pos2) -> bool {
        if pos.y <= IrisEditorOverlay::MENUBAR_HEIGHT
            || self
                .iris_overlay
                .is_point_over_overlay(irisui::prelude::Point::new(pos.x, pos.y))
        {
            return true;
        }

        // 1. Outside 3D viewport -> 100% over an editor UI panel (Hierarchy, Inspector, Assets, Menus, etc.)
        if !self.last_viewport_rect.contains(pos) {
            return true;
        }

        // 2. Open popups / context menus
        if egui::Popup::is_any_open(&self.context) {
            return true;
        }

        // 3. Floating dialogs (Preferences, About, Loading overlay, etc.)
        self.ui_rects.iter().any(|rect| rect.contains(pos))
    }
}