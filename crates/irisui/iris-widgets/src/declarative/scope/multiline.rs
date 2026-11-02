// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Multi-line Text Editor Widget
//!
//! Provides a hardware-accelerated, GPU SDF multi-line code and text editor widget on [`UiScope`].
//! Features line number gutters, active line highlighting, sticky column navigation,
//! click-to-caret positioning, and virtualized vertical line rendering.

use super::core::UiScope;
use crate::controls::multiline::{
    DEFAULT_GUTTER_PADDING, GutterMetrics, GutterStyle, MultilineTextState,
};
use crate::declarative::scope::input_box::interaction::find_cursor_idx_at_x;
use crate::declarative::types::WidgetResponse;
use crate::scroll_area::ScrollBarGeometry;
use crate::scroll_area::style::ScrollAreaStyle;
use iris_core::{AlignItems, Color, Insets, Style, TextAlign, WidgetCursor, WidgetRole};
use iris_text::measure_text_advance;

/// Configuration properties for rendering an interactive multi-line code/text editor.
#[derive(Clone, Debug)]
pub struct MultilineEditorProps {
    /// Persistent 64-bit semantic tag for interaction and focus tracking.
    pub tag: u64,
    /// Font size in logical pixels for code lines.
    pub font_size: f32,
    /// Height in logical pixels for each individual text line.
    pub line_height: f32,
    /// Total widget container height in logical pixels.
    pub height: f32,
    /// Whether to display the line number gutter on the left.
    pub show_gutter: bool,
    /// Visual styling for the line number gutter.
    pub gutter_style: GutterStyle,
    /// Container background color.
    pub background: Color,
    /// Border stroke color.
    pub border_color: Color,
    /// Subtle background highlight color applied across the active line containing the caret.
    pub active_line_bg: Color,
    /// Color of the blinking caret indicator.
    pub caret_color: Color,
    /// Whether the caret is currently in its visible phase of the blink cycle.
    pub blink_caret: bool,
}

impl MultilineEditorProps {
    /// Constructs a new property descriptor with standard IDE defaults.
    #[must_use]
    pub fn new(tag: u64) -> Self {
        Self {
            tag,
            font_size: 12.0,
            line_height: 20.0,
            height: 220.0,
            show_gutter: true,
            gutter_style: GutterStyle::default(),
            background: Color::from_u8(18, 19, 24, 255),
            border_color: Color::from_u8(38, 40, 48, 255),
            active_line_bg: Color::from_u8(30, 33, 42, 140),
            caret_color: Color::from_u8(0, 220, 255, 255),
            blink_caret: true,
        }
    }

    /// Sets the height of the editor container.
    #[must_use]
    pub const fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets whether the line number gutter is rendered.
    #[must_use]
    pub const fn with_gutter(mut self, show: bool) -> Self {
        self.show_gutter = show;
        self
    }

    /// Overrides the visual caret color.
    #[must_use]
    pub const fn with_caret_color(mut self, color: Color) -> Self {
        self.caret_color = color;
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a declarative multi-line code/text editor widget bound to a [`MultilineTextState`].
    ///
    /// Features:
    /// - Automatic click-to-caret positioning with sub-pixel horizontal glyph advance matching.
    /// - Sticky vertical navigation (`preferred_column`) across lines of varying lengths.
    /// - Line number gutter with active line number highlighting.
    /// - Enter key newline insertion, Backspace/Delete editing, and Home/End navigation.
    ///
    /// # Arguments
    /// * `tag` - Unique 64-bit semantic tag for focus and hit-testing.
    /// * `height` - Display height in logical pixels.
    /// * `state` - Mutable reference to the multi-line editing buffer state.
    pub fn multiline_editor(
        &mut self,
        tag: u64,
        height: f32,
        state: &mut MultilineTextState,
    ) -> WidgetResponse {
        let props = MultilineEditorProps::new(tag).with_height(height);
        self.multiline_editor_with_props(&props, state)
    }

    /// Emits a declarative multi-line editor with customized visual properties.
    pub fn multiline_editor_with_props(
        &mut self,
        props: &MultilineEditorProps,
        state: &mut MultilineTextState,
    ) -> WidgetResponse {
        let mut response = WidgetResponse::default();
        let container_id = self.tree.create_node();
        response.id = container_id;

        let container_rect = self
            .tree
            .tag_rect(props.tag)
            .or(self.tree.event_state.frame_clicked_rect)
            .unwrap_or_else(|| self.available_rect());

        let total_content_h = (state.line_count() as f32 * props.line_height).max(props.height);
        let max_scroll = (total_content_h - props.height).max(0.0);

        // Sync scroll from UiTree persistent state (updated by dispatch_event from mouse wheel or scrollbar)
        if props.tag != 0 {
            let tree_scroll = self.tree.scroll_offset(props.tag);
            if (tree_scroll - state.scroll_y).abs() > 0.01 && max_scroll > 0.0 {
                state.scroll_y = tree_scroll.clamp(0.0, max_scroll);
            }
        }

        // 1. Configure Editor Root Node
        if let Some(node) = self.tree.get_mut(container_id) {
            node.set_name("MultilineEditor");
            node.tag = props.tag;
            node.role = WidgetRole::TextInput;
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Text);
            node.content_size = iris_core::geometry::Size::new(0.0, total_content_h);
            node.style.scroll_offset_y = state.scroll_y;
            node.set_style(
                Style::new()
                    .flex_grow(1.0)
                    .height(props.height)
                    .flex_col()
                    .background(props.background)
                    .border(1.0, props.border_color)
                    .border_radius(4.0)
                    .clip_children(true)
                    .scroll_offset_y(state.scroll_y),
            );
        }
        let _ = self.tree.add_child(self.parent, container_id);

        let gutter_metrics =
            GutterMetrics::compute(state.line_count(), 7.2, DEFAULT_GUTTER_PADDING);
        let gutter_w = if props.show_gutter {
            gutter_metrics.width
        } else {
            0.0
        };
        let divider_w = if props.show_gutter { 1.0 } else { 0.0 };
        let code_pad_left = 8.0;
        let text_start_x = container_rect.x + gutter_w + divider_w + code_pad_left;

        let track_tag = props.tag.wrapping_add(0x1000);
        let thumb_tag = props.tag.wrapping_add(0x2000);
        let is_thumb_captured = self
            .tree
            .event_state
            .pointer_capture_tag
            .is_some_and(|t| t == thumb_tag);

        // 2. Resolve Focus and Interaction States
        let (clicked, hovered, _) = self.check_interaction(container_id);
        if hovered {
            response
                .flags
                .insert(crate::declarative::types::ResponseFlags::HOVERED);
        }

        let is_pressed = self.tree.event_state.pressed == Some(container_id)
            || self.tree.event_state.pressed_tag == Some(props.tag)
            || self.tree.event_state.pointer_capture == Some(container_id)
            || self.tree.event_state.pointer_capture_tag == Some(props.tag);

        // 3. Handle Mouse Click Caret Positioning & Drag Selection
        if (clicked || (is_pressed && self.tree.event_state.frame_clicked_pos.is_some()))
            && !is_thumb_captured
        {
            self.tree.event_state.focused_tag = Some(props.tag);
            self.tree.event_state.focused = Some(container_id);
            response
                .flags
                .insert(crate::declarative::types::ResponseFlags::CLICKED);

            if let Some(click_pos) = self.tree.event_state.frame_clicked_pos {
                let on_scrollbar = max_scroll > 0.0 && click_pos.x >= container_rect.right() - 14.0;
                if !on_scrollbar {
                    let rel_y = (click_pos.y - (container_rect.y + 4.0) + state.scroll_y).max(0.0);
                    let line_idx = (rel_y / props.line_height).floor() as usize;
                    let clicked_line = line_idx.min(state.line_count().saturating_sub(1));

                    let line_content = state.line_str(clicked_line).unwrap_or("");
                    let char_col = if click_pos.x <= text_start_x {
                        0
                    } else {
                        let rel_x = click_pos.x - text_start_x;
                        let byte_idx = find_cursor_idx_at_x(line_content, rel_x, props.font_size);
                        line_content[..byte_idx].chars().count()
                    };

                    let is_double_click = self.tree.event_state.frame_double_clicked_id
                        == Some(container_id)
                        || self.tree.event_state.frame_double_clicked_tag == Some(props.tag);

                    if is_double_click {
                        state.select_all();
                    } else {
                        state.set_cursor(clicked_line, char_col);
                        state.preferred_column = None;
                        state.selection = None;
                        state.selection_anchor = Some((clicked_line, char_col));
                    }
                }
            }
        }

        if !is_thumb_captured {
            let drag_pos = if is_pressed && self.tree.event_state.is_pointer_drag_active {
                self.tree
                    .event_state
                    .last_pointer_pos
                    .or(self.tree.event_state.frame_clicked_pos)
            } else {
                None
            };

            if let Some(pos) = drag_pos {
                let on_scrollbar = max_scroll > 0.0 && pos.x >= container_rect.right() - 14.0;
                if !on_scrollbar {
                    let rel_y = (pos.y - (container_rect.y + 4.0) + state.scroll_y).max(0.0);
                    let line_idx = (rel_y / props.line_height).floor() as usize;
                    let hover_line = line_idx.min(state.line_count().saturating_sub(1));

                    let line_content = state.line_str(hover_line).unwrap_or("");
                    let hover_col = if pos.x <= text_start_x {
                        0
                    } else {
                        let rel_x = pos.x - text_start_x;
                        let byte_idx = find_cursor_idx_at_x(line_content, rel_x, props.font_size);
                        line_content[..byte_idx].chars().count()
                    };

                    if let Some(anchor) = state.selection_anchor {
                        if anchor != (hover_line, hover_col) {
                            state.set_selection(anchor, (hover_line, hover_col));
                        } else {
                            state.selection = None;
                        }
                        state.set_cursor(hover_line, hover_col);
                    }
                }
            }
        }

        let is_focused = self.tree.event_state.focused_tag == Some(props.tag)
            || self.tree.event_state.focused == Some(container_id);

        // 4. Handle Keyboard Input when Focused
        if is_focused {
            let mut key_consumed = false;
            let mut should_scroll_to_cursor = false;
            if self.tree.event_state.frame_key_pressed
                && let Some(key) = self.tree.event_state.frame_key
            {
                let outcome = state.handle_key(
                    key,
                    self.tree.event_state.frame_ctrl_held,
                    self.tree.event_state.frame_shift_held,
                );
                if outcome.consumed {
                    key_consumed = true;
                }
                if outcome.cursor_moved || outcome.text_changed {
                    should_scroll_to_cursor = true;
                }
                if outcome.text_changed {
                    response
                        .flags
                        .insert(crate::declarative::types::ResponseFlags::CHANGED);
                }
            }

            if !self.tree.event_state.frame_ctrl_held
                && !key_consumed
                && let Some(text_in) = &self.tree.event_state.frame_text_input
                && !text_in.is_empty()
                && !text_in
                    .chars()
                    .any(|c| c.is_control() && c != '\n' && c != '\t')
            {
                state.insert_str(text_in);
                should_scroll_to_cursor = true;
                response
                    .flags
                    .insert(crate::declarative::types::ResponseFlags::CHANGED);
            }

            if should_scroll_to_cursor {
                let cursor_top = state.cursor.0 as f32 * props.line_height;
                let cursor_bottom = cursor_top + props.line_height;
                if cursor_top < state.scroll_y {
                    state.scroll_y = cursor_top;
                } else if cursor_bottom > state.scroll_y + props.height {
                    state.scroll_y = cursor_bottom - props.height;
                }
                state.scroll_y = state.scroll_y.clamp(0.0, max_scroll);
                if props.tag != 0 {
                    self.tree.set_scroll_offset(props.tag, state.scroll_y);
                }
            }
        }

        state.scroll_y = state.scroll_y.clamp(0.0, max_scroll);
        if props.tag != 0 {
            self.tree.set_max_scroll(props.tag, max_scroll);
        }

        // 5. Layout & Render Lines Hierarchy
        let visible_range = GutterMetrics::visible_line_range(
            state.scroll_y,
            props.height,
            props.line_height,
            state.line_count(),
            2,
        );

        let top_spacer_h = visible_range.start as f32 * props.line_height;
        if top_spacer_h > 0.0 {
            let spacer_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(spacer_id) {
                node.set_name("MultilineTopSpacer");
                node.set_style(Style::new().height(top_spacer_h).flex_shrink(0.0));
            }
            let _ = self.tree.add_child(container_id, spacer_id);
        }

        let gutter_width = if props.show_gutter {
            gutter_metrics.width
        } else {
            0.0
        };

        for line_idx in visible_range.clone() {
            let is_cursor_line = line_idx == state.cursor.0;
            let row_id = self.tree.create_node();

            let row_bg = if is_cursor_line && is_focused {
                props.active_line_bg
            } else {
                Color::TRANSPARENT
            };

            if let Some(node) = self.tree.get_mut(row_id) {
                node.set_style(
                    Style::new()
                        .flex_grow(1.0)
                        .height(props.line_height)
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .background(row_bg),
                );
            }
            let _ = self.tree.add_child(container_id, row_id);

            // 5a. Gutter Line Number Column (Centered, clean typography)
            if props.show_gutter {
                let gutter_cell_id = self.tree.create_node();
                if let Some(node) = self.tree.get_mut(gutter_cell_id) {
                    node.set_name("GutterCell");
                    node.set_text(gutter_metrics.format_line_number(line_idx));
                    node.font_size = 11.0;
                    node.line_height = props.line_height;
                    node.text_align = TextAlign::Center;
                    node.text_color = if is_cursor_line && is_focused {
                        props.gutter_style.active_line_color
                    } else {
                        props.gutter_style.text_color
                    };
                    node.set_style(
                        Style::new()
                            .width(gutter_width)
                            .height(props.line_height)
                            .flex_shrink(0.0),
                    );
                }
                let _ = self.tree.add_child(row_id, gutter_cell_id);

                // 1px Vertical divider line between gutter and code text
                let divider_id = self.tree.create_node();
                if let Some(node) = self.tree.get_mut(divider_id) {
                    node.set_name("GutterDivider");
                    node.interactive = false;
                    node.set_style(
                        Style::new()
                            .width(1.0)
                            .height(props.line_height)
                            .flex_shrink(0.0)
                            .background(props.gutter_style.divider_color),
                    );
                }
                let _ = self.tree.add_child(row_id, divider_id);
            }

            // 5b. Code / Text Content Column
            let text_cell_id = self.tree.create_node();
            let line_content = state.line_str(line_idx).unwrap_or("");
            if let Some(node) = self.tree.get_mut(text_cell_id) {
                node.set_name("TextCell");
                node.set_text(line_content);
                node.font_size = props.font_size;
                node.line_height = props.line_height;
                node.text_align = TextAlign::Left;
                node.text_color = Color::from_u8(220, 225, 235, 255);
                node.set_style(
                    Style::new()
                        .flex_grow(1.0)
                        .height(props.line_height)
                        .margin_insets(Insets::new(0.0, 0.0, 0.0, code_pad_left)),
                );
            }
            let _ = self.tree.add_child(row_id, text_cell_id);

            // 5c. Selection Highlight Capsule
            if let Some((start, end)) = state.selection {
                let (s_line, s_col) = start.min(end);
                let (e_line, e_col) = start.max(end);

                if line_idx >= s_line && line_idx <= e_line {
                    let (sel_x, sel_w) = if s_line == e_line {
                        let s_byte = state
                            .char_col_to_byte_offset(line_idx, s_col)
                            .saturating_sub(state.line_starts[line_idx])
                            .min(line_content.len());
                        let e_byte = state
                            .char_col_to_byte_offset(line_idx, e_col)
                            .saturating_sub(state.line_starts[line_idx])
                            .min(line_content.len());
                        let sx = measure_text_advance(&line_content[..s_byte], props.font_size);
                        let sw =
                            measure_text_advance(&line_content[s_byte..e_byte], props.font_size)
                                .max(4.0);
                        (sx, sw)
                    } else if line_idx == s_line {
                        let s_byte = state
                            .char_col_to_byte_offset(line_idx, s_col)
                            .saturating_sub(state.line_starts[line_idx])
                            .min(line_content.len());
                        let sx = measure_text_advance(&line_content[..s_byte], props.font_size);
                        let sw = (measure_text_advance(&line_content[s_byte..], props.font_size)
                            + 8.0)
                            .max(8.0);
                        (sx, sw)
                    } else if line_idx == e_line {
                        let e_byte = state
                            .char_col_to_byte_offset(line_idx, e_col)
                            .saturating_sub(state.line_starts[line_idx])
                            .min(line_content.len());
                        let sw = measure_text_advance(&line_content[..e_byte], props.font_size);
                        (0.0, sw)
                    } else {
                        let sw = measure_text_advance(line_content, props.font_size) + 8.0;
                        (0.0, sw)
                    };

                    let sel_id = self.tree.create_node();
                    if let Some(node) = self.tree.get_mut(sel_id) {
                        node.set_name("SelectionCapsule");
                        node.interactive = false;
                        node.set_style(
                            Style::new()
                                .position_absolute()
                                .left(sel_x)
                                .top(1.0)
                                .width(sel_w)
                                .height(props.line_height - 2.0)
                                .background(Color::rgba(0.20, 0.45, 0.85, 0.35))
                                .border_radius(2.0),
                        );
                    }
                    let _ = self.tree.add_child(text_cell_id, sel_id);
                }
            }

            // 5d. Blinking Caret Indicator
            if is_cursor_line && is_focused && props.blink_caret {
                let cursor_char_col = state.cursor.1;
                let byte_offset = state.char_col_to_byte_offset(line_idx, cursor_char_col);
                let line_start = state.line_starts[line_idx];
                let rel_byte = byte_offset
                    .saturating_sub(line_start)
                    .min(line_content.len());
                let caret_x_advance =
                    measure_text_advance(&line_content[..rel_byte], props.font_size);

                let caret_id = self.tree.create_node();
                if let Some(node) = self.tree.get_mut(caret_id) {
                    node.set_name("TextInputCaret");
                    node.role = WidgetRole::TextInputCaret;
                    node.interactive = false;
                    node.tag = caret_x_advance.to_bits() as u64;
                    node.set_style(
                        Style::new()
                            .position_absolute()
                            .left(caret_x_advance)
                            .top(2.0)
                            .width(1.5)
                            .height(props.line_height - 4.0)
                            .background(props.caret_color)
                            .border_radius(0.75),
                    );
                }
                let _ = self.tree.add_child(text_cell_id, caret_id);
            }
        }

        let remaining_lines = state.line_count().saturating_sub(visible_range.end);
        if remaining_lines > 0 {
            let bottom_spacer_h = remaining_lines as f32 * props.line_height;
            let spacer_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(spacer_id) {
                node.set_name("MultilineBottomSpacer");
                node.set_style(Style::new().height(bottom_spacer_h).flex_shrink(0.0));
            }
            let _ = self.tree.add_child(container_id, spacer_id);
        }

        // 6. Autonomous Vertical ScrollBar Indicator & Drag Thumb
        if max_scroll > 0.0 {
            let scroll_style = ScrollAreaStyle::dark_default();

            if let Some(geom) = ScrollBarGeometry::compute_vertical(
                container_rect,
                total_content_h,
                state.scroll_y,
                &scroll_style,
            ) {
                let is_dragging = self
                    .tree
                    .event_state
                    .pointer_capture
                    .is_some_and(|cid| self.tree.get(cid).is_some_and(|n| n.tag == thumb_tag));
                let cursor_pos = self.tree.event_state.last_pointer_pos;

                let mut child_scope = self.child_scope(container_id);
                child_scope.scrollbar_vertical(
                    geom,
                    container_rect,
                    is_dragging,
                    cursor_pos,
                    track_tag,
                    thumb_tag,
                );
            }
        }

        response
    }
}