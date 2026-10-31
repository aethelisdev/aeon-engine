// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Input Box Rendering Implementations
//!
//! Provides rendering methods for single-line text input fields, selection capsules,
//! blinking carets, and clear buttons on [`UiScope`].

use super::interaction::{find_cursor_idx_at_x, process_text_input_keys};
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{Color, Style, TextAlign, WidgetCursor, WidgetRole};
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
        let parent_layer = self.tree.effective_layer(self.parent);

        let _ = self.tree.add_child(self.parent, node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("TextInput");
            node.tag = tag;
            if parent_layer == iris_core::UiLayer::Modal {
                node.layer = iris_core::UiLayer::Modal;
            }
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
            let is_double_click = self.tree.event_state.frame_double_clicked_id == Some(node_id)
                || (edit_key != 0
                    && self.tree.event_state.frame_double_clicked_tag == Some(edit_key))
                || self
                    .tree
                    .event_state
                    .frame_double_clicked_id
                    .is_some_and(|did| self.tree.is_descendant_of(did, node_id));

            let (cur, all_sel) = if is_double_click {
                (text.len(), true)
            } else if let Some(click_pos) = self.tree.event_state.frame_clicked_pos {
                let node_rect = self
                    .tree
                    .get(node_id)
                    .map(|n| n.computed_rect)
                    .filter(|r| r.width > 0.0)
                    .or_else(|| {
                        self.tree
                            .event_state
                            .frame_clicked_id
                            .and_then(|cid| self.tree.get(cid))
                            .map(|n| n.computed_rect)
                            .filter(|r| r.width > 0.0)
                    })
                    .or(self.tree.event_state.frame_clicked_rect);
                let mut cur = text.len();
                if let Some(rect) = node_rect {
                    let text_start_x = rect.x + 8.0;
                    let rel_x = (click_pos.x - text_start_x).max(0.0);
                    cur = find_cursor_idx_at_x(text, rel_x, font_size);
                }
                (cur, false)
            } else if !is_editing_this {
                (text.len(), true)
            } else {
                (text.len(), false)
            };

            self.tree
                .event_state
                .start_text_edit(Some(node_id), edit_key, text, cur, all_sel);
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
            let ctrl_held = self.tree.event_state.frame_ctrl_held;

            let res = process_text_input_keys(
                text,
                &mut cur,
                &mut all_sel,
                key_pressed,
                key,
                incoming,
                ctrl_held,
            );

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
}