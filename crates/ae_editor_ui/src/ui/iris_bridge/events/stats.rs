// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interaction and scrolling subsystem for the Performance Stats & Telemetry panel overlay.
//!
//! Evaluates clicks and toggles directly via hardware hit-testing ([`UiTree::hit_test_target`])
//! and semantic tags without retaining coordinate rectangles.
//!

use crate::ui::iris_bridge::stats::{is_stats_tag, resolve_stats_action};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles mouse clicks, checkbox toggles, and mouse wheel scrolling for the Performance Stats panel.
    ///
    /// Evaluates target elements directly via hardware hit-testing on the [`UiTree`]
    /// matching against [`STATS_TAG_TOGGLE_WIREFRAME`] and [`STATS_TAG_TOGGLE_GRID`].
    pub(crate) fn handle_stats_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let _ = self.stats.last_rect?;
        let mut result = IrisOverlayEventResult::default();

        // 1. Mouse wheel scrolling
        if let WindowEvent::MouseWheel { delta, .. } = event {
            let delta_y = match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 24.0,
                winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
            };
            let old_scroll = self.stats.scroll_y;
            self.stats.scroll_y = (self.stats.scroll_y - delta_y).clamp(0.0, self.stats.max_scroll);
            if (self.stats.scroll_y - old_scroll).abs() > 0.001 {
                self.notifier
                    .tag_redraw(crate::ui::panel_layout::PanelId::Stats);
                self.chrome.needs_scroll_sync = true;
            }
            result.consumed = true;
            return Some(result);
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