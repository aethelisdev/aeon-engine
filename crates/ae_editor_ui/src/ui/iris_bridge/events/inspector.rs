// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interaction, numeric dragging, color picking, and text editing subsystem for the Scene Inspector panel overlay.

use crate::ui::iris_bridge::inspector::tags::TAG_INSPECTOR_DROPDOWN_ITEM_BASE;
use crate::ui::iris_bridge::inspector::{self, ComponentCategory, InspectorAction};
use crate::ui::iris_bridge::types::{
    InspectorColorDragMode, InspectorNumberDragState, IrisEditorOverlay, IrisOverlayEventResult,
};
use crate::ui::workbench::render::inspector_actions::read_inspector_number_value;
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Dispatches mouse wheel scrolling, keyboard editing, menu hover, and click interactions for the docked Scene Inspector.
    pub(crate) fn handle_inspector_window_event(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
    ) -> Option<IrisOverlayEventResult> {
        // 0. Mouse Wheel Scrolling for Inspector scrollable cards container
        if let WindowEvent::MouseWheel { delta, .. } = event {
            let mut result = IrisOverlayEventResult::default();
            let delta_y = match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 24.0,
                winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
            };
            let old_scroll = self.inspector.scroll_y;
            self.inspector.scroll_y = (self.inspector.scroll_y - delta_y).max(0.0);
            if (self.inspector.scroll_y - old_scroll).abs() > 0.001 {
                self.notifier
                    .tag_redraw(crate::ui::panel_layout::PanelId::Inspector);
                self.chrome.needs_scroll_sync = true;
            }
            result.consumed = true;
            return Some(result);
        }

        // 1. Add Component Menu Hover & Submenu Cascade
        if let Some(hover_res) = self.handle_inspector_menu_hover(event) {
            return Some(hover_res);
        }

        // 2. Keyboard / IME Input for active text, numeric, rename, or hex fields
        if let Some(key_res) = self.handle_inspector_keyboard_input(event) {
            return Some(key_res);
        }

        // 3. Mouse Click Interactions (Dropdowns, Color Picker, Number/Text focus, Buttons)
        self.handle_inspector_click_event(event, world)
    }

    /// Handles hover events cascading submenus inside the Add Component popup.
    fn handle_inspector_menu_hover(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        if let WindowEvent::CursorMoved { .. } = event
            && self.inspector.is_add_menu_open
            && let Some(hit) = self
                .overlay_tree
                .tree()
                .hit_test_target(self.cursor_pos())
                .or_else(|| self.tree.hit_test_target(self.cursor_pos()))
            && hit.layer == UiLayer::Popup
        {
            if hit.role == WidgetRole::DropdownItem
                && let Some(cat) = ComponentCategory::from_tag(hit.tag)
                && self.inspector.active_submenu != Some(cat)
            {
                self.inspector.active_submenu = Some(cat);
                self.chrome.needs_layout_rebuild = true;
                self.notifier.tag_all();
            }
            return Some(IrisOverlayEventResult {
                consumed: true,
                ..Default::default()
            });
        }
        None
    }

    /// Handles mouse click interactions inside the Inspector panel.
    fn handle_inspector_click_event(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
    ) -> Option<IrisOverlayEventResult> {
        let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button,
            ..
        } = event
        else {
            return None;
        };

        let mut result = IrisOverlayEventResult::default();
        let click_point = self.cursor_pos();
        let ui_button = match button {
            WinitMouseButton::Left => MouseButton::Left,
            WinitMouseButton::Right => MouseButton::Right,
            WinitMouseButton::Middle => MouseButton::Middle,
            _ => MouseButton::Left,
        };

        let entity_opt = self.inspector.inspected_entity;

        // 1. Check if an active dropdown popup is open and clicked
        if let Some(active_dd) = self.inspector.active_dropdown {
            if let Some(hit) = self
                .overlay_tree
                .tree()
                .hit_test_target(click_point)
                .or_else(|| self.tree.hit_test_target(click_point))
            {
                if hit.layer == UiLayer::Popup && hit.role == WidgetRole::DropdownItem {
                    let opt_idx = (hit.tag - TAG_INSPECTOR_DROPDOWN_ITEM_BASE) as usize;
                    if let Some(entity) = entity_opt {
                        self.inspector
                            .interactions
                            .actions
                            .push(InspectorAction::SelectDropdown(entity, active_dd, opt_idx));
                    }
                    self.inspector.active_dropdown = None;
                    result.consumed = true;
                    return Some(result);
                }
                if hit.layer == UiLayer::Popup && hit.role == WidgetRole::DropdownPopup {
                    result.consumed = true;
                    return Some(result);
                }
            }
            self.inspector.active_dropdown = None;
        }

        // 1c. Check if Add Component Cascading Menu is open and clicked
        if self.inspector.is_add_menu_open
            && ui_button == MouseButton::Left
            && let Some(hit) = self
                .overlay_tree
                .tree()
                .hit_test_target(click_point)
                .or_else(|| self.tree.hit_test_target(click_point))
            && hit.layer == UiLayer::Popup
        {
            if hit.role == WidgetRole::DropdownItem {
                if let Some(cat) = ComponentCategory::from_tag(hit.tag) {
                    self.inspector.active_submenu = Some(cat);
                    self.notifier.tag_all();
                    result.consumed = true;
                    return Some(result);
                } else if let Some(comp_name) =
                    inspector::add_menu::resolve_component_name_from_tag(hit.tag)
                {
                    if let Some(entity) = entity_opt {
                        self.inspector
                            .interactions
                            .actions
                            .push(InspectorAction::AddComponent(entity, comp_name));
                    }
                    self.inspector.is_add_menu_open = false;
                    self.inspector.active_submenu = None;
                    self.notifier.tag_all();
                    result.consumed = true;
                    return Some(result);
                }
            }
            // Clicked inside popup container background
            result.consumed = true;
            return Some(result);
        }

        // 1b. Check if 2D HSV Color Picker is open and clicked
        if self.inspector.is_color_picker_open
            && let Some(hit) = self
                .overlay_tree
                .tree()
                .hit_test_target(click_point)
                .or_else(|| self.tree.hit_test_target(click_point))
            && hit.layer == UiLayer::Popup
        {
            if inspector::tags::resolve_color_picker_close_tag(hit.tag) {
                self.inspector.is_color_picker_open = false;
                if let Some(entity) = entity_opt {
                    self.inspector
                        .interactions
                        .actions
                        .push(InspectorAction::CommitColorEdit(entity));
                }
                result.consumed = true;
                return Some(result);
            }
            if inspector::tags::resolve_color_picker_sv_box_tag(hit.tag) {
                let mut state = HsvColorPickerState {
                    hue: self.inspector.hsv[0],
                    saturation: self.inspector.hsv[1],
                    value: self.inspector.hsv[2],
                    alpha: 1.0,
                };
                state.update_from_sv_point(click_point, hit.rect);
                self.inspector.hsv = [state.hue, state.saturation, state.value];
                let color = state.to_color();
                if let Some(entity) = entity_opt {
                    self.inspector
                        .interactions
                        .actions
                        .push(InspectorAction::StartColorEdit(entity));
                    self.inspector
                        .interactions
                        .actions
                        .push(InspectorAction::LiveSetObjectColor(entity, color));
                }
                self.inspector.color_drag_mode = Some(InspectorColorDragMode::SaturationValue);
                result.consumed = true;
                return Some(result);
            }
            if inspector::tags::resolve_color_picker_hue_bar_tag(hit.tag) {
                let mut state = HsvColorPickerState {
                    hue: self.inspector.hsv[0],
                    saturation: self.inspector.hsv[1],
                    value: self.inspector.hsv[2],
                    alpha: 1.0,
                };
                state.update_from_hue_point(click_point, hit.rect);
                self.inspector.hsv = [state.hue, state.saturation, state.value];
                let color = state.to_color();
                if let Some(entity) = entity_opt {
                    self.inspector
                        .interactions
                        .actions
                        .push(InspectorAction::StartColorEdit(entity));
                    self.inspector
                        .interactions
                        .actions
                        .push(InspectorAction::LiveSetObjectColor(entity, color));
                }
                self.inspector.color_drag_mode = Some(InspectorColorDragMode::Hue);
                result.consumed = true;
                return Some(result);
            }
            if inspector::tags::resolve_color_picker_card_tag(hit.tag) {
                result.consumed = true;
                return Some(result);
            }
        }

        // Close color picker or add menu if clicked outside
        if self.inspector.is_color_picker_open {
            let hit = self
                .overlay_tree
                .tree()
                .hit_test_target(click_point)
                .or_else(|| self.tree.hit_test_target(click_point));
            let inside_picker = hit.is_some_and(|h| {
                h.layer == UiLayer::Popup
                    && (inspector::tags::resolve_color_picker_card_tag(h.tag)
                        || inspector::tags::resolve_color_picker_sv_box_tag(h.tag)
                        || inspector::tags::resolve_color_picker_hue_bar_tag(h.tag)
                        || inspector::tags::resolve_color_picker_close_tag(h.tag))
            });
            if !inside_picker {
                self.inspector.is_color_picker_open = false;
                if let Some(entity) = entity_opt {
                    self.inspector
                        .interactions
                        .actions
                        .push(InspectorAction::CommitColorEdit(entity));
                }
            }
        }
        if self.inspector.is_add_menu_open {
            self.inspector.is_add_menu_open = false;
            self.inspector.active_submenu = None;
        }

        // 2. Resolve interactive UiTree node hit (O(1) semantic tag)
        let hit_opt = self
            .hit_test_panel(crate::ui::panel_layout::PanelId::Inspector, click_point)
            .or_else(|| self.tree.hit_test_target(click_point));

        if let Some(mut hit) = hit_opt {
            if hit.tag == 0 {
                hit.tag =
                    self.resolve_panel_tag(crate::ui::panel_layout::PanelId::Inspector, hit.id);
            }
            // 2a. Entity Name Input
            if inspector::resolve_entity_name_input_tag(hit.tag) {
                if let Some(entity) = entity_opt {
                    if let Some((cur_ent, ref mut input)) = self.inspector.rename_input {
                        if cur_ent == entity && input.has_selection() {
                            input.clear_selection();
                            self.inspector.rename_is_all_selected = false;
                        } else if cur_ent != entity {
                            let current_name = self.inspector.inspected_entity_name.clone();
                            self.inspector.start_rename(entity, &current_name);
                        }
                    } else {
                        let current_name = self.inspector.inspected_entity_name.clone();
                        self.inspector.start_rename(entity, &current_name);
                    }
                }
                self.focus_manager
                    .set_focus_tag(inspector::entity_name_input_tag());
                self.inspector.active_number_input = None;
                self.inspector.active_text_input = None;
                self.inspector.active_text_input_state = None;
                self.inspector.hex_buffer = None;
                result.consumed = true;
                return Some(result);
            }

            // 2c. Number Inputs (Transform & Components, zero rectangular loops)
            if let Some((num_id, min_val, max_val, sensitivity)) =
                inspector::resolve_inspector_number_input_tag(hit.tag)
            {
                if let Some((ent, input)) = self.inspector.rename_input.take() {
                    let final_name = input.text().trim().to_string();
                    if !final_name.is_empty() {
                        self.inspector
                            .interactions
                            .actions
                            .push(InspectorAction::RenameEntity(ent, final_name));
                    }
                    self.inspector.clear_rename();
                }
                if let Some(prev) = self.inspector.active_number_input.take()
                    && prev.id != num_id
                {
                    if let Ok(v) =
                        inspector::evaluate_inspector_math(&prev.buffer, prev.initial_val)
                    {
                        let clamped_v = prev.id.clamp_value(v.clamp(prev.min_val, prev.max_val));
                        self.inspector
                            .interactions
                            .actions
                            .push(InspectorAction::SetNumberValue(
                                prev.entity,
                                prev.id,
                                clamped_v,
                            ));
                        self.inspector
                            .interactions
                            .actions
                            .push(InspectorAction::CommitNumberEdit(prev.entity, prev.id));
                    } else {
                        self.inspector.edit_start_snapshot = None;
                    }
                }

                if let Some(entity) = entity_opt {
                    let cur_val = match num_id {
                        inspector::InspectorNumberInputId::RotX => {
                            self.inspector.inspector_euler[0]
                        }
                        inspector::InspectorNumberInputId::RotY => {
                            self.inspector.inspector_euler[1]
                        }
                        inspector::InspectorNumberInputId::RotZ => {
                            self.inspector.inspector_euler[2]
                        }
                        _ => world
                            .map(|w| read_inspector_number_value(w, entity, num_id))
                            .unwrap_or(0.0),
                    };

                    self.inspector
                        .interactions
                        .actions
                        .push(InspectorAction::StartNumberEdit(entity, num_id));
                    self.inspector.drag_number = Some(InspectorNumberDragState::new(
                        entity,
                        num_id,
                        click_point.x,
                        cur_val,
                        min_val,
                        max_val,
                        sensitivity,
                    ));
                }
                result.consumed = true;
                return Some(result);
            }

            // 2e. Component Text Inputs
            if let Some(text_id) = inspector::resolve_inspector_text_input_tag(hit.tag) {
                if let Some((prev_ent, prev_id, prev_buf)) = self.inspector.active_text_input.take()
                    && prev_id != text_id
                {
                    self.inspector
                        .interactions
                        .actions
                        .push(InspectorAction::SetTextValue(prev_ent, prev_id, prev_buf));
                    self.inspector.active_text_input_state = None;
                }
                let cur_val = self
                    .tree
                    .get(hit.id)
                    .and_then(|node| {
                        if let Some(ref t) = node.text {
                            Some(t.clone())
                        } else {
                            node.children
                                .first()
                                .and_then(|&cid| self.tree.get(cid).and_then(|c| c.text.clone()))
                        }
                    })
                    .unwrap_or_default();

                if let Some(entity) = entity_opt {
                    let mut input = irisui::prelude::TextInputState::new(&cur_val);
                    input.select_all();
                    self.inspector.active_text_input_state = Some((entity, text_id, input));
                    self.inspector.active_text_input = Some((entity, text_id, cur_val));
                }
                self.focus_manager.set_focus_tag(hit.tag);
                self.inspector.active_number_input = None;
                if let Some((ent, input)) = self.inspector.rename_input.take() {
                    let final_name = input.text().trim().to_string();
                    if !final_name.is_empty() {
                        self.inspector
                            .interactions
                            .actions
                            .push(InspectorAction::RenameEntity(ent, final_name));
                    }
                    self.inspector.clear_rename();
                }
                self.inspector.hex_buffer = None;
                result.consumed = true;
                return Some(result);
            }

            // 2f. All other semantic tags (Delete button, Checkbox, Dropdown pill, Color swatch, Hex input, Swatches, etc.)
            let mut actions = Vec::new();
            if inspector::resolve_inspector_tag_click(
                hit.tag,
                entity_opt,
                &self.inspector.saved_swatches,
                &mut actions,
            ) {
                for action in actions {
                    match action {
                        InspectorAction::OpenAddComponentMenu(_) => {
                            if self.inspector.is_add_menu_open {
                                self.inspector.is_add_menu_open = false;
                                self.inspector.active_submenu = None;
                            } else {
                                self.inspector.is_add_menu_open = true;
                                self.inspector.active_submenu = None;
                                self.menubar.active_menu = None;
                                self.hierarchy.is_add_menu_open = false;
                                self.hierarchy.active_context_menu = None;
                            }
                        }
                        InspectorAction::SelectDropdown(_ent, dd_id, _) => {
                            if self.inspector.active_dropdown == Some(dd_id) {
                                self.inspector.active_dropdown = None;
                            } else {
                                self.inspector.active_dropdown = Some(dd_id);
                            }
                        }
                        InspectorAction::FocusRename => {
                            if let Some(entity) = entity_opt {
                                let current_name = self.inspector.inspected_entity_name.clone();
                                self.inspector.start_rename(entity, &current_name);
                            }
                            self.inspector.active_number_input = None;
                            self.inspector.active_text_input = None;
                            self.inspector.hex_buffer = None;
                        }
                        InspectorAction::FocusHexInput => {
                            if let Some(entity) = entity_opt {
                                self.inspector.hex_buffer = Some((entity, String::from("#")));
                            }
                            self.inspector.active_number_input = None;
                            self.inspector.active_text_input = None;
                        }
                        InspectorAction::ToggleColorPicker => {
                            self.inspector.is_color_picker_open =
                                !self.inspector.is_color_picker_open;
                            self.inspector.active_number_input = None;
                            self.inspector.active_text_input = None;
                            self.inspector.hex_buffer = None;
                        }
                        other => {
                            self.inspector.interactions.actions.push(other);
                        }
                    }
                }
                result.consumed = true;
                return Some(result);
            }
        }

        // Commit active number input if clicked outside
        if let Some(session) = self.inspector.active_number_input.take() {
            if let Ok(v) = inspector::evaluate_inspector_math(&session.buffer, session.initial_val)
            {
                let clamped_v = session
                    .id
                    .clamp_value(v.clamp(session.min_val, session.max_val));
                self.inspector
                    .interactions
                    .actions
                    .push(InspectorAction::SetNumberValue(
                        session.entity,
                        session.id,
                        clamped_v,
                    ));
                self.inspector
                    .interactions
                    .actions
                    .push(InspectorAction::CommitNumberEdit(
                        session.entity,
                        session.id,
                    ));
            } else {
                self.inspector.edit_start_snapshot = None;
            }
        }

        // Commit active text input if clicked outside
        if let Some((ent, id, buf)) = self.inspector.active_text_input.take() {
            self.inspector
                .interactions
                .actions
                .push(InspectorAction::SetTextValue(ent, id, buf));
            self.inspector.active_text_input_state = None;
        }

        // Commit active rename input if clicked outside
        if let Some((ent, input)) = self.inspector.rename_input.take() {
            let final_name = input.text().trim().to_string();
            if !final_name.is_empty() {
                self.inspector
                    .interactions
                    .actions
                    .push(InspectorAction::RenameEntity(ent, final_name));
            }
            self.inspector.clear_rename();
        }
        self.focus_manager.clear_focus();

        // Commit active hex input if clicked outside
        if let Some((ent, buf)) = self.inspector.hex_buffer.take() {
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
                self.inspector
                    .interactions
                    .actions
                    .push(InspectorAction::SetObjectColor(
                        ent,
                        Color::rgba(r, g, b, 1.0),
                    ));
            }
        }

        None
    }
}