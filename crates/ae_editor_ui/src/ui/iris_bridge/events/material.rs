// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Window Event Routing
//!
//! Dispatches mouse clicks, scroll wheel deltas, and button activations
//! for the Iris UI Material & Surface Studio panel.
//!

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use winit::event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};

impl IrisEditorOverlay {
    /// Routes window events to the Material & Surface Studio panel when active.
    /// Returns `Some(IrisOverlayEventResult)` if the event was consumed by the material panel.
    pub(crate) fn handle_material_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let targets = self.material_targets.as_ref()?;
        let mut result = IrisOverlayEventResult::default();

        // 1. Mouse Click handling
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos;
            if let Some(action) = super::super::material::handle_material_click(
                click_point,
                self.material_selected_entity,
                targets,
            ) {
                self.material_actions.push(action);
                result.consumed = true;
                return Some(result);
            }

            if targets.panel_rect.contains_point(click_point) {
                result.consumed = true;
                return Some(result);
            }
        }

        // 2. Mouse Wheel Scroll handling
        if let WindowEvent::MouseWheel { delta, .. } = event
            && targets.panel_rect.contains_point(self.cursor_pos)
        {
            let delta_y = match delta {
                MouseScrollDelta::LineDelta(_, y) => *y,
                MouseScrollDelta::PixelDelta(pos) => (pos.y as f32) / 20.0,
            };

            self.material_scroll_y = super::super::material::handle_material_scroll(
                delta_y,
                self.material_scroll_y,
                targets,
            );
            result.consumed = true;
            return Some(result);
        }

        None
    }
}