// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event routing logic for the floating Preferences configuration dialog
//! using 100% declarative semantic tags and O(1) hit testing.

use super::super::preferences::{
    self, PREF_TAG_CARD, PREF_TAG_CLOSE, PREF_TAG_CONTENT_VIEW, PREF_TAG_SCROLLBAR_THUMB,
    PREF_TAG_SCROLLBAR_TRACK, PREF_TAG_TITLEBAR, PreferencesAction, PreferencesSliderId,
    is_preferences_tag, parse_dropdown_item_tag, parse_dropdown_tag, parse_number_tag,
    parse_section_tag, parse_slider_tag, parse_tab_tag, parse_toggle_tag,
};
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

        let mut result = IrisOverlayEventResult::default();

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.chrome.cursor_pos = Point::new(position.x as f32, position.y as f32);

                // 1. Titlebar Dragging
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

                // 2. Slider Continuous Dragging
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

                // 3. Scrollbar Thumb Dragging
                if let Some((start_cursor_y, start_scroll_y)) =
                    self.preferences.active_scrollbar_drag
                {
                    let delta_y = self.cursor_pos().y - start_cursor_y;
                    let max_scroll = self.preferences.max_scroll_y;
                    let content_h = preferences::PREF_CARD_HEIGHT - preferences::TITLEBAR_HEIGHT;
                    let total_h = content_h + max_scroll;
                    let content_rect = self.preferences.content_rect.unwrap_or_default();
                    let style = ScrollAreaStyle {
                        thickness: 6.0,
                        inset: 3.0,
                        ..ScrollAreaStyle::dark_default()
                    };
                    if let Some(geom) = ScrollBarGeometry::compute_vertical(
                        content_rect,
                        total_h,
                        self.preferences.scroll_y,
                        &style,
                    ) {
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

        let card_rect = self.preferences.card_rect?;
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
                            if let Ok(mut val) = buffer.trim().parse::<f32>() {
                                val = val.clamp(slider_id.min_val(), slider_id.max_val());
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
                        self.preferences.dropdown_trigger_rect = None;
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
                    && card_rect.contains_point(self.cursor_pos())
                {
                    let scroll_y = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 28.0,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };
                    self.preferences.scroll_y = (self.preferences.scroll_y - scroll_y)
                        .clamp(0.0, self.preferences.max_scroll_y);
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

                // 1. If Preferences' active dropdown popup is open
                if let Some(dd_id) = self.preferences.dropdown {
                    if let Some(ref hit) = hit_target
                        && let Some(selected_idx) = parse_dropdown_item_tag(hit.tag)
                    {
                        result.preferences_action =
                            Some(PreferencesAction::SelectDropdownItem(dd_id, selected_idx));
                        self.preferences.dropdown = None;
                        self.preferences.dropdown_trigger_rect = None;
                        result.consumed = true;
                        return Some(result);
                    } else {
                        // Dismiss active dropdown on clicking outside
                        self.preferences.dropdown = None;
                        self.preferences.dropdown_trigger_rect = None;
                    }
                }

                // If cursor is over an external active foreground popup, do not intercept
                if hit_target
                    .as_ref()
                    .is_some_and(|h| h.layer == UiLayer::Popup && !is_preferences_tag(h.tag))
                {
                    return None;
                }

                // Legitimate click: dismiss floating menus
                self.hierarchy.is_add_menu_open = false;
                self.hierarchy.active_submenu = None;
                self.hierarchy.active_sub_submenu = None;
                self.inspector.active_dropdown = None;
                self.inspector.is_add_menu_open = false;

                // If clicked outside the active number input box, commit and close it
                if let Some((slider_id, buffer)) = self.preferences.active_number_input.take()
                    && let Ok(mut val) = buffer.trim().parse::<f32>()
                {
                    val = val.clamp(slider_id.min_val(), slider_id.max_val());
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

                // Semantic Tag Hit Routing
                if let Some(ref hit) = hit_target
                    && is_preferences_tag(hit.tag)
                {
                    let tag = hit.tag;

                    // Close button
                    if tag == PREF_TAG_CLOSE {
                        result.close_preferences = true;
                        self.preferences.drag_offset = None;
                        self.preferences.active_slider_drag = None;
                        self.preferences.dropdown = None;
                        self.preferences.dropdown_trigger_rect = None;
                        self.preferences.active_number_input = None;
                        result.consumed = true;
                        return Some(result);
                    }

                    // Titlebar drag start
                    if tag == PREF_TAG_TITLEBAR {
                        self.preferences.drag_offset = Some(Point::new(
                            click_point.x - card_rect.x,
                            click_point.y - card_rect.y,
                        ));
                        result.consumed = true;
                        return Some(result);
                    }

                    // Sidebar Tab Selection
                    if let Some(tab_idx) = parse_tab_tag(tag) {
                        self.preferences.tab = tab_idx;
                        self.preferences.dropdown = None;
                        self.preferences.dropdown_trigger_rect = None;
                        self.preferences.active_number_input = None;
                        self.preferences.scroll_y = 0.0;
                        self.preferences.active_scrollbar_drag = None;
                        self.notifier.tag_all();
                        result.preferences_action = Some(PreferencesAction::SelectTab(tab_idx));
                        result.consumed = true;
                        return Some(result);
                    }

                    // Scrollbar Thumb Drag
                    if tag == PREF_TAG_SCROLLBAR_THUMB {
                        self.preferences.active_scrollbar_drag =
                            Some((click_point.y, self.preferences.scroll_y));
                        self.notifier.tag_all();
                        result.consumed = true;
                        return Some(result);
                    }

                    // Scrollbar Track Click
                    if tag == PREF_TAG_SCROLLBAR_TRACK {
                        let content_rect = self.preferences.content_rect.unwrap_or_default();
                        let total_h = content_rect.height + self.preferences.max_scroll_y;
                        let style = ScrollAreaStyle {
                            thickness: 6.0,
                            inset: 3.0,
                            ..ScrollAreaStyle::dark_default()
                        };
                        if let Some(geom) = ScrollBarGeometry::compute_vertical(
                            content_rect,
                            total_h,
                            self.preferences.scroll_y,
                            &style,
                        ) {
                            let new_scroll = ScrollBarGeometry::scroll_from_track_click(
                                click_point.y,
                                geom.track_rect.y,
                                geom.track_rect.height,
                                geom.thumb_rect.height,
                                self.preferences.max_scroll_y,
                            );
                            self.preferences.scroll_y =
                                new_scroll.clamp(0.0, self.preferences.max_scroll_y);
                            self.preferences.active_scrollbar_drag =
                                Some((click_point.y, self.preferences.scroll_y));
                            self.notifier.tag_all();
                            result.consumed = true;
                            return Some(result);
                        }
                    }

                    // Collapsible Section Toggle
                    if let Some(sec_id) = parse_section_tag(tag) {
                        if self.preferences.collapsed_sections.contains(sec_id) {
                            self.preferences.collapsed_sections.remove(sec_id);
                        } else {
                            self.preferences.collapsed_sections.insert(sec_id);
                        }
                        result.preferences_action = Some(PreferencesAction::ToggleSection(sec_id));
                        result.consumed = true;
                        return Some(result);
                    }

                    // ComboBox Trigger
                    if let Some(dd_id) = parse_dropdown_tag(tag) {
                        if self.preferences.dropdown == Some(dd_id) {
                            self.preferences.dropdown = None;
                            self.preferences.dropdown_trigger_rect = None;
                        } else {
                            self.preferences.dropdown = Some(dd_id);
                            self.preferences.dropdown_trigger_rect = Some(hit.rect);
                        }
                        result.consumed = true;
                        return Some(result);
                    }

                    // Toggle Switch / Checkbox
                    if let Some(toggle_id) = parse_toggle_tag(tag) {
                        result.preferences_action = Some(PreferencesAction::Toggle(toggle_id));
                        result.consumed = true;
                        return Some(result);
                    }

                    // Continuous Slider Track Click & Drag
                    if let Some(slider_id) = parse_slider_tag(tag) {
                        let min_val = slider_id.min_val();
                        let max_val = slider_id.max_val();
                        self.preferences.active_slider_drag =
                            Some((slider_id, hit.rect, min_val, max_val));
                        let norm = ((click_point.x - hit.rect.x) / hit.rect.width).clamp(0.0, 1.0);
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

                    // Direct Numeric Input Box
                    if let Some(slider_id) = parse_number_tag(tag) {
                        let cur_val = slider_id.min_val(); // fallback or active
                        let initial_str = slider_id.format_val(cur_val);
                        self.preferences.active_number_input = Some((slider_id, initial_str));
                        self.preferences.active_slider_drag = None;
                        self.preferences.dropdown = None;
                        self.preferences.dropdown_trigger_rect = None;
                        result.consumed = true;
                        return Some(result);
                    }

                    // Background card / content view click absorption
                    if tag == PREF_TAG_CARD || tag == PREF_TAG_CONTENT_VIEW {
                        result.consumed = true;
                        return Some(result);
                    }
                }

                // If click is inside card rect, consume it so it doesn't pass through to canvas
                if card_rect.contains_point(click_point) {
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