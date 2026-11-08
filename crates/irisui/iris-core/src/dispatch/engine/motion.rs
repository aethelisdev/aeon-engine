// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Mouse Motion & Drag Tracking Dispatch Handlers
//!
//! Evaluates cursor movement inputs, autonomous drag inception, pointer capture overrides,
//! tooltip hover delays, and active cursor style resolution.

use std::time::Instant;

use super::scrollbar::{resolve_effective_cursor, update_scrollbar_drag};
use crate::dispatch::types::{DispatchResult, TOOLTIP_DEFAULT_HOVER_DELAY_MS, TooltipState};
use crate::event::MouseButton;
use crate::geometry::Point;
use crate::node::{WidgetCursor, WidgetRole};
use crate::response::{ResponseFlags, WidgetResponse};
use crate::tree::UiTree;

/// Processes cursor motion inputs, updating hover states and servicing active pointer capture.
pub(crate) fn handle_mouse_move(tree: &mut UiTree, point: Point) -> DispatchResult {
    tree.event_state.frame_drag_released_id = None;
    tree.event_state.frame_drag_released_tag = None;

    // 1. Pointer capture overrides normal hit-testing (drag gesture in progress)
    let mut captured_id = tree
        .event_state
        .pointer_capture
        .filter(|&id| tree.get(id).is_some())
        .or_else(|| {
            let tag = tree.event_state.pointer_capture_tag?;
            let id = tree.find_node_by_tag(tag)?;
            tree.event_state.pointer_capture = Some(id);
            Some(id)
        });

    // Autonomous drag capture inception: If a button was pressed down and mouse moved past drag threshold,
    // establish pointer capture on the pressed interactive/tagged widget node
    let pressed_id = tree
        .event_state
        .pressed
        .filter(|&id| tree.get(id).is_some())
        .or_else(|| {
            let tag = tree.event_state.pressed_tag?;
            let id = tree.find_node_by_tag(tag)?;
            tree.event_state.pressed = Some(id);
            Some(id)
        });

    if captured_id.is_none()
        && let Some(pressed_id) = pressed_id
        && tree.event_state.pressed_button == Some(MouseButton::Left)
        && let Some(origin) = tree.event_state.press_origin
    {
        let dx = point.x - origin.x;
        let dy = point.y - origin.y;
        if (dx * dx + dy * dy) > 9.0 {
            let is_draggable = tree.event_state.pressed_tag.is_some()
                || tree.get(pressed_id).is_some_and(|n| {
                    n.tag != 0
                        || n.role.is_interactive()
                        || n.role == WidgetRole::ModalTitlebar
                        || n.role == WidgetRole::FloatingTitlebar
                });
            if is_draggable {
                let capture_tag = tree.event_state.pressed_tag.or_else(|| {
                    let n = tree.get(pressed_id)?;
                    if n.tag != 0 {
                        Some(n.tag)
                    } else {
                        let anc = tree.resolve_ancestor_tag(pressed_id);
                        if anc != 0 { Some(anc) } else { None }
                    }
                });
                tree.event_state.pointer_capture = Some(pressed_id);
                tree.event_state.pointer_capture_tag = capture_tag;
                captured_id = Some(pressed_id);
            }
        }
    }

    if let Some(captured_id) = captured_id
        && let Some((role, cursor, tag, rect)) = tree.get(captured_id).map(|node| {
            (
                node.role,
                node.cursor.or_else(|| node.role.default_cursor()),
                node.tag,
                node.computed_rect,
            )
        })
    {
        if rect.width > 0.0 {
            tree.event_state.frame_clicked_rect = Some(rect);
        }

        // Special case: Autonomous ScrollBarThumb dragging
        if role == WidgetRole::ScrollBarThumb
            && let Some(press_origin) = tree.event_state.press_origin
        {
            tree.event_state.is_pointer_drag_active = true;
            let delta_y = point.y - press_origin.y;
            let initial_scroll = tree.event_state.initial_scroll_y;
            update_scrollbar_drag(tree, captured_id, delta_y, initial_scroll);
        } else if role == WidgetRole::ColorPickerArea {
            tree.event_state.is_pointer_drag_active = true;
        } else if role == WidgetRole::NumericInput {
            // Dual click-to-edit vs drag scrubber:
            // Dragging only activates once pointer moves past 3.0px threshold (dist_sq > 9.0)
            let is_past_threshold = tree.event_state.press_origin.is_some_and(|origin| {
                let dx = point.x - origin.x;
                let dy = point.y - origin.y;
                (dx * dx + dy * dy) > 9.0
            });
            if is_past_threshold {
                tree.event_state.is_pointer_drag_active = true;
            }
        } else {
            let is_past_threshold = tree.event_state.press_origin.is_none_or(|origin| {
                let dx = point.x - origin.x;
                let dy = point.y - origin.y;
                (dx * dx + dy * dy) > 9.0
            });
            if is_past_threshold {
                tree.event_state.is_pointer_drag_active = true;
            }
        }

        let is_editing_numeric = role == WidgetRole::NumericInput
            && tree
                .event_state
                .active_text_edit
                .as_ref()
                .is_some_and(|s| s.tag != 0 && (tag == 0 || s.tag == tag));

        let effective_cursor = if is_editing_numeric {
            Some(WidgetCursor::Text)
        } else if tree.event_state.is_pointer_drag_active {
            if role == WidgetRole::NumericInput {
                Some(WidgetCursor::EwResize)
            } else {
                cursor
            }
        } else {
            cursor
        };

        let mut flags = ResponseFlags::HOVERED;
        if tree.event_state.pressed_button.is_some() {
            flags = flags.union(ResponseFlags::PRESSED);
        }

        if tree.event_state.is_pointer_drag_active && !is_editing_numeric {
            let delta = if let Some(last_pos) = tree.event_state.last_pointer_pos {
                Point::new(point.x - last_pos.x, point.y - last_pos.y)
            } else if let Some(origin) = tree.event_state.press_origin {
                Point::new(point.x - origin.x, point.y - origin.y)
            } else {
                Point::ZERO
            };
            tree.event_state.frame_drag_delta = Some(delta);
            tree.event_state.last_pointer_pos = Some(point);
            flags = flags.union(ResponseFlags::DRAGGED);
        } else {
            tree.event_state.frame_drag_delta = None;
            tree.event_state.last_pointer_pos = Some(point);
        }

        return DispatchResult {
            consumed: true,
            needs_redraw: true,
            target: Some(captured_id),
            target_tag: tag,
            cursor: effective_cursor,
            response: Some(WidgetResponse::from_flags(captured_id, rect, flags)),
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: None,
            next_timeout: None,
        };
    }

    // 2. Perform layered hit-testing against active tree
    let hit_id = tree.hit_test_layered(point);
    tree.event_state.hovered = hit_id;

    let now = Instant::now();
    let mut next_timeout = None;
    let mut needs_redraw = false;

    if let Some(target_id) = hit_id {
        // Find cursor override traversing upward if child does not specify one
        let cursor = resolve_effective_cursor(tree, target_id);
        let (tag, rect) = tree
            .get(target_id)
            .map(|n| (n.tag, n.computed_rect))
            .unwrap_or((0, crate::geometry::Rect::ZERO));

        // Evaluate autonomous hover timer and tooltip state
        let is_same_target = if let Some(ref current) = tree.event_state.active_tooltip {
            current.target_id == target_id
                || (tag != 0 && current.target_tag == tag)
                || (current.target_tag != 0
                    && tree.resolve_ancestor_tag(target_id) == current.target_tag)
                || tree.is_descendant_of(target_id, current.target_id)
        } else {
            false
        };

        if is_same_target {
            if let Some(ref mut tooltip) = tree.event_state.active_tooltip {
                let elapsed = now.duration_since(tooltip.hover_start);
                if !tooltip.is_open {
                    if elapsed >= tooltip.delay {
                        tooltip.is_open = true;
                        needs_redraw = true;
                    } else {
                        next_timeout = Some(tooltip.delay.saturating_sub(elapsed));
                    }
                }
            }
        } else {
            if tree.event_state.has_open_tooltip() {
                needs_redraw = true;
            }
            let default_delay = std::time::Duration::from_millis(TOOLTIP_DEFAULT_HOVER_DELAY_MS);
            next_timeout = Some(default_delay);
            tree.event_state.active_tooltip = Some(TooltipState {
                target_id,
                target_tag: tag,
                anchor_pos: point,
                hover_start: now,
                delay: default_delay,
                is_open: false,
            });
        }

        let mut flags = ResponseFlags::HOVERED;
        if tree.event_state.pressed == Some(target_id) {
            flags = flags.union(ResponseFlags::PRESSED);
        }

        DispatchResult {
            consumed: tree.event_state.is_pointer_drag_active,
            needs_redraw,
            target: Some(target_id),
            target_tag: tag,
            cursor,
            response: Some(WidgetResponse::from_flags(target_id, rect, flags)),
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: None,
            next_timeout,
        }
    } else {
        if tree.event_state.has_open_tooltip() {
            needs_redraw = true;
        }
        tree.event_state.active_tooltip = None;
        DispatchResult {
            consumed: false,
            needs_redraw,
            target: None,
            target_tag: 0,
            cursor: None,
            response: None,
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: None,
            next_timeout: None,
        }
    }
}