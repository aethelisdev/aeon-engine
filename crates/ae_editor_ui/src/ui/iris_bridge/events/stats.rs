// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interaction and scrolling subsystem for the Performance Stats & Telemetry panel overlay.
//!
//! Evaluates clicks and toggles directly via hardware hit-testing ([`UiTree::hit_test_target`])
//! and semantic tags without retaining coordinate rectangles.
//!

use crate::ui::iris_bridge::stats::{is_stats_tag, resolve_stats_action};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::{ScrollAreaFrame, ScrollAreaStyle, handle_scroll_area_event};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles mouse clicks, checkbox toggles, and mouse wheel scrolling for the Performance Stats panel.
    ///
    /// Delegates scrollbar dragging and mouse wheel navigation directly to the
    /// framework's autonomous [`handle_scroll_area_event`] subsystem.
    pub(crate) fn handle_stats_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let panel_rect = self.stats.last_rect?;
        let mut result = IrisOverlayEventResult::default();
        let cursor = self.cursor_pos();

        // 1. Autonomous ScrollArea Event Routing (Wheel Scrolling, Track Jump & Thumb Dragging)
        if let Some(ui_event) = super::adapter::winit_to_ui_event(event, cursor) {
            let vp_h = panel_rect.height.max(10.0);
            let total_content_height = vp_h + self.stats.max_scroll;
            let scroll_frame = ScrollAreaFrame::from_geometry(
                panel_rect,
                total_content_height,
                self.stats.scroll_state.scroll_y,
                &ScrollAreaStyle::dark_default(),
            );

            let scroll_res = handle_scroll_area_event(
                &mut self.stats.scroll_state,
                &ui_event,
                &scroll_frame,
                None,
            );

            if scroll_res.consumed {
                self.stats.sync_scroll_y();
                if scroll_res.capture_pointer {
                    self.set_pointer_capture(crate::ui::panel_layout::PanelId::Stats);
                }
                if scroll_res.release_pointer {
                    self.release_pointer_capture();
                }
                self.notifier
                    .tag_redraw(crate::ui::panel_layout::PanelId::Stats);
                self.chrome.needs_scroll_sync = true;
                self.chrome.needs_layout_rebuild = true;
                result.consumed = true;
                return Some(result);
            }
        }

        // 2. Mouse click handling
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos();

            let hit_opt = self
                .hit_test_panel(crate::ui::panel_layout::PanelId::Stats, click_point)
                .or_else(|| self.tree.hit_test_target(click_point));

            if let Some(hit) = hit_opt {
                let effective_tag = if hit.tag != 0 {
                    hit.tag
                } else {
                    self.resolve_panel_tag(crate::ui::panel_layout::PanelId::Stats, hit.id)
                };

                if let Some(action) = resolve_stats_action(effective_tag) {
                    self.stats.actions.push(action);
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                    result.consumed = true;
                    return Some(result);
                }

                if is_stats_tag(effective_tag) {
                    result.consumed = true;
                    return Some(result);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::stats::types::{
        STATS_TAG_TOGGLE_GRID, STATS_TAG_TOGGLE_WIREFRAME, StatsPanelAction, StatsPanelState,
    };
    use irisui::prelude::{Point, Rect, ScrollAreaFrame, ScrollAreaStyle, UiEvent};

    #[test]
    fn test_stats_panel_scroll_state_synchronization() {
        let mut stats = StatsPanelState {
            max_scroll: 500.0,
            ..Default::default()
        };
        stats.scroll_state.scroll_y = 75.0;

        stats.sync_scroll_y();
        assert_eq!(stats.interactions.scroll_y, 75.0);
        assert_eq!(stats.scroll_y, 75.0);
    }

    #[test]
    fn test_stats_panel_autonomous_scroll_wheel_and_bounds() {
        let panel_rect = Rect::new(100.0, 100.0, 300.0, 400.0);
        let mut stats = StatsPanelState {
            last_rect: Some(panel_rect),
            max_scroll: 500.0,
            ..Default::default()
        };

        let vp_h = panel_rect.height.max(10.0);
        let total_content_height = vp_h + stats.max_scroll;
        let scroll_frame = ScrollAreaFrame::from_geometry(
            panel_rect,
            total_content_height,
            stats.scroll_state.scroll_y,
            &ScrollAreaStyle::dark_default(),
        );

        // 1. Wheel Scroll Down (-1.0 line tick -> raw_delta = -(-1.0) * 28.0 = +28.0px)
        let ui_wheel_down = UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: -1.0,
            point: Point::new(150.0, 150.0),
        };

        let res =
            handle_scroll_area_event(&mut stats.scroll_state, &ui_wheel_down, &scroll_frame, None);
        assert!(res.consumed);
        stats.sync_scroll_y();
        assert_eq!(stats.scroll_state.scroll_y, 28.0);
        assert_eq!(stats.interactions.scroll_y, 28.0);

        // 2. Wheel Scroll past top bound (+10.0 lines tick -> clamps to 0.0)
        let ui_wheel_up = UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: 10.0,
            point: Point::new(150.0, 150.0),
        };
        let res_up =
            handle_scroll_area_event(&mut stats.scroll_state, &ui_wheel_up, &scroll_frame, None);
        assert!(res_up.consumed);
        stats.sync_scroll_y();
        assert_eq!(stats.scroll_state.scroll_y, 0.0);
        assert_eq!(stats.interactions.scroll_y, 0.0);

        // 3. Wheel Scroll past bottom bound (-50.0 lines tick -> clamps to max_scroll = 500.0)
        let ui_wheel_bottom = UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: -50.0,
            point: Point::new(150.0, 150.0),
        };
        let res_bottom = handle_scroll_area_event(
            &mut stats.scroll_state,
            &ui_wheel_bottom,
            &scroll_frame,
            None,
        );
        assert!(res_bottom.consumed);
        stats.sync_scroll_y();
        assert_eq!(stats.scroll_state.scroll_y, 500.0);
        assert_eq!(stats.interactions.scroll_y, 500.0);
    }

    #[test]
    fn test_stats_panel_scroll_outside_viewport_ignored() {
        let panel_rect = Rect::new(100.0, 100.0, 300.0, 400.0);
        let mut stats = StatsPanelState {
            last_rect: Some(panel_rect),
            max_scroll: 500.0,
            ..Default::default()
        };

        let vp_h = panel_rect.height.max(10.0);
        let total_content_height = vp_h + stats.max_scroll;
        let scroll_frame = ScrollAreaFrame::from_geometry(
            panel_rect,
            total_content_height,
            stats.scroll_state.scroll_y,
            &ScrollAreaStyle::dark_default(),
        );

        // Cursor OUTSIDE Stats panel (50.0, 50.0)
        let ui_wheel_outside = UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: -28.0,
            point: Point::new(50.0, 50.0),
        };

        let res = handle_scroll_area_event(
            &mut stats.scroll_state,
            &ui_wheel_outside,
            &scroll_frame,
            None,
        );
        assert!(!res.consumed);
        assert_eq!(stats.scroll_state.scroll_y, 0.0);
    }

    #[test]
    fn test_stats_panel_toggle_actions_resolution() {
        assert_eq!(
            resolve_stats_action(STATS_TAG_TOGGLE_WIREFRAME),
            Some(StatsPanelAction::ToggleWireframe)
        );
        assert_eq!(
            resolve_stats_action(STATS_TAG_TOGGLE_GRID),
            Some(StatsPanelAction::ToggleGrid)
        );
        assert_eq!(resolve_stats_action(0x9999), None);
    }
}