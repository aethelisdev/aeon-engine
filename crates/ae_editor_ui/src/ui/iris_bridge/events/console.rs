// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console Window Event Routing
//!
//! Dispatches mouse clicks, autonomous scrollbar dragging, track jumps,
//! wheel scrolling, and keyboard search filtering for the Iris UI Developer Console panel.
//!

use crate::ui::iris_bridge::console::CONSOLE_TOOLBAR_HEIGHT;
use crate::ui::iris_bridge::console::types::{
    CONSOLE_TAG_SEARCH_CLEAR, CONSOLE_TAG_SEARCH_INPUT, is_console_tag,
};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::{Rect, ScrollAreaFrame, ScrollAreaStyle, handle_scroll_area_event};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Routes window events to the Developer Console panel when active.
    ///
    /// Delegates scrollbar dragging, track jumps, and mouse wheel navigation
    /// to the framework's autonomous [`handle_scroll_area_event`] subsystem.
    pub(crate) fn handle_console_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let mut result = IrisOverlayEventResult::default();
        let cursor = self.cursor_pos();

        // 1. Autonomous ScrollArea Event Routing (Thumb Drag, Track Jump, Wheel Scroll)
        if let Some(ui_event) = super::winit_to_ui_event(event, cursor) {
            let panel_rect = self
                .console
                .panel_rect
                .unwrap_or(Rect::new(0.0, 0.0, 300.0, 200.0));
            let vp_h = (panel_rect.height - CONSOLE_TOOLBAR_HEIGHT).max(10.0);
            let vp_rect = Rect::new(
                panel_rect.x,
                panel_rect.y + CONSOLE_TOOLBAR_HEIGHT,
                panel_rect.width,
                vp_h,
            );
            let total_content_height = vp_h + self.console.max_scroll_y;
            let scroll_frame = ScrollAreaFrame::from_geometry(
                vp_rect,
                total_content_height,
                self.console.interactions.scroll_y,
                &ScrollAreaStyle::dark_default(),
            );

            // Synchronize active scroll offset into autonomous scroll state
            self.console.scroll_state.scroll_y = self.console.interactions.scroll_y;

            let scroll_res = handle_scroll_area_event(
                &mut self.console.scroll_state,
                &ui_event,
                &scroll_frame,
                None,
            );

            if scroll_res.consumed {
                self.console.interactions.scroll_y = scroll_res.scroll_y;
                self.console.auto_scroll = false;

                if scroll_res.capture_pointer {
                    self.set_pointer_capture(crate::ui::panel_layout::PanelId::Console);
                }
                if scroll_res.release_pointer {
                    self.release_pointer_capture();
                }

                self.chrome.needs_scroll_sync = true;
                result.consumed = true;
                return Some(result);
            }
        }

        // 2. Mouse Click Handling on Console Toolbar and Action Buttons
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = cursor;

            // Direct toolbar action evaluator
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
                        self.set_focus_tag(CONSOLE_TAG_SEARCH_INPUT);
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

            // Semantic tag hit-testing fallback
            let hit_opt = self
                .hit_test_panel(crate::ui::panel_layout::PanelId::Console, click_point)
                .or_else(|| self.tree.hit_test_target(click_point));

            if let Some(mut hit) = hit_opt {
                if hit.tag == 0 {
                    hit.tag =
                        self.resolve_panel_tag(crate::ui::panel_layout::PanelId::Console, hit.id);
                }
                if is_console_tag(hit.tag) {
                    if hit.tag == CONSOLE_TAG_SEARCH_INPUT {
                        self.set_focus_tag(CONSOLE_TAG_SEARCH_INPUT);
                    } else if hit.tag == CONSOLE_TAG_SEARCH_CLEAR {
                        self.console.search_input.clear();
                    } else {
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

        None
    }
}