// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event routing, hit-testing, and interaction dispatch subsystem for Iris UI editor overlays.

pub mod menubar;
pub mod modals;
pub mod preferences;

use super::hierarchy::{self, HierarchyAction};
use super::stats::StatsPanelAction;
use super::types::{IrisEditorOverlay, IrisOverlayEventResult};
use super::viewport_hud::ViewportHudAction;
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Returns true if the given coordinate is over the menubar, status bar, or active dropdown/modal.
    pub fn is_point_over_overlay(&self, point: Point) -> bool {
        if self.about_targets.is_some()
            || self.delete_targets.is_some()
            || self.new_folder_targets.is_some()
            || self.rename_targets.is_some()
            || self.loading_targets.is_some()
        {
            return true;
        }
        if let Some(ref targets) = self.preferences_targets
            && (targets.card_rect.contains_point(point)
                || targets
                    .active_dropdown_popup_rect
                    .is_some_and(|r| r.contains_point(point)))
        {
            return true;
        }
        // Floating dropdown popup from menubar has highest z-order above docked panels
        if let Some(dd_rect) = self.dropdown_rect
            && dd_rect.contains_point(point)
        {
            return true;
        }
        if let Some(ref targets) = self.hierarchy_targets {
            if let Some(sub_rect) = targets.active_submenu_rect
                && sub_rect.contains_point(point)
            {
                return true;
            }
            if let Some(add_rect) = targets.active_add_menu_rect
                && add_rect.contains_point(point)
            {
                return true;
            }
            if let Some((_, menu_rect, _, _)) = targets.active_context_menu
                && menu_rect.contains_point(point)
            {
                return true;
            }
            if targets.panel_rect.contains_point(point) {
                return true;
            }
        }
        if let Some(ref hud) = self.viewport_hud_targets {
            if let Some(dd_rect) = hud.active_dropdown_popup_rect
                && dd_rect.contains_point(point)
            {
                return true;
            }
            if hud.buttons.iter().any(|(_, r)| r.contains_point(point))
                || hud
                    .dropdown_triggers
                    .iter()
                    .any(|(_, r)| r.contains_point(point))
                || hud
                    .compass_knobs
                    .iter()
                    .any(|(_, r)| r.contains_point(point))
                || hud
                    .billboard_icons
                    .iter()
                    .any(|(_, r)| r.contains_point(point))
            {
                return true;
            }
        }
        if let Some(ref targets) = self.stats_targets
            && targets.panel_rect.contains_point(point)
        {
            return true;
        }
        if point.y <= Self::MENUBAR_HEIGHT {
            return true;
        }
        if self.screen_height > Self::STATUS_BAR_HEIGHT
            && point.y >= (self.screen_height - Self::STATUS_BAR_HEIGHT)
        {
            return true;
        }
        false
    }

    /// Intercepts and processes window mouse input and cursor movement events.
    pub fn handle_event(&mut self, event: &WindowEvent) -> IrisOverlayEventResult {
        // ALWAYS update real-time cursor_pos at the very start of handle_event
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.cursor_pos = Point::new(position.x as f32, position.y as f32);
        }

        let mut result = IrisOverlayEventResult::default();

        // 0a. If Loading splash is active, consume all interaction to block underlying clicks
        if self.loading_targets.is_some() {
            result.consumed = true;
            return result;
        }

        // 0b. If Preferences panel is active, intercept its events
        if let Some(pref_result) = self.handle_preferences_event(event) {
            return pref_result;
        }

        // 0c. If any Modal dialogue is active (About, Delete, New Folder, Rename)
        if let Some(modal_result) = self.handle_modal_events(event) {
            return modal_result;
        }

        // 0d. Top Menubar and Floating Dropdown events (HIGHEST PRIORITY ABOVE DOCKED PANELS!)
        if let Some(mb_res) = self.handle_menubar_event(event) {
            return mb_res;
        }

        // 0e. If Scene Hierarchy panel is active, intercept clicks, context menus, and add menus
        if let Some(ref hier_targets) = self.hierarchy_targets
            && let WindowEvent::MouseInput {
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
                        self.hierarchy_active_context_menu = None;
                        self.active_menu = None;
                        self.viewport_hud_dropdown = None;
                        self.preferences_dropdown = None;
                    }
                    HierarchyAction::CloseAddMenu => {
                        self.hierarchy_is_add_menu_open = false;
                        self.hierarchy_active_submenu = None;
                    }
                    HierarchyAction::OpenSubmenu(sub) => {
                        self.hierarchy_active_submenu = Some(sub);
                    }
                    HierarchyAction::CloseSubmenu => {
                        self.hierarchy_active_submenu = None;
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
                return result;
            }
        }

        // 0f. If Viewport HUD is active, intercept clicks and dropdown interactions
        if let Some(ref hud_targets) = self.viewport_hud_targets
            && let WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } = event
        {
            let click_point = self.cursor_pos;

            // 1. If an active dropdown popup is open
            if self.viewport_hud_dropdown.is_some() {
                for (action, rect, _) in &hud_targets.active_dropdown_items {
                    if rect.contains_point(click_point) {
                        self.viewport_hud_actions.push(action.clone());
                        self.viewport_hud_dropdown = None;
                        result.consumed = true;
                        return result;
                    }
                }

                if let Some(popup_rect) = hud_targets.active_dropdown_popup_rect
                    && !popup_rect.contains_point(click_point)
                {
                    self.viewport_hud_dropdown = None;
                }
            }

            // 2. Check dropdown triggers
            for (dd_id, rect) in &hud_targets.dropdown_triggers {
                if rect.contains_point(click_point) {
                    self.viewport_hud_dropdown = if self.viewport_hud_dropdown == Some(*dd_id) {
                        None
                    } else {
                        Some(*dd_id)
                    };
                    result.consumed = true;
                    return result;
                }
            }

            // 3. Check toolbar buttons
            for (action, rect) in &hud_targets.buttons {
                if rect.contains_point(click_point) {
                    self.viewport_hud_actions.push(action.clone());
                    result.consumed = true;
                    return result;
                }
            }

            // 4. Check compass knobs
            for (action, rect) in &hud_targets.compass_knobs {
                if rect.contains_point(click_point) {
                    self.viewport_hud_actions.push(action.clone());
                    result.consumed = true;
                    return result;
                }
            }

            // 5. Check billboard icons
            for (ent, rect) in &hud_targets.billboard_icons {
                if rect.contains_point(click_point) {
                    self.viewport_hud_actions
                        .push(ViewportHudAction::SelectEntity(*ent));
                    result.consumed = true;
                    return result;
                }
            }
        }

        // 0g. If Stats panel is active, intercept checkbox clicks
        if let Some(ref stats_targets) = self.stats_targets
            && let WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } = event
        {
            let click_point = self.cursor_pos;
            if let Some(wire_rect) = stats_targets.wireframe_checkbox_rect
                && wire_rect.contains_point(click_point)
            {
                self.stats_actions.push(StatsPanelAction::ToggleWireframe);
                result.consumed = true;
                return result;
            }
            if let Some(grid_rect) = stats_targets.grid_checkbox_rect
                && grid_rect.contains_point(click_point)
            {
                self.stats_actions.push(StatsPanelAction::ToggleGrid);
                result.consumed = true;
                return result;
            }
            if stats_targets.panel_rect.contains_point(click_point) {
                result.consumed = true;
                return result;
            }
        }

        // 0h. Mouse Wheel scrolling for Stats panel, Hierarchy panel, and Preferences dialog
        if let WindowEvent::MouseWheel { delta, .. } = event {
            let delta_y = match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 24.0,
                winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
            };
            if let Some(ref targets) = self.stats_targets
                && targets.panel_rect.contains_point(self.cursor_pos)
            {
                self.stats_scroll_y = (self.stats_scroll_y - delta_y).max(0.0);
                result.consumed = true;
                return result;
            }
            if let Some(ref targets) = self.hierarchy_targets
                && targets.panel_rect.contains_point(self.cursor_pos)
            {
                self.hierarchy_scroll_y = (self.hierarchy_scroll_y - delta_y).max(0.0);
                result.consumed = true;
                return result;
            }
            if let Some(ref targets) = self.preferences_targets
                && targets.card_rect.contains_point(self.cursor_pos)
            {
                self.preferences_scroll_y = (self.preferences_scroll_y - delta_y).max(0.0);
                result.consumed = true;
                return result;
            }
        }

        // 0i. Hierarchy Add Menu hover and submenu cascade
        if let WindowEvent::CursorMoved { .. } = event
            && let Some(ref targets) = self.hierarchy_targets
            && self.hierarchy_is_add_menu_open
        {
            let in_submenu = targets
                .active_submenu_rect
                .is_some_and(|r| r.contains_point(self.cursor_pos));
            let in_add_menu = targets
                .active_add_menu_rect
                .is_some_and(|r| r.contains_point(self.cursor_pos));

            if !in_submenu && in_add_menu {
                for (item_rect, target_payload) in &targets.add_menu_items {
                    if item_rect.contains_point(self.cursor_pos) {
                        if let Ok(submenu_id) = target_payload {
                            self.hierarchy_active_submenu = Some(*submenu_id);
                        } else {
                            self.hierarchy_active_submenu = None;
                        }
                        break;
                    }
                }
            }
        }

        result
    }
}