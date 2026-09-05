// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Window Event Routing
//!
//! Dispatches mouse clicks, playhead needle dragging, and transport controls
//! for the Iris UI Animation Timeline Studio panel.
//!

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Routes window events to the Animation Timeline Studio panel when active.
    /// Returns `Some(IrisOverlayEventResult)` if the event was consumed by the timeline.
    pub(crate) fn handle_timeline_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let targets = self.timeline_targets.as_ref()?;
        let mut result = IrisOverlayEventResult::default();

        // 1. Mouse Click handling
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos;
            if let Some((action, start_dragging)) = super::super::timeline::handle_timeline_click(
                targets,
                click_point,
                self.timeline_selected_entity,
            ) {
                if start_dragging {
                    self.timeline_is_dragging = true;
                }
                self.timeline_actions.push(action);
                result.consumed = true;
                return Some(result);
            }

            if targets.panel_rect.contains_point(click_point) {
                result.consumed = true;
                return Some(result);
            }
        }

        // 2. Mouse Release handling (terminate playhead dragging)
        if let WindowEvent::MouseInput {
            state: ElementState::Released,
            button: WinitMouseButton::Left,
            ..
        } = event
            && self.timeline_is_dragging
        {
            self.timeline_is_dragging = false;
            result.consumed = true;
            return Some(result);
        }

        // 3. Cursor movement while dragging scrubber
        if let WindowEvent::CursorMoved { .. } = event
            && self.timeline_is_dragging
            && let Some(action) =
                super::super::timeline::handle_timeline_drag(targets, self.cursor_pos)
        {
            self.timeline_actions.push(action);
            result.consumed = true;
            return Some(result);
        }

        None
    }
}