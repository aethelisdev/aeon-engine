// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Mouse Click & Press Dispatch Handlers
//!
//! Evaluates mouse button down/up transitions, focus acquisition, double-click windowing,
//! ancestor role bubbling (titlebars, inputs, buttons, checkboxes), and click confirmations.

use std::time::Instant;

use super::scrollbar::{
    find_associated_viewport_and_thumb, find_associated_viewport_and_track,
    perform_track_jump_scroll, resolve_effective_cursor,
};
use crate::dispatch::types::{
    DOUBLE_CLICK_MAX_DELAY_MS, DOUBLE_CLICK_MAX_DISTANCE_SQ, DispatchResult,
};
use crate::event::MouseButton;
use crate::geometry::{Point, Rect};
use crate::id::WidgetId;
use crate::node::{UiLayer, WidgetRole};
use crate::response::{ResponseFlags, WidgetResponse};
use crate::tree::UiTree;

/// Processes mouse button press inputs, initiating press states, focus, and drag capture.
pub(crate) fn handle_mouse_down(
    tree: &mut UiTree,
    button: MouseButton,
    point: Point,
) -> DispatchResult {
    tree.close_tooltip();
    tree.event_state.frame_clicked_id = None;
    tree.event_state.frame_clicked_tag = None;
    tree.event_state.frame_double_clicked_id = None;
    tree.event_state.frame_double_clicked_tag = None;
    tree.event_state.frame_secondary_clicked_id = None;
    tree.event_state.frame_secondary_clicked_tag = None;
    tree.event_state.frame_drag_released_id = None;
    tree.event_state.frame_drag_released_tag = None;
    tree.event_state.frame_clicked_pos = None;
    tree.event_state.frame_clicked_rect = None;
    tree.event_state.frame_drag_delta = None;

    let hit_id = tree.hit_test_layered(point);

    // Autonomous popup, context menu, combobox, and menu bar dismiss when clicking outside
    let had_popups_dismissed = if tree.has_open_popup() {
        let is_inside_open_popup = if let Some(hid) = hit_id {
            // Check if clicked widget is inside the context menu container
            let inside_context_menu = if let Some(active) = tree.event_state.active_context_menu {
                if let Some(menu_cid) = active.container_id {
                    hid == menu_cid || tree.is_descendant_of(hid, menu_cid)
                } else {
                    false
                }
            } else {
                false
            };

            // Check if clicked widget belongs to a popup layer or elevated dropdown role
            let is_popup_widget = tree.effective_layer(hid) >= UiLayer::Popup
                || tree
                    .find_ancestor_with_role(hid, WidgetRole::DropdownPopup)
                    .is_some()
                || tree.get(hid).is_some_and(|n| {
                    matches!(
                        n.role,
                        WidgetRole::DropdownPopup | WidgetRole::DropdownItem | WidgetRole::Tooltip
                    ) || (tree.active_menu_bar_tag.is_some()
                        && Some(n.tag) == tree.active_menu_bar_tag)
                        || (tree.active_combobox_tag.is_some()
                            && Some(n.tag) == tree.active_combobox_tag)
                });

            inside_context_menu || is_popup_widget
        } else {
            false
        };

        if !is_inside_open_popup {
            tree.close_all_popups()
        } else {
            false
        }
    } else {
        false
    };

    tree.event_state.pressed = hit_id;
    tree.event_state.pressed_button = Some(button);
    tree.event_state.press_origin = Some(point);
    tree.event_state.last_pointer_pos = Some(point);

    if let Some(target_id) = hit_id {
        // Synchronize keyboard focus with the newly clicked interactive target
        let tag = tree.get(target_id).map(|n| n.tag).unwrap_or(0);
        let effective_tag = if tag != 0 {
            tag
        } else {
            tree.resolve_ancestor_tag(target_id)
        };
        tree.event_state.pressed_tag = if effective_tag != 0 {
            Some(effective_tag)
        } else {
            None
        };
        tree.event_state.focused = Some(target_id);
        tree.event_state.focused_tag = if effective_tag != 0 {
            Some(effective_tag)
        } else {
            None
        };

        let role = tree
            .get(target_id)
            .map(|n| n.role)
            .unwrap_or(WidgetRole::Default);

        // If clicking on non-button child of a ModalTitlebar, FloatingTitlebar, NumericInput, ColorPickerArea, or TextInput, bubble target to interactive ancestor.
        // ScrollBarThumb and ScrollBarTrack are autonomous interactive controls and must never bubble to ancestors.
        let (effective_target_id, effective_role) = if role != WidgetRole::Button
            && role != WidgetRole::ScrollBarThumb
            && role != WidgetRole::ScrollBarTrack
        {
            if let Some(tb_id) = tree
                .find_ancestor_with_role(target_id, WidgetRole::ModalTitlebar)
                .or_else(|| tree.find_ancestor_with_role(target_id, WidgetRole::FloatingTitlebar))
            {
                let tb_role = tree
                    .get(tb_id)
                    .map_or(WidgetRole::ModalTitlebar, |n| n.role);
                (tb_id, tb_role)
            } else if let Some(num_id) =
                tree.find_ancestor_with_role(target_id, WidgetRole::NumericInput)
            {
                (num_id, WidgetRole::NumericInput)
            } else if let Some(cp_id) =
                tree.find_ancestor_with_role(target_id, WidgetRole::ColorPickerArea)
            {
                (cp_id, WidgetRole::ColorPickerArea)
            } else if let Some(ti_id) =
                tree.find_ancestor_with_role(target_id, WidgetRole::TextInput)
            {
                (ti_id, WidgetRole::TextInput)
            } else if !role.is_interactive()
                && let Some(btn_id) = tree.find_ancestor_with_role(target_id, WidgetRole::Button)
            {
                (btn_id, WidgetRole::Button)
            } else if !role.is_interactive()
                && let Some(cb_id) = tree.find_ancestor_with_role(target_id, WidgetRole::Checkbox)
            {
                (cb_id, WidgetRole::Checkbox)
            } else {
                (target_id, role)
            }
        } else {
            (target_id, role)
        };

        let effective_tag = if let Some(n) = tree.get(effective_target_id)
            && n.tag != 0
        {
            n.tag
        } else if tag != 0 {
            tag
        } else {
            tree.resolve_ancestor_tag(target_id)
        };

        tree.event_state.pressed = Some(effective_target_id);
        tree.event_state.pressed_tag = if effective_tag != 0 {
            Some(effective_tag)
        } else {
            None
        };
        tree.event_state.focused = Some(effective_target_id);
        tree.event_state.focused_tag = if effective_tag != 0 {
            Some(effective_tag)
        } else {
            None
        };

        // Special case: Autonomous ScrollBarThumb capture inception
        if effective_role == WidgetRole::ScrollBarThumb && button == MouseButton::Left {
            tree.event_state.pointer_capture = Some(effective_target_id);
            tree.event_state.pointer_capture_tag = if effective_tag != 0 {
                Some(effective_tag)
            } else {
                tree.get(effective_target_id)
                    .map(|n| n.tag)
                    .filter(|&t| t != 0)
            };
            if let Some((vp_id, _)) = find_associated_viewport_and_track(tree, effective_target_id)
            {
                tree.event_state.initial_scroll_y = tree
                    .get(vp_id)
                    .map(|v| v.style.scroll_offset_y)
                    .unwrap_or(0.0);
            }
        } else if effective_role == WidgetRole::ScrollBarTrack && button == MouseButton::Left {
            // Special case: Autonomous ScrollBarTrack click-to-jump gesture
            if let Some((vp_id, thumb_id)) =
                find_associated_viewport_and_thumb(tree, effective_target_id)
            {
                perform_track_jump_scroll(tree, vp_id, effective_target_id, thumb_id, point);
                tree.event_state.pointer_capture = Some(thumb_id);
                tree.event_state.pointer_capture_tag =
                    tree.get(thumb_id).map(|n| n.tag).filter(|&t| t != 0);
                tree.event_state.initial_scroll_y = tree
                    .get(vp_id)
                    .map(|v| v.style.scroll_offset_y)
                    .unwrap_or(0.0);
            }
        } else if (effective_role == WidgetRole::NumericInput
            || effective_role == WidgetRole::TextInput
            || effective_role == WidgetRole::ModalTitlebar
            || effective_role == WidgetRole::FloatingTitlebar
            || effective_role == WidgetRole::ColorPickerArea)
            && button == MouseButton::Left
        {
            // Autonomous pointer capture inception for drag scrubbers, draggable modal/floating titlebars, and color pickers
            tree.event_state.pointer_capture = Some(effective_target_id);
            tree.event_state.pointer_capture_tag = if effective_tag != 0 {
                Some(effective_tag)
            } else {
                tree.get(effective_target_id)
                    .map(|n| n.tag)
                    .filter(|&t| t != 0)
            };
        }

        let rect = tree
            .get(effective_target_id)
            .map(|n| n.computed_rect)
            .unwrap_or(crate::geometry::Rect::ZERO);
        if effective_role == WidgetRole::TextInput && button == MouseButton::Left {
            tree.event_state.frame_clicked_pos = Some(point);
            tree.event_state.frame_clicked_rect = Some(rect);
        }
        let cursor = resolve_effective_cursor(tree, effective_target_id);

        let is_interactive = effective_role.is_interactive()
            || effective_tag != 0
            || tree.event_state.pointer_capture.is_some();

        // A mouse press is strictly a down state (HOVERED | PRESSED), NEVER a click
        let flags = ResponseFlags::HOVERED | ResponseFlags::PRESSED;

        DispatchResult {
            consumed: is_interactive || had_popups_dismissed,
            needs_redraw: is_interactive || had_popups_dismissed,
            target: Some(target_id),
            target_tag: tag,
            cursor,
            response: Some(WidgetResponse::from_flags(target_id, rect, flags)),
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: None,
            next_timeout: None,
        }
    } else {
        // Clicking on empty area clears active keyboard and tag focus
        tree.event_state.clear_focus();
        if had_popups_dismissed {
            DispatchResult {
                consumed: true,
                needs_redraw: true,
                ..DispatchResult::ignored()
            }
        } else {
            DispatchResult::ignored()
        }
    }
}

/// Records a confirmed mouse click and evaluates whether it constitutes a double-click gesture.
pub(crate) fn record_click_and_check_double(
    tree: &mut UiTree,
    target_id: WidgetId,
    tag: Option<u64>,
    point: Point,
    rect: Rect,
) -> bool {
    let effective_tag = tag.unwrap_or(0);
    let is_double_click = if let (Some(last_pos), Some(last_time)) = (
        tree.event_state.last_click_pos,
        tree.event_state.last_click_time,
    ) {
        let elapsed = last_time.elapsed().as_millis();
        let dx = point.x - last_pos.x;
        let dy = point.y - last_pos.y;
        let dist_sq = dx * dx + dy * dy;
        let id_or_tag_matches = tree.event_state.last_click_id == Some(target_id)
            || (effective_tag != 0
                && tree
                    .event_state
                    .last_click_tag
                    .is_some_and(|lt| lt == effective_tag))
            || (effective_tag != 0
                && tree
                    .event_state
                    .last_click_id
                    .and_then(|lid| tree.get(lid))
                    .map_or(0, |n| n.tag)
                    == effective_tag)
            || rect.contains_point(last_pos);
        id_or_tag_matches
            && elapsed <= DOUBLE_CLICK_MAX_DELAY_MS
            && dist_sq <= DOUBLE_CLICK_MAX_DISTANCE_SQ
    } else {
        false
    };

    if is_double_click {
        tree.event_state.frame_double_clicked_id = Some(target_id);
        tree.event_state.frame_double_clicked_tag = if effective_tag != 0 {
            Some(effective_tag)
        } else {
            None
        };
        tree.event_state.last_click_id = None;
        tree.event_state.last_click_tag = None;
        tree.event_state.last_click_pos = None;
        tree.event_state.last_click_time = None;
    } else {
        tree.event_state.frame_double_clicked_id = None;
        tree.event_state.frame_double_clicked_tag = None;
        tree.event_state.last_click_id = Some(target_id);
        tree.event_state.last_click_tag = if effective_tag != 0 {
            Some(effective_tag)
        } else {
            None
        };
        tree.event_state.last_click_pos = Some(point);
        tree.event_state.last_click_time = Some(Instant::now());
    }

    is_double_click
}

/// Processes mouse button release inputs, detecting click and double-click gestures.
pub(crate) fn handle_mouse_up(
    tree: &mut UiTree,
    button: MouseButton,
    point: Point,
) -> DispatchResult {
    // 1. Check if an active pointer capture is being released
    let captured_id = tree
        .event_state
        .pointer_capture
        .filter(|&id| tree.get(id).is_some())
        .or_else(|| {
            let tag = tree.event_state.pointer_capture_tag?;
            tree.find_node_by_tag(tag)
        });

    let captured_info = captured_id.and_then(|id| {
        tree.get(id).map(|n| {
            (
                id,
                n.tag,
                n.computed_rect,
                n.cursor.or_else(|| n.role.default_cursor()),
            )
        })
    });

    if let Some((captured_id, tag, rect, cursor)) = captured_info {
        let press_origin = tree.event_state.press_origin;
        let original_button = tree.event_state.pressed_button;
        let is_drag_gesture = tree.event_state.is_pointer_drag_active
            || press_origin.is_some_and(|origin| {
                let dx = point.x - origin.x;
                let dy = point.y - origin.y;
                (dx * dx + dy * dy) > 9.0 // > 3.0 px drag threshold
            });

        let captured_tag = if tag != 0 {
            Some(tag)
        } else {
            tree.event_state.pointer_capture_tag.or_else(|| {
                let anc = tree.resolve_ancestor_tag(captured_id);
                if anc != 0 { Some(anc) } else { None }
            })
        };

        tree.event_state.pointer_capture = None;
        tree.event_state.pointer_capture_tag = None;
        tree.event_state.pressed = None;
        tree.event_state.pressed_tag = None;
        tree.event_state.pressed_button = None;
        tree.event_state.press_origin = None;
        tree.event_state.last_pointer_pos = None;
        tree.event_state.initial_scroll_y = 0.0;
        tree.event_state.frame_drag_delta = None;
        tree.event_state.is_pointer_drag_active = false;
        tree.event_state.text_drag_anchor = None;

        let mut flags = ResponseFlags::empty();

        if !is_drag_gesture && original_button == Some(button) {
            match button {
                MouseButton::Left => {
                    flags = flags.union(ResponseFlags::CLICKED);
                    tree.event_state.frame_clicked_id = Some(captured_id);
                    tree.event_state.frame_clicked_tag = captured_tag;
                    tree.event_state.frame_clicked_pos = Some(point);
                    tree.event_state.frame_clicked_rect = Some(rect);
                    tree.event_state.focused = Some(captured_id);
                    tree.event_state.focused_tag = captured_tag;

                    if record_click_and_check_double(tree, captured_id, captured_tag, point, rect) {
                        flags = flags.union(ResponseFlags::DOUBLE_CLICKED);
                    }
                }
                MouseButton::Right => {
                    flags = flags.union(ResponseFlags::SECONDARY_CLICKED);
                    tree.event_state.frame_secondary_clicked_id = Some(captured_id);
                    tree.event_state.frame_secondary_clicked_tag = captured_tag;
                    tree.event_state.frame_clicked_pos = Some(point);
                    tree.event_state.frame_clicked_rect = Some(rect);
                }
                _ => {}
            }
        } else if is_drag_gesture && original_button == Some(button) {
            flags = flags.union(ResponseFlags::DRAG_RELEASED);
            tree.event_state.frame_drag_released_id = Some(captured_id);
            tree.event_state.frame_drag_released_tag = captured_tag;
            tree.event_state.frame_clicked_pos = None;
            tree.event_state.frame_clicked_rect = Some(rect);
        }

        return DispatchResult {
            consumed: true,
            needs_redraw: true,
            target: Some(captured_id),
            target_tag: tag,
            cursor,
            response: Some(WidgetResponse::from_flags(captured_id, rect, flags)),
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: None,
            next_timeout: None,
        };
    }

    tree.event_state.pointer_capture = None;
    tree.event_state.pointer_capture_tag = None;

    // 2. Normal click detection: pressed and released on identical widget with identical button
    let hit_id = tree.hit_test_layered(point);
    let original_pressed = tree.event_state.pressed;
    let original_pressed_tag = tree.event_state.pressed_tag;
    let original_button = tree.event_state.pressed_button;
    let original_press_origin = tree.event_state.press_origin;
    tree.event_state.pressed = None;
    tree.event_state.pressed_tag = None;
    tree.event_state.pressed_button = None;
    tree.event_state.press_origin = None;
    tree.event_state.last_pointer_pos = None;
    tree.event_state.initial_scroll_y = 0.0;

    if let Some(target_id) = hit_id {
        let (tag, rect) = tree
            .get(target_id)
            .map(|n| (n.tag, n.computed_rect))
            .unwrap_or((0, crate::geometry::Rect::ZERO));
        let cursor = resolve_effective_cursor(tree, target_id);

        let role = tree
            .get(target_id)
            .map(|n| n.role)
            .unwrap_or(WidgetRole::Default);

        // If clicking on non-interactive child of an interactive ancestor (ModalTitlebar, FloatingTitlebar, NumericInput, ColorPickerArea, TextInput, or Button), bubble target
        let (effective_target_id, effective_role) = if role != WidgetRole::Button {
            if let Some(tb_id) = tree
                .find_ancestor_with_role(target_id, WidgetRole::ModalTitlebar)
                .or_else(|| tree.find_ancestor_with_role(target_id, WidgetRole::FloatingTitlebar))
            {
                let tb_role = tree
                    .get(tb_id)
                    .map_or(WidgetRole::ModalTitlebar, |n| n.role);
                (tb_id, tb_role)
            } else if let Some(num_id) =
                tree.find_ancestor_with_role(target_id, WidgetRole::NumericInput)
            {
                (num_id, WidgetRole::NumericInput)
            } else if let Some(cp_id) =
                tree.find_ancestor_with_role(target_id, WidgetRole::ColorPickerArea)
            {
                (cp_id, WidgetRole::ColorPickerArea)
            } else if let Some(ti_id) =
                tree.find_ancestor_with_role(target_id, WidgetRole::TextInput)
            {
                (ti_id, WidgetRole::TextInput)
            } else if !role.is_interactive()
                && let Some(btn_id) = tree.find_ancestor_with_role(target_id, WidgetRole::Button)
            {
                (btn_id, WidgetRole::Button)
            } else if !role.is_interactive()
                && let Some(cb_id) = tree.find_ancestor_with_role(target_id, WidgetRole::Checkbox)
            {
                (cb_id, WidgetRole::Checkbox)
            } else {
                (target_id, role)
            }
        } else {
            (target_id, role)
        };

        let effective_tag = if let Some(n) = tree.get(effective_target_id)
            && n.tag != 0
        {
            n.tag
        } else if tag != 0 {
            tag
        } else {
            tree.resolve_ancestor_tag(target_id)
        };

        let is_interactive = effective_role.is_interactive()
            || effective_tag != 0
            || tree.event_state.pointer_capture.is_some();

        let effective_rect = tree
            .get(effective_target_id)
            .map(|n| n.computed_rect)
            .unwrap_or(rect);

        // Click confirmed if release target and button match initial press across frames:
        // 1. Direct WidgetId match (same generation/frame)
        // 2. Persistent semantic tag match (across tree rebuilds)
        // 3. Coordinate bounds match (press_origin and release point both inside widget computed_rect)
        let matches_pressed = original_pressed == Some(target_id)
            || original_pressed == Some(effective_target_id)
            || (original_pressed_tag.is_some() && original_pressed_tag == Some(effective_tag))
            || (original_press_origin.is_some_and(|origin| effective_rect.contains_point(origin))
                && effective_rect.contains_point(point));

        let is_drag_gesture = tree.event_state.is_pointer_drag_active;

        if matches_pressed && original_button == Some(button) {
            let mut flags = ResponseFlags::HOVERED;

            if is_drag_gesture {
                flags = flags.union(ResponseFlags::DRAG_RELEASED);
                tree.event_state.frame_drag_released_id = Some(effective_target_id);
                tree.event_state.frame_drag_released_tag = if effective_tag != 0 {
                    Some(effective_tag)
                } else {
                    None
                };
                tree.event_state.frame_clicked_pos = None;
                tree.event_state.frame_clicked_rect = Some(effective_rect);
            } else {
                tree.event_state.frame_clicked_pos = Some(point);
                tree.event_state.frame_clicked_rect = Some(effective_rect);

                match button {
                    MouseButton::Left => {
                        flags = flags.union(ResponseFlags::CLICKED);
                        tree.event_state.frame_clicked_id = Some(effective_target_id);
                        tree.event_state.frame_clicked_tag = if effective_tag != 0 {
                            Some(effective_tag)
                        } else {
                            None
                        };
                        tree.event_state.focused = Some(effective_target_id);
                        tree.event_state.focused_tag = if effective_tag != 0 {
                            Some(effective_tag)
                        } else {
                            None
                        };

                        if record_click_and_check_double(
                            tree,
                            effective_target_id,
                            if effective_tag != 0 {
                                Some(effective_tag)
                            } else {
                                None
                            },
                            point,
                            effective_rect,
                        ) {
                            flags = flags.union(ResponseFlags::DOUBLE_CLICKED);
                        }
                    }
                    MouseButton::Right => {
                        flags = flags.union(ResponseFlags::SECONDARY_CLICKED);
                        tree.event_state.frame_secondary_clicked_id = Some(effective_target_id);
                        tree.event_state.frame_secondary_clicked_tag = if effective_tag != 0 {
                            Some(effective_tag)
                        } else {
                            None
                        };
                    }
                    MouseButton::Middle => {
                        flags = flags.union(ResponseFlags::MIDDLE_CLICKED);
                    }
                    MouseButton::Other(_) => {}
                }
            }

            DispatchResult {
                consumed: is_interactive,
                needs_redraw: true,
                target: Some(effective_target_id),
                target_tag: effective_tag,
                cursor,
                response: Some(WidgetResponse::from_flags(
                    effective_target_id,
                    effective_rect,
                    flags,
                )),
                scroll_delta: (0.0, 0.0),
                key: None,
                is_key_pressed: false,
                text_input: None,
                next_timeout: None,
            }
        } else {
            DispatchResult {
                consumed: false,
                needs_redraw: false,
                target: Some(target_id),
                target_tag: tag,
                cursor,
                response: Some(WidgetResponse::from_flags(
                    target_id,
                    rect,
                    ResponseFlags::HOVERED,
                )),
                scroll_delta: (0.0, 0.0),
                key: None,
                is_key_pressed: false,
                text_input: None,
                next_timeout: None,
            }
        }
    } else {
        DispatchResult::ignored()
    }
}