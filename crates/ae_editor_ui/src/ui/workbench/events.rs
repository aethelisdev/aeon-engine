// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Window Event Coordinator
//!
//! Forwards winit window events to Iris UI overlays, native dock coordinator,
//! keyboard hotkey handlers, and manages modal/dialog lifecycle state.

pub mod dock_interaction;
pub mod hit_test;
pub mod keyboard;

use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::workbench::state::EngineUi;
use irisui::prelude::Rect;
use winit::event::WindowEvent;
use winit::window::Window;

impl EngineUi {
    /// Forwards winit window events to Iris UI and the native dock coordinator.
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        // Synchronize active floating window boundaries with IrisEditorOverlay for occlusion testing
        self.iris_overlay.chrome.floating_window_rects = self
            .layout_state
            .dock_state
            .floating_windows
            .iter()
            .map(|w| Rect::new(w.rect.x, w.rect.y, w.rect.width, w.rect.height))
            .collect();

        let zoom = self.scale_factor();
        let scaled_event;
        let event_ref = match event {
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                scaled_event = WindowEvent::CursorMoved {
                    device_id: *device_id,
                    position: winit::dpi::PhysicalPosition::new(
                        position.x / (zoom as f64),
                        position.y / (zoom as f64),
                    ),
                };
                &scaled_event
            }
            other => other,
        };

        let iris_res = self.iris_overlay.handle_event(event_ref);
        if let Some(act) = iris_res.ui_action {
            self.pending_actions.push(act);
        }
        if let Some(panel) = iris_res.toggle_panel {
            self.layout_state.activate_or_open(panel);
        }
        if let Some((leaf, tab_idx)) = iris_res.activate_dock_tab {
            let _ = self
                .layout_state
                .dock_state
                .tree
                .set_active_tab(leaf, tab_idx);
            self.iris_overlay.chrome.active_dock_overflow = None;
            self.iris_overlay.chrome.needs_layout_rebuild = true;
            return true;
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
            if let crate::ui::iris_bridge::PreferencesAction::SetUiScale(s) = pref_act {
                self.ui_zoom_factor = s;
            }
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
            let _ = crate::assets::file_ops::delete_asset_or_folder(&target);
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
            let _ = crate::assets::file_ops::create_subfolder(&parent, &folder_name);
            self.iris_overlay.modals.new_folder_buffer.clear();
            self.asset_browser.new_folder_name.clear();
        }
        if iris_res.cancel_new_folder {
            self.asset_browser.new_folder_parent = None;
            self.iris_overlay.modals.new_folder_buffer.clear();
            self.asset_browser.new_folder_name.clear();
        }

        if let Some(new_name) = iris_res.apply_rename
            && let Some(ren) = self.asset_browser.rename_state.take()
        {
            let _ = crate::assets::file_ops::rename_asset_or_folder(&ren.target_path, &new_name);
            self.iris_overlay.modals.rename_buffer.clear();
        }
        if iris_res.cancel_rename {
            self.asset_browser.rename_state = None;
            self.iris_overlay.modals.rename_buffer.clear();
        }

        if iris_res.clear_console_entries {
            self.console_entries.clear();
            self.console_last_count = 0;
        }

        // Process keyboard shortcuts and search live-typing
        if self.handle_keyboard_shortcut(event) {
            return true;
        }

        let p = self.iris_overlay.cursor_pos();
        let win_size = window.inner_size();
        let screen_w = win_size.width as f32 / zoom;
        let screen_h = win_size.height as f32 / zoom;
        let workspace_rect = Rect::new(
            0.0,
            IrisEditorOverlay::MENUBAR_HEIGHT,
            screen_w,
            (screen_h - IrisEditorOverlay::MENUBAR_HEIGHT - IrisEditorOverlay::STATUS_BAR_HEIGHT)
                .max(0.0),
        );

        // Enforce workspace boundary clamping on all floating windows
        self.layout_state.clamp_floating_windows(
            screen_w,
            screen_h,
            IrisEditorOverlay::MENUBAR_HEIGHT,
            IrisEditorOverlay::STATUS_BAR_HEIGHT,
        );

        // Process native dock cursor movements and mouse inputs
        let mut dock_consumed = false;
        if matches!(event, WindowEvent::CursorMoved { .. }) {
            dock_consumed = self.handle_dock_cursor_moved(p, screen_w, screen_h, workspace_rect);
        } else if matches!(event, WindowEvent::MouseInput { .. }) {
            dock_consumed = self.handle_dock_mouse_input(p, event);
        }

        // Apply requested cursor
        let requested_cursor = self.iris_overlay.requested_cursor_icon();
        if requested_cursor != winit::window::CursorIcon::Default {
            window.set_cursor(requested_cursor);
        } else {
            window.set_cursor(winit::window::CursorIcon::Default);
        }

        let consumed = iris_res.consumed || dock_consumed;
        if consumed {
            self.iris_overlay.notifier.tag_all();
            self.iris_overlay.chrome.needs_layout_rebuild = true;
        }
        consumed
    }
}