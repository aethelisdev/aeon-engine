// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Keyboard & IME Dispatch Handlers
//!
//! Routes keyboard navigation, character text inputs, autonomous numeric undo shortcuts (`Ctrl+Z`, `Ctrl+Y`),
//! and IME composition events to the focused widget.

use crate::dispatch::types::DispatchResult;
use crate::event::{ImeEvent, KeyCode};
use crate::node::WidgetRole;
use crate::response::{ResponseFlags, WidgetResponse};
use crate::tree::UiTree;

/// Evaluates `Ctrl+Z`, `Ctrl+Y`, and `Ctrl+Shift+Z` shortcuts against the autonomous [`crate::dispatch::NumericUndoStack`]
/// when no inline text buffer session is currently active.
pub(crate) fn try_handle_numeric_undo_shortcut(
    tree: &mut UiTree,
    key: KeyCode,
    text: Option<&str>,
    is_pressed: bool,
    ctrl_held: bool,
    shift_held: bool,
) -> Option<DispatchResult> {
    if !is_pressed
        || !ctrl_held
        || tree.event_state.active_text_edit.is_some()
        || !tree.event_state.numeric_undo.is_enabled()
    {
        return None;
    }

    let ch = match key {
        KeyCode::Character(c) => Some(c.to_ascii_lowercase()),
        _ => text.and_then(|s| s.chars().next().map(|c| c.to_ascii_lowercase())),
    };

    let target_tag = if !shift_held && ch == Some('z') {
        tree.event_state.numeric_undo.undo()?
    } else if (!shift_held && ch == Some('y')) || (shift_held && ch == Some('z')) {
        tree.event_state.numeric_undo.redo()?
    } else {
        return None;
    };

    let target_id = tree
        .find_node_by_tag(target_tag)
        .or(tree.event_state.focused);
    tree.event_state.focused = target_id;
    tree.event_state.focused_tag = Some(target_tag);
    let rect = target_id
        .and_then(|id| tree.get(id).map(|n| n.computed_rect))
        .unwrap_or(crate::geometry::Rect::ZERO);
    let flags = ResponseFlags::FOCUSED | ResponseFlags::CHANGED | ResponseFlags::SUBMITTED;

    Some(DispatchResult {
        consumed: true,
        needs_redraw: true,
        target: target_id,
        target_tag,
        cursor: None,
        response: target_id.map(|id| WidgetResponse::from_flags(id, rect, flags)),
        scroll_delta: (0.0, 0.0),
        key: Some(key),
        is_key_pressed: true,
        text_input: None,
        next_timeout: None,
    })
}

/// Routes keyboard navigation and text inputs to the currently focused widget.
pub(crate) fn handle_keyboard(
    tree: &mut UiTree,
    key: KeyCode,
    text: Option<String>,
    is_pressed: bool,
    ctrl_held: bool,
    shift_held: bool,
) -> DispatchResult {
    tree.event_state.frame_clicked_id = None;
    tree.event_state.frame_clicked_tag = None;
    tree.event_state.frame_drag_delta = None;

    if is_pressed && key == KeyCode::Escape && (tree.has_open_popup() || tree.has_open_tooltip()) {
        tree.close_all_popups();
        tree.close_tooltip();
        return DispatchResult {
            consumed: true,
            needs_redraw: true,
            key: Some(key),
            is_key_pressed: true,
            ..Default::default()
        };
    }

    if let Some(res) = try_handle_numeric_undo_shortcut(
        tree,
        key,
        text.as_deref(),
        is_pressed,
        ctrl_held,
        shift_held,
    ) {
        return res;
    }

    let target_id = tree.event_state.focused.or_else(|| {
        if let Some(tag) = tree.event_state.focused_tag {
            tree.find_node_by_tag(tag)
        } else if let Some(ref edit) = tree.event_state.active_text_edit {
            tree.event_state.focused_tag = Some(edit.tag);
            tree.find_node_by_tag(edit.tag)
        } else {
            None
        }
    });

    if let Some(focused_id) = target_id {
        tree.event_state.focused = Some(focused_id);
        tree.event_state.frame_key = Some(key);
        tree.event_state.frame_text_input = if is_pressed { text.clone() } else { None };
        tree.event_state.frame_key_pressed = is_pressed;
        tree.event_state.frame_ctrl_held = ctrl_held;
        tree.event_state.frame_shift_held = shift_held;

        let tag = tree
            .event_state
            .focused_tag
            .or_else(|| tree.get(focused_id).map(|n| n.tag).filter(|&t| t != 0))
            .unwrap_or(0);
        if tag != 0 && tree.event_state.focused_tag.is_none() {
            tree.event_state.focused_tag = Some(tag);
        }
        let rect = tree
            .get(focused_id)
            .map(|n| n.computed_rect)
            .unwrap_or(crate::geometry::Rect::ZERO);

        let node_role = tree.get(focused_id).map(|n| n.role);
        let has_text_edit = tree.event_state.active_text_edit.is_some();
        let is_text_input = has_text_edit || matches!(node_role, Some(WidgetRole::TextInput));

        let mut flags = ResponseFlags::FOCUSED;
        let consumed;

        if is_text_input {
            consumed = true;
            if is_pressed
                && (text.is_some()
                    || matches!(
                        key,
                        KeyCode::Backspace | KeyCode::Delete | KeyCode::Enter | KeyCode::Space
                    ))
            {
                flags = flags.union(ResponseFlags::CHANGED);
            }
            if is_pressed && key == KeyCode::Enter {
                flags = flags.union(ResponseFlags::SUBMITTED);
            }
        } else {
            // Non-text widgets (buttons, panels, rows) only consume activation and navigation keys
            consumed = match key {
                KeyCode::Enter | KeyCode::Space => {
                    if matches!(
                        node_role,
                        Some(WidgetRole::Button | WidgetRole::Checkbox | WidgetRole::DropdownItem)
                    ) {
                        if is_pressed {
                            flags = flags.union(ResponseFlags::CLICKED);
                        }
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Tab | KeyCode::Escape => true,
                _ => false,
            };

            if !consumed {
                tree.event_state.frame_key = None;
                tree.event_state.frame_text_input = None;
            }
        }

        DispatchResult {
            consumed,
            needs_redraw: is_pressed && consumed,
            target: Some(focused_id),
            target_tag: tag,
            cursor: None,
            response: Some(WidgetResponse::from_flags(focused_id, rect, flags)),
            scroll_delta: (0.0, 0.0),
            key: if consumed { Some(key) } else { None },
            is_key_pressed: is_pressed,
            text_input: if is_pressed && consumed { text } else { None },
            next_timeout: None,
        }
    } else {
        tree.event_state.frame_key = None;
        tree.event_state.frame_text_input = None;
        tree.event_state.frame_key_pressed = false;
        DispatchResult::ignored()
    }
}

/// Routes IME preedit and commit events to the currently focused widget.
pub(crate) fn handle_ime(tree: &mut UiTree, ime_event: ImeEvent) -> DispatchResult {
    tree.event_state.frame_clicked_id = None;
    tree.event_state.frame_clicked_tag = None;
    tree.event_state.frame_drag_delta = None;

    if let Some(focused_id) = tree.event_state.focused {
        let committed_text = match ime_event {
            ImeEvent::Commit(ref s) => Some(s.clone()),
            ImeEvent::Preedit(ref s, _) => Some(s.clone()),
            _ => None,
        };

        tree.event_state.frame_key = None;
        tree.event_state.frame_text_input = committed_text.clone();
        tree.event_state.frame_key_pressed = true;

        let tag = tree.event_state.focused_tag.unwrap_or(0);
        let rect = tree
            .get(focused_id)
            .map(|n| n.computed_rect)
            .unwrap_or(crate::geometry::Rect::ZERO);

        let mut flags = ResponseFlags::FOCUSED;
        if committed_text.is_some() {
            flags = flags.union(ResponseFlags::CHANGED);
        }

        DispatchResult {
            consumed: true,
            needs_redraw: committed_text.is_some(),
            target: Some(focused_id),
            target_tag: tag,
            cursor: None,
            response: Some(WidgetResponse::from_flags(focused_id, rect, flags)),
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: committed_text,
            next_timeout: None,
        }
    } else {
        tree.event_state.frame_key = None;
        tree.event_state.frame_text_input = None;
        tree.event_state.frame_key_pressed = false;
        DispatchResult::ignored()
    }
}