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
        if let Some(ref targets) = self.inspector_targets {
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
            if targets.scroll_container_rect.contains_point(point)
                || targets.add_component_btn_rect.contains_point(point)
                || targets.save_prefab_btn_rect.contains_point(point)
                || targets.name_input_rect.contains_point(point)
            {
                return true;
            }
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
        let mut result = IrisOverlayEventResult::default();

        // ALWAYS update real-time cursor_pos at the very start of handle_event
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.cursor_pos = Point::new(position.x as f32, position.y as f32);

            // Continuous horizontal mouse drag for Inspector numeric fields
            if let Some(ref mut drag) = self.inspector_drag_number {
                let delta = self.cursor_pos.x - drag.start_x;
                if delta.abs() > 2.0 {
                    drag.has_dragged = true;
                    self.inspector_active_number_input = None;
                    let new_val = (drag.start_val + delta * drag.sensitivity)
                        .clamp(drag.min_val, drag.max_val);
                    self.inspector_actions
                        .push(super::inspector::InspectorAction::SetNumberValue(
                            drag.id, new_val,
                        ));
                    result.consumed = true;
                    return result;
                }
            }
        }

        // Mouse Release handler for Inspector numeric drag
        if let WindowEvent::MouseInput {
            state: ElementState::Released,
            button: WinitMouseButton::Left,
            ..
        } = event
            && let Some(drag) = self.inspector_drag_number.take()
        {
            if !drag.has_dragged {
                // Click in-place without dragging -> activate direct numeric text editing
                self.inspector_active_number_input =
                    Some((drag.id, format!("{:.2}", drag.start_val)));
            }
            result.consumed = true;
            return result;
        }

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
            if let Some(ref targets) = self.inspector_targets
                && targets
                    .scroll_container_rect
                    .contains_point(self.cursor_pos)
            {
                self.inspector_scroll_y = (self.inspector_scroll_y - delta_y).max(0.0);
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

        // 0j. Inspector Add Menu hover and category submenu cascade
        if let WindowEvent::CursorMoved { .. } = event
            && let Some(ref targets) = self.inspector_targets
            && self.inspector_is_add_menu_open
        {
            let in_submenu = targets
                .active_submenu_rect
                .is_some_and(|r| r.contains_point(self.cursor_pos));
            let in_add_menu = targets
                .active_add_menu_rect
                .is_some_and(|r| r.contains_point(self.cursor_pos));

            if !in_submenu && in_add_menu {
                for &(cat, item_rect) in &targets.add_menu_categories {
                    if item_rect.contains_point(self.cursor_pos) {
                        self.inspector_active_submenu = Some(cat);
                        break;
                    }
                }
            }
        }

        // 0k. Inspector Keyboard Input (Number input editing and Entity Rename)
        if self.inspector_active_number_input.is_some() || self.inspector_rename_buffer.is_some() {
            match event {
                WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                    if let Some((_, ref mut buf)) = self.inspector_active_number_input {
                        for c in text.chars() {
                            if c.is_ascii_digit() || c == '.' || c == '-' {
                                buf.push(c);
                            }
                        }
                        result.consumed = true;
                        return result;
                    }
                    if let Some(ref mut buf) = self.inspector_rename_buffer {
                        buf.push_str(text);
                        result.consumed = true;
                        return result;
                    }
                }
                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key),
                            text,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    if self.inspector_active_number_input.is_some() {
                        match *key {
                            winit::keyboard::KeyCode::Escape => {
                                self.inspector_active_number_input = None;
                                result.consumed = true;
                                return result;
                            }
                            winit::keyboard::KeyCode::Enter
                            | winit::keyboard::KeyCode::NumpadEnter => {
                                if let Some((id, buf)) = self.inspector_active_number_input.take()
                                    && let Ok(v) = buf.trim().parse::<f32>()
                                {
                                    self.inspector_actions.push(
                                        super::inspector::InspectorAction::SetNumberValue(id, v),
                                    );
                                }
                                result.consumed = true;
                                return result;
                            }
                            winit::keyboard::KeyCode::Backspace => {
                                if let Some((_, ref mut buf)) = self.inspector_active_number_input {
                                    buf.pop();
                                }
                                result.consumed = true;
                                return result;
                            }
                            _ => {
                                if let Some(t) = text
                                    && let Some((_, ref mut buf)) =
                                        self.inspector_active_number_input
                                {
                                    for c in t.chars() {
                                        if c.is_ascii_digit() || c == '.' || c == '-' {
                                            buf.push(c);
                                        }
                                    }
                                    result.consumed = true;
                                    return result;
                                }
                            }
                        }
                    }

                    if self.inspector_rename_buffer.is_some() {
                        match *key {
                            winit::keyboard::KeyCode::Escape => {
                                self.inspector_rename_buffer = None;
                                result.consumed = true;
                                return result;
                            }
                            winit::keyboard::KeyCode::Enter
                            | winit::keyboard::KeyCode::NumpadEnter => {
                                if let Some(buf) = self.inspector_rename_buffer.take()
                                    && !buf.trim().is_empty()
                                {
                                    self.inspector_actions
                                        .push(super::inspector::InspectorAction::RenameEntity(buf));
                                }
                                result.consumed = true;
                                return result;
                            }
                            winit::keyboard::KeyCode::Backspace => {
                                if let Some(ref mut buf) = self.inspector_rename_buffer {
                                    buf.pop();
                                }
                                result.consumed = true;
                                return result;
                            }
                            _ => {
                                if let Some(t) = text
                                    && let Some(ref mut buf) = self.inspector_rename_buffer
                                    && !t.chars().any(|c| c.is_control())
                                {
                                    buf.push_str(t);
                                    result.consumed = true;
                                    return result;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // 0l. Inspector click interactions
        if let Some(ref insp_targets) = self.inspector_targets
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

            // 1. Check if an active dropdown popup is open and clicked
            if let Some(active_dd) = self.inspector_active_dropdown {
                if let Some(popup_rect) = insp_targets.active_dropdown_popup_rect
                    && popup_rect.contains_point(click_point)
                {
                    for &(opt_idx, item_rect) in &insp_targets.dropdown_items {
                        if item_rect.contains_point(click_point) {
                            self.inspector_actions.push(
                                super::inspector::InspectorAction::SelectDropdown(
                                    active_dd, opt_idx,
                                ),
                            );
                            self.inspector_active_dropdown = None;
                            result.consumed = true;
                            return result;
                        }
                    }
                    result.consumed = true;
                    return result;
                }
                // Click outside closed the dropdown
                self.inspector_active_dropdown = None;
            }

            // 2. Check if a number input box is clicked
            for &(num_id, box_rect, min_val, max_val, cur_val) in &insp_targets.number_inputs {
                if box_rect.contains_point(click_point) {
                    // If switching from another active number input, commit previous first
                    if let Some((prev_id, prev_buf)) = self.inspector_active_number_input.take()
                        && prev_id != num_id
                        && let Ok(v) = prev_buf.trim().parse::<f32>()
                    {
                        self.inspector_actions.push(
                            super::inspector::InspectorAction::SetNumberValue(prev_id, v),
                        );
                    }
                    let sensitivity = match num_id {
                        super::inspector::InspectorNumberInputId::RotX
                        | super::inspector::InspectorNumberInputId::RotY
                        | super::inspector::InspectorNumberInputId::RotZ
                        | super::inspector::InspectorNumberInputId::CharacterMaxSlope => 0.5,
                        super::inspector::InspectorNumberInputId::ScaleX
                        | super::inspector::InspectorNumberInputId::ScaleY
                        | super::inspector::InspectorNumberInputId::ScaleZ => 0.01,
                        super::inspector::InspectorNumberInputId::PosX
                        | super::inspector::InspectorNumberInputId::PosY
                        | super::inspector::InspectorNumberInputId::PosZ
                        | super::inspector::InspectorNumberInputId::RigidBodyMass
                        | super::inspector::InspectorNumberInputId::RigidBodyGravity
                        | super::inspector::InspectorNumberInputId::ColliderBoxX
                        | super::inspector::InspectorNumberInputId::ColliderBoxY
                        | super::inspector::InspectorNumberInputId::ColliderBoxZ
                        | super::inspector::InspectorNumberInputId::ColliderHalfHeight
                        | super::inspector::InspectorNumberInputId::ColliderRadius
                        | super::inspector::InspectorNumberInputId::ColliderCenterY => 0.05,
                        _ => 0.02,
                    };
                    self.inspector_drag_number =
                        Some(crate::ui::iris_bridge::types::InspectorNumberDragState {
                            id: num_id,
                            start_x: click_point.x,
                            start_val: cur_val,
                            min_val,
                            max_val,
                            sensitivity,
                            has_dragged: false,
                        });
                    result.consumed = true;
                    return result;
                }
            }

            // If click was outside number input while editing, commit value
            if let Some((id, buf)) = self.inspector_active_number_input.take()
                && let Ok(v) = buf.trim().parse::<f32>()
            {
                self.inspector_actions
                    .push(super::inspector::InspectorAction::SetNumberValue(id, v));
            }

            let mut actions = Vec::new();
            let consumed = super::inspector::handle_inspector_click(
                click_point,
                ui_button,
                insp_targets,
                &mut actions,
            );

            for action in actions {
                match action {
                    super::inspector::InspectorAction::OpenAddComponentMenu(_) => {
                        self.inspector_is_add_menu_open = true;
                        self.inspector_active_submenu = None;
                        self.active_menu = None;
                        self.hierarchy_is_add_menu_open = false;
                        self.hierarchy_active_context_menu = None;
                    }
                    super::inspector::InspectorAction::CloseAddComponentMenu => {
                        self.inspector_is_add_menu_open = false;
                        self.inspector_active_submenu = None;
                    }
                    super::inspector::InspectorAction::OpenAddSubmenu(cat) => {
                        self.inspector_active_submenu = Some(cat);
                    }
                    super::inspector::InspectorAction::CloseAddSubmenu => {
                        self.inspector_active_submenu = None;
                    }
                    super::inspector::InspectorAction::SelectDropdown(dd_id, _) => {
                        if self.inspector_active_dropdown == Some(dd_id) {
                            self.inspector_active_dropdown = None;
                        } else {
                            self.inspector_active_dropdown = Some(dd_id);
                        }
                    }
                    super::inspector::InspectorAction::FocusRename => {
                        self.inspector_rename_buffer = Some(String::new());
                    }
                    other => {
                        self.inspector_actions.push(other);
                    }
                }
            }

            if consumed {
                result.consumed = true;
                return result;
            }
        }

        result
    }
}