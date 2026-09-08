// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interaction, numeric dragging, color picking, and text editing subsystem for the Scene Inspector panel overlay.

use crate::ui::iris_bridge::inspector::{self, InspectorAction};
use crate::ui::iris_bridge::types::{
    InspectorColorDragMode, InspectorNumberDragState, InspectorNumberInputSession,
    IrisEditorOverlay, IrisOverlayEventResult,
};
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Dispatches active continuous mouse dragging and release events across the entire window for the Inspector.
    pub(crate) fn handle_inspector_drag_events(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        // 1. Continuous Mouse Motion (Numeric drag and HSV color picker drag)
        if let Some(motion_res) = self.handle_inspector_drag_motion(event) {
            return Some(motion_res);
        }

        // 2. Mouse Release (Commit or transition numeric / color dragging)
        if let Some(release_res) = self.handle_inspector_mouse_release(event) {
            return Some(release_res);
        }

        None
    }

    /// Dispatches keyboard editing, menu hover, and click interactions for the docked Scene Inspector.
    pub(crate) fn handle_inspector_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        // 1. Add Component Menu Hover & Submenu Cascade
        self.handle_inspector_menu_hover(event);

        // 2. Keyboard / IME Input for active text, numeric, rename, or hex fields
        if let Some(key_res) = self.handle_inspector_keyboard_input(event) {
            return Some(key_res);
        }

        // 3. Mouse Click Interactions (Dropdowns, Color Picker, Number/Text focus, Buttons)
        self.handle_inspector_click_event(event)
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

        // Continuous horizontal mouse drag for Inspector numeric fields
        if let Some(ref mut drag) = self.inspector_drag_number {
            let delta = self.cursor_pos.x - drag.start_x;
            if delta.abs() > 2.0 {
                drag.has_dragged = true;
                self.inspector_active_number_input = None;
                let speed_mult = if self.shift_held {
                    5.0
                } else if self.alt_held {
                    0.1
                } else {
                    1.0
                };
                let new_val = (drag.start_val + delta * drag.sensitivity * speed_mult)
                    .clamp(drag.min_val, drag.max_val);
                self.inspector_actions.push(InspectorAction::SetNumberValue(
                    drag.entity,
                    drag.id,
                    new_val,
                ));
                result.consumed = true;
                return Some(result);
            }
        }

        // Continuous mouse drag for Inspector 2D HSV Color Picker
        if let Some(mode) = self.inspector_color_drag_mode
            && let Some(ref insp) = self.inspector_targets
            && let Some(entity) = insp.inspected_entity
        {
            match mode {
                InspectorColorDragMode::SaturationValue => {
                    if let Some(sv_rect) = insp.color_picker_sv_box_rect {
                        let s = ((self.cursor_pos.x - sv_rect.x) / sv_rect.width).clamp(0.0, 1.0);
                        let v = (1.0 - (self.cursor_pos.y - sv_rect.y) / sv_rect.height)
                            .clamp(0.0, 1.0);
                        self.inspector_hsv[1] = s;
                        self.inspector_hsv[2] = v;
                        let col = hsv_to_rgb(self.inspector_hsv[0], s, v);
                        self.inspector_actions
                            .push(InspectorAction::SetObjectColor(entity, col));
                        result.consumed = true;
                        return Some(result);
                    }
                }
                InspectorColorDragMode::Hue => {
                    if let Some(hue_rect) = insp.color_picker_hue_bar_rect {
                        let h = (((self.cursor_pos.y - hue_rect.y) / hue_rect.height) * 360.0)
                            .clamp(0.0, 360.0);
                        self.inspector_hsv[0] = h;
                        let col = hsv_to_rgb(h, self.inspector_hsv[1], self.inspector_hsv[2]);
                        self.inspector_actions
                            .push(InspectorAction::SetObjectColor(entity, col));
                        result.consumed = true;
                        return Some(result);
                    }
                }
            }
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
        if self.inspector_color_drag_mode.take().is_some() {
            result.consumed = true;
            return Some(result);
        }
        if let Some(drag) = self.inspector_drag_number.take() {
            if !drag.has_dragged {
                // Click in-place without dragging -> activate direct numeric text editing with Select All
                let initial_str = if drag.start_val.fract().abs() < 1e-4 {
                    format!("{:.1}", drag.start_val)
                } else {
                    format!("{:.3}", drag.start_val)
                };
                let cursor_idx = initial_str.len();
                self.inspector_active_number_input = Some(InspectorNumberInputSession {
                    entity: drag.entity,
                    id: drag.id,
                    buffer: initial_str,
                    cursor_idx,
                    is_all_selected: true,
                    initial_val: drag.start_val,
                    min_val: drag.min_val,
                    max_val: drag.max_val,
                });
            } else {
                self.inspector_actions
                    .push(InspectorAction::CommitNumberEdit(drag.entity, drag.id));
            }
            result.consumed = true;
            return Some(result);
        }

        None
    }

    /// Handles hover events cascading submenus inside the Add Component popup.
    fn handle_inspector_menu_hover(&mut self, event: &WindowEvent) {
        if let WindowEvent::CursorMoved { .. } = event
            && let Some(ref targets) = self.inspector_targets
            && self.inspector_is_add_menu_open
        {
            let in_submenu = targets
                .active_submenu_rect
                .is_some_and(|r| r.contains_point(self.cursor_pos));
            let in_add_menu = targets
                .active_add_menu_rect
                .is_some_and(|r| r.contains_point(self.cursor_pos));

            if !in_submenu && in_add_menu {
                for &(cat, item_rect) in &targets.add_menu_categories {
                    if item_rect.contains_point(self.cursor_pos) {
                        self.inspector_active_submenu = Some(cat);
                        break;
                    }
                }
            }
        }
    }

    /// Handles mouse click interactions inside the Inspector panel.
    fn handle_inspector_click_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let insp_targets = self.inspector_targets.as_ref()?;
        let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button,
            ..
        } = event
        else {
            return None;
        };

        let mut result = IrisOverlayEventResult::default();
        let click_point = self.cursor_pos;
        let ui_button = match button {
            WinitMouseButton::Left => MouseButton::Left,
            WinitMouseButton::Right => MouseButton::Right,
            WinitMouseButton::Middle => MouseButton::Middle,
            _ => MouseButton::Left,
        };

        let entity_opt = insp_targets.inspected_entity;

        // 1. Check if an active dropdown popup is open and clicked
        if let Some(active_dd) = self.inspector_active_dropdown {
            if let Some(popup_rect) = insp_targets.active_dropdown_popup_rect
                && popup_rect.contains_point(click_point)
            {
                for &(opt_idx, item_rect) in &insp_targets.dropdown_items {
                    if item_rect.contains_point(click_point) {
                        if let Some(entity) = entity_opt {
                            self.inspector_actions
                                .push(InspectorAction::SelectDropdown(entity, active_dd, opt_idx));
                        }
                        self.inspector_active_dropdown = None;
                        result.consumed = true;
                        return Some(result);
                    }
                }
                result.consumed = true;
                return Some(result);
            }
            self.inspector_active_dropdown = None;
        }

        // 1b. Check if 2D HSV Color Picker is open and clicked
        if self.inspector_is_color_picker_open {
            if let Some(close_btn) = insp_targets.color_picker_close_btn_rect
                && close_btn.contains_point(click_point)
            {
                self.inspector_is_color_picker_open = false;
                result.consumed = true;
                return Some(result);
            }

            if let Some(sv_rect) = insp_targets.color_picker_sv_box_rect
                && sv_rect.contains_point(click_point)
            {
                let s = ((click_point.x - sv_rect.x) / sv_rect.width).clamp(0.0, 1.0);
                let v = (1.0 - (click_point.y - sv_rect.y) / sv_rect.height).clamp(0.0, 1.0);
                self.inspector_hsv[1] = s;
                self.inspector_hsv[2] = v;
                let col = hsv_to_rgb(self.inspector_hsv[0], s, v);
                if let Some(entity) = entity_opt {
                    self.inspector_actions
                        .push(InspectorAction::SetObjectColor(entity, col));
                }
                self.inspector_color_drag_mode = Some(InspectorColorDragMode::SaturationValue);
                result.consumed = true;
                return Some(result);
            }

            if let Some(hue_rect) = insp_targets.color_picker_hue_bar_rect
                && hue_rect.contains_point(click_point)
            {
                let h =
                    (((click_point.y - hue_rect.y) / hue_rect.height) * 360.0).clamp(0.0, 360.0);
                self.inspector_hsv[0] = h;
                let col = hsv_to_rgb(h, self.inspector_hsv[1], self.inspector_hsv[2]);
                if let Some(entity) = entity_opt {
                    self.inspector_actions
                        .push(InspectorAction::SetObjectColor(entity, col));
                }
                self.inspector_color_drag_mode = Some(InspectorColorDragMode::Hue);
                result.consumed = true;
                return Some(result);
            }
        }

        // 2. Check if a string text input box is clicked
        for &(text_id, box_rect, ref cur_val) in &insp_targets.text_inputs {
            if box_rect.contains_point(click_point) {
                if let Some((prev_ent, prev_id, prev_buf)) = self.inspector_active_text_input.take()
                    && prev_id != text_id
                {
                    self.inspector_actions
                        .push(InspectorAction::SetTextValue(prev_ent, prev_id, prev_buf));
                }
                if let Some(entity) = entity_opt {
                    self.inspector_active_text_input = Some((entity, text_id, cur_val.clone()));
                }
                self.inspector_active_number_input = None;
                self.inspector_rename_buffer = None;
                self.inspector_hex_buffer = None;
                result.consumed = true;
                return Some(result);
            }
        }

        // 2b. Check if a number input box is clicked
        for &(num_id, box_rect, min_val, max_val, cur_val) in &insp_targets.number_inputs {
            if box_rect.contains_point(click_point) {
                if let Some(prev) = self.inspector_active_number_input.take()
                    && prev.id != num_id
                {
                    if let Ok(v) =
                        inspector::evaluate_inspector_math(&prev.buffer, prev.initial_val)
                    {
                        let clamped_v = prev.id.clamp_value(v.clamp(prev.min_val, prev.max_val));
                        self.inspector_actions.push(InspectorAction::SetNumberValue(
                            prev.entity,
                            prev.id,
                            clamped_v,
                        ));
                        self.inspector_actions
                            .push(InspectorAction::CommitNumberEdit(prev.entity, prev.id));
                    } else {
                        self.inspector_edit_start_snapshot = None;
                    }
                }
                let sensitivity = match num_id {
                    inspector::InspectorNumberInputId::UiOffsetX
                    | inspector::InspectorNumberInputId::UiOffsetY
                    | inspector::InspectorNumberInputId::UiSizeW
                    | inspector::InspectorNumberInputId::UiSizeH => 1.0,
                    inspector::InspectorNumberInputId::UiFontSize
                    | inspector::InspectorNumberInputId::UiBorderWidth
                    | inspector::InspectorNumberInputId::UiCornerRadius
                    | inspector::InspectorNumberInputId::UiProgressVal
                    | inspector::InspectorNumberInputId::UiProgressMin
                    | inspector::InspectorNumberInputId::UiProgressMax
                    | inspector::InspectorNumberInputId::UiZIndex => 0.5,
                    inspector::InspectorNumberInputId::UiAlpha
                    | inspector::InspectorNumberInputId::UiPivotX
                    | inspector::InspectorNumberInputId::UiPivotY => 0.01,
                    inspector::InspectorNumberInputId::RotX
                    | inspector::InspectorNumberInputId::RotY
                    | inspector::InspectorNumberInputId::RotZ
                    | inspector::InspectorNumberInputId::CharacterMaxSlope => 0.5,
                    inspector::InspectorNumberInputId::ScaleX
                    | inspector::InspectorNumberInputId::ScaleY
                    | inspector::InspectorNumberInputId::ScaleZ => 0.01,
                    inspector::InspectorNumberInputId::PosX
                    | inspector::InspectorNumberInputId::PosY
                    | inspector::InspectorNumberInputId::PosZ => 0.1,
                    inspector::InspectorNumberInputId::RigidBodyMass
                    | inspector::InspectorNumberInputId::RigidBodyGravity
                    | inspector::InspectorNumberInputId::ColliderBoxX
                    | inspector::InspectorNumberInputId::ColliderBoxY
                    | inspector::InspectorNumberInputId::ColliderBoxZ
                    | inspector::InspectorNumberInputId::ColliderHalfHeight
                    | inspector::InspectorNumberInputId::ColliderRadius
                    | inspector::InspectorNumberInputId::ColliderCenterY => 0.05,
                    _ => 0.05,
                };
                if let Some(entity) = entity_opt {
                    self.inspector_actions
                        .push(InspectorAction::StartNumberEdit(entity, num_id));
                    self.inspector_drag_number = Some(InspectorNumberDragState {
                        entity,
                        id: num_id,
                        start_x: click_point.x,
                        start_val: cur_val,
                        min_val,
                        max_val,
                        sensitivity,
                        has_dragged: false,
                    });
                }
                result.consumed = true;
                return Some(result);
            }
        }

        // Commit active number input if clicked outside
        if let Some(session) = self.inspector_active_number_input.take() {
            if let Ok(v) = inspector::evaluate_inspector_math(&session.buffer, session.initial_val)
            {
                let clamped_v = session
                    .id
                    .clamp_value(v.clamp(session.min_val, session.max_val));
                self.inspector_actions.push(InspectorAction::SetNumberValue(
                    session.entity,
                    session.id,
                    clamped_v,
                ));
                self.inspector_actions
                    .push(InspectorAction::CommitNumberEdit(
                        session.entity,
                        session.id,
                    ));
            } else {
                self.inspector_edit_start_snapshot = None;
            }
        }

        // Commit active text input if clicked outside
        if let Some((ent, id, buf)) = self.inspector_active_text_input.take() {
            self.inspector_actions
                .push(InspectorAction::SetTextValue(ent, id, buf));
        }

        // Commit active hex input if clicked outside
        if let Some(ref hex_rect) = insp_targets.hex_input_rect
            && !hex_rect.contains_point(click_point)
            && let Some((ent, buf)) = self.inspector_hex_buffer.take()
        {
            let clean_hex = buf.trim_start_matches('#');
            if (clean_hex.len() == 6 || clean_hex.len() == 3)
                && let Ok(rgb) = u32::from_str_radix(clean_hex, 16)
            {
                let (r, g, b) = if clean_hex.len() == 6 {
                    (
                        ((rgb >> 16) & 0xFF) as f32 / 255.0,
                        ((rgb >> 8) & 0xFF) as f32 / 255.0,
                        (rgb & 0xFF) as f32 / 255.0,
                    )
                } else {
                    (
                        (((rgb >> 8) & 0xF) * 17) as f32 / 255.0,
                        (((rgb >> 4) & 0xF) * 17) as f32 / 255.0,
                        ((rgb & 0xF) * 17) as f32 / 255.0,
                    )
                };
                self.inspector_actions.push(InspectorAction::SetObjectColor(
                    ent,
                    Color::rgba(r, g, b, 1.0),
                ));
            }
        }

        // Close color picker if clicked outside
        if self.inspector_is_color_picker_open {
            let inside_picker = insp_targets
                .color_picker_popup_rect
                .is_some_and(|r| r.contains_point(click_point));
            let inside_swatch = insp_targets
                .color_swatch_rect
                .is_some_and(|r| r.contains_point(click_point));
            if !inside_picker && !inside_swatch {
                self.inspector_is_color_picker_open = false;
            }
        }

        let mut actions = Vec::new();
        let consumed =
            inspector::handle_inspector_click(click_point, ui_button, insp_targets, &mut actions);

        for action in actions {
            match action {
                InspectorAction::OpenAddComponentMenu(_) => {
                    self.inspector_is_add_menu_open = true;
                    self.inspector_active_submenu = None;
                    self.active_menu = None;
                    self.hierarchy_is_add_menu_open = false;
                    self.hierarchy_active_context_menu = None;
                }
                InspectorAction::CloseAddComponentMenu => {
                    self.inspector_is_add_menu_open = false;
                    self.inspector_active_submenu = None;
                }
                InspectorAction::OpenAddSubmenu(cat) => {
                    self.inspector_active_submenu = Some(cat);
                }
                InspectorAction::CloseAddSubmenu => {
                    self.inspector_active_submenu = None;
                }
                InspectorAction::SelectDropdown(_ent, dd_id, _) => {
                    if self.inspector_active_dropdown == Some(dd_id) {
                        self.inspector_active_dropdown = None;
                    } else {
                        self.inspector_active_dropdown = Some(dd_id);
                    }
                }
                InspectorAction::FocusRename => {
                    if let Some(entity) = entity_opt {
                        self.inspector_rename_buffer = Some((entity, String::new()));
                    }
                }
                InspectorAction::FocusHexInput => {
                    if let Some(entity) = entity_opt {
                        self.inspector_hex_buffer = Some((entity, String::from("#")));
                    }
                }
                InspectorAction::ToggleColorPicker => {
                    self.inspector_is_color_picker_open = !self.inspector_is_color_picker_open;
                }
                other => {
                    self.inspector_actions.push(other);
                }
            }
        }

        if consumed {
            result.consumed = true;
            return Some(result);
        }

        None
    }
}