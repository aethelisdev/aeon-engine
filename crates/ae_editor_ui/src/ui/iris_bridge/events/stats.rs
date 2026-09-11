// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interaction and scrolling subsystem for the Performance Stats & Telemetry panel overlay.

use crate::ui::iris_bridge::stats::StatsPanelAction;
use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles mouse clicks and checkbox toggles for the Performance Stats panel.
    pub(crate) fn handle_stats_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let stats_targets = self.stats_targets.as_ref()?;
        let mut result = IrisOverlayEventResult::default();

        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos;
            if let Some(wire_rect) = stats_targets.wireframe_checkbox_rect
                && wire_rect.contains_point(click_point)
            {
                self.stats_actions.push(StatsPanelAction::ToggleWireframe);
                self.stats_revision = self.stats_revision.wrapping_add(1);
                result.consumed = true;
                return Some(result);
            }
            if let Some(grid_rect) = stats_targets.grid_checkbox_rect
                && grid_rect.contains_point(click_point)
            {
                self.stats_actions.push(StatsPanelAction::ToggleGrid);
                self.stats_revision = self.stats_revision.wrapping_add(1);
                result.consumed = true;
                return Some(result);
            }
            if stats_targets.panel_rect.contains_point(click_point) {
                result.consumed = true;
                return Some(result);
            }
        }

        None
    }

    /// Handles mouse wheel scrolling for docked panels (Stats, Hierarchy, Preferences, Inspector).
    pub(crate) fn handle_panel_mouse_wheel(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let WindowEvent::MouseWheel { delta, .. } = event else {
            return None;
        };

        let mut result = IrisOverlayEventResult::default();
        let delta_y = match delta {
            winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 24.0,
            winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
        };

        if let Some(ref targets) = self.stats_targets
            && targets.panel_rect.contains_point(self.cursor_pos)
        {
            self.stats_scroll_y = (self.stats_scroll_y - delta_y).max(0.0);
            self.stats_revision = self.stats_revision.wrapping_add(1);
            result.consumed = true;
            return Some(result);
        }
        if let Some(ref targets) = self.hierarchy_targets
            && targets.panel_rect.contains_point(self.cursor_pos)
        {
            self.hierarchy_scroll_y = (self.hierarchy_scroll_y - delta_y).max(0.0);
            result.consumed = true;
            return Some(result);
        }
        if let Some(ref targets) = self.preferences_targets
            && targets.card_rect.contains_point(self.cursor_pos)
        {
            self.preferences_scroll_y = (self.preferences_scroll_y - delta_y).max(0.0);
            result.consumed = true;
            return Some(result);
        }
        if let Some(ref targets) = self.inspector_targets
            && targets
                .scroll_container_rect
                .contains_point(self.cursor_pos)
        {
            self.inspector_scroll_y = (self.inspector_scroll_y - delta_y).max(0.0);
            result.consumed = true;
            return Some(result);
        }

        None
    }
}