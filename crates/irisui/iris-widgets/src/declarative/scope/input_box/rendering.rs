// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Input Box Rendering Implementations
//!
//! Provides rendering methods for single-line text input fields, selection capsules,
//! blinking carets, and clear buttons on [`UiScope`].
//!

use super::interaction::{find_cursor_idx_at_x, process_text_input_keys};
use super::types::{InputBoxProps, calculate_visible_slice_and_caret};
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::{ResponseFlags, WidgetResponse};
use iris_core::{
    AlignItems, Color, JustifyContent, Style, TextAlign, WidgetCursor, WidgetId, WidgetRole,
};
use iris_text::measure_text_advance;

impl<'a> UiScope<'a> {
    /// Emits a single-line declarative text input field bound to a mutable string buffer in-place.
    pub fn text_input(
        &mut self,
        placeholder: impl Into<String>,
        text: &mut String,
    ) -> WidgetResponse {
        self.text_input_with_width(placeholder, 0.0, text)
    }

    /// Emits a single-line declarative text input field constrained to an explicit width.
    pub fn text_input_with_width(
        &mut self,
        placeholder: impl Into<String>,
        width: f32,
        text: &mut String,
    ) -> WidgetResponse {
        let placeholder_str = placeholder.into();
        let (visible, _) = crate::declarative::types::split_label_id(&placeholder_str);
        let tag = self.tag_for(&placeholder_str);
        self.text_input_with_width_tagged(visible, tag, width, text)
    }

    /// Emits a single-line declarative text input field with an explicit semantic tag.
    pub fn text_input_tagged(
        &mut self,
        placeholder: impl Into<String>,
        tag: u64,
        text: &mut String,
    ) -> WidgetResponse {
        self.text_input_with_width_tagged(placeholder, tag, 0.0, text)
    }

    /// Emits a single-line declarative text input field with an explicit semantic tag and fixed width.
    ///
    /// Autonomous UX lifecycle:
    /// 1. First click selects the entire text buffer with a blue selection capsule.
    /// 2. Arrow keys or subsequent clicks deactivate all-selected mode and place the caret into cursor mode.
    /// 3. Typing in all-selected mode replaces the selection buffer; Backspace clears it completely.
    /// 4. Pressing Enter submits the input, clears focus, and emits [`WidgetResponse::submitted`].
    pub fn text_input_with_width_tagged(
        &mut self,
        placeholder: impl Into<String>,
        tag: u64,
        width: f32,
        text: &mut String,
    ) -> WidgetResponse {
        let node_id = self.tree.create_node();
        let placeholder_str = placeholder.into();

        let _ = self.tree.add_child(self.parent, node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("TextInput");
            node.tag = tag;
        }

        let edit_key = if tag != 0 { tag } else { node_id.as_ffi() };
        let font_size = 11.0;

        let (clicked, hovered, _) = self.check_interaction(node_id);

        let is_editing_this = self
            .tree
            .event_state
            .active_text_edit
            .as_ref()
            .is_some_and(|(k, ..)| *k == edit_key);

        let is_clicked_outside = if let Some(clicked_tag) = self.tree.event_state.frame_clicked_tag
        {
            tag != 0 && clicked_tag != tag
        } else if let Some(clicked_id) = self.tree.event_state.frame_clicked_id {
            clicked_id != node_id && !self.tree.is_descendant_of(clicked_id, node_id)
        } else {
            false
        };

        if is_clicked_outside && is_editing_this {
            self.tree.event_state.clear_focus();
        }

        if clicked {
            let (cur_idx, all_sel) = if !is_editing_this {
                (text.len(), true)
            } else {
                let mut cur = text.len();
                if let Some(click_pos) = self.tree.event_state.frame_clicked_pos
                    && let Some(node) = self.tree.get(node_id)
                    && node.computed_rect.width > 0.0
                {
                    let rel_x = click_pos.x - (node.computed_rect.x + 8.0);
                    cur = find_cursor_idx_at_x(text, rel_x, font_size);
                }
                (cur, false)
            };
            self.tree
                .event_state
                .start_text_edit(Some(node_id), edit_key, text, cur_idx, all_sel);
        }

        let is_focused = self
            .tree
            .event_state
            .active_text_edit
            .as_ref()
            .is_some_and(|(k, ..)| *k == edit_key);

        if is_focused {
            self.tree.event_state.focused = Some(node_id);
            if tag != 0 {
                self.tree.event_state.focused_tag = Some(tag);
            }
        }

        let mut changed = false;
        let mut submitted = false;

        let (cursor_idx, is_all_sel) = if is_focused {
            let (mut cur, mut all_sel) = self
                .tree
                .event_state
                .active_text_edit
                .as_ref()
                .filter(|(k, ..)| *k == edit_key)
                .map(|(_, _, c, s)| (*c, *s))
                .unwrap_or((text.len(), false));

            let key_pressed = self.tree.event_state.frame_key_pressed;
            let key = self.tree.event_state.frame_key;
            let incoming = self.tree.event_state.frame_text_input.as_deref();

            let res =
                process_text_input_keys(text, &mut cur, &mut all_sel, key_pressed, key, incoming);

            if res.changed {
                changed = true;
            }
            if res.submitted {
                submitted = true;
            }
            if res.should_clear_focus {
                self.tree.event_state.clear_focus();
            } else {
                self.tree
                    .event_state
                    .start_text_edit(Some(node_id), edit_key, text, cur, all_sel);
            }
            (cur, all_sel)
        } else {
            (0, false)
        };

        let is_still_focused = self
            .tree
            .event_state
            .active_text_edit
            .as_ref()
            .is_some_and(|(k, ..)| *k == edit_key);
        let border_color = if is_still_focused {
            Color::rgba(0.0, 0.90, 1.0, 0.95)
        } else if hovered {
            Color::rgba(0.35, 0.40, 0.52, 0.95)
        } else {
            Color::rgba(0.20, 0.23, 0.30, 0.60)
        };

        let bg_color = if hovered || is_still_focused {
            Color::rgba(0.08, 0.09, 0.12, 0.98)
        } else {
            Color::rgba(0.06, 0.07, 0.09, 0.95)
        };

        let mut box_style = Style::new()
            .height(24.0)
            .border_radius(4.0)
            .border(1.0, border_color)
            .hover_border(1.0, Color::rgba(0.35, 0.40, 0.52, 0.95))
            .focus_border(1.0, Color::rgba(0.0, 0.90, 1.0, 0.95))
            .background(bg_color)
            .hover_background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .focus_background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .clip_children(true);

        if width > 0.0 {
            box_style = box_style.width(width);
        } else {
            box_style = box_style.flex_grow(1.0);
        }

        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::TextInput;
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Text);
            node.set_style(box_style);
        }

        if is_still_focused && is_all_sel && !text.is_empty() {
            let sel_w = measure_text_advance(text, font_size).max(4.0);
            let sel_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(sel_id) {
                node.set_name("SelectionCapsule");
                node.interactive = false;
                node.set_style(
                    Style::new()
                        .position_absolute()
                        .left(8.0)
                        .top(3.5)
                        .width(sel_w + 2.0)
                        .height(17.0)
                        .background(Color::rgba(0.14, 0.46, 0.88, 0.95))
                        .border_radius(3.0),
                );
            }
            let _ = self.tree.add_child(node_id, sel_id);
        }

        let text_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(text_id) {
            node.role = WidgetRole::Default;
            node.tag = tag;
            let display_str = if text.is_empty() {
                placeholder_str.as_str()
            } else {
                text.as_str()
            };
            node.set_text(display_str);
            node.font_size = font_size;
            node.line_height = 24.0;
            node.text_align = TextAlign::Left;
            node.text_color = if text.is_empty() {
                Color::rgba(0.42, 0.45, 0.55, 1.0)
            } else if is_still_focused && is_all_sel {
                Color::WHITE
            } else {
                Color::rgba(0.92, 0.94, 0.98, 1.0)
            };
            node.focus_text_color = Some(Color::rgba(0.92, 0.94, 0.98, 1.0));
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(8.0)
                    .top(0.0)
                    .height(24.0),
            );
        }
        let _ = self.tree.add_child(node_id, text_id);

        if is_still_focused {
            let caret_offset = measure_text_advance(&text[..cursor_idx.min(text.len())], font_size);
            let caret_x = 8.0 + caret_offset;
            let caret_id = self.tree.create_node();
            let caret_color = self.theme().text_caret;
            if let Some(node) = self.tree.get_mut(caret_id) {
                node.set_name("TextInputCaret");
                node.role = WidgetRole::TextInputCaret;
                node.tag = tag;
                node.interactive = false;
                node.set_style(
                    Style::new()
                        .position_absolute()
                        .left(caret_x)
                        .top(5.0)
                        .width(1.5)
                        .height(14.0)
                        .background(caret_color)
                        .border_radius(0.75),
                );
            }
            let _ = self.tree.add_child(node_id, caret_id);
        }

        let mut resp = WidgetResponse::new(node_id, clicked, hovered, changed);
        if is_still_focused {
            resp = resp.with_focused(true);
        }
        if submitted {
            resp = resp.with_submitted(true);
        }
        resp
    }

    /// Emits a styled text input display box with placeholder support and an optional blinking caret.
    pub fn input_box(
        &mut self,
        text: &str,
        placeholder: &str,
        text_width: f32,
        cursor_blink_visible: bool,
    ) -> WidgetId {
        let box_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(box_id) {
            node.role = WidgetRole::TextInput;
            node.cursor = Some(WidgetCursor::Text);
            node.set_style(
                Style::new()
                    .height(28.0)
                    .border_radius(4.0)
                    .border(1.0, Color::rgba(0.0, 0.85, 0.95, 0.80))
                    .background(Color::rgba(0.05, 0.05, 0.07, 1.0)),
            );
        }
        let _ = self.tree.add_child(self.parent, box_id);

        let text_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(text_id) {
            node.role = WidgetRole::Default;
            let display = if text.is_empty() { placeholder } else { text };
            node.set_text(display);
            node.font_size = 12.0;
            node.line_height = 28.0;
            node.text_align = TextAlign::Left;
            node.text_color = if text.is_empty() {
                Color::rgba(0.45, 0.45, 0.52, 1.0)
            } else {
                Color::WHITE
            };
        }
        let _ = self.tree.add_child(box_id, text_id);

        if cursor_blink_visible {
            let caret_offset = if text.is_empty() { 0.0_f32 } else { text_width };
            let caret_color = self.theme().text_caret;
            let caret_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(caret_id) {
                node.role = WidgetRole::TextInputCaret;
                node.tag = caret_offset.to_bits() as u64;
                node.set_style(Style::new().background(caret_color).border_radius(0.75));
            }
            let _ = self.tree.add_child(box_id, caret_id);
        }

        box_id
    }

    /// Emits a text input box with placeholder, explicit width, and caret blink indicator, tagged with a semantic tag.
    pub fn input_box_tagged(&mut self, props: &InputBoxProps<'_>, tag: u64) -> WidgetResponse {
        let box_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(box_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, box_id);

        let edit_key = if tag != 0 { tag } else { box_id.as_ffi() };
        let font_size = props.font_size;
        let text_start_x = if props.icon.is_some() { 26.0 } else { 8.0 };

        let (clicked, hovered, _) = self.check_interaction(box_id);

        let is_editing_this = self
            .tree
            .event_state
            .active_text_edit
            .as_ref()
            .is_some_and(|(k, ..)| *k == edit_key);

        let is_clicked_outside = if let Some(clicked_tag) = self.tree.event_state.frame_clicked_tag
        {
            tag != 0 && clicked_tag != tag && Some(clicked_tag) != props.clear_tag
        } else if let Some(clicked_id) = self.tree.event_state.frame_clicked_id {
            clicked_id != box_id && !self.tree.is_descendant_of(clicked_id, box_id)
        } else {
            false
        };

        if is_clicked_outside && is_editing_this {
            self.tree.event_state.clear_text_edit();
        }

        if clicked {
            let (cur_idx, all_sel, initial_text) = if !is_editing_this {
                (props.text.len(), true, props.text.to_string())
            } else {
                let active_buf = self
                    .tree
                    .event_state
                    .text_edit_buffer(edit_key)
                    .unwrap_or(props.text)
                    .to_string();
                let mut cur = active_buf.len();
                if let Some(click_pos) = self.tree.event_state.frame_clicked_pos
                    && let Some(node) = self.tree.get(box_id)
                    && node.computed_rect.width > 0.0
                {
                    let rel_x = click_pos.x - (node.computed_rect.x + text_start_x);
                    cur = find_cursor_idx_at_x(&active_buf, rel_x, font_size);
                }
                (cur, false, active_buf)
            };
            self.tree.event_state.start_text_edit(
                Some(box_id),
                edit_key,
                &initial_text,
                cur_idx,
                all_sel,
            );
        }

        let is_focused = self
            .tree
            .event_state
            .active_text_edit
            .as_ref()
            .is_some_and(|(k, ..)| *k == edit_key);

        if is_focused {
            self.tree.event_state.focused = Some(box_id);
            if tag != 0 {
                self.tree.event_state.focused_tag = Some(tag);
            }
        }

        let mut changed = false;
        let mut submitted = false;

        let (active_text, cursor_idx, is_all_sel) = if is_focused {
            let edit_session = self.tree.event_state.active_text_edit.take();
            if let Some((k, mut buffer, mut cur, mut all_sel)) = edit_session {
                if let Some(clear_tag) = props.clear_tag
                    && self.tree.event_state.frame_clicked_tag == Some(clear_tag)
                {
                    buffer.clear();
                    cur = 0;
                    all_sel = false;
                    changed = true;
                }

                let key_pressed = self.tree.event_state.frame_key_pressed;
                let key = self.tree.event_state.frame_key;
                let incoming = self.tree.event_state.frame_text_input.as_deref();

                let res = process_text_input_keys(
                    &mut buffer,
                    &mut cur,
                    &mut all_sel,
                    key_pressed,
                    key,
                    incoming,
                );

                if res.changed {
                    changed = true;
                }
                if res.submitted {
                    submitted = true;
                }

                if res.should_clear_focus {
                    self.tree.event_state.clear_text_edit();
                    (buffer, cur, all_sel)
                } else {
                    let active_buf_clone = buffer.clone();
                    self.tree.event_state.active_text_edit = Some((k, buffer, cur, all_sel));
                    (active_buf_clone, cur, all_sel)
                }
            } else {
                (props.text.to_string(), props.cursor_byte_idx, false)
            }
        } else {
            (props.text.to_string(), props.cursor_byte_idx, false)
        };

        let is_still_focused = self
            .tree
            .event_state
            .active_text_edit
            .as_ref()
            .is_some_and(|(k, ..)| *k == edit_key);

        let border_color = if is_still_focused || props.is_focused {
            Color::rgba(0.0, 0.90, 1.0, 0.95)
        } else if hovered {
            Color::rgba(0.35, 0.40, 0.52, 0.95)
        } else {
            Color::rgba(0.20, 0.23, 0.30, 0.60)
        };
        let bg_color = if hovered || is_still_focused {
            Color::rgba(0.08, 0.09, 0.12, 0.98)
        } else {
            Color::rgba(0.06, 0.07, 0.09, 0.95)
        };

        let mut box_style = Style::new()
            .height(24.0)
            .border_radius(4.0)
            .border(1.0, border_color)
            .hover_border(1.0, Color::rgba(0.35, 0.40, 0.52, 0.95))
            .focus_border(1.0, Color::rgba(0.0, 0.90, 1.0, 0.95))
            .background(bg_color)
            .hover_background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .focus_background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .clip_children(true);

        if props.width > 0.0 {
            box_style = box_style.width(props.width);
        } else {
            box_style = box_style.flex_grow(1.0);
        }

        if let Some(node) = self.tree.get_mut(box_id) {
            node.role = WidgetRole::TextInput;
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Text);
            node.set_style(box_style);
        }

        if let Some(icon_str) = props.icon {
            let icon_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(icon_id) {
                node.tag = tag;
                node.set_name("SearchIcon");
                node.interactive = false;
                node.role = WidgetRole::TextInputIcon;
                node.set_text(icon_str);
                node.font_size = 11.0;
                node.line_height = 24.0;
                node.text_align = TextAlign::Center;
                node.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
                node.set_style(
                    Style::new()
                        .position_absolute()
                        .left(7.0)
                        .top(0.0)
                        .width(14.0)
                        .height(24.0),
                );
            }
            let _ = self.tree.add_child(box_id, icon_id);
        }

        let pad_end = if props.clear_tag.is_some() && !active_text.is_empty() {
            24.0
        } else {
            8.0
        };
        let max_visible_w = if props.width > 0.0 {
            (props.width - text_start_x - pad_end).max(0.0)
        } else {
            0.0
        };

        let effective_cursor_idx = if is_still_focused {
            cursor_idx
        } else {
            props.cursor_byte_idx
        };

        let (visible_start, visible_end, caret_offset) =
            if !active_text.is_empty() && max_visible_w > 0.0 {
                calculate_visible_slice_and_caret(
                    &active_text,
                    effective_cursor_idx,
                    font_size,
                    max_visible_w,
                )
            } else if !active_text.is_empty() {
                let valid_idx = effective_cursor_idx.min(active_text.len());
                let offset = iris_text::measure_text_advance(&active_text[..valid_idx], font_size);
                (0, active_text.len(), offset)
            } else {
                (0, 0, 0.0)
            };

        let effective_selection = if is_still_focused && is_all_sel {
            Some((0, active_text.len()))
        } else {
            props.selection
        };

        if let Some((start_byte, end_byte)) = effective_selection
            && !active_text.is_empty()
        {
            let start_byte = start_byte.min(active_text.len());
            let end_byte = end_byte.min(active_text.len());
            let (sel_min_b, sel_max_b) = if start_byte <= end_byte {
                (start_byte, end_byte)
            } else {
                (end_byte, start_byte)
            };
            let sel_vis_min = sel_min_b.max(visible_start).min(visible_end);
            let sel_vis_max = sel_max_b.max(visible_start).min(visible_end);
            if sel_vis_max > sel_vis_min {
                let sel_start_x = iris_text::measure_text_advance(
                    &active_text[visible_start..sel_vis_min],
                    font_size,
                );
                let sel_end_x = iris_text::measure_text_advance(
                    &active_text[visible_start..sel_vis_max],
                    font_size,
                );
                let sel_w = (sel_end_x - sel_start_x).max(2.0);
                let sel_x = text_start_x + sel_start_x;
                let sel_id = self.tree.create_node();
                if let Some(node) = self.tree.get_mut(sel_id) {
                    node.set_name("SelectionCapsule");
                    node.interactive = false;
                    node.set_style(
                        Style::new()
                            .position_absolute()
                            .left(sel_x)
                            .top(3.5)
                            .width(sel_w)
                            .height(17.0)
                            .background(Color::rgba(0.14, 0.46, 0.88, 0.95))
                            .border_radius(3.0),
                    );
                }
                let _ = self.tree.add_child(box_id, sel_id);
            }
        }

        let display_text = if active_text.is_empty() {
            props.placeholder
        } else {
            &active_text[visible_start..visible_end]
        };

        let text_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(text_id) {
            node.role = WidgetRole::Default;
            node.interactive = false;
            node.tag = tag;
            node.set_text(display_text);
            node.font_size = font_size;
            node.line_height = 24.0;
            node.text_align = TextAlign::Left;
            node.text_color = if active_text.is_empty() {
                Color::rgba(0.42, 0.45, 0.55, 1.0)
            } else {
                Color::rgba(0.92, 0.94, 0.98, 1.0)
            };
            node.focus_text_color = Some(Color::rgba(0.92, 0.94, 0.98, 1.0));
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(text_start_x)
                    .top(0.0)
                    .width(max_visible_w)
                    .height(24.0),
            );
        }
        let _ = self.tree.add_child(box_id, text_id);

        if (is_still_focused || props.is_focused) && props.cursor_blink_visible {
            let caret_x = text_start_x + caret_offset;
            let caret_id = self.tree.create_node();
            let caret_color = props.caret_color.unwrap_or(self.theme().text_caret);
            if let Some(node) = self.tree.get_mut(caret_id) {
                node.set_name("TextInputCaret");
                node.role = WidgetRole::TextInputCaret;
                node.interactive = false;
                node.tag = caret_offset.to_bits() as u64;
                node.set_style(
                    Style::new()
                        .position_absolute()
                        .left(caret_x)
                        .top(5.0)
                        .width(1.5)
                        .height(14.0)
                        .background(caret_color)
                        .border_radius(0.75),
                );
            }
            let _ = self.tree.add_child(box_id, caret_id);
        }

        if let Some(clear_tag) = props.clear_tag
            && !active_text.is_empty()
        {
            let clear_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(clear_id) {
                node.tag = clear_tag;
                node.set_name("SearchClearButton");
                node.role = WidgetRole::Button;
                node.interactive = true;
                node.cursor = Some(WidgetCursor::Pointer);
                node.set_text("✖");
                node.font_size = 11.0;
                node.line_height = 18.0;
                node.text_align = TextAlign::Center;
                node.text_color = Color::rgba(0.60, 0.63, 0.72, 1.0);
                node.hover_text_color = Some(Color::WHITE);
                node.set_style(
                    Style::new()
                        .position_absolute()
                        .right(4.0)
                        .top(3.0)
                        .width(16.0)
                        .height(18.0)
                        .border_radius(3.0)
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center),
                );
            }
            let _ = self.tree.add_child(box_id, clear_id);
        }

        let mut response = self.response(box_id);
        if changed {
            response.flags = response.flags.union(ResponseFlags::CHANGED);
        }
        if submitted {
            response.flags = response.flags.union(ResponseFlags::SUBMITTED);
        }
        if is_still_focused {
            response.flags = response.flags.union(ResponseFlags::FOCUSED);
        }
        response
    }
}