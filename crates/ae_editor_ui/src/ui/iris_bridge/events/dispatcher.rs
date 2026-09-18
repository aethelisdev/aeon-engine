// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Top-level event orchestration and interaction routing for Iris UI editor overlays.

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::*;
use winit::event::WindowEvent;

impl IrisEditorOverlay {
    /// Intercepts and processes window mouse input and cursor movement events across all active Iris UI subsystems.
    pub fn handle_event(&mut self, event: &WindowEvent) -> IrisOverlayEventResult {
        let result = self.dispatch_window_event_internal(event);
        // Reactive Event Invalidation: If any UI element consumed this event or produced
        // an action/state change, immediately flag the UI tree as dirty so the next frame
        // reflects the change instantly, even if the mouse cursor does not move a single pixel.
        if result.consumed
            || result.preferences_action.is_some()
            || result.ui_action.is_some()
            || result.toggle_panel.is_some()
            || result.open_preferences
            || result.close_preferences
            || result.open_about
            || result.close_about
            || result.confirm_delete
            || result.cancel_delete
            || result.create_folder.is_some()
            || result.apply_rename.is_some()
            || result.reset_layout
        {
            self.notifier.tag_all();
        }
        result
    }

    /// Internal routing pipeline that tests UI layers in strict Z-order.
    fn dispatch_window_event_internal(&mut self, event: &WindowEvent) -> IrisOverlayEventResult {
        let mut result = IrisOverlayEventResult::default();

        // 1. Track modifier keys for accelerated / fine-tune dragging
        if let WindowEvent::ModifiersChanged(modifiers) = event {
            self.chrome.shift_held = modifiers.state().shift_key();
            self.chrome.alt_held = modifiers.state().alt_key();
            self.chrome.ctrl_held = modifiers.state().control_key();
        }

        // 2. Real-time cursor position tracking
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.chrome.cursor_pos = Point::new(position.x as f32, position.y as f32);
        }

        // 3. Inspector active dragging / scrubber motion & release (runs globally)
        if let Some(insp_drag_res) = self.handle_inspector_drag_events(event) {
            return insp_drag_res;
        }

        // 3b. Active Preferences Drag Interaction (Window dragging or slider dragging)
        // Must be handled before menubar or any other widget, regardless of cursor position,
        // so that window drag continues smoothly across panels/menubar and releases cleanly.
        if let Some(pref_drag_res) = self.handle_preferences_drag_events(event) {
            return pref_drag_res;
        }

        // 3c. Active Asset Drag Interaction (Global mouse release and Escape cancellation)
        // If an asset item is being dragged from the Content Browser, it must receive
        // the release or cancellation event everywhere across the window, cleanly ending
        // the drag or spawning onto the viewport.
        if let Some(asset_drag_res) = self.handle_asset_drag_events(event) {
            return asset_drag_res;
        }

        // 4. Loading Splash Screen (blocks all underlying interactions)
        if self.modals.loading_targets.is_some() {
            result.consumed = true;
            return result;
        }

        // 5. Top Menubar and Dropdowns (Prioritized above modal dialogs whenever a dropdown
        // is open or the cursor is positioned over the menubar header)
        if (self.menubar.active_menu.is_some() || self.cursor_pos().y <= Self::MENUBAR_HEIGHT)
            && let Some(mb_res) = self.handle_menubar_event(event)
        {
            return mb_res;
        }

        // 5b. Active Floating Popups (Hierarchy, Inspector, UI Designer, Dock Overflow)
        // These belong to UiLayer::Popup; events inside them MUST be handled before modal dialogs!
        if self.is_point_over_popup(self.cursor_pos()) {
            if let Some(hier_res) = self.handle_hierarchy_window_event(event) {
                return hier_res;
            }
            if let Some(insp_res) = self.handle_inspector_window_event(event) {
                return insp_res;
            }
            if let Some(ui_res) = self.handle_ui_designer_window_event(event) {
                return ui_res;
            }
            if let Some(dock_res) = self.handle_dock_overflow_event(event) {
                return dock_res;
            }
        }

        // 6. Generic Modal Dialogs (About, Delete, New Folder, Rename, Asset Preview)
        // Topmost modal cards; must be evaluated before Preferences so clicks and close actions
        // on active modal dialogs are never intercepted by the background Preferences panel.
        if let Some(modal_result) = self.handle_modal_events(event) {
            return modal_result;
        }

        // 7. Preferences Floating Dialog
        if let Some(pref_result) = self.handle_preferences_event(event) {
            return pref_result;
        }

        // 8. Top Menubar and Dropdowns (Fallback interaction route)
        if let Some(mb_res) = self.handle_menubar_event(event) {
            return mb_res;
        }

        // 8b. Viewport HUD Controls (Interactive on docked or floating viewports)
        if let Some(hud_res) = self.handle_viewport_hud_window_event(event) {
            return hud_res;
        }

        // 9. Floating Window Occlusion Check:
        // If cursor is over an active floating window, docked panels must NOT claim the event!
        let is_cursor_over_floating = self
            .chrome
            .floating_window_rects
            .iter()
            .any(|r| r.contains_point(self.cursor_pos()));

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