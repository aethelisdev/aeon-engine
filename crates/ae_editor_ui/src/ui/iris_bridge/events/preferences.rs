// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event routing logic for the floating Preferences configuration dialog
//! using 100% declarative semantic tags and O(1) hit testing.

use super::super::preferences::{
    PREF_CARD_HEIGHT, PREF_CARD_WIDTH, PreferencesAction, PreferencesTagTarget, TITLEBAR_HEIGHT,
    is_preferences_tag, parse_dropdown_item_tag, resolve_preferences_tag,
};
use super::super::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles active continuous mouse dragging and release interactions for the Preferences dialog.
    ///
    /// Must be invoked at high priority in event dispatch (Step 3b), before the menubar
    /// or docked panels, so that window dragging and scrollbar dragging continue smoothly across
    /// any panel boundary or menubar, and mouse release is reliably captured anywhere on screen.
    pub(crate) fn handle_preferences_drag_events(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        if self.preferences.drag_offset.is_none()
            && self.preferences.active_scrollbar_drag.is_none()
            && self.preferences.active_drag_tag.is_none()
        {
            return None;
        }

        let mut result = IrisOverlayEventResult::default();

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.chrome.cursor_pos = Point::new(position.x as f32, position.y as f32);

                // 1. Generic Declarative Slider / Float Dragging
                if let Some(active_tag) = self.preferences.active_drag_tag {
                    let delta_x = self.cursor_pos().x - self.chrome.last_cursor_pos.x;
                    let delta_y = self.cursor_pos().y - self.chrome.last_cursor_pos.y;
                    self.preferences.pending_interaction_events.push((
                        active_tag,
                        InteractionEvent::Drag {
                            delta: Point::new(delta_x, delta_y),
                        },
                    ));
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                    result.consumed = true;
                    return Some(result);
                }

                // 2. Titlebar Dragging
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

                // 3. Scrollbar Thumb Dragging
                if let Some((start_cursor_y, start_scroll_y)) =
                    self.preferences.active_scrollbar_drag
                {
                    let delta_y = self.cursor_pos().y - start_cursor_y;
                    let max_scroll = self.preferences.max_scroll_y;
                    let content_h = PREF_CARD_HEIGHT - TITLEBAR_HEIGHT;
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
                        self.chrome.needs_scroll_sync = true;
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
                if self.preferences.active_drag_tag.is_some() {
                    self.preferences.active_drag_tag = None;
                    self.notifier.tag_all();
                    result.consumed = true;
                    return Some(result);
                }
                if self.preferences.active_scrollbar_drag.is_some() {
                    self.preferences.active_scrollbar_drag = None;
                    self.chrome.needs_scroll_sync = true;
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
                // If active inline numeric input is editing
                if let Some((tag, ref mut buffer, ref mut is_all_selected)) =
                    self.preferences.active_number_input
                {
                    if *key == winit::keyboard::KeyCode::Enter
                        || *key == winit::keyboard::KeyCode::NumpadEnter
                    {
                        let committed_tag = tag;
                        let committed_text = buffer.clone();
                        self.preferences.active_number_input = None;
                        self.preferences.pending_interaction_events.push((
                            committed_tag,
                            InteractionEvent::TextInput {
                                text: committed_text,
                            },
                        ));
                        self.notifier.tag_all();
                        self.chrome.needs_layout_rebuild = true;
                        result.consumed = true;
                        return Some(result);
                    } else if *key == winit::keyboard::KeyCode::Escape {
                        self.preferences.active_number_input = None;
                        self.notifier.tag_all();
                        self.chrome.needs_layout_rebuild = true;
                        result.consumed = true;
                        return Some(result);
                    } else if *key == winit::keyboard::KeyCode::Backspace {
                        if *is_all_selected {
                            buffer.clear();
                            *is_all_selected = false;
                        } else {
                            buffer.pop();
                        }
                        self.notifier.tag_all();
                        self.chrome.needs_layout_rebuild = true;
                        result.consumed = true;
                        return Some(result);
                    } else if let Some(input_chars) = text {
                        if *is_all_selected {
                            buffer.clear();
                            *is_all_selected = false;
                        }
                        for ch in input_chars.chars() {
                            if ch.is_ascii_digit() || ch == '.' || ch == '-' {
                                buffer.push(ch);
                            }
                        }
                        self.notifier.tag_all();
                        self.chrome.needs_layout_rebuild = true;
                        result.consumed = true;
                        return Some(result);
                    }
                }

                if *key == winit::keyboard::KeyCode::Escape {
                    if self.preferences.dropdown.is_some() {
                        self.preferences.dropdown = None;
                        self.preferences.dropdown_trigger_rect = None;
                    } else {
                        result.close_preferences = true;
                        self.preferences.drag_offset = None;
                        self.preferences.active_scrollbar_drag = None;
                    }
                    result.consumed = true;
                    return Some(result);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let cursor = self.cursor_pos();
                let is_over_card = cursor.x >= card_rect.x
                    && cursor.x <= card_rect.right()
                    && cursor.y >= card_rect.y
                    && cursor.y <= card_rect.bottom();
                if !self.is_point_over_popup(cursor) && is_over_card {
                    let scroll_y = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 28.0,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };
                    self.preferences.scroll_y = (self.preferences.scroll_y - scroll_y)
                        .clamp(0.0, self.preferences.max_scroll_y);
                    self.chrome.needs_scroll_sync = true;
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

                // Commit and dismiss any active number input on clicking elsewhere
                if let Some((input_tag, buffer, _)) = self.preferences.active_number_input.take() {
                    let clicked_same_tag = hit_target.as_ref().is_some_and(|h| h.tag == input_tag);
                    if !clicked_same_tag {
                        self.preferences
                            .pending_interaction_events
                            .push((input_tag, InteractionEvent::TextInput { text: buffer }));
                    }
                }

                // Dispatch declarative two-way property interactions and begin drag tracking
                if let Some(ref hit) = hit_target
                    && hit.tag != 0
                {
                    self.preferences.pending_interaction_events.push((
                        hit.tag,
                        InteractionEvent::Click {
                            button: MouseButton::Left,
                        },
                    ));
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;

                    if hit.role == WidgetRole::NumericInput {
                        if hit.cursor == Some(WidgetCursor::Text) {
                            let initial_text = self
                                .tree
                                .get(hit.id)
                                .and_then(|node| {
                                    if let Some(ref t) = node.text {
                                        Some(t.as_str())
                                    } else {
                                        node.children.first().and_then(|&child_id| {
                                            self.tree.get(child_id).and_then(|c| c.text.as_deref())
                                        })
                                    }
                                })
                                .unwrap_or("")
                                .chars()
                                .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                                .collect::<String>();
                            self.preferences.active_number_input =
                                Some((hit.tag, initial_text, true));
                        } else {
                            self.preferences.active_drag_tag = Some(hit.tag);
                        }
                    }
                }

                // Semantic Tag Hit Routing
                if let Some(ref hit) = hit_target
                    && let Some(target) = resolve_preferences_tag(hit.tag)
                {
                    match target {
                        PreferencesTagTarget::Close => {
                            result.close_preferences = true;
                            self.preferences.drag_offset = None;
                            self.preferences.dropdown = None;
                            self.preferences.dropdown_trigger_rect = None;
                            result.consumed = true;
                            return Some(result);
                        }
                        PreferencesTagTarget::Titlebar => {
                            self.preferences.drag_offset = Some(Point::new(
                                click_point.x - card_rect.x,
                                click_point.y - card_rect.y,
                            ));
                            result.consumed = true;
                            return Some(result);
                        }
                        PreferencesTagTarget::Tab(tab_idx) => {
                            self.preferences.tab = tab_idx;
                            self.preferences.dropdown = None;
                            self.preferences.dropdown_trigger_rect = None;
                            self.preferences.scroll_y = 0.0;
                            self.preferences.active_scrollbar_drag = None;
                            self.notifier.tag_all();
                            result.preferences_action = Some(PreferencesAction::SelectTab(tab_idx));
                            result.consumed = true;
                            return Some(result);
                        }
                        PreferencesTagTarget::ScrollbarThumb => {
                            self.preferences.active_scrollbar_drag =
                                Some((click_point.y, self.preferences.scroll_y));
                            self.chrome.needs_scroll_sync = true;
                            result.consumed = true;
                            return Some(result);
                        }
                        PreferencesTagTarget::ScrollbarTrack => {
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
                                self.chrome.needs_scroll_sync = true;
                                result.consumed = true;
                                return Some(result);
                            }
                        }
                        PreferencesTagTarget::Section(sec_id) => {
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
                        PreferencesTagTarget::Dropdown(dd_id) => {
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
                        PreferencesTagTarget::Toggle(toggle_id) => {
                            result.preferences_action = Some(PreferencesAction::Toggle(toggle_id));
                            result.consumed = true;
                            return Some(result);
                        }
                        PreferencesTagTarget::DropdownItem(_) => {}
                    }
                }

                if let Some(ref hit) = hit_target
                    && is_preferences_tag(hit.tag)
                {
                    result.consumed = true;
                    return Some(result);
                }

                // If click is inside card rect, consume it so it doesn't pass through to canvas
                let is_over_card = click_point.x >= card_rect.x
                    && click_point.x <= card_rect.right()
                    && click_point.y >= card_rect.y
                    && click_point.y <= card_rect.bottom();
                if is_over_card {
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
    let max_x = (screen_width - PREF_CARD_WIDTH).max(0.0);
    let max_y = (screen_height - PREF_CARD_HEIGHT).max(28.0);
    let new_x = (cursor_pos.x - drag_offset.x).clamp(0.0, max_x);
    let new_y = (cursor_pos.y - drag_offset.y).clamp(28.0, max_y);
    Point::new(new_x, new_y)
}