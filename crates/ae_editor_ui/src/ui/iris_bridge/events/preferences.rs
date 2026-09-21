// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event routing logic for the floating Preferences configuration dialog.

use super::super::preferences::{self, PreferencesAction, PreferencesSliderId};
use super::super::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles active continuous mouse dragging and release interactions for the Preferences dialog.
    ///
    /// Must be invoked at high priority in event dispatch (Step 3b), before the menubar
    /// or docked panels, so that window dragging and slider dragging continue smoothly across
    /// any panel boundary or menubar, and mouse release is reliably captured anywhere on screen.
    pub(crate) fn handle_preferences_drag_events(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        if self.preferences.drag_offset.is_none()
            && self.preferences.active_slider_drag.is_none()
            && self.preferences.active_scrollbar_drag.is_none()
        {
            return None;
        }

        let targets = self.preferences.targets.as_ref()?;
        let mut result = IrisOverlayEventResult::default();

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.chrome.cursor_pos = Point::new(position.x as f32, position.y as f32);
                if let Some(drag_offset) = self.preferences.drag_offset {
                    self.preferences.pos = Some(calculate_preferences_drag_pos(
                        self.cursor_pos(),
                        drag_offset,
                        self.screen_width,
                        self.screen_height,
                    ));
                    self.notifier.tag_all();
                    result.consumed = true;
                    return Some(result);
                }
                if let Some((slider_id, track_rect, min_val, max_val)) =
                    self.preferences.active_slider_drag
                {
                    let norm =
                        ((self.cursor_pos().x - track_rect.x) / track_rect.width).clamp(0.0, 1.0);
                    let mut val = min_val + norm * (max_val - min_val);
                    if slider_id == PreferencesSliderId::PhysicsFrequency {
                        val = preferences::PHYSICS_HZ_PRESETS
                            .iter()
                            .copied()
                            .min_by(|a, b| (a - val).abs().total_cmp(&(b - val).abs()))
                            .unwrap_or(val);
                    }
                    result.preferences_action =
                        Some(PreferencesAction::SetSliderValue(slider_id, val));
                    result.consumed = true;
                    return Some(result);
                }
                if let Some((start_cursor_y, start_scroll_y)) =
                    self.preferences.active_scrollbar_drag
                    && let Some(geom) = targets.scrollbar
                {
                    let delta_y = self.cursor_pos().y - start_cursor_y;
                    let max_scroll =
                        (targets.total_content_height - targets.content_rect.height).max(0.0);
                    let scroll_delta = ScrollBarGeometry::scroll_from_thumb_drag(
                        delta_y,
                        geom.track_rect.height,
                        geom.thumb_rect.height,
                        max_scroll,
                    );
                    self.preferences.scroll_y =
                        (start_scroll_y + scroll_delta).clamp(0.0, max_scroll);
                    self.notifier.tag_all();
                    result.consumed = true;
                    return Some(result);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: WinitMouseButton::Left,
                ..
            } => {
                if self.preferences.drag_offset.is_some() {
                    self.preferences.drag_offset = None;
                    result.consumed = true;
                    return Some(result);
                }
                if self.preferences.active_slider_drag.is_some() {
                    self.preferences.active_slider_drag = None;
                    result.consumed = true;
                    return Some(result);
                }
                if self.preferences.active_scrollbar_drag.is_some() {
                    self.preferences.active_scrollbar_drag = None;
                    self.notifier.tag_all();
                    result.consumed = true;
                    return Some(result);
                }
            }
            _ => {}
        }

        None
    }

    /// Handles keyboard, mouse cursor dragging, and click interactions for the Preferences dialog.
    pub(crate) fn handle_preferences_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        // Fast-path active drag motion and mouse release
        if let Some(drag_res) = self.handle_preferences_drag_events(event) {
            return Some(drag_res);
        }

        let targets = self.preferences.targets.as_ref()?;
        let mut result = IrisOverlayEventResult::default();

        match event {
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
                if let Some((slider_id, ref mut buffer)) = self.preferences.active_number_input {
                    match *key {
                        winit::keyboard::KeyCode::Escape => {
                            self.preferences.active_number_input = None;
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                            if let Some(&(_, _, min_val, max_val, _)) = targets
                                .number_inputs
                                .iter()
                                .find(|(id, _, _, _, _)| *id == slider_id)
                                && let Ok(mut val) = buffer.trim().parse::<f32>()
                            {
                                val = val.clamp(min_val, max_val);
                                if slider_id == PreferencesSliderId::PhysicsFrequency {
                                    val = preferences::PHYSICS_HZ_PRESETS
                                        .iter()
                                        .copied()
                                        .min_by(|a, b| (a - val).abs().total_cmp(&(b - val).abs()))
                                        .unwrap_or(val);
                                }
                                result.preferences_action =
                                    Some(PreferencesAction::SetSliderValue(slider_id, val));
                            }
                            self.preferences.active_number_input = None;
                            result.consumed = true;
                            return Some(result);
                        }
                        winit::keyboard::KeyCode::Backspace => {
                            buffer.pop();
                            result.consumed = true;
                            return Some(result);
                        }
                        _ => {
                            if let Some(t) = text {
                                for c in t.chars() {
                                    if c.is_ascii_digit()
                                        || (c == '.' && !buffer.contains('.'))
                                        || (c == '-' && buffer.is_empty())
                                    {
                                        buffer.push(c);
                                    }
                                }
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                    }
                }

                if *key == winit::keyboard::KeyCode::Escape {
                    if self.preferences.dropdown.is_some() {
                        self.preferences.dropdown = None;
                    } else {
                        result.close_preferences = true;
                        self.preferences.drag_offset = None;
                        self.preferences.active_slider_drag = None;
                        self.preferences.active_scrollbar_drag = None;
                        self.preferences.active_number_input = None;
                    }
                    result.consumed = true;
                    return Some(result);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if !self.is_point_over_popup(self.cursor_pos())
                    && (targets.content_rect.contains_point(self.cursor_pos())
                        || targets.card_rect.contains_point(self.cursor_pos()))
                {
                    let scroll_y = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 28.0,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };
                    let max_scroll =
                        (targets.total_content_height - targets.content_rect.height).max(0.0);
                    self.preferences.scroll_y =
                        (self.preferences.scroll_y - scroll_y).clamp(0.0, max_scroll);
                    self.notifier.tag_all();
                    result.consumed = true;
                    return Some(result);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.chrome.cursor_pos = Point::new(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: WinitMouseButton::Left,
                ..
            } => {}
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } => {
                let click_point = self.cursor_pos();
                let hit_target = self.tree.hit_test_target(click_point);

                // 1. If Preferences' own active dropdown popup is open, query the hit widget directly
                if let Some(dd_id) = self.preferences.dropdown {
                    if let Some(ref hit) = hit_target
                        && hit.layer == UiLayer::Popup
                        && hit.role == WidgetRole::DropdownItem
                    {
                        let selected_idx = hit.tag as usize;
                        result.preferences_action =
                            Some(PreferencesAction::SelectDropdownItem(dd_id, selected_idx));
                        self.preferences.dropdown = None;
                        result.consumed = true;
                        return Some(result);
                    } else {
                        // Clicked outside dropdown items; dismiss the active dropdown
                        self.preferences.dropdown = None;
                    }
                }

                // Occlusion: If cursor is over an external active foreground popup, Preferences must NOT intercept the click
                if hit_target
                    .as_ref()
                    .is_some_and(|h| h.layer == UiLayer::Popup)
                {
                    return None;
                }

                // Legitimate click on Preferences: dismiss any open floating menus
                self.hierarchy.is_add_menu_open = false;
                self.hierarchy.active_submenu = None;
                self.hierarchy.active_sub_submenu = None;
                self.inspector.active_dropdown = None;
                self.inspector.is_add_menu_open = false;

                // 2. Direct numeric input box clicks
                for &(slider_id, box_rect, _, _, cur_val) in &targets.number_inputs {
                    if box_rect.contains_point(click_point) {
                        let initial_str = match slider_id {
                            PreferencesSliderId::PhysicsFrequency
                            | PreferencesSliderId::UndoHistoryLimit
                            | PreferencesSliderId::CloudAltitude
                            | PreferencesSliderId::FogDistance => format!("{:.0}", cur_val),
                            PreferencesSliderId::ShadowBias => format!("{:.4}", cur_val),
                            _ => format!("{:.2}", cur_val),
                        };
                        self.preferences.active_number_input = Some((slider_id, initial_str));
                        self.preferences.active_slider_drag = None;
                        self.preferences.dropdown = None;
                        result.consumed = true;
                        return Some(result);
                    }
                }

                // If clicked outside active number box, commit and close it
                if let Some((slider_id, buffer)) = self.preferences.active_number_input.take()
                    && let Some(&(_, _, min_val, max_val, _)) = targets
                        .number_inputs
                        .iter()
                        .find(|(id, _, _, _, _)| *id == slider_id)
                    && let Ok(mut val) = buffer.trim().parse::<f32>()
                {
                    val = val.clamp(min_val, max_val);
                    if slider_id == PreferencesSliderId::PhysicsFrequency {
                        val = preferences::PHYSICS_HZ_PRESETS
                            .iter()
                            .copied()
                            .min_by(|a, b| (a - val).abs().total_cmp(&(b - val).abs()))
                            .unwrap_or(val);
                    }
                    result.preferences_action =
                        Some(PreferencesAction::SetSliderValue(slider_id, val));
                }

                // 3. Close button
                if targets.close_button.contains_point(click_point) {
                    result.close_preferences = true;
                    self.preferences.drag_offset = None;
                    self.preferences.active_slider_drag = None;
                    self.preferences.dropdown = None;
                    self.preferences.active_number_input = None;
                    result.consumed = true;
                    return Some(result);
                }

                // 4. Titlebar dragging
                if targets.title_bar_rect.contains_point(click_point) {
                    let card_x = targets.card_rect.x;
                    let card_y = targets.card_rect.y;
                    self.preferences.drag_offset =
                        Some(Point::new(click_point.x - card_x, click_point.y - card_y));
                    result.consumed = true;
                    return Some(result);
                }

                // 5. Tab clicks
                for &(tab_idx, tab_rect) in &targets.tabs {
                    if tab_rect.contains_point(click_point) {
                        self.preferences.tab = tab_idx;
                        self.preferences.dropdown = None;
                        self.preferences.active_number_input = None;
                        self.preferences.scroll_y = 0.0;
                        self.preferences.active_scrollbar_drag = None;
                        self.notifier.tag_all();
                        result.preferences_action = Some(PreferencesAction::SelectTab(tab_idx));
                        result.consumed = true;
                        return Some(result);
                    }
                }

                // 5b. Scrollbar thumb drag or track click
                if let Some(geom) = targets.scrollbar {
                    let thumb_hit_rect = Rect::new(
                        geom.thumb_rect.x - 6.0,
                        geom.thumb_rect.y,
                        geom.thumb_rect.width + 12.0,
                        geom.thumb_rect.height,
                    );
                    let track_hit_rect = Rect::new(
                        geom.track_rect.x - 6.0,
                        geom.track_rect.y,
                        geom.track_rect.width + 12.0,
                        geom.track_rect.height,
                    );

                    if thumb_hit_rect.contains_point(click_point) {
                        self.preferences.active_scrollbar_drag =
                            Some((click_point.y, self.preferences.scroll_y));
                        self.notifier.tag_all();
                        result.consumed = true;
                        return Some(result);
                    } else if track_hit_rect.contains_point(click_point) {
                        let max_scroll =
                            (targets.total_content_height - targets.content_rect.height).max(0.0);
                        let new_scroll = ScrollBarGeometry::scroll_from_track_click(
                            click_point.y,
                            geom.track_rect.y,
                            geom.track_rect.height,
                            geom.thumb_rect.height,
                            max_scroll,
                        );
                        self.preferences.scroll_y = new_scroll.clamp(0.0, max_scroll);
                        self.preferences.active_scrollbar_drag =
                            Some((click_point.y, self.preferences.scroll_y));
                        self.notifier.tag_all();
                        result.consumed = true;
                        return Some(result);
                    }
                }

                // 6. Content Area Interactive Elements (Dropdowns, Toggles, Sliders, Section Toggles)
                if targets.content_rect.contains_point(click_point) {
                    for &(sec_id, sec_rect) in &targets.section_toggles {
                        if sec_rect.contains_point(click_point) {
                            if self.preferences.collapsed_sections.contains(sec_id) {
                                self.preferences.collapsed_sections.remove(sec_id);
                            } else {
                                self.preferences.collapsed_sections.insert(sec_id);
                            }
                            result.preferences_action =
                                Some(PreferencesAction::ToggleSection(sec_id));
                            result.consumed = true;
                            return Some(result);
                        }
                    }

                    for &(dd_id, dd_rect) in &targets.dropdowns {
                        if dd_rect.contains_point(click_point) {
                            if self.preferences.dropdown == Some(dd_id) {
                                self.preferences.dropdown = None;
                            } else {
                                self.preferences.dropdown = Some(dd_id);
                            }
                            result.consumed = true;
                            return Some(result);
                        }
                    }

                    for &(toggle_id, toggle_rect) in &targets.toggles {
                        if toggle_rect.contains_point(click_point) {
                            result.preferences_action = Some(PreferencesAction::Toggle(toggle_id));
                            result.consumed = true;
                            return Some(result);
                        }
                    }

                    for &(slider_id, track_rect, min_val, max_val, _) in &targets.sliders {
                        if track_rect.contains_point(click_point) {
                            self.preferences.active_slider_drag =
                                Some((slider_id, track_rect, min_val, max_val));
                            let norm =
                                ((click_point.x - track_rect.x) / track_rect.width).clamp(0.0, 1.0);
                            let mut val = min_val + norm * (max_val - min_val);
                            if slider_id == PreferencesSliderId::PhysicsFrequency {
                                val = preferences::PHYSICS_HZ_PRESETS
                                    .iter()
                                    .copied()
                                    .min_by(|a, b| (a - val).abs().total_cmp(&(b - val).abs()))
                                    .unwrap_or(val);
                            }
                            result.preferences_action =
                                Some(PreferencesAction::SetSliderValue(slider_id, val));
                            result.consumed = true;
                            return Some(result);
                        }
                    }
                }

                // 7. If click is inside card, consume it so it doesn't click through to underlying canvas
                if targets.card_rect.contains_point(click_point) {
                    result.consumed = true;
                    return Some(result);
                }
            }
            _ => {}
        }

        None
    }
}

/// Calculates the clamped screen position for the Preferences dialog during dragging.
///
/// Ensures the dialog cannot be dragged above the menubar (y >= 28.0) or beyond the screen edges,
/// while allowing continuous movement across docked panel boundaries.
#[inline]
pub fn calculate_preferences_drag_pos(
    cursor_pos: Point,
    drag_offset: Point,
    screen_width: f32,
    screen_height: f32,
) -> Point {
    let max_x = (screen_width - preferences::PREF_CARD_WIDTH).max(0.0);
    let max_y = (screen_height - preferences::PREF_CARD_HEIGHT).max(28.0);
    let new_x = (cursor_pos.x - drag_offset.x).clamp(0.0, max_x);
    let new_y = (cursor_pos.y - drag_offset.y).clamp(28.0, max_y);
    Point::new(new_x, new_y)
}