// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Continuous mouse dragging and release subsystem for Scene Inspector numeric scrubbers and color pickers.
//!
//! Provides global mouse motion interception, sensitivity modifiers (Shift/Ctrl), Escape cancellation,
//! and in-place click detection for transitioning into direct numeric editing.
//!

use crate::ui::iris_bridge::inspector::{self, InspectorAction};
use crate::ui::iris_bridge::types::{
    InspectorColorDragMode, InspectorNumberInputSession, IrisEditorOverlay, IrisOverlayEventResult,
};
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Dispatches active continuous mouse dragging and release events across the entire window for the Inspector.
    ///
    /// Intercepts Escape key cancellation, continuous horizontal motion, and mouse button release.
    pub(crate) fn handle_inspector_drag_events(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        // 1. Escape key cancellation for active numeric dragging
        if let WindowEvent::KeyboardInput { event, .. } = event
            && event.state == ElementState::Pressed
            && event.logical_key == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape)
            && let Some(mut drag_state) = self.inspector.drag_number.take()
        {
            let res = drag_state.drag.cancel();
            if let Some(val) = res.value {
                let entity = drag_state.entity;
                let id = drag_state.id;
                let clamped_val = id.clamp_value(val as f32);
                self.inspector
                    .interactions
                    .actions
                    .push(InspectorAction::SetNumberValue(entity, id, clamped_val));
            }
            return Some(IrisOverlayEventResult {
                consumed: true,
                ..Default::default()
            });
        }

        // 2. Continuous Mouse Motion (Numeric drag and HSV color picker drag)
        if let Some(motion_res) = self.handle_inspector_drag_motion(event) {
            return Some(motion_res);
        }

        // 3. Mouse Release (Commit or transition numeric / color dragging)
        if let Some(release_res) = self.handle_inspector_mouse_release(event) {
            return Some(release_res);
        }

        None
    }

    /// Handles continuous horizontal mouse dragging for numeric scrubbers and 2D HSV color canvas.
    fn handle_inspector_drag_motion(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let WindowEvent::CursorMoved { .. } = event else {
            return None;
        };
        let mut result = IrisOverlayEventResult::default();

        let cursor_x = self.cursor_pos().x;
        let shift_held = self.chrome.shift_held;

        // Continuous horizontal mouse drag for Inspector numeric fields
        if let Some(ref mut drag_state) = self.inspector.drag_number {
            let res = drag_state
                .drag
                .update(cursor_x, shift_held, self.chrome.ctrl_held);
            if res.has_dragged
                && let Some(new_val) = res.value
            {
                let entity = drag_state.entity;
                let id = drag_state.id;
                let clamped_val = id.clamp_value(new_val as f32);

                match id {
                    inspector::InspectorNumberInputId::RotX => {
                        self.inspector.inspector_euler[0] = clamped_val;
                    }
                    inspector::InspectorNumberInputId::RotY => {
                        self.inspector.inspector_euler[1] = clamped_val;
                    }
                    inspector::InspectorNumberInputId::RotZ => {
                        self.inspector.inspector_euler[2] = clamped_val;
                    }
                    _ => {}
                }

                self.inspector.active_number_input = None;
                self.inspector
                    .interactions
                    .actions
                    .push(InspectorAction::SetNumberValue(entity, id, clamped_val));
                result.consumed = true;
                return Some(result);
            }
        }

        // Continuous mouse drag for Inspector 2D HSV Color Picker
        if let Some(mode) = self.inspector.color_drag_mode
            && let Some(entity) = self.inspector.inspected_entity
        {
            let cursor = self.cursor_pos();
            let mut state = HsvColorPickerState {
                hue: self.inspector.hsv[0],
                saturation: self.inspector.hsv[1],
                value: self.inspector.hsv[2],
                alpha: 1.0,
            };

            match mode {
                InspectorColorDragMode::SaturationValue => {
                    let sv_rect = self
                        .overlay_tree
                        .tree()
                        .iter()
                        .find(|(_, n)| n.tag == inspector::tags::TAG_INSPECTOR_COLOR_PICKER_SV_BOX)
                        .or_else(|| {
                            self.tree.iter().find(|(_, n)| {
                                n.tag == inspector::tags::TAG_INSPECTOR_COLOR_PICKER_SV_BOX
                            })
                        })
                        .map(|(_, n)| n.computed_rect)
                        .unwrap_or(Rect::new(cursor.x - 80.0, cursor.y - 65.0, 160.0, 130.0));
                    state.update_from_sv_point(cursor, sv_rect);
                }
                InspectorColorDragMode::Hue => {
                    let hue_rect = self
                        .overlay_tree
                        .tree()
                        .iter()
                        .find(|(_, n)| n.tag == inspector::tags::TAG_INSPECTOR_COLOR_PICKER_HUE_BAR)
                        .or_else(|| {
                            self.tree.iter().find(|(_, n)| {
                                n.tag == inspector::tags::TAG_INSPECTOR_COLOR_PICKER_HUE_BAR
                            })
                        })
                        .map(|(_, n)| n.computed_rect)
                        .unwrap_or(Rect::new(cursor.x - 9.0, cursor.y - 65.0, 18.0, 130.0));
                    state.update_from_hue_point(cursor, hue_rect);
                }
            }

            self.inspector.hsv = [state.hue, state.saturation, state.value];
            let col = state.to_color();
            self.inspector
                .actions
                .push(InspectorAction::LiveSetObjectColor(entity, col));
            result.consumed = true;
            return Some(result);
        }

        None
    }

    /// Handles mouse release terminating numeric drags and color picker drags.
    fn handle_inspector_mouse_release(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let WindowEvent::MouseInput {
            state: ElementState::Released,
            button: WinitMouseButton::Left,
            ..
        } = event
        else {
            return None;
        };

        let mut result = IrisOverlayEventResult::default();
        if self.inspector.color_drag_mode.take().is_some() {
            if let Some(entity) = self.inspector.inspected_entity {
                self.inspector
                    .actions
                    .push(InspectorAction::CommitColorEdit(entity));
            }
            result.consumed = true;
            return Some(result);
        }
        if let Some(mut drag_state) = self.inspector.drag_number.take() {
            let res = drag_state.drag.finish();
            if res.is_click_in_place {
                // Click in-place without dragging -> activate direct numeric text editing with Select All
                let start_val = drag_state.start_val();
                let initial_str = if start_val.fract().abs() < 1e-4 {
                    format!("{:.1}", start_val)
                } else {
                    format!("{:.3}", start_val)
                };
                let cursor_idx = initial_str.len();
                self.inspector.active_number_input = Some(InspectorNumberInputSession {
                    entity: drag_state.entity,
                    id: drag_state.id,
                    buffer: initial_str,
                    cursor_idx,
                    is_all_selected: true,
                    initial_val: start_val,
                    min_val: drag_state.drag.min_val as f32,
                    max_val: drag_state.drag.max_val as f32,
                });
            } else {
                self.inspector
                    .actions
                    .push(InspectorAction::CommitNumberEdit(
                        drag_state.entity,
                        drag_state.id,
                    ));
            }
            result.consumed = true;
            return Some(result);
        }

        None
    }
}