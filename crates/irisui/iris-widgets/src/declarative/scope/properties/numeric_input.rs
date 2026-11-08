// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inline Numeric Text Editing Interaction Engine
//!
//! Provides a unified interaction driver connecting scalar and vector numeric components
//! ([`crate::declarative::widget::NumericDrag`], [`crate::declarative::widget::DragVec3`], sliders)
//! directly to [`iris_core::dispatch::ActiveTextEditSession`] and [`crate::declarative::scope::input_box::process_text_input_keys`].
//!
//! Enforces mathematical expression filtering ([`crate::declarative::scope::input_box::TextInputFilter::NumericExpression`]),
//! clipboard integration (`Ctrl+C`, `Ctrl+X`, `Ctrl+V`), autonomous undo/redo (`Ctrl+Z`, `Ctrl+Y`),
//! and arithmetic evaluation upon focus loss or `Enter` submission.

use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::input_box::{
    TextInputFilter, TextInputKeyContext, find_cursor_idx_at_x, process_text_input_keys,
};
use iris_core::WidgetId;
use iris_text::measure_text_advance;

/// Configuration parameters governing inline numeric editing boundaries and precision formatting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumericEditConfig {
    /// Lower boundary clamp for the parsed value.
    pub min_val: f32,
    /// Upper boundary clamp for the parsed value.
    pub max_val: f32,
    /// Precision decimal places for floating-point formatting.
    pub decimals: usize,
}

impl NumericEditConfig {
    /// Creates a new configuration with explicit min, max, and precision decimal places.
    #[inline]
    pub const fn new(min_val: f32, max_val: f32, decimals: usize) -> Self {
        Self {
            min_val,
            max_val,
            decimals,
        }
    }
}

/// Execution outcome produced by evaluating an inline numeric edit frame.
#[derive(Debug, Clone, PartialEq)]
pub struct NumericEditResult {
    /// Whether the underlying numeric value was altered by an evaluated expression.
    pub value_changed: bool,
    /// Whether the edit session was committed and submitted (e.g. via `Enter` or clicking outside).
    pub submitted: bool,
    /// Whether the widget node is actively focused in direct text editing mode.
    pub is_editing: bool,
    /// Raw underlying textual string value currently stored in the edit buffer.
    pub raw_text: String,
    /// Formatted string rendered on the widget node, including an optional blinking caret indicator.
    pub display_text: String,
    /// Optional active selection range `(start_byte_idx, end_byte_idx)` where `start < end`.
    pub selection: Option<(usize, usize)>,
    /// Caret byte index within the buffer.
    pub cursor_byte_idx: usize,
}

/// Evaluates and coordinates inline text editing for scalar and vector numeric inputs.
///
/// Binds  to [`iris_core::dispatch::ActiveTextEditSession`] and [`process_text_input_keys`]
/// using [`TextInputFilter::NumericExpression`].
///
/// # Arguments
///
/// * `scope` - Active declarative scope.
/// * `node_id` - Identifier of the UI node representing the clickable numeric pill.
/// * `tag` - Semantic hash or explicit tag identifying the numeric field across rebuilds.
/// * `value` - Mutable reference to the target float value.
/// * `config` - Bounds and precision settings ([`NumericEditConfig`]).
/// * `clicked` - Whether the numeric pill was clicked in this frame.
///
/// # Returns
///
/// A [`NumericEditResult`] with the updated interaction and rendering parameters.
pub fn handle_numeric_edit_interaction(
    scope: &mut UiScope<'_>,
    node_id: WidgetId,
    tag: u64,
    value: &mut f32,
    config: NumericEditConfig,
    clicked: bool,
) -> NumericEditResult {
    let min_val = config.min_val;
    let max_val = config.max_val;
    let decimals = config.decimals;
    let mut value_changed = false;
    let mut submitted = false;

    if let Some(restored) = scope.tree.event_state.numeric_undo.take_pending_scalar(tag) {
        let next = restored.clamp(min_val, max_val);
        if (next - *value).abs() > f32::EPSILON {
            *value = next;
            value_changed = true;
        }
        submitted = true;
    }

    let mut is_editing = scope
        .tree
        .event_state
        .active_text_edit
        .as_ref()
        .is_some_and(|s| s.tag == tag);

    let node_rect = scope
        .tree
        .get(node_id)
        .map(|n| n.computed_rect)
        .filter(|r| r.width > 0.0)
        .or_else(|| {
            scope
                .tree
                .event_state
                .pointer_capture
                .and_then(|cid| scope.tree.get(cid))
                .map(|n| n.computed_rect)
                .filter(|r| r.width > 0.0)
        })
        .or(scope.tree.event_state.frame_clicked_rect);

    let is_captured = scope.tree.event_state.pointer_capture == Some(node_id)
        || (tag != 0 && scope.tree.event_state.pointer_capture_tag == Some(tag));
    let is_pressed = scope.tree.event_state.pressed == Some(node_id)
        || (tag != 0 && scope.tree.event_state.pressed_tag == Some(tag));
    let is_drag_active = scope.tree.event_state.is_pointer_drag_active && is_captured;
    let is_drag_released = scope.tree.event_state.frame_drag_released_id == Some(node_id)
        || (tag != 0 && scope.tree.event_state.frame_drag_released_tag == Some(tag))
        || scope
            .tree
            .event_state
            .frame_drag_released_id
            .is_some_and(|did| scope.tree.is_descendant_of(did, node_id));

    let is_clicked_outside = if let Some(clicked_tag) = scope.tree.event_state.frame_clicked_tag {
        clicked_tag != tag
    } else if let Some(clicked_id) = scope.tree.event_state.frame_clicked_id {
        clicked_id != node_id && !scope.tree.is_descendant_of(clicked_id, node_id)
    } else {
        false
    };

    if clicked {
        if is_editing {
            let is_double_click = scope.tree.event_state.frame_double_clicked_id == Some(node_id)
                || (tag != 0 && scope.tree.event_state.frame_double_clicked_tag == Some(tag))
                || scope
                    .tree
                    .event_state
                    .frame_double_clicked_id
                    .is_some_and(|did| scope.tree.is_descendant_of(did, node_id));

            if let Some(mut session) = scope.tree.event_state.active_text_edit.take() {
                if is_double_click {
                    let len = session.buffer.len();
                    session.selection = if len > 0 { Some((0, len)) } else { None };
                    session.cursor_byte_idx = len;
                    scope.tree.event_state.text_drag_anchor = Some(len);
                } else if let Some(click_pos) = scope.tree.event_state.frame_clicked_pos
                    && let Some(rect) = node_rect
                {
                    let font_size = 10.5;
                    let text_w = measure_text_advance(&session.buffer, font_size);
                    let text_start_x = rect.x + (rect.width - text_w).max(0.0) * 0.5;
                    let rel_x = (click_pos.x - text_start_x).max(0.0);
                    let target_cur = find_cursor_idx_at_x(&session.buffer, rel_x, font_size);
                    session.cursor_byte_idx = target_cur;
                    session.selection = None;
                    scope.tree.event_state.text_drag_anchor = Some(target_cur);
                } else {
                    session.selection = None;
                    session.cursor_byte_idx = session.buffer.len();
                }
                scope.tree.event_state.active_text_edit = Some(session);
            }
            scope.tree.event_state.focused = Some(node_id);
            if tag != 0 {
                scope.tree.event_state.focused_tag = Some(tag);
            }
        } else {
            let initial_buf = crate::declarative::types::format_compact_float(*value, decimals);
            let len = initial_buf.len();
            scope.tree.event_state.start_text_edit(
                Some(node_id),
                tag,
                &initial_buf,
                len,
                Some((0, len)),
            );
            scope.tree.event_state.text_drag_anchor = Some(len);
            is_editing = true;
        }
    }

    // Move caret immediately on mouse press inception while already in active text editing mode
    if is_editing
        && !is_clicked_outside
        && !is_drag_released
        && is_pressed
        && scope.tree.event_state.text_drag_anchor.is_none()
        && let Some(pos) = scope
            .tree
            .event_state
            .frame_clicked_pos
            .or(scope.tree.event_state.last_pointer_pos)
        && let Some(rect) = node_rect
        && let Some(mut session) = scope.tree.event_state.active_text_edit.take()
    {
        let font_size = 10.5;
        let text_w = measure_text_advance(&session.buffer, font_size);
        let text_start_x = rect.x + (rect.width - text_w).max(0.0) * 0.5;
        let rel_x = (pos.x - text_start_x).max(0.0);
        let target_cur = find_cursor_idx_at_x(&session.buffer, rel_x, font_size);
        session.cursor_byte_idx = target_cur;
        session.selection = None;
        scope.tree.event_state.text_drag_anchor = Some(target_cur);
        scope.tree.event_state.active_text_edit = Some(session);
    }

    // Coordinate mouse drag-to-select range across character boundaries while editing
    if is_editing
        && is_drag_active
        && let Some(pointer_pos) = scope.tree.event_state.last_pointer_pos
        && let Some(rect) = node_rect
        && let Some(mut session) = scope.tree.event_state.active_text_edit.take()
    {
        let font_size = 10.5;
        let text_w = measure_text_advance(&session.buffer, font_size);
        let text_start_x = rect.x + (rect.width - text_w).max(0.0) * 0.5;
        let rel_x = (pointer_pos.x - text_start_x).max(0.0);
        let target_cur = find_cursor_idx_at_x(&session.buffer, rel_x, font_size);
        session.cursor_byte_idx = target_cur;
        if let Some(anchor) = scope.tree.event_state.text_drag_anchor {
            if anchor != target_cur {
                session.selection = Some((anchor.min(target_cur), anchor.max(target_cur)));
            } else {
                session.selection = None;
            }
        }
        scope.tree.event_state.active_text_edit = Some(session);
    }

    if is_drag_released {
        scope.tree.event_state.text_drag_anchor = None;
    }

    if is_editing {
        scope.tree.event_state.focused = Some(node_id);
        if tag != 0 {
            scope.tree.event_state.focused_tag = Some(tag);
        }

        if is_clicked_outside {
            if let Some(session) = scope.tree.event_state.active_text_edit.take()
                && let Some(parsed) =
                    crate::declarative::types::eval_numeric_expression(&session.buffer)
            {
                let next = parsed.clamp(min_val, max_val);
                if (next - *value).abs() > f32::EPSILON {
                    scope
                        .tree
                        .event_state
                        .numeric_undo
                        .push_scalar(tag, *value, next);
                    *value = next;
                    value_changed = true;
                }
                submitted = true;
            }
            if scope.tree.event_state.focused_tag == Some(tag) {
                scope.tree.event_state.clear_focus();
            }
            is_editing = false;
        } else {
            let key_pressed = scope.tree.event_state.frame_key_pressed;
            let key = scope.tree.event_state.frame_key;
            let incoming = scope.tree.event_state.frame_text_input.as_deref();
            let ctrl_held = scope.tree.event_state.frame_ctrl_held;
            let shift_held = scope.tree.event_state.frame_shift_held;

            let ctx = TextInputKeyContext::new(key_pressed, key, incoming, ctrl_held, shift_held)
                .with_filter(TextInputFilter::NumericExpression);

            if let Some(mut session) = scope.tree.event_state.active_text_edit.take() {
                let res = process_text_input_keys(
                    &mut session.buffer,
                    &mut session.cursor_byte_idx,
                    &mut session.selection,
                    ctx,
                    &mut session.undo_stack,
                );

                if res.submitted || res.should_clear_focus {
                    if res.submitted
                        && let Some(parsed) =
                            crate::declarative::types::eval_numeric_expression(&session.buffer)
                    {
                        let next = parsed.clamp(min_val, max_val);
                        if (next - *value).abs() > f32::EPSILON {
                            scope
                                .tree
                                .event_state
                                .numeric_undo
                                .push_scalar(tag, *value, next);
                            *value = next;
                            value_changed = true;
                        }
                        submitted = true;
                    }
                    scope.tree.event_state.clear_text_edit();
                    if res.should_clear_focus {
                        scope.tree.event_state.clear_focus();
                    } else {
                        scope.tree.event_state.focused = Some(node_id);
                        if tag != 0 {
                            scope.tree.event_state.focused_tag = Some(tag);
                        }
                    }
                    is_editing = false;
                } else {
                    scope.tree.event_state.active_text_edit = Some(session);
                }
            }
        }
    }

    let (raw_text, display_text, selection, cursor_byte_idx) = if is_editing {
        let session = scope
            .tree
            .event_state
            .active_text_edit
            .as_ref()
            .expect("active_text_edit invariant guaranteed when is_editing is true");
        let raw = session.buffer.clone();
        let sel = session.selection;
        let cur = session.cursor_byte_idx.min(raw.len());
        let disp = raw.clone();

        (raw, disp, sel, cur)
    } else {
        let raw = crate::declarative::types::format_compact_float(*value, decimals);
        let disp = raw.clone();
        let len = raw.len();
        (raw, disp, None, len)
    };

    NumericEditResult {
        value_changed,
        submitted,
        is_editing,
        raw_text,
        display_text,
        selection,
        cursor_byte_idx,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::UiTree;
    use iris_core::geometry::{Point, Rect};

    #[test]
    fn test_numeric_input_subpixel_click_cursor_positioning() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");

        let node_id = tree.create_node();
        let _ = tree.add_child(root, node_id);
        if let Some(node) = tree.get_mut(node_id) {
            node.computed_rect = Rect::new(100.0, 50.0, 48.0, 22.0);
        }

        let tag = 5555;
        let mut val = 42.5;
        let config = NumericEditConfig::new(0.0, 100.0, 1);

        // 1. First click: enters editing mode and selects all text ("42.5")
        {
            let mut scope = UiScope::new(&mut tree, root);
            let res =
                handle_numeric_edit_interaction(&mut scope, node_id, tag, &mut val, config, true);
            assert!(res.is_editing);
            assert_eq!(res.raw_text, "42.5");
            assert_eq!(res.selection, Some((0, 4)));
        }

        // 2. Second click: clicks inside the text at a specific coordinate
        // Text is centered inside [100.0, 148.0]. Center is 124.0.
        tree.event_state.frame_clicked_pos = Some(Point::new(124.0, 60.0));
        tree.event_state.frame_clicked_rect = Some(Rect::new(100.0, 50.0, 48.0, 22.0));
        {
            let mut scope = UiScope::new(&mut tree, root);
            let res =
                handle_numeric_edit_interaction(&mut scope, node_id, tag, &mut val, config, true);
            assert!(res.is_editing);
            assert_eq!(res.selection, None);
            // Caret should be positioned between characters (1 or 2), not forced to the end
            assert!(res.cursor_byte_idx > 0 && res.cursor_byte_idx < 4);
        }

        // 3. Double click: selects all text again
        tree.event_state.frame_double_clicked_id = Some(node_id);
        {
            let mut scope = UiScope::new(&mut tree, root);
            let res =
                handle_numeric_edit_interaction(&mut scope, node_id, tag, &mut val, config, true);
            assert!(res.is_editing);
            assert_eq!(res.selection, Some((0, 4)));
        }
    }

    #[test]
    fn test_numeric_input_drag_to_select() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");

        let node_id = tree.create_node();
        let _ = tree.add_child(root, node_id);
        if let Some(node) = tree.get_mut(node_id) {
            node.computed_rect = Rect::new(100.0, 50.0, 60.0, 22.0);
        }

        let tag = 7777;
        let mut val = 123.45;
        let config = NumericEditConfig::new(0.0, 1000.0, 2);

        // 1. Enter edit mode
        {
            let mut scope = UiScope::new(&mut tree, root);
            let _ =
                handle_numeric_edit_interaction(&mut scope, node_id, tag, &mut val, config, true);
        }

        // 2. Press down at beginning of text to set drag anchor
        tree.event_state.pressed = Some(node_id);
        tree.event_state.frame_clicked_pos = Some(Point::new(110.0, 60.0));
        tree.event_state.text_drag_anchor = None;
        {
            let mut scope = UiScope::new(&mut tree, root);
            let _ =
                handle_numeric_edit_interaction(&mut scope, node_id, tag, &mut val, config, false);
        }
        assert!(tree.event_state.text_drag_anchor.is_some());

        // 3. Drag to the right
        tree.event_state.pointer_capture = Some(node_id);
        tree.event_state.is_pointer_drag_active = true;
        tree.event_state.last_pointer_pos = Some(Point::new(140.0, 60.0));
        {
            let mut scope = UiScope::new(&mut tree, root);
            let res =
                handle_numeric_edit_interaction(&mut scope, node_id, tag, &mut val, config, false);
            assert!(res.is_editing);
            assert!(
                res.selection.is_some(),
                "Drag gesture must create an active text selection range"
            );
            let (s, e) = res.selection.unwrap();
            assert!(s < e);
        }
    }
}