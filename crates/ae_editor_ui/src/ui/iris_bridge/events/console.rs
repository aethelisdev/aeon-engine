// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Window Event Routing
//!
//! Dispatches mouse clicks, wheel scrolling, and keyboard search filtering
//! for the Iris UI Developer Console panel.
//!

use crate::ui::iris_bridge::console::CONSOLE_TOOLBAR_HEIGHT;
use crate::ui::iris_bridge::console::types::{
    CONSOLE_TAG_SCROLLBAR_THUMB, CONSOLE_TAG_SCROLLBAR_TRACK, CONSOLE_TAG_SEARCH_CLEAR,
    CONSOLE_TAG_SEARCH_INPUT, is_console_tag,
};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::{Rect, ScrollAreaStyle, ScrollBarGeometry};
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
                        self.focus_manager.set_focus_tag(CONSOLE_TAG_SEARCH_INPUT);
                        result.consumed = true;
                        return Some(result);
                    }
                    super::super::console::ConsoleAction::ClearSearch => {
                        self.console.search_input.clear();
                    }
                    super::super::console::ConsoleAction::CopyLog(_) => {}
                }
                self.notifier.tag_all();
                self.chrome.needs_layout_rebuild = true;
                result.consumed = true;
                return Some(result);
            }

            let hit_opt = self
                .hit_test_panel(crate::ui::panel_layout::PanelId::Console, click_point)
                .or_else(|| self.tree.hit_test_target(click_point));

            if let Some(mut hit) = hit_opt {
                if hit.tag == 0 {
                    hit.tag =
                        self.resolve_panel_tag(crate::ui::panel_layout::PanelId::Console, hit.id);
                }
                if is_console_tag(hit.tag) {
                    if hit.tag == CONSOLE_TAG_SCROLLBAR_THUMB {
                        self.console.active_scrollbar_drag =
                            Some((click_point.y, self.console.interactions.scroll_y));
                        self.set_pointer_capture(crate::ui::panel_layout::PanelId::Console);
                        self.console.auto_scroll = false;
                        self.chrome.needs_scroll_sync = true;
                        result.consumed = true;
                        return Some(result);
                    } else if hit.tag == CONSOLE_TAG_SCROLLBAR_TRACK {
                        let track_y = hit.rect.y;
                        let track_h = hit.rect.height.max(10.0);
                        let thumb_h = ((track_h / (self.console.max_scroll_y + track_h)) * track_h)
                            .clamp(20.0, track_h);
                        let new_scroll =
                            irisui::prelude::ScrollBarGeometry::scroll_from_track_click(
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
                        self.set_pointer_capture(crate::ui::panel_layout::PanelId::Console);
                        self.chrome.needs_scroll_sync = true;
                        result.consumed = true;
                        return Some(result);
                    }

                    if hit.tag == CONSOLE_TAG_SEARCH_INPUT {
                        self.focus_manager.set_focus_tag(CONSOLE_TAG_SEARCH_INPUT);
                    } else if hit.tag == CONSOLE_TAG_SEARCH_CLEAR {
                        self.console.search_input.clear();
                    } else {
                        if self.focus_manager.is_tag_focused(CONSOLE_TAG_SEARCH_INPUT) {
                            self.focus_manager.clear_focus();
                        }
                        match hit.tag {
                            crate::ui::iris_bridge::console::CONSOLE_TAG_CLEAR => {
                                if let Ok(mut lock) = ae_editor::editor_logger::LOGGER.logs.lock() {
                                    lock.clear();
                                }
                                ae_editor::editor_logger::LOGGER
                                    .log_count
                                    .store(0, std::sync::atomic::Ordering::Relaxed);
                                self.console.interactions.scroll_y = 0.0;
                                result.clear_console_entries = true;
                            }
                            crate::ui::iris_bridge::console::CONSOLE_TAG_FILTER_ALL => {
                                self.console.filter =
                                    crate::ui::iris_bridge::console::ConsoleFilterLevel::All;
                            }
                            crate::ui::iris_bridge::console::CONSOLE_TAG_FILTER_ERROR => {
                                self.console.filter =
                                    crate::ui::iris_bridge::console::ConsoleFilterLevel::Error;
                            }
                            crate::ui::iris_bridge::console::CONSOLE_TAG_FILTER_WARN => {
                                self.console.filter =
                                    crate::ui::iris_bridge::console::ConsoleFilterLevel::Warn;
                            }
                            crate::ui::iris_bridge::console::CONSOLE_TAG_FILTER_INFO => {
                                self.console.filter =
                                    crate::ui::iris_bridge::console::ConsoleFilterLevel::Info;
                            }
                            crate::ui::iris_bridge::console::CONSOLE_TAG_FILTER_DEBUG => {
                                self.console.filter =
                                    crate::ui::iris_bridge::console::ConsoleFilterLevel::Debug;
                            }
                            crate::ui::iris_bridge::console::CONSOLE_TAG_AUTOSCROLL => {
                                self.console.auto_scroll = !self.console.auto_scroll;
                            }
                            _ => {}
                        }
                    }

                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                    result.consumed = true;
                    return Some(result);
                }
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
            self.release_pointer_capture();
            self.chrome.needs_scroll_sync = true;
            result.consumed = true;
            return Some(result);
        }

        // 1c. Scrollbar continuous mouse drag
        if let WindowEvent::CursorMoved { .. } = event
            && let Some((start_y, start_scroll)) = self.console.active_scrollbar_drag
        {
            let cursor = self.cursor_pos();
            let delta_y = cursor.y - start_y;
            let panel_rect = self
                .console
                .panel_rect
                .unwrap_or(Rect::new(0.0, 0.0, 300.0, 200.0));
            let vp_h = (panel_rect.height - CONSOLE_TOOLBAR_HEIGHT).max(10.0);
            let total_h = vp_h + self.console.max_scroll_y;
            let vp_rect = Rect::new(0.0, CONSOLE_TOOLBAR_HEIGHT, panel_rect.width, vp_h);
            let scroll_style = ScrollAreaStyle::dark_default();
            if let Some(geom) =
                ScrollBarGeometry::compute_vertical(vp_rect, total_h, start_scroll, &scroll_style)
            {
                let delta_scroll = ScrollBarGeometry::scroll_from_thumb_drag(
                    delta_y,
                    geom.track_rect.height,
                    geom.thumb_rect.height,
                    self.console.max_scroll_y,
                );
                self.console.interactions.scroll_y =
                    (start_scroll + delta_scroll).clamp(0.0, self.console.max_scroll_y);
                self.chrome.needs_scroll_sync = true;
                result.consumed = true;
                return Some(result);
            }
        }

        // 2. Mouse Wheel scroll handling
        let cursor = self.cursor_pos();
        let is_over_console = self
            .hit_test_panel(crate::ui::panel_layout::PanelId::Console, cursor)
            .is_some_and(|hit| is_console_tag(hit.tag))
            || self
                .tree
                .hit_test_target(cursor)
                .is_some_and(|hit| is_console_tag(hit.tag));

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
            self.chrome.needs_scroll_sync = true;
            result.consumed = true;
            return Some(result);
        }

        None
    }
}