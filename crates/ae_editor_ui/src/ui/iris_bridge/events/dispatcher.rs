// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Top-level event orchestration and interaction routing for Iris UI editor overlays.

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::*;
use winit::event::WindowEvent;

impl IrisEditorOverlay {
    /// Intercepts and processes window mouse input and cursor movement events across all active Iris UI subsystems.
    pub fn handle_event(&mut self, event: &WindowEvent) -> IrisOverlayEventResult {
        let mut result = IrisOverlayEventResult::default();

        // 1. Track modifier keys for accelerated / fine-tune dragging
        if let WindowEvent::ModifiersChanged(modifiers) = event {
            self.shift_held = modifiers.state().shift_key();
            self.alt_held = modifiers.state().alt_key();
            self.ctrl_held = modifiers.state().control_key();
        }

        // 2. Real-time cursor position tracking
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.cursor_pos = Point::new(position.x as f32, position.y as f32);
        }

        // 3. Inspector active dragging / scrubber motion & release (runs globally)
        if let Some(insp_drag_res) = self.handle_inspector_drag_events(event) {
            return insp_drag_res;
        }

        // 4. Loading Splash Screen (blocks all underlying interactions)
        if self.loading_targets.is_some() {
            result.consumed = true;
            return result;
        }

        // 5. Top Menubar and Dropdowns (Prioritized above modal dialogs whenever a dropdown
        // is open or the cursor is positioned over the menubar header)
        if (self.active_menu.is_some() || self.cursor_pos.y <= Self::MENUBAR_HEIGHT)
            && let Some(mb_res) = self.handle_menubar_event(event)
        {
            return mb_res;
        }

        // 6. Preferences Modal Dialog
        if let Some(pref_result) = self.handle_preferences_event(event) {
            return pref_result;
        }

        // 7. Generic Modal Dialogs (About, Delete, New Folder, Rename, Asset Preview)
        if let Some(modal_result) = self.handle_modal_events(event) {
            return modal_result;
        }

        // 8. Top Menubar and Dropdowns (Fallback interaction route)
        if let Some(mb_res) = self.handle_menubar_event(event) {
            return mb_res;
        }

        // 8. Viewport HUD Controls (Interactive on docked or floating viewports)
        if let Some(hud_res) = self.handle_viewport_hud_window_event(event) {
            return hud_res;
        }

        // 9. Floating Window Occlusion Check:
        // If cursor is over an active floating window, docked panels must NOT claim the event!
        let is_cursor_over_floating = self
            .floating_window_rects
            .iter()
            .any(|r| r.contains_point(self.cursor_pos));

        if !is_cursor_over_floating {
            // 10. Docked Panels Event Dispatch
            if let Some(hier_res) = self.handle_hierarchy_window_event(event) {
                return hier_res;
            }

            if let Some(stats_res) = self.handle_stats_window_event(event) {
                return stats_res;
            }

            if let Some(console_res) = self.handle_console_window_event(event) {
                return console_res;
            }

            if let Some(assets_res) = self.handle_assets_window_event(event) {
                return assets_res;
            }

            if let Some(timeline_res) = self.handle_timeline_window_event(event) {
                return timeline_res;
            }

            if let Some(mat_res) = self.handle_material_window_event(event) {
                return mat_res;
            }

            if let Some(ui_res) = self.handle_ui_designer_window_event(event) {
                return ui_res;
            }

            if let Some(scroll_res) = self.handle_panel_mouse_wheel(event) {
                return scroll_res;
            }

            if let Some(insp_res) = self.handle_inspector_window_event(event) {
                return insp_res;
            }
        }

        result
    }
}