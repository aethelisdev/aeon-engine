// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event routing logic for the native dock tab overflow dropdown menu.
//!

use super::super::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles mouse hovering, clicks, and dismissal for the active dock tab overflow popup menu.
    pub(crate) fn handle_dock_overflow_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let (active_leaf, _) = self.chrome.active_dock_overflow?;
        let frame = self.chrome.native_dock_frame.as_ref()?;
        let mut result = IrisOverlayEventResult::default();

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.chrome.cursor_pos = Point::new(position.x as f32, position.y as f32);
                if let Some(ref overflow_rect) = frame.active_overflow_rect
                    && overflow_rect.contains_point(self.chrome.cursor_pos)
                {
                    result.consumed = true;
                    return Some(result);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } => {
                let click_point = self.cursor_pos();

                // 1. Check if clicking on any item inside the active overflow dropdown menu
                if let Some(target) = frame
                    .overflow_item_targets
                    .iter()
                    .find(|t| t.rect.contains_point(click_point))
                {
                    result.activate_dock_tab = Some((target.leaf, target.tab_index));
                    self.chrome.active_dock_overflow = None;
                    self.chrome.needs_layout_rebuild = true;
                    result.consumed = true;
                    return Some(result);
                }

                // 2. Check if clicking on the chevron button itself (toggle / close)
                if let Some(target) = frame
                    .chevron_targets
                    .iter()
                    .find(|t| t.rect.contains_point(click_point))
                {
                    if target.leaf == active_leaf {
                        self.chrome.active_dock_overflow = None;
                    } else {
                        self.chrome.active_dock_overflow = Some((target.leaf, target.rect));
                    }
                    self.chrome.needs_layout_rebuild = true;
                    result.consumed = true;
                    return Some(result);
                }

                // 3. Clicked anywhere else outside the overflow popup -> dismiss menu
                self.chrome.active_dock_overflow = None;
                self.chrome.needs_layout_rebuild = true;
                result.consumed = true;
                return Some(result);
            }
            _ => {}
        }

        None
    }
}