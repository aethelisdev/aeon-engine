// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Window Event Routing
//!
//! Routes window mouse input, canvas dragging, viewport panning, and zoom scrolling
//! to the Iris UI 2D Visual UI Designer panel.
//!

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use crate::ui::iris_bridge::ui_designer::{
    UiDesignerAction, handle_ui_designer_click, handle_ui_designer_drag, handle_ui_designer_scroll,
};
use winit::event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};

impl IrisEditorOverlay {
    /// Routes window events to the 2D Visual UI Designer panel when active.
    /// Returns `Some(IrisOverlayEventResult)` if the event was consumed by the UI Designer.
    pub(crate) fn handle_ui_designer_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let targets = self.ui_designer_targets.as_ref()?;
        let mut result = IrisOverlayEventResult::default();

        // ── 1. Mouse Click Handling ───────────────────────────────────────────
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos;
            let click_res = handle_ui_designer_click(
                click_point,
                targets,
                self.ui_designer_is_aspect_open,
                self.ui_designer_is_add_menu_open,
            );

            if let Some(action) = click_res.action {
                match action {
                    UiDesignerAction::ToggleAspectDropdown => {
                        self.ui_designer_is_aspect_open = !self.ui_designer_is_aspect_open;
                        self.ui_designer_is_add_menu_open = false;
                    }
                    UiDesignerAction::ToggleAddMenu => {
                        self.ui_designer_is_add_menu_open = !self.ui_designer_is_add_menu_open;
                        self.ui_designer_is_aspect_open = false;
                    }
                    UiDesignerAction::ClosePopups => {
                        self.ui_designer_is_aspect_open = false;
                        self.ui_designer_is_add_menu_open = false;
                    }
                    UiDesignerAction::SetAspectRatio(ratio) => {
                        self.ui_designer_is_aspect_open = false;
                        self.ui_designer_actions
                            .push(UiDesignerAction::SetAspectRatio(ratio));
                    }
                    UiDesignerAction::SpawnElement(elem) => {
                        self.ui_designer_is_add_menu_open = false;
                        self.ui_designer_actions
                            .push(UiDesignerAction::SpawnElement(elem));
                    }
                    other => {
                        self.ui_designer_actions.push(other);
                    }
                }
                result.consumed = true;
            }

            if let Some(drag) = click_res.start_element_drag {
                self.ui_designer_drag_state = Some(drag);
                self.ui_designer_last_cursor = click_point;
                result.consumed = true;
            }

            if click_res.start_canvas_pan {
                self.ui_designer_is_panning = true;
                self.ui_designer_last_cursor = click_point;
                result.consumed = true;
            }

            if targets.panel_rect.contains_point(click_point) {
                result.consumed = true;
                return Some(result);
            }
        }

        // ── 2. Mouse Dragging & Panning (CursorMoved) ─────────────────────────
        if let WindowEvent::CursorMoved { .. } = event
            && (self.ui_designer_drag_state.is_some() || self.ui_designer_is_panning)
        {
            let delta = [
                self.cursor_pos.x - self.ui_designer_last_cursor.x,
                self.cursor_pos.y - self.ui_designer_last_cursor.y,
            ];
            self.ui_designer_last_cursor = self.cursor_pos;

            if let Some(drag_action) = handle_ui_designer_drag(
                self.cursor_pos,
                delta,
                self.ui_designer_drag_state.as_ref(),
                self.ui_designer_is_panning,
                targets,
            ) {
                self.ui_designer_actions.push(drag_action);
            }
            result.consumed = true;
            return Some(result);
        }

        // ── 3. Mouse Release (Terminate element dragging and canvas panning) ──
        if let WindowEvent::MouseInput {
            state: ElementState::Released,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let had_drag = self.ui_designer_drag_state.take().is_some();
            let had_pan = std::mem::take(&mut self.ui_designer_is_panning);
            if had_drag || had_pan {
                result.consumed = true;
                return Some(result);
            }
        }

        // ── 4. Mouse Wheel Scroll (Canvas Zoom) ────────────────────────────────
        if let WindowEvent::MouseWheel { delta, .. } = event
            && targets.panel_rect.contains_point(self.cursor_pos)
        {
            let delta_y = match delta {
                MouseScrollDelta::LineDelta(_, y) => *y,
                MouseScrollDelta::PixelDelta(pos) => (pos.y as f32) / 20.0,
            };

            if let Some(scroll_action) =
                handle_ui_designer_scroll(self.cursor_pos, delta_y, targets)
            {
                self.ui_designer_actions.push(scroll_action);
            }
            result.consumed = true;
            return Some(result);
        }

        None
    }
}