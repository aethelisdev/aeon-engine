// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchical UI Event Dispatcher & Interaction State (`iris-core::dispatch`)
//!
//! Provides the central [`dispatch_ui_event`] engine and [`DispatchResult`] interaction models.
//!
//! Evaluates incoming [`UiEvent`] windowing inputs against the active [`UiTree`] arena,
//! coordinating z-order layer priority ([`crate::node::UiLayer`]), GPU scissor clipping boundaries,
//! pointer capture for uninterrupted drags, keyboard focus routing, and upward scroll event bubbling.
//!

use std::time::Instant;

use crate::event::{ImeEvent, KeyCode, MouseButton, UiEvent};
use crate::geometry::Point;
use crate::id::WidgetId;
use crate::node::WidgetCursor;
use crate::response::{ResponseFlags, WidgetResponse};
use crate::tree::UiTree;

/// Maximum allowable time delta in milliseconds between consecutive clicks to qualify as a double-click.
pub const DOUBLE_CLICK_MAX_DELAY_MS: u128 = 400;

/// Maximum allowable squared Euclidean distance in pixels between consecutive clicks for double-click.
pub const DOUBLE_CLICK_MAX_DISTANCE_SQ: f32 = 25.0;

/// Persistent tracking state of active user interaction within a [`UiTree`].
///
/// Maintained across frames to orchestrate pointer capture during drags,
/// hover state transitions, double-click gesture detection, and keyboard focus routing.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EventState {
    /// Active widget node currently capturing all pointer movements and releases.
    pub pointer_capture: Option<WidgetId>,
    /// Deepest interactive widget node currently underneath the pointer cursor.
    pub hovered: Option<WidgetId>,
    /// Widget node where the mouse button was originally pressed down.
    pub pressed: Option<WidgetId>,
    /// Mouse button variant (Left, Right, Middle) that initiated the current press gesture.
    pub pressed_button: Option<MouseButton>,
    /// Widget node currently holding active keyboard navigation or text input focus.
    pub focused: Option<WidgetId>,
    /// Persistent 64-bit semantic tag currently holding keyboard focus, if assigned.
    pub focused_tag: Option<u64>,
    /// Screen coordinate where the primary mouse press initiated.
    pub press_origin: Option<Point>,
    /// Last confirmed click target widget ID for double-click gesture detection.
    pub last_click_id: Option<WidgetId>,
    /// Screen coordinate of the last confirmed click gesture.
    pub last_click_pos: Option<Point>,
    /// Monotonic timestamp of the last confirmed click gesture.
    pub last_click_time: Option<Instant>,
}

impl EventState {
    /// Constructs a clean, unpopulated interaction state.
    #[inline]
    pub const fn new() -> Self {
        Self {
            pointer_capture: None,
            hovered: None,
            pressed: None,
            pressed_button: None,
            focused: None,
            focused_tag: None,
            press_origin: None,
            last_click_id: None,
            last_click_pos: None,
            last_click_time: None,
        }
    }

    /// Resets transient interaction states while preserving persistent focus.
    #[inline]
    pub fn reset_transient_pointer(&mut self) {
        self.pointer_capture = None;
        self.hovered = None;
        self.pressed = None;
        self.pressed_button = None;
        self.press_origin = None;
        self.last_click_id = None;
        self.last_click_pos = None;
        self.last_click_time = None;
    }

    /// Clears active keyboard and semantic tag focus.
    #[inline]
    pub fn clear_focus(&mut self) {
        self.focused = None;
        self.focused_tag = None;
    }
}

/// Comprehensive outcome of dispatching a [`UiEvent`] through the hierarchical UI graph.
///
/// Returned directly by [`UiTree::dispatch_event`] to inform the caller whether the event
/// was consumed by a widget, which node was targeted, what hardware cursor should display,
/// and any attached scroll/keyboard payloads.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DispatchResult {
    /// Whether the interaction was consumed by a target widget node in the hierarchy.
    pub consumed: bool,
    /// The target widget node ID that processed or originated the event.
    pub target: Option<WidgetId>,
    /// The 64-bit persistent semantic tag of the target widget node, if assigned (`0` if none).
    pub target_tag: u64,
    /// Desired mouse cursor shape override requested by the hovered or capturing widget.
    pub cursor: Option<WidgetCursor>,
    /// The immediate-mode declarative interaction response emitted for the targeted widget.
    pub response: Option<WidgetResponse>,
    /// Scroll offset delta `(delta_x, delta_y)` delivered by mouse wheel interactions.
    pub scroll_delta: (f32, f32),
    /// Virtual keyboard key associated with this interaction, if a keyboard event was processed.
    pub key: Option<KeyCode>,
    /// Whether the keyboard key is currently pressed down.
    pub is_key_pressed: bool,
    /// Printable text or committed character string delivered by typing or IME input.
    pub text_input: Option<String>,
}

impl DispatchResult {
    /// Constructs an unhandled pass-through result indicating the event was ignored.
    #[inline]
    pub const fn ignored() -> Self {
        Self {
            consumed: false,
            target: None,
            target_tag: 0,
            cursor: None,
            response: None,
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: None,
        }
    }

    /// Constructs a consumed result targeted at a specific widget node.
    #[inline]
    pub const fn consumed_at(target: WidgetId, target_tag: u64) -> Self {
        Self {
            consumed: true,
            target: Some(target),
            target_tag,
            cursor: None,
            response: None,
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: None,
        }
    }

    /// Returns `true` if the event was absorbed by a widget node.
    #[inline]
    pub const fn is_consumed(&self) -> bool {
        self.consumed
    }

    /// Attaches a hardware cursor override to this dispatch result.
    #[inline]
    pub const fn with_cursor(mut self, cursor: Option<WidgetCursor>) -> Self {
        self.cursor = cursor;
        self
    }

    /// Attaches an immediate-mode interaction response to this dispatch result.
    #[inline]
    pub const fn with_response(mut self, response: Option<WidgetResponse>) -> Self {
        self.response = response;
        self
    }

    /// Attaches a mouse scroll delta `(delta_x, delta_y)` to this dispatch result.
    #[inline]
    pub const fn with_scroll_delta(mut self, delta_x: f32, delta_y: f32) -> Self {
        self.scroll_delta = (delta_x, delta_y);
        self
    }

    /// Attaches keyboard key data to this dispatch result.
    #[inline]
    pub const fn with_key(mut self, key: Option<KeyCode>, is_pressed: bool) -> Self {
        self.key = key;
        self.is_key_pressed = is_pressed;
        self
    }

    /// Attaches typed or committed text input to this dispatch result.
    #[inline]
    pub fn with_text(mut self, text: Option<String>) -> Self {
        self.text_input = text;
        self
    }
}

/// Dispatches an incoming [`UiEvent`] windowing input through the [`UiTree`] hierarchy.
///
/// # Execution Pipeline
/// 1. **Pointer Capture Evaluation**: If [`EventState::pointer_capture`] is set, all pointer
///    events are routed exclusively to the capturing widget, bypassing spatial hit-testing.
/// 2. **Layered Spatial Hit-Testing**: Evaluates pointer coordinates across layer priority
///    stacking contexts with automatic modal window occlusion filtering.
/// 3. **Hierarchical Event Bubbling**: If a leaf child node does not consume the interaction,
///    the event traverses upward through ancestor nodes until handled or reaching the root.
/// 4. **State Machine Transitions**: Automatically synchronizes hover, pressed, click,
///    double-click, and focus states on [`EventState`].
pub fn dispatch_ui_event(tree: &mut UiTree, event: &UiEvent) -> DispatchResult {
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
        let cursor = node.cursor.or_else(|| node.role.default_cursor());
        let tag = node.tag;
        let rect = node.computed_rect;
        let flags = ResponseFlags::HOVERED | ResponseFlags::DRAGGED;

        return DispatchResult {
            consumed: true,
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

/// Processes mouse button press inputs, initiating press states and updating focus.
fn handle_mouse_down(tree: &mut UiTree, button: MouseButton, point: Point) -> DispatchResult {
    let hit_id = tree.hit_test_layered(point);
    tree.event_state.pressed = hit_id;
    tree.event_state.pressed_button = Some(button);
    tree.event_state.press_origin = Some(point);

    if let Some(target_id) = hit_id {
        // Synchronize keyboard focus with the newly clicked interactive target
        let tag = tree.get(target_id).map(|n| n.tag).unwrap_or(0);
        tree.event_state.focused = Some(target_id);
        tree.event_state.focused_tag = if tag != 0 { Some(tag) } else { None };

        let rect = tree
            .get(target_id)
            .map(|n| n.computed_rect)
            .unwrap_or(crate::geometry::Rect::ZERO);
        let cursor = resolve_effective_cursor(tree, target_id);

        // A mouse press is strictly a down state (HOVERED | PRESSED), NEVER a click
        let flags = ResponseFlags::HOVERED | ResponseFlags::PRESSED;

        DispatchResult {
            consumed: true,
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
    let original_button = tree.event_state.pressed_button;
    tree.event_state.pressed = None;
    tree.event_state.pressed_button = None;
    tree.event_state.press_origin = None;

    if let Some(target_id) = hit_id {
        let (tag, rect) = tree
            .get(target_id)
            .map(|n| (n.tag, n.computed_rect))
            .unwrap_or((0, crate::geometry::Rect::ZERO));
        let cursor = resolve_effective_cursor(tree, target_id);

        // Click confirmed if release target and button match initial press
        if original_pressed == Some(target_id) && original_button == Some(button) {
            let mut flags = ResponseFlags::HOVERED;

            match button {
                MouseButton::Left => {
                    flags = flags.union(ResponseFlags::CLICKED);

                    let is_double_click = if let (Some(last_id), Some(last_pos), Some(last_time)) = (
                        tree.event_state.last_click_id,
                        tree.event_state.last_click_pos,
                        tree.event_state.last_click_time,
                    ) {
                        let elapsed = last_time.elapsed().as_millis();
                        let dx = point.x - last_pos.x;
                        let dy = point.y - last_pos.y;
                        let dist_sq = dx * dx + dy * dy;
                        last_id == target_id
                            && elapsed <= DOUBLE_CLICK_MAX_DELAY_MS
                            && dist_sq <= DOUBLE_CLICK_MAX_DISTANCE_SQ
                    } else {
                        false
                    };

                    if is_double_click {
                        flags = flags.union(ResponseFlags::DOUBLE_CLICKED);
                        tree.event_state.last_click_id = None;
                        tree.event_state.last_click_pos = None;
                        tree.event_state.last_click_time = None;
                    } else {
                        tree.event_state.last_click_id = Some(target_id);
                        tree.event_state.last_click_pos = Some(point);
                        tree.event_state.last_click_time = Some(Instant::now());
                    }
                }
                MouseButton::Right => {
                    flags = flags.union(ResponseFlags::SECONDARY_CLICKED);
                }
                MouseButton::Middle => {
                    flags = flags.union(ResponseFlags::MIDDLE_CLICKED);
                }
                MouseButton::Other(_) => {}
            }

            DispatchResult {
                consumed: true,
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
            DispatchResult {
                consumed: false,
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

    // Bubble upward to find the nearest scroll-handling ancestor (container with clip_children).
    // Interactive children (buttons, sliders, inputs) must NOT stop this bubbling.
    let mut current = Some(hit_id);
    while let Some(curr_id) = current {
        if let Some(node) = tree.get(curr_id) {
            if node.style.clip_children {
                let tag = node.tag;
                let rect = node.computed_rect;

                // Directly update the container's vertical scroll offset in the tree
                if let Some(mut_node) = tree.get_mut(curr_id) {
                    mut_node.style.scroll_offset_y =
                        (mut_node.style.scroll_offset_y - delta_y).max(0.0);
                }

                return DispatchResult {
                    consumed: true,
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
    tree: &UiTree,
    key: KeyCode,
    text: Option<String>,
    is_pressed: bool,
) -> DispatchResult {
    if let Some(focused_id) = tree.event_state.focused {
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
        DispatchResult::ignored()
    }
}

/// Routes IME preedit and commit events to the currently focused widget.
fn handle_ime(tree: &UiTree, ime_event: ImeEvent) -> DispatchResult {
    if let Some(focused_id) = tree.event_state.focused {
        let tag = tree.event_state.focused_tag.unwrap_or(0);
        let rect = tree
            .get(focused_id)
            .map(|n| n.computed_rect)
            .unwrap_or(crate::geometry::Rect::ZERO);

        let committed_text = match ime_event {
            ImeEvent::Commit(ref s) => Some(s.clone()),
            ImeEvent::Preedit(ref s, _) => Some(s.clone()),
            _ => None,
        };

        let mut flags = ResponseFlags::FOCUSED;
        if committed_text.is_some() {
            flags = flags.union(ResponseFlags::CHANGED);
        }

        DispatchResult {
            consumed: true,
            target: Some(focused_id),
            target_tag: tag,
            cursor: None,
            response: Some(WidgetResponse::from_flags(focused_id, rect, flags)),
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: true,
            text_input: committed_text,
        }
    } else {
        DispatchResult::ignored()
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    #[test]
    fn test_ui_tree_dispatch_event_hit_test_and_capture() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        let child = tree.create_node();
        tree.add_child(root, child).unwrap();

        // Configure child bounds
        let child_node = tree.get_mut(child).unwrap();
        child_node.visible = true;
        child_node.interactive = true;
        child_node.computed_rect = Rect::new(100.0, 100.0, 50.0, 50.0);
        child_node.tag = 42;

        // 1. MouseMove over child -> Hover detected
        let move_evt = UiEvent::MouseMove {
            point: Point::new(120.0, 120.0),
        };
        let res = tree.dispatch_event(&move_evt);
        assert!(!res.is_consumed()); // Mouse move alone is not consuming
        assert_eq!(res.target, Some(child));
        assert_eq!(res.target_tag, 42);
        assert!(res.response.map(|r| r.hovered()).unwrap_or(false));
        assert_eq!(tree.event_state.hovered, Some(child));

        // 2. MouseDown on child -> Press & Focus captured
        let down_evt = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(120.0, 120.0),
        };
        let res_down = tree.dispatch_event(&down_evt);
        assert!(res_down.is_consumed());
        assert_eq!(res_down.target, Some(child));
        assert!(res_down.response.map(|r| r.pressed()).unwrap_or(false));
        assert_eq!(tree.event_state.focused, Some(child));
        assert_eq!(tree.event_state.focused_tag, Some(42));

        // 3. MouseUp on child -> Click detected!
        let up_evt = UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(120.0, 120.0),
        };
        let res_up = tree.dispatch_event(&up_evt);
        assert!(res_up.is_consumed());
        assert_eq!(res_up.target, Some(child));
        assert!(res_up.response.map(|r| r.clicked()).unwrap_or(false));

        // 4. Pointer capture test: capture moves away from child
        tree.event_state.pointer_capture = Some(child);
        let capture_move = UiEvent::MouseMove {
            point: Point::new(500.0, 500.0), // Far away outside child bounds
        };
        let res_capture = tree.dispatch_event(&capture_move);
        assert!(res_capture.is_consumed());
        assert_eq!(res_capture.target, Some(child));
        assert!(res_capture.response.map(|r| r.dragged()).unwrap_or(false));

        // Release capture
        let capture_up = UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(500.0, 500.0),
        };
        let res_cap_up = tree.dispatch_event(&capture_up);
        assert!(res_cap_up.is_consumed());
        assert_eq!(tree.event_state.pointer_capture, None);
    }

    #[test]
    fn test_ui_tree_dispatch_event_double_click() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        let child = tree.create_node();
        tree.add_child(root, child).unwrap();

        let child_node = tree.get_mut(child).unwrap();
        child_node.visible = true;
        child_node.interactive = true;
        child_node.computed_rect = Rect::new(50.0, 50.0, 100.0, 40.0);

        let pt = Point::new(60.0, 60.0);

        // 1st click: down + up
        tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: pt,
        });
        let res1 = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: pt,
        });
        assert!(res1.response.map(|r| r.clicked()).unwrap_or(false));
        assert!(!res1.response.map(|r| r.double_clicked()).unwrap_or(false));

        // 2nd quick click: down + up
        tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: pt,
        });
        let res2 = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: pt,
        });
        assert!(res2.response.map(|r| r.clicked()).unwrap_or(false));
        assert!(
            res2.response.map(|r| r.double_clicked()).unwrap_or(false),
            "Second consecutive click must emit DOUBLE_CLICKED"
        );
    }

    #[test]
    fn test_ui_tree_dispatch_event_scroll_bubbling_past_interactive_button() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();

        // Scroll container with clip_children
        let scroll_container = tree.create_node();
        tree.add_child(root, scroll_container).unwrap();
        {
            let sc = tree.get_mut(scroll_container).unwrap();
            sc.visible = true;
            sc.style.clip_children = true;
            sc.style.scroll_offset_y = 10.0;
            sc.computed_rect = Rect::new(0.0, 0.0, 300.0, 400.0);
            sc.tag = 100;
        }

        // Interactive child button inside scroll container
        let button = tree.create_node();
        tree.add_child(scroll_container, button).unwrap();
        {
            let btn = tree.get_mut(button).unwrap();
            btn.visible = true;
            btn.interactive = true; // Interactive MUST NOT block scroll bubbling
            btn.computed_rect = Rect::new(20.0, 20.0, 100.0, 30.0);
            btn.tag = 200;
        }

        // Scroll over the interactive button: delta_y = 15.0
        let scroll_evt = UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: 15.0,
            point: Point::new(30.0, 25.0), // Inside button bounds
        };

        let res = tree.dispatch_event(&scroll_evt);
        assert!(res.is_consumed(), "Scroll event must be consumed");
        assert_eq!(
            res.target,
            Some(scroll_container),
            "Scroll event must bubble past interactive button to the scroll container"
        );
        assert_eq!(res.target_tag, 100);
        assert_eq!(res.scroll_delta, (0.0, 15.0));

        // Verify container's scroll_offset_y was updated: (10.0 - 15.0).max(0.0) = 0.0
        let sc = tree.get(scroll_container).unwrap();
        assert_eq!(sc.style.scroll_offset_y, 0.0);
    }

    #[test]
    fn test_ui_tree_dispatch_event_keyboard_and_ime() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        let input_box = tree.create_node();
        tree.add_child(root, input_box).unwrap();

        {
            let node = tree.get_mut(input_box).unwrap();
            node.visible = true;
            node.interactive = true;
            node.computed_rect = Rect::new(10.0, 10.0, 200.0, 30.0);
            node.tag = 999;
        }

        // Focus input box by clicking it
        tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(15.0, 15.0),
        });
        assert_eq!(tree.event_state.focused, Some(input_box));

        // Type a character 'A'
        let key_evt = UiEvent::KeyboardInput {
            key: KeyCode::Character('A'),
            text: Some("A".to_string()),
            is_pressed: true,
        };
        let key_res = tree.dispatch_event(&key_evt);
        assert!(key_res.is_consumed());
        assert_eq!(key_res.target, Some(input_box));
        assert_eq!(key_res.key, Some(KeyCode::Character('A')));
        assert!(key_res.is_key_pressed);
        assert_eq!(key_res.text_input.as_deref(), Some("A"));
        assert!(key_res.response.map(|r| r.changed()).unwrap_or(false));

        // Send IME commit event
        let ime_evt = UiEvent::Ime(ImeEvent::Commit("ğ".to_string()));
        let ime_res = tree.dispatch_event(&ime_evt);
        assert!(ime_res.is_consumed());
        assert_eq!(ime_res.target, Some(input_box));
        assert_eq!(ime_res.text_input.as_deref(), Some("ğ"));
        assert!(ime_res.response.map(|r| r.changed()).unwrap_or(false));
    }

    #[test]
    fn test_ui_tree_dispatch_event_right_and_middle_click_isolation() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        let button = tree.create_node();
        tree.add_child(root, button).unwrap();

        {
            let node = tree.get_mut(button).unwrap();
            node.visible = true;
            node.interactive = true;
            node.computed_rect = Rect::new(50.0, 50.0, 100.0, 40.0);
            node.tag = 777;
        }

        let pt = Point::new(60.0, 60.0);

        // 1. Right click: down + up
        let down_right = tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Right,
            point: pt,
        });
        // Down phase is only PRESSED, never SECONDARY_CLICKED
        assert!(down_right.response.map(|r| r.pressed()).unwrap_or(false));
        assert!(
            !down_right
                .response
                .map(|r| r.secondary_clicked())
                .unwrap_or(false)
        );

        let up_right = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Right,
            point: pt,
        });
        // Up phase confirms click: must emit SECONDARY_CLICKED, NEVER primary CLICKED
        assert!(
            up_right
                .response
                .map(|r| r.secondary_clicked())
                .unwrap_or(false)
        );
        assert!(
            !up_right.response.map(|r| r.clicked()).unwrap_or(false),
            "Right click MUST NOT emit primary CLICKED flag"
        );

        // 2. Middle click: down + up
        tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Middle,
            point: pt,
        });
        let up_middle = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Middle,
            point: pt,
        });
        // Up phase confirms click: must emit MIDDLE_CLICKED, NEVER primary CLICKED
        assert!(
            up_middle
                .response
                .map(|r| r.middle_clicked())
                .unwrap_or(false)
        );
        assert!(
            !up_middle.response.map(|r| r.clicked()).unwrap_or(false),
            "Middle click MUST NOT emit primary CLICKED flag"
        );
    }
}