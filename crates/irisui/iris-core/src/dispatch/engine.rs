// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchical Event Dispatch Pipeline & Autonomous Interactors (`iris-core::dispatch::engine`)
//!
//! Orchestrates the routing of windowing [`UiEvent`] inputs against the active [`UiTree`] hierarchy.
//! Includes autonomous scrollbar thumb dragging, track click-to-jump, and wheel bubbling.
//!

use std::time::Instant;

use super::types::{DOUBLE_CLICK_MAX_DELAY_MS, DOUBLE_CLICK_MAX_DISTANCE_SQ, DispatchResult};
use crate::event::{ImeEvent, KeyCode, MouseButton, UiEvent};
use crate::geometry::Point;
use crate::id::WidgetId;
use crate::node::{UiLayer, WidgetCursor, WidgetRole};
use crate::response::{ResponseFlags, WidgetResponse};
use crate::tree::UiTree;

/// Dispatches an incoming [`UiEvent`] windowing input through the [`UiTree`] hierarchy.
///
/// # Execution Pipeline
/// 1. **Pointer Capture Evaluation**: If pointer capture is active, all pointer
///    events are routed exclusively to the capturing widget, bypassing spatial hit-testing.
/// 2. **Layered Spatial Hit-Testing**: Evaluates pointer coordinates across layer priority
///    stacking contexts with automatic modal window occlusion filtering.
/// 3. **Autonomous Scroll Interactivity**: Automatically handles scrollbar thumb drags,
///    track click-to-jump gestures, and mouse wheel bubbling.
/// 4. **State Machine Transitions**: Synchronizes hover, pressed, click, double-click,
///    and keyboard focus transitions on [`super::types::EventState`].
pub fn dispatch_ui_event(tree: &mut UiTree, event: &UiEvent) -> DispatchResult {
    // Clear transient frame click markers when starting new gestures or non-pointer events,
    // preserving clicks across immediate micro-motion until the frame finishes or a new press occurs
    if matches!(
        event,
        UiEvent::MouseDown { .. } | UiEvent::MouseScroll { .. }
    ) {
        tree.event_state.frame_clicked_id = None;
        tree.event_state.frame_clicked_tag = None;
        tree.event_state.frame_double_clicked_id = None;
        tree.event_state.frame_double_clicked_tag = None;
        tree.event_state.frame_secondary_clicked_id = None;
        tree.event_state.frame_secondary_clicked_tag = None;
    }

    match event {
        UiEvent::MouseMove { point } => handle_mouse_move(tree, *point),
        UiEvent::MouseDown { button, point } => handle_mouse_down(tree, *button, *point),
        UiEvent::MouseUp { button, point } => handle_mouse_up(tree, *button, *point),
        UiEvent::MouseScroll {
            delta_x,
            delta_y,
            point,
        } => handle_mouse_scroll(tree, *delta_x, *delta_y, *point),
        UiEvent::KeyboardInput {
            key,
            text,
            is_pressed,
        } => handle_keyboard(tree, *key, text.clone(), *is_pressed),
        UiEvent::Ime(ime_event) => handle_ime(tree, ime_event.clone()),
    }
}

/// Processes cursor motion inputs, updating hover states and servicing active pointer capture.
fn handle_mouse_move(tree: &mut UiTree, point: Point) -> DispatchResult {
    // 1. Pointer capture overrides normal hit-testing (drag gesture in progress)
    if let Some((captured_id, node)) = tree
        .event_state
        .pointer_capture
        .and_then(|id| tree.get(id).map(|node| (id, node)))
    {
        // Special case: Autonomous ScrollBarThumb dragging
        if node.role == WidgetRole::ScrollBarThumb
            && let Some(press_origin) = tree.event_state.press_origin
        {
            let delta_y = point.y - press_origin.y;
            let initial_scroll = tree.event_state.initial_scroll_y;
            update_scrollbar_drag(tree, captured_id, delta_y, initial_scroll);
        }

        let delta = if let Some(last_pos) = tree.event_state.last_pointer_pos {
            Point::new(point.x - last_pos.x, point.y - last_pos.y)
        } else if let Some(origin) = tree.event_state.press_origin {
            Point::new(point.x - origin.x, point.y - origin.y)
        } else {
            Point::ZERO
        };
        tree.event_state.frame_drag_delta = Some(delta);
        tree.event_state.last_pointer_pos = Some(point);

        let node = tree.get(captured_id).unwrap();
        let cursor = node.cursor.or_else(|| node.role.default_cursor());
        let tag = node.tag;
        let rect = node.computed_rect;
        let flags = ResponseFlags::HOVERED | ResponseFlags::DRAGGED;

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
        };
    }

    // 2. Perform layered hit-testing against active tree
    let hit_id = tree.hit_test_layered(point);
    tree.event_state.hovered = hit_id;

    if let Some(target_id) = hit_id {
        // Find cursor override traversing upward if child does not specify one
        let cursor = resolve_effective_cursor(tree, target_id);
        let (tag, rect) = tree
            .get(target_id)
            .map(|n| (n.tag, n.computed_rect))
            .unwrap_or((0, crate::geometry::Rect::ZERO));

        let mut flags = ResponseFlags::HOVERED;
        if tree.event_state.pressed == Some(target_id) {
            flags = flags.union(ResponseFlags::PRESSED);
        }

        DispatchResult {
            consumed: false, // Normal hover motion does not consume the event
            needs_redraw: false,
            target: Some(target_id),
            target_tag: tag,
            cursor,
            response: Some(WidgetResponse::from_flags(target_id, rect, flags)),
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: None,
        }
    } else {
        DispatchResult::ignored()
    }
}

/// Processes mouse button press inputs, initiating press states, focus, and drag capture.
fn handle_mouse_down(tree: &mut UiTree, button: MouseButton, point: Point) -> DispatchResult {
    tree.event_state.frame_clicked_id = None;
    tree.event_state.frame_clicked_tag = None;
    tree.event_state.frame_double_clicked_id = None;
    tree.event_state.frame_double_clicked_tag = None;
    tree.event_state.frame_secondary_clicked_id = None;
    tree.event_state.frame_secondary_clicked_tag = None;
    tree.event_state.frame_clicked_pos = None;
    tree.event_state.frame_drag_delta = None;

    let hit_id = tree.hit_test_layered(point);

    // Autonomous context menu dismiss when clicking outside its bounds
    if let Some(active) = tree.event_state.active_context_menu {
        let is_inside_menu = if let Some(menu_cid) = active.container_id {
            hit_id == Some(menu_cid)
                || hit_id.is_some_and(|hid| {
                    tree.is_descendant_of(hid, menu_cid)
                        || tree.get(hid).is_some_and(|n| n.layer == UiLayer::Popup)
                })
        } else {
            hit_id.is_some_and(|hid| tree.get(hid).is_some_and(|n| n.layer == UiLayer::Popup))
        };
        if !is_inside_menu {
            tree.event_state.active_context_menu = None;
        }
    }

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

        // Special case: Autonomous ScrollBarThumb capture inception
        if role == WidgetRole::ScrollBarThumb && button == MouseButton::Left {
            tree.event_state.pointer_capture = Some(target_id);
            if let Some((vp_id, _)) = find_associated_viewport_and_track(tree, target_id) {
                tree.event_state.initial_scroll_y = tree
                    .get(vp_id)
                    .map(|v| v.style.scroll_offset_y)
                    .unwrap_or(0.0);
            }
        } else if role == WidgetRole::ScrollBarTrack && button == MouseButton::Left {
            // Special case: Autonomous ScrollBarTrack click-to-jump gesture
            if let Some((vp_id, thumb_id)) = find_associated_viewport_and_thumb(tree, target_id) {
                perform_track_jump_scroll(tree, vp_id, target_id, thumb_id, point);
                tree.event_state.pointer_capture = Some(thumb_id);
                tree.event_state.initial_scroll_y = tree
                    .get(vp_id)
                    .map(|v| v.style.scroll_offset_y)
                    .unwrap_or(0.0);
            }
        } else if role == WidgetRole::NumericInput && button == MouseButton::Left {
            // Autonomous NumericInput pointer capture inception for drag scrubbers
            tree.event_state.pointer_capture = Some(target_id);
        }

        let rect = tree
            .get(target_id)
            .map(|n| n.computed_rect)
            .unwrap_or(crate::geometry::Rect::ZERO);
        let cursor = resolve_effective_cursor(tree, target_id);

        let is_interactive = role.is_interactive()
            || effective_tag != 0
            || tree.event_state.pointer_capture.is_some();

        // A mouse press is strictly a down state (HOVERED | PRESSED), NEVER a click
        let flags = ResponseFlags::HOVERED | ResponseFlags::PRESSED;

        DispatchResult {
            consumed: is_interactive,
            needs_redraw: is_interactive,
            target: Some(target_id),
            target_tag: tag,
            cursor,
            response: Some(WidgetResponse::from_flags(target_id, rect, flags)),
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: None,
        }
    } else {
        // Clicking on empty area clears active keyboard and tag focus
        tree.event_state.clear_focus();
        DispatchResult::ignored()
    }
}

/// Processes mouse button release inputs, detecting click and double-click gestures.
fn handle_mouse_up(tree: &mut UiTree, button: MouseButton, point: Point) -> DispatchResult {
    // 1. Check if an active pointer capture is being released
    if let Some(captured_id) = tree.event_state.pointer_capture {
        tree.event_state.pointer_capture = None;
        tree.event_state.pressed = None;
        tree.event_state.pressed_button = None;
        tree.event_state.press_origin = None;
        tree.event_state.last_pointer_pos = None;
        tree.event_state.initial_scroll_y = 0.0;
        tree.event_state.frame_drag_delta = None;

        let (tag, rect, cursor) = tree
            .get(captured_id)
            .map(|n| {
                (
                    n.tag,
                    n.computed_rect,
                    n.cursor.or_else(|| n.role.default_cursor()),
                )
            })
            .unwrap_or((0, crate::geometry::Rect::ZERO, None));

        return DispatchResult {
            consumed: true,
            needs_redraw: true,
            target: Some(captured_id),
            target_tag: tag,
            cursor,
            response: Some(WidgetResponse::from_flags(
                captured_id,
                rect,
                ResponseFlags::empty(),
            )),
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: None,
        };
    }

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

        let effective_tag = if tag != 0 {
            tag
        } else {
            tree.resolve_ancestor_tag(target_id)
        };

        let role = tree
            .get(target_id)
            .map(|n| n.role)
            .unwrap_or(WidgetRole::Default);
        let is_interactive = role.is_interactive()
            || effective_tag != 0
            || tree.event_state.pointer_capture.is_some();

        // Click confirmed if release target and button match initial press across frames:
        // 1. Direct WidgetId match (same generation/frame)
        // 2. Persistent semantic tag match (across tree rebuilds)
        // 3. Coordinate bounds match (press_origin and release point both inside widget computed_rect)
        let matches_pressed = original_pressed == Some(target_id)
            || (original_pressed_tag.is_some() && original_pressed_tag == Some(effective_tag))
            || (original_press_origin.is_some_and(|origin| rect.contains_point(origin))
                && rect.contains_point(point));

        if matches_pressed && original_button == Some(button) {
            let mut flags = ResponseFlags::HOVERED;

            tree.event_state.frame_clicked_pos = Some(point);

            match button {
                MouseButton::Left => {
                    flags = flags.union(ResponseFlags::CLICKED);
                    tree.event_state.frame_clicked_id = Some(target_id);
                    tree.event_state.frame_clicked_tag = if effective_tag != 0 {
                        Some(effective_tag)
                    } else {
                        None
                    };

                    let is_double_click = if let (Some(last_id), Some(last_pos), Some(last_time)) = (
                        tree.event_state.last_click_id,
                        tree.event_state.last_click_pos,
                        tree.event_state.last_click_time,
                    ) {
                        let elapsed = last_time.elapsed().as_millis();
                        let dx = point.x - last_pos.x;
                        let dy = point.y - last_pos.y;
                        let dist_sq = dx * dx + dy * dy;
                        let id_or_tag_matches = last_id == target_id
                            || (effective_tag != 0
                                && tree.get(last_id).map_or(0, |n| n.tag) == effective_tag)
                            || rect.contains_point(last_pos);
                        id_or_tag_matches
                            && elapsed <= DOUBLE_CLICK_MAX_DELAY_MS
                            && dist_sq <= DOUBLE_CLICK_MAX_DISTANCE_SQ
                    } else {
                        false
                    };

                    if is_double_click {
                        flags = flags.union(ResponseFlags::DOUBLE_CLICKED);
                        tree.event_state.frame_double_clicked_id = Some(target_id);
                        tree.event_state.frame_double_clicked_tag = if effective_tag != 0 {
                            Some(effective_tag)
                        } else {
                            None
                        };
                        tree.event_state.last_click_id = None;
                        tree.event_state.last_click_pos = None;
                        tree.event_state.last_click_time = None;
                    } else {
                        tree.event_state.frame_double_clicked_id = None;
                        tree.event_state.frame_double_clicked_tag = None;
                        tree.event_state.last_click_id = Some(target_id);
                        tree.event_state.last_click_pos = Some(point);
                        tree.event_state.last_click_time = Some(Instant::now());
                    }
                }
                MouseButton::Right => {
                    flags = flags.union(ResponseFlags::SECONDARY_CLICKED);
                    tree.event_state.frame_secondary_clicked_id = Some(target_id);
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

            DispatchResult {
                consumed: is_interactive,
                needs_redraw: true,
                target: Some(target_id),
                target_tag: effective_tag,
                cursor,
                response: Some(WidgetResponse::from_flags(target_id, rect, flags)),
                scroll_delta: (0.0, 0.0),
                key: None,
                is_key_pressed: false,
                text_input: None,
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
            }
        }
    } else {
        DispatchResult::ignored()
    }
}

/// Traverses upward through ancestors looking for a scroll container (`clip_children`).
///
/// Note: Interactive child widgets (e.g. buttons, inputs) deliberately DO NOT intercept
/// or block mouse scroll events, allowing bubbling to reach the enclosing scroll container.
fn handle_mouse_scroll(
    tree: &mut UiTree,
    delta_x: f32,
    delta_y: f32,
    point: Point,
) -> DispatchResult {
    let Some(hit_id) = tree.hit_test_layered(point) else {
        return DispatchResult::ignored();
    };

    // Bubble upward to find the nearest scroll-handling ancestor.
    // Interactive children (buttons, sliders, inputs) must NOT stop this bubbling.
    let mut current = Some(hit_id);
    while let Some(curr_id) = current {
        if let Some(node) = tree.get(curr_id) {
            let is_explicit_viewport = node.role == WidgetRole::ScrollAreaViewport;
            let is_generic_scrollable = node.role != WidgetRole::Button
                && node.role != WidgetRole::TextInput
                && (node.style.clip_children && node.tag != 0);

            let is_scroll_container = is_explicit_viewport || is_generic_scrollable;

            let has_overflow = if is_scroll_container {
                if node.content_size.height > 0.0 {
                    node.content_size.height > node.computed_rect.height + 0.5
                        || node.style.scroll_offset_y > 0.0
                } else {
                    true
                }
            } else {
                false
            };

            if is_scroll_container {
                if has_overflow {
                    let tag = node.tag;
                    let rect = node.computed_rect;
                    let max_scroll = if node.content_size.height > node.computed_rect.height {
                        node.content_size.height - node.computed_rect.height
                    } else if node.content_size.height == 0.0 {
                        f32::MAX
                    } else {
                        node.style.scroll_offset_y
                    };

                    // Directly update the container's vertical scroll offset in the tree
                    let new_scroll = if let Some(mut_node) = tree.get_mut(curr_id) {
                        let ns = (mut_node.style.scroll_offset_y - delta_y).clamp(0.0, max_scroll);
                        mut_node.style.scroll_offset_y = ns;
                        ns
                    } else {
                        0.0
                    };
                    if tag != 0 {
                        tree.set_scroll_offset(tag, new_scroll);
                    }

                    // Autonomous synchronization of attached scrollbar thumb geometry
                    sync_scrollbar_thumb_geometry(tree, curr_id);

                    return DispatchResult {
                        consumed: true,
                        needs_redraw: true,
                        target: Some(curr_id),
                        target_tag: tag,
                        cursor: None,
                        response: Some(WidgetResponse::from_flags(
                            curr_id,
                            rect,
                            ResponseFlags::CHANGED,
                        )),
                        scroll_delta: (delta_x, delta_y),
                        key: None,
                        is_key_pressed: false,
                        text_input: None,
                    };
                } else {
                    return DispatchResult {
                        consumed: true,
                        needs_redraw: false,
                        target: Some(curr_id),
                        target_tag: node.tag,
                        cursor: None,
                        response: Some(WidgetResponse::from_flags(
                            curr_id,
                            node.computed_rect,
                            ResponseFlags::HOVERED,
                        )),
                        scroll_delta: (0.0, 0.0),
                        key: None,
                        is_key_pressed: false,
                        text_input: None,
                    };
                }
            }
            current = node.parent;
        } else {
            break;
        }
    }

    DispatchResult::ignored()
}

/// Routes keyboard navigation and text inputs to the currently focused widget.
fn handle_keyboard(
    tree: &mut UiTree,
    key: KeyCode,
    text: Option<String>,
    is_pressed: bool,
) -> DispatchResult {
    tree.event_state.frame_clicked_id = None;
    tree.event_state.frame_clicked_tag = None;
    tree.event_state.frame_drag_delta = None;

    if is_pressed && key == KeyCode::Escape && tree.event_state.active_context_menu.is_some() {
        tree.event_state.active_context_menu = None;
        return DispatchResult {
            consumed: true,
            needs_redraw: true,
            key: Some(key),
            is_key_pressed: true,
            ..Default::default()
        };
    }

    if let Some(focused_id) = tree.event_state.focused {
        tree.event_state.frame_key = Some(key);
        tree.event_state.frame_text_input = text.clone();
        tree.event_state.frame_key_pressed = is_pressed;

        let tag = tree.event_state.focused_tag.unwrap_or(0);
        let rect = tree
            .get(focused_id)
            .map(|n| n.computed_rect)
            .unwrap_or(crate::geometry::Rect::ZERO);

        let mut flags = ResponseFlags::FOCUSED;
        if is_pressed
            && (text.is_some()
                || matches!(
                    key,
                    KeyCode::Backspace | KeyCode::Delete | KeyCode::Enter | KeyCode::Space
                ))
        {
            flags = flags.union(ResponseFlags::CHANGED);
        }

        DispatchResult {
            consumed: true,
            needs_redraw: is_pressed,
            target: Some(focused_id),
            target_tag: tag,
            cursor: None,
            response: Some(WidgetResponse::from_flags(focused_id, rect, flags)),
            scroll_delta: (0.0, 0.0),
            key: Some(key),
            is_key_pressed: is_pressed,
            text_input: text,
        }
    } else {
        tree.event_state.frame_key = None;
        tree.event_state.frame_text_input = None;
        tree.event_state.frame_key_pressed = false;
        DispatchResult::ignored()
    }
}

/// Routes IME preedit and commit events to the currently focused widget.
fn handle_ime(tree: &mut UiTree, ime_event: ImeEvent) -> DispatchResult {
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
        }
    } else {
        tree.event_state.frame_key = None;
        tree.event_state.frame_text_input = None;
        tree.event_state.frame_key_pressed = false;
        DispatchResult::ignored()
    }
}

/// Translates a thumb vertical displacement delta into a proportional viewport scroll offset.
fn update_scrollbar_drag(tree: &mut UiTree, thumb_id: WidgetId, delta_y: f32, initial_scroll: f32) {
    let Some((vp_id, track_id)) = find_associated_viewport_and_track(tree, thumb_id) else {
        return;
    };

    let (track_rect, thumb_h) = {
        let Some(track) = tree.get(track_id) else {
            return;
        };
        let Some(thumb) = tree.get(thumb_id) else {
            return;
        };
        (track.computed_rect, thumb.computed_rect.height)
    };

    let max_scroll_y = {
        let Some(vp) = tree.get(vp_id) else {
            return;
        };
        (vp.content_size.height - vp.computed_rect.height).max(0.0)
    };

    if max_scroll_y <= 0.0 {
        return;
    }

    let travel = (track_rect.height - thumb_h).max(1.0);
    let delta_scroll = (delta_y / travel) * max_scroll_y;
    let new_scroll = (initial_scroll + delta_scroll).clamp(0.0, max_scroll_y);

    let vp_tag = tree.get(vp_id).map_or(0, |n| n.tag);
    if let Some(vp) = tree.get_mut(vp_id) {
        vp.style.scroll_offset_y = new_scroll;
    }
    if vp_tag != 0 {
        tree.set_scroll_offset(vp_tag, new_scroll);
    }

    let ratio = (new_scroll / max_scroll_y).clamp(0.0, 1.0);
    let new_thumb_y = track_rect.y + ratio * travel;
    if let Some(thumb) = tree.get_mut(thumb_id) {
        thumb.computed_rect.y = new_thumb_y;
    }
}

/// Calculates and performs a direct jump scroll when the scrollbar track is clicked.
fn perform_track_jump_scroll(
    tree: &mut UiTree,
    vp_id: WidgetId,
    track_id: WidgetId,
    thumb_id: WidgetId,
    point: Point,
) {
    let (track_rect, thumb_h) = {
        let Some(track) = tree.get(track_id) else {
            return;
        };
        let Some(thumb) = tree.get(thumb_id) else {
            return;
        };
        (track.computed_rect, thumb.computed_rect.height)
    };

    let max_scroll_y = {
        let Some(vp) = tree.get(vp_id) else {
            return;
        };
        (vp.content_size.height - vp.computed_rect.height).max(0.0)
    };

    if max_scroll_y <= 0.0 {
        return;
    }

    let travel = (track_rect.height - thumb_h).max(1.0);
    let click_offset_y = point.y - track_rect.y - (thumb_h * 0.5);
    let ratio = (click_offset_y / travel).clamp(0.0, 1.0);
    let new_scroll = ratio * max_scroll_y;

    let vp_tag = tree.get(vp_id).map_or(0, |n| n.tag);
    if let Some(vp) = tree.get_mut(vp_id) {
        vp.style.scroll_offset_y = new_scroll;
    }
    if vp_tag != 0 {
        tree.set_scroll_offset(vp_tag, new_scroll);
    }

    let new_thumb_y = track_rect.y + ratio * travel;
    if let Some(thumb) = tree.get_mut(thumb_id) {
        thumb.computed_rect.y = new_thumb_y;
    }
}

/// Synchronizes a scrollbar thumb node's vertical coordinate with its associated viewport's offset.
fn sync_scrollbar_thumb_geometry(tree: &mut UiTree, viewport_id: WidgetId) {
    let (scroll_y, max_scroll_y) = {
        let Some(vp) = tree.get(viewport_id) else {
            return;
        };
        let max_scroll = (vp.content_size.height - vp.computed_rect.height).max(0.0);
        (vp.style.scroll_offset_y, max_scroll)
    };

    let Some((track_id, thumb_id)) = find_track_and_thumb_for_viewport(tree, viewport_id) else {
        return;
    };

    let (track_rect, thumb_h) = {
        let Some(track) = tree.get(track_id) else {
            return;
        };
        let Some(thumb) = tree.get(thumb_id) else {
            return;
        };
        (track.computed_rect, thumb.computed_rect.height)
    };

    let travel = (track_rect.height - thumb_h).max(0.0);
    let ratio = if max_scroll_y > 0.0 {
        (scroll_y / max_scroll_y).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let new_thumb_y = track_rect.y + ratio * travel;
    if let Some(thumb) = tree.get_mut(thumb_id) {
        thumb.computed_rect.y = new_thumb_y;
    }
}

/// Locates the associated viewport and scrollbar track for a given scrollbar thumb node.
fn find_associated_viewport_and_track(
    tree: &UiTree,
    thumb_id: WidgetId,
) -> Option<(WidgetId, WidgetId)> {
    let parent_id = tree.get(thumb_id)?.parent?;
    let parent = tree.get(parent_id)?;

    // If parent is directly the viewport
    if parent.role == WidgetRole::ScrollAreaViewport || parent.style.clip_children {
        let track_id = parent.children.iter().copied().find(|&c| {
            tree.get(c)
                .is_some_and(|n| n.role == WidgetRole::ScrollBarTrack)
        })?;
        return Some((parent_id, track_id));
    }

    // If thumb and track are siblings of the viewport under a shared parent container
    let vp_id = parent.children.iter().copied().find(|&c| {
        tree.get(c).is_some_and(|n| {
            n.role == WidgetRole::ScrollAreaViewport || (n.style.clip_children && c != thumb_id)
        })
    })?;

    let track_id = parent.children.iter().copied().find(|&c| {
        tree.get(c)
            .is_some_and(|n| n.role == WidgetRole::ScrollBarTrack)
    })?;

    Some((vp_id, track_id))
}

/// Locates the associated viewport and scrollbar thumb for a given scrollbar track node.
fn find_associated_viewport_and_thumb(
    tree: &UiTree,
    track_id: WidgetId,
) -> Option<(WidgetId, WidgetId)> {
    let parent_id = tree.get(track_id)?.parent?;
    let parent = tree.get(parent_id)?;

    if parent.role == WidgetRole::ScrollAreaViewport || parent.style.clip_children {
        let thumb_id = parent.children.iter().copied().find(|&c| {
            tree.get(c)
                .is_some_and(|n| n.role == WidgetRole::ScrollBarThumb)
        })?;
        return Some((parent_id, thumb_id));
    }

    let vp_id = parent.children.iter().copied().find(|&c| {
        tree.get(c).is_some_and(|n| {
            n.role == WidgetRole::ScrollAreaViewport || (n.style.clip_children && c != track_id)
        })
    })?;

    let thumb_id = parent.children.iter().copied().find(|&c| {
        tree.get(c)
            .is_some_and(|n| n.role == WidgetRole::ScrollBarThumb)
    })?;

    Some((vp_id, thumb_id))
}

/// Locates the scrollbar track and thumb nodes belonging to a viewport node.
fn find_track_and_thumb_for_viewport(
    tree: &UiTree,
    viewport_id: WidgetId,
) -> Option<(WidgetId, WidgetId)> {
    let vp = tree.get(viewport_id)?;

    // Check children first
    let track_child = vp.children.iter().copied().find(|&c| {
        tree.get(c)
            .is_some_and(|n| n.role == WidgetRole::ScrollBarTrack)
    });
    let thumb_child = vp.children.iter().copied().find(|&c| {
        tree.get(c)
            .is_some_and(|n| n.role == WidgetRole::ScrollBarThumb)
    });

    if let (Some(t), Some(th)) = (track_child, thumb_child) {
        return Some((t, th));
    }

    // Check sibling elements under parent
    let parent_id = vp.parent?;
    let parent = tree.get(parent_id)?;

    let track_sib = parent.children.iter().copied().find(|&c| {
        tree.get(c)
            .is_some_and(|n| n.role == WidgetRole::ScrollBarTrack)
    })?;
    let thumb_sib = parent.children.iter().copied().find(|&c| {
        tree.get(c)
            .is_some_and(|n| n.role == WidgetRole::ScrollBarThumb)
    })?;

    Some((track_sib, thumb_sib))
}

/// Resolves the effective mouse cursor icon by checking the node and bubbling to ancestors.
fn resolve_effective_cursor(tree: &UiTree, start_id: WidgetId) -> Option<WidgetCursor> {
    let mut current = Some(start_id);
    while let Some(curr_id) = current {
        if let Some(node) = tree.get(curr_id) {
            if let Some(cursor) = node.cursor.or_else(|| node.role.default_cursor()) {
                return Some(cursor);
            }
            current = node.parent;
        } else {
            break;
        }
    }
    None
}