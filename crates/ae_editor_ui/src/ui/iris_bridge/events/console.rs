// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Window Event Routing
//!
//! Dispatches mouse clicks, wheel scrolling, and keyboard search filtering
//! for the Iris UI Developer Console panel.
//!

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Routes window events to the Developer Console panel when active.
    ///
    /// Returns `Some(IrisOverlayEventResult)` if the event was consumed by the console.
    pub(crate) fn handle_console_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let mut result = IrisOverlayEventResult::default();

        // 1. Mouse Click handling
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos();
            if let Some(action) =
                super::super::console::handle_console_click(&self.tree, click_point)
            {
                match action {
                    super::super::console::ConsoleAction::ClearLogs => {
                        if let Ok(mut lock) = ae_editor::editor_logger::LOGGER.logs.lock() {
                            lock.clear();
                        }
                        ae_editor::editor_logger::LOGGER
                            .log_count
                            .store(0, std::sync::atomic::Ordering::Relaxed);
                        self.console.interactions.scroll_y = 0.0;
                        result.clear_console_entries = true;
                    }
                    super::super::console::ConsoleAction::SetFilter(level) => {
                        self.console.filter = level;
                    }
                    super::super::console::ConsoleAction::ToggleAutoScroll => {
                        self.console.auto_scroll = !self.console.auto_scroll;
                    }
                    super::super::console::ConsoleAction::FocusSearch => {
                        self.console.interactions.is_search_focused = true;
                    }
                    super::super::console::ConsoleAction::ClearSearch => {
                        self.console.interactions.search_query.clear();
                    }
                    super::super::console::ConsoleAction::CopyLog(_) => {}
                }
                self.notifier.tag_all();
                self.chrome.needs_layout_rebuild = true;
                result.consumed = true;
                return Some(result);
            }

            if let Some(hit) = self.tree.hit_test_target(click_point)
                && irisui::prelude::is_console_tag(hit.tag)
            {
                if hit.tag == irisui::prelude::CONSOLE_TAG_SCROLLBAR_THUMB {
                    self.console.active_scrollbar_drag =
                        Some((click_point.y, self.console.interactions.scroll_y));
                    self.console.auto_scroll = false;
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                    result.consumed = true;
                    return Some(result);
                } else if hit.tag == irisui::prelude::CONSOLE_TAG_SCROLLBAR_TRACK {
                    let track_y = hit.rect.y;
                    let track_h = hit.rect.height.max(10.0);
                    let thumb_h = ((track_h / (self.console.max_scroll_y + track_h)) * track_h)
                        .clamp(20.0, track_h);
                    let new_scroll = irisui::prelude::ScrollBarGeometry::scroll_from_track_click(
                        click_point.y,
                        track_y,
                        track_h,
                        thumb_h,
                        self.console.max_scroll_y,
                    );
                    self.console.interactions.scroll_y =
                        new_scroll.clamp(0.0, self.console.max_scroll_y);
                    self.console.auto_scroll = false;
                    self.console.active_scrollbar_drag =
                        Some((click_point.y, self.console.interactions.scroll_y));
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                    result.consumed = true;
                    return Some(result);
                }

                if hit.tag != irisui::prelude::CONSOLE_TAG_SEARCH_INPUT {
                    self.console.interactions.is_search_focused = false;
                }
                self.notifier.tag_all();
                self.chrome.needs_layout_rebuild = true;
                result.consumed = true;
                return Some(result);
            }
        }

        // 1b. Scrollbar drag release
        if let WindowEvent::MouseInput {
            state: ElementState::Released,
            button: WinitMouseButton::Left,
            ..
        } = event
            && self.console.active_scrollbar_drag.is_some()
        {
            self.console.active_scrollbar_drag = None;
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
            result.consumed = true;
            return Some(result);
        }

        // 1c. Scrollbar continuous mouse drag
        if let WindowEvent::CursorMoved { .. } = event
            && let Some((start_y, start_scroll)) = self.console.active_scrollbar_drag
        {
            let cursor = self.cursor_pos();
            let delta_y = cursor.y - start_y;
            let mut track_h = 200.0_f32;
            let mut thumb_h = 30.0_f32;
            if let Some(root_id) = self.tree.root() {
                self.tree.traverse_depth_first(root_id, &mut |_id, node| {
                    if node.tag == irisui::prelude::CONSOLE_TAG_SCROLLBAR_TRACK {
                        track_h = node.computed_rect.height;
                    } else if node.tag == irisui::prelude::CONSOLE_TAG_SCROLLBAR_THUMB {
                        thumb_h = node.computed_rect.height;
                    }
                });
            }
            let delta_scroll = irisui::prelude::ScrollBarGeometry::scroll_from_thumb_drag(
                delta_y,
                track_h,
                thumb_h,
                self.console.max_scroll_y,
            );
            self.console.interactions.scroll_y =
                (start_scroll + delta_scroll).clamp(0.0, self.console.max_scroll_y);
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
            result.consumed = true;
            return Some(result);
        }

        // 2. Mouse Wheel scroll handling
        let cursor = self.cursor_pos();
        let is_over_console = self
            .console
            .panel_rect
            .map_or(false, |rect| rect.contains_point(cursor))
            || self
                .tree
                .hit_test_target(cursor)
                .map_or(false, |hit| irisui::prelude::is_console_tag(hit.tag));

        if let WindowEvent::MouseWheel { delta, .. } = event
            && is_over_console
        {
            let delta_lines = match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 24.0,
            };
            super::super::console::handle_console_scroll(
                delta_lines,
                self.console.max_scroll_y,
                &mut self.console.interactions.scroll_y,
                &mut self.console.auto_scroll,
            );
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
            result.consumed = true;
            return Some(result);
        }

        // 3. Search query typing when search input is focused
        if self.console.is_search_focused {
            match event {
                WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                    self.console.search_query.push_str(text);
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                    result.consumed = true;
                    return Some(result);
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
                } => match *key {
                    winit::keyboard::KeyCode::Escape
                    | winit::keyboard::KeyCode::Enter
                    | winit::keyboard::KeyCode::NumpadEnter => {
                        self.console.is_search_focused = false;
                        self.notifier.tag_all();
                        self.chrome.needs_layout_rebuild = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    winit::keyboard::KeyCode::Backspace => {
                        self.console.search_query.pop();
                        self.notifier.tag_all();
                        self.chrome.needs_layout_rebuild = true;
                        result.consumed = true;
                        return Some(result);
                    }
                    _ => {
                        if let Some(t) = text
                            && !t.chars().any(|c| c.is_control())
                        {
                            self.console.search_query.push_str(t);
                            self.notifier.tag_all();
                            self.chrome.needs_layout_rebuild = true;
                            result.consumed = true;
                            return Some(result);
                        }
                    }
                },
                _ => {}
            }
        }

        None
    }
}