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
    /// Returns `Some(IrisOverlayEventResult)` if the event was consumed by the console.
    pub(crate) fn handle_console_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let targets = self.console_targets.as_ref()?;
        let mut result = IrisOverlayEventResult::default();

        // 1. Mouse Click handling
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos;
            if let Some(action) = super::super::console::handle_console_click(targets, click_point)
            {
                match action {
                    super::super::console::ConsoleAction::ClearLogs => {
                        if let Ok(mut lock) = ae_editor::editor_logger::LOGGER.logs.lock() {
                            lock.clear();
                        }
                        ae_editor::editor_logger::LOGGER
                            .log_count
                            .store(0, std::sync::atomic::Ordering::Relaxed);
                        self.console_scroll_y = 0.0;
                        result.clear_console_entries = true;
                    }
                    super::super::console::ConsoleAction::SetFilter(level) => {
                        self.console_filter = level;
                    }
                    super::super::console::ConsoleAction::ToggleAutoScroll => {
                        self.console_auto_scroll = !self.console_auto_scroll;
                    }
                    super::super::console::ConsoleAction::FocusSearch => {
                        self.console_is_search_focused = true;
                    }
                    super::super::console::ConsoleAction::ClearSearch => {
                        self.console_search_query.clear();
                    }
                    super::super::console::ConsoleAction::CopyLog(_) => {}
                }
                result.consumed = true;
                return Some(result);
            }

            if targets.panel_rect.contains_point(click_point) {
                if !targets.search_input_rect.contains_point(click_point) {
                    self.console_is_search_focused = false;
                }
                result.consumed = true;
                return Some(result);
            }
        }

        // 2. Mouse Wheel scroll handling
        if let WindowEvent::MouseWheel { delta, .. } = event
            && targets.panel_rect.contains_point(self.cursor_pos)
        {
            let delta_lines = match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 24.0,
            };
            super::super::console::handle_console_scroll(
                targets,
                self.cursor_pos,
                delta_lines,
                &mut self.console_scroll_y,
                &mut self.console_auto_scroll,
            );
            result.consumed = true;
            return Some(result);
        }

        // 3. Search query typing when search input is focused
        if self.console_is_search_focused {
            match event {
                WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                    self.console_search_query.push_str(text);
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
                        self.console_is_search_focused = false;
                        result.consumed = true;
                        return Some(result);
                    }
                    winit::keyboard::KeyCode::Backspace => {
                        self.console_search_query.pop();
                        result.consumed = true;
                        return Some(result);
                    }
                    _ => {
                        if let Some(t) = text
                            && !t.chars().any(|c| c.is_control())
                        {
                            self.console_search_query.push_str(t);
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