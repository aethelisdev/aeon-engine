// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Window Event Routing
//!
//! Dispatches mouse clicks, autonomous scrollbar dragging, track jumps,
//! wheel scrolling, and button activations for the Iris UI Material & Surface Studio panel.
//!

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::{
    InteractionEvent, MouseButton, Rect, ScrollAreaFrame, ScrollAreaStyle, handle_scroll_area_event,
};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Routes window events to the Material & Surface Studio panel when active.
    ///
    /// Delegates scrollbar dragging, track jumps, and mouse wheel navigation
    /// to the framework's autonomous [`handle_scroll_area_event`] subsystem.
    pub(crate) fn handle_material_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let panel_rect = self.material.panel_rect?;
        let mut result = IrisOverlayEventResult::default();
        let cursor = self.cursor_pos();

        // 0. Autonomous ScrollArea Event Routing (Thumb Drag, Track Jump, Wheel Scroll)
        if let Some(ui_event) = super::winit_to_ui_event(event, cursor) {
            let vp_h = (panel_rect.height - super::super::material::header::MATERIAL_HEADER_HEIGHT)
                .max(10.0);
            let total_content_height = vp_h + self.material.max_scroll_y;
            let vp_rect = Rect::new(
                panel_rect.x,
                panel_rect.y + super::super::material::header::MATERIAL_HEADER_HEIGHT,
                panel_rect.width,
                vp_h,
            );
            let scroll_frame = ScrollAreaFrame::from_geometry(
                vp_rect,
                total_content_height,
                self.material.interactions.scroll_y,
                &ScrollAreaStyle::dark_default(),
            );

            self.material.scroll_state.scroll_y = self.material.interactions.scroll_y;
            let scroll_res = handle_scroll_area_event(
                &mut self.material.scroll_state,
                &ui_event,
                &scroll_frame,
                None,
            );

            if scroll_res.consumed {
                self.material.interactions.scroll_y = scroll_res.scroll_y;
                if scroll_res.capture_pointer {
                    self.set_pointer_capture(crate::ui::panel_layout::PanelId::MaterialEditor);
                }
                if scroll_res.release_pointer {
                    self.release_pointer_capture();
                }
                self.chrome.needs_scroll_sync = true;
                result.consumed = true;
                return Some(result);
            }
        }

        // 1. Mouse Click handling
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = cursor;

            let hit_opt = self
                .hit_test_panel(
                    crate::ui::panel_layout::PanelId::MaterialEditor,
                    click_point,
                )
                .or_else(|| self.tree.hit_test_target(click_point));

            if let Some(mut hit) = hit_opt {
                if hit.tag == 0 {
                    hit.tag = self.resolve_panel_tag(
                        crate::ui::panel_layout::PanelId::MaterialEditor,
                        hit.id,
                    );
                }

                // Scrollbar thumb and track are handled autonomously above
                if hit.tag == super::super::material::types::MATERIAL_TAG_SCROLLBAR_THUMB
                    || hit.tag == super::super::material::types::MATERIAL_TAG_SCROLLBAR_TRACK
                {
                    result.consumed = true;
                    return Some(result);
                }

                self.material.pending_interaction_events.push((
                    hit.tag,
                    InteractionEvent::Click {
                        button: MouseButton::Left,
                    },
                ));

                if let Some(action) = super::super::material::handle_material_click(
                    hit.tag,
                    self.material.selected_entity,
                    self.material.active_model,
                ) {
                    self.material.actions.push(action);
                    result.consumed = true;
                    return Some(result);
                }
            }

            let is_over_panel = click_point.x >= panel_rect.x
                && click_point.x <= panel_rect.right()
                && click_point.y >= panel_rect.y
                && click_point.y <= panel_rect.bottom();
            if is_over_panel {
                result.consumed = true;
                return Some(result);
            }
        }

        None
    }
}