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
        if iris_res.open_about {
            self.show_about = true;
        }
        if iris_res.close_about {
            self.show_about = false;
        }
        // Always pass event to egui state so pointer and drag states never get starved or desynchronized
        let response = self.state.on_window_event(window, event);

        if iris_res.consumed {
            if let Some(ref targets) = self.iris_overlay.about_targets {
                let p = self.iris_overlay.cursor_pos;
                if targets.header_close_rect.contains_point(p)
                    || targets.bottom_close_rect.contains_point(p)
                    || targets.link_rect.contains_point(p)
                {
                    window.set_cursor(winit::window::CursorIcon::Pointer);
                } else {
                    window.set_cursor(winit::window::CursorIcon::Default);
                }
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