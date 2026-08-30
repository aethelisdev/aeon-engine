// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event routing logic for the floating Preferences configuration dialog.

use super::super::preferences::{self, PreferencesAction, PreferencesSliderId};
use super::super::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles keyboard, mouse cursor dragging, and click interactions for the Preferences dialog.
    pub(crate) fn handle_preferences_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let targets = self.preferences_targets.as_ref()?;
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
                if let Some((slider_id, ref mut buffer)) = self.active_number_input {
                    match *key {
                        winit::keyboard::KeyCode::Escape => {
                            self.active_number_input = None;
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
                            self.active_number_input = None;
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
                    if self.preferences_dropdown.is_some() {
                        self.preferences_dropdown = None;
                    } else {
                        result.close_preferences = true;
                        self.preferences_drag_offset = None;
                        self.active_slider_drag = None;
                        self.active_number_input = None;
                    }
                    result.consumed = true;
                    return Some(result);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if targets.content_rect.contains_point(self.cursor_pos) {
                    let scroll_y = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 28.0,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };
                    let max_scroll = (targets.total_content_height - targets.content_rect.height
                        + 32.0)
                        .max(0.0);
                    self.preferences_scroll_y =
                        (self.preferences_scroll_y - scroll_y).clamp(0.0, max_scroll);
                    result.consumed = true;
                    return Some(result);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = Point::new(position.x as f32, position.y as f32);
                if let Some(drag_offset) = self.preferences_drag_offset {
                    let max_x = (self.screen_width - preferences::PREF_CARD_WIDTH).max(0.0);
                    let max_y = (self.screen_height - preferences::PREF_CARD_HEIGHT).max(28.0);
                    let new_x = (self.cursor_pos.x - drag_offset.x).clamp(0.0, max_x);
                    let new_y = (self.cursor_pos.y - drag_offset.y).clamp(28.0, max_y);
                    self.preferences_pos = Some(Point::new(new_x, new_y));
                    result.consumed = true;
                    return Some(result);
                }
                if let Some((slider_id, track_rect, min_val, max_val)) = self.active_slider_drag {
                    let norm =
                        ((self.cursor_pos.x - track_rect.x) / track_rect.width).clamp(0.0, 1.0);
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
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: WinitMouseButton::Left,
                ..
            } => {
                if self.preferences_drag_offset.is_some() {
                    self.preferences_drag_offset = None;
                    result.consumed = true;
                    return Some(result);
                }
                if self.active_slider_drag.is_some() {
                    self.active_slider_drag = None;
                    result.consumed = true;
                    return Some(result);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } => {
                let click_point = self.cursor_pos;

                // 1. If an active dropdown popup is open
                if let Some(popup_rect) = targets.active_dropdown_popup_rect {
                    if popup_rect.contains_point(click_point) {
                        if let Some(&(idx, _, _)) = targets
                            .active_dropdown_items
                            .iter()
                            .find(|(_, r, _)| r.contains_point(click_point))
                            && let Some(dd_id) = self.preferences_dropdown
                        {
                            result.preferences_action =
                                Some(PreferencesAction::SelectDropdownItem(dd_id, idx));
                            self.preferences_dropdown = None;
                            result.consumed = true;
                            return Some(result);
                        }
                    } else {
                        self.preferences_dropdown = None;
                    }
                }

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
                        self.active_number_input = Some((slider_id, initial_str));
                        self.active_slider_drag = None;
                        self.preferences_dropdown = None;
                        result.consumed = true;
                        return Some(result);
                    }
                }

                // If clicked outside active number box, commit and close it
                if let Some((slider_id, buffer)) = self.active_number_input.take()
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
                    self.preferences_drag_offset = None;
                    self.active_slider_drag = None;
                    self.preferences_dropdown = None;
                    self.active_number_input = None;
                    result.consumed = true;
                    return Some(result);
                }

                // 4. Titlebar dragging
                if targets.title_bar_rect.contains_point(click_point) {
                    let card_x = targets.card_rect.x;
                    let card_y = targets.card_rect.y;
                    self.preferences_drag_offset =
                        Some(Point::new(click_point.x - card_x, click_point.y - card_y));
                    result.consumed = true;
                    return Some(result);
                }

                // 5. Tab clicks
                for &(tab_idx, tab_rect) in &targets.tabs {
                    if tab_rect.contains_point(click_point) {
                        self.preferences_tab = tab_idx;
                        self.preferences_dropdown = None;
                        self.active_number_input = None;
                        self.preferences_scroll_y = 0.0;
                        result.preferences_action = Some(PreferencesAction::SelectTab(tab_idx));
                        result.consumed = true;
                        return Some(result);
                    }
                }

                // 6. Content Area Interactive Elements (Dropdowns, Toggles, Sliders, Section Toggles)
                if targets.content_rect.contains_point(click_point) {
                    for &(sec_id, sec_rect) in &targets.section_toggles {
                        if sec_rect.contains_point(click_point) {
                            if self.collapsed_sections.contains(sec_id) {
                                self.collapsed_sections.remove(sec_id);
                            } else {
                                self.collapsed_sections.insert(sec_id);
                            }
                            result.preferences_action =
                                Some(PreferencesAction::ToggleSection(sec_id));
                            result.consumed = true;
                            return Some(result);
                        }
                    }

                    for &(dd_id, dd_rect) in &targets.dropdowns {
                        if dd_rect.contains_point(click_point) {
                            if self.preferences_dropdown == Some(dd_id) {
                                self.preferences_dropdown = None;
                            } else {
                                self.preferences_dropdown = Some(dd_id);
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
                            self.active_slider_drag =
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