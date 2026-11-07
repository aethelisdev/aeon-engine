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
    TextInputFilter, TextInputKeyContext, process_text_input_keys,
};
use iris_core::WidgetId;

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

    if clicked {
        if is_editing {
            if let Some(mut session) = scope.tree.event_state.active_text_edit.take() {
                session.selection = None;
                session.cursor_byte_idx = session.buffer.len();
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
            is_editing = true;
        }
    }

    if is_editing {
        scope.tree.event_state.focused = Some(node_id);
        if tag != 0 {
            scope.tree.event_state.focused_tag = Some(tag);
        }

        let is_clicked_outside = if let Some(clicked_tag) = scope.tree.event_state.frame_clicked_tag
        {
            clicked_tag != tag
        } else if let Some(clicked_id) = scope.tree.event_state.frame_clicked_id {
            clicked_id != node_id
        } else {
            false
        };

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

        let disp = if sel.is_some() {
            raw.clone()
        } else {
            let blink_caret = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(true, |d| (d.as_millis() / 500).is_multiple_of(2));
            if blink_caret {
                let (left, right) = raw.split_at(cur);
                format!("{}|{}", left, right)
            } else {
                raw.clone()
            }
        };

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