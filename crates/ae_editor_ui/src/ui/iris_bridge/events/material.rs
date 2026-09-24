// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Window Event Routing
//!
//! Dispatches mouse clicks, scroll wheel deltas, scrollbar dragging, and button activations
//! for the Iris UI Material & Surface Studio panel.
//!

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};

impl IrisEditorOverlay {
    /// Routes window events to the Material & Surface Studio panel when active.
    ///
    /// Returns `Some(IrisOverlayEventResult)` if the event was consumed by the material panel.
    pub(crate) fn handle_material_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let panel_rect = self.material.panel_rect?;
        let mut result = IrisOverlayEventResult::default();

        // 1. Mouse Click handling
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos();

            if let Some(hit) = self.tree.hit_test_target(click_point) {
                if hit.tag == super::super::material::types::MATERIAL_TAG_SCROLLBAR_THUMB {
                    self.material.active_scrollbar_drag =
                        Some((click_point.y, self.material.interactions.scroll_y));
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                    result.consumed = true;
                    return Some(result);
                } else if hit.tag == super::super::material::types::MATERIAL_TAG_SCROLLBAR_TRACK {
                    let track_y = hit.rect.y;
                    let track_h = hit.rect.height.max(10.0);
                    let thumb_h = ((track_h / (self.material.max_scroll_y + track_h)) * track_h)
                        .clamp(20.0, track_h);
                    let new_scroll = ScrollBarGeometry::scroll_from_track_click(
                        click_point.y,
                        track_y,
                        track_h,
                        thumb_h,
                        self.material.max_scroll_y,
                    );
                    self.material.interactions.scroll_y =
                        new_scroll.clamp(0.0, self.material.max_scroll_y);
                    self.material.active_scrollbar_drag =
                        Some((click_point.y, self.material.interactions.scroll_y));
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
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

            if panel_rect.contains_point(click_point) {
                result.consumed = true;
                return Some(result);
            }
        }

        // 1b. Scrollbar drag release
        if let WindowEvent::MouseInput {
            state: ElementState::Released,
            button: WinitMouseButton::Left,
            ..
        } = event
            && self.material.active_scrollbar_drag.is_some()
        {
            self.material.active_scrollbar_drag = None;
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
            result.consumed = true;
            return Some(result);
        }

        // 1c. Scrollbar continuous mouse drag
        if let WindowEvent::CursorMoved { .. } = event
            && let Some((start_y, start_scroll)) = self.material.active_scrollbar_drag
        {
            let cursor = self.cursor_pos();
            let delta_y = cursor.y - start_y;
            let mut track_h = 200.0_f32;
            let mut thumb_h = 30.0_f32;
            if let Some(root_id) = self.tree.root() {
                self.tree.traverse_depth_first(root_id, &mut |_id, node| {
                    if node.tag == super::super::material::types::MATERIAL_TAG_SCROLLBAR_TRACK {
                        track_h = node.computed_rect.height;
                    } else if node.tag
                        == super::super::material::types::MATERIAL_TAG_SCROLLBAR_THUMB
                    {
                        thumb_h = node.computed_rect.height;
                    }
                });
            }
            let delta_scroll = ScrollBarGeometry::scroll_from_thumb_drag(
                delta_y,
                track_h,
                thumb_h,
                self.material.max_scroll_y,
            );
            self.material.interactions.scroll_y =
                (start_scroll + delta_scroll).clamp(0.0, self.material.max_scroll_y);
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
            result.consumed = true;
            return Some(result);
        }

        // 2. Mouse Wheel Scroll handling
        if let WindowEvent::MouseWheel { delta, .. } = event
            && panel_rect.contains_point(self.cursor_pos())
        {
            let delta_y = match delta {
                MouseScrollDelta::LineDelta(_, y) => *y,
                MouseScrollDelta::PixelDelta(pos) => (pos.y as f32) / 20.0,
            };

            let scroll_step = 24.0;
            self.material.scroll_y = (self.material.scroll_y - delta_y * scroll_step)
                .clamp(0.0, self.material.max_scroll_y);
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
            result.consumed = true;
            return Some(result);
        }

        None
    }
}