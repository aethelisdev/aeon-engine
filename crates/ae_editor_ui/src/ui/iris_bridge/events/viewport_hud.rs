// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interaction and event handling subsystem for the 3D Viewport HUD overlay.

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use crate::ui::iris_bridge::viewport_hud::ViewportHudAction;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles mouse interactions and dropdown selections for the 3D Viewport HUD.
    pub(crate) fn handle_viewport_hud_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let hud_targets = self.viewport_hud_targets.as_ref()?;
        let mut result = IrisOverlayEventResult::default();

        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos;

            // 1. If an active dropdown popup is open
            if self.viewport_hud_dropdown.is_some() {
                for (action, rect, _) in &hud_targets.active_dropdown_items {
                    if rect.contains_point(click_point) {
                        self.viewport_hud_actions.push(action.clone());
                        self.viewport_hud_dropdown = None;
                        result.consumed = true;
                        return Some(result);
                    }
                }

                if let Some(popup_rect) = hud_targets.active_dropdown_popup_rect
                    && !popup_rect.contains_point(click_point)
                {
                    self.viewport_hud_dropdown = None;
                }
            }

            // 2. Check dropdown triggers
            for (dd_id, rect) in &hud_targets.dropdown_triggers {
                if rect.contains_point(click_point) {
                    self.viewport_hud_dropdown = if self.viewport_hud_dropdown == Some(*dd_id) {
                        None
                    } else {
                        Some(*dd_id)
                    };
                    result.consumed = true;
                    return Some(result);
                }
            }

            // 3. Check toolbar buttons
            for (action, rect) in &hud_targets.buttons {
                if rect.contains_point(click_point) {
                    self.viewport_hud_actions.push(action.clone());
                    result.consumed = true;
                    return Some(result);
                }
            }

            // 4. Check compass knobs
            for (action, rect) in &hud_targets.compass_knobs {
                if rect.contains_point(click_point) {
                    self.viewport_hud_actions.push(action.clone());
                    result.consumed = true;
                    return Some(result);
                }
            }

            // 5. Check billboard icons
            for (ent, rect) in &hud_targets.billboard_icons {
                if rect.contains_point(click_point) {
                    self.viewport_hud_actions
                        .push(ViewportHudAction::SelectEntity(*ent));
                    result.consumed = true;
                    return Some(result);
                }
            }
        }

        None
    }
}