// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # ScrollArea Interaction and Event Handling (`iris-widgets::scroll_area::events`)
//!
//! Autonomous event handling for scrollbar thumb dragging, track click-to-jump,
//! pointer capture lifecycles, and mouse cursor feedback.
//!

use super::area::ScrollAreaFrame;
use super::scroll_bar::ScrollBarGeometry;
use super::wheel::{WheelScrollConfig, evaluate_wheel_scroll};
use iris_core::event::{MouseButton, UiEvent};
use iris_core::node::WidgetCursor;

/// Active drag interaction state for a scrollbar thumb.
///
/// Tracks the origin vertical coordinate where the pointer was pressed and the
/// initial scroll offset at drag inception to calculate linear projections without drift.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollbarDragState {
    /// Vertical cursor coordinate (Y) in logical pixels at the start of the drag.
    pub start_cursor_y: f32,
    /// Vertical scroll offset in logical pixels at the start of the drag.
    pub initial_scroll_y: f32,
}

impl ScrollbarDragState {
    /// Creates a new scrollbar drag state with the specified origin cursor coordinate and initial scroll.
    #[inline]
    pub const fn new(start_cursor_y: f32, initial_scroll_y: f32) -> Self {
        Self {
            start_cursor_y,
            initial_scroll_y,
        }
    }
}

/// Persistent interaction state for a scrollable container.
///
/// Retains the current vertical scroll position and any ongoing scrollbar thumb
/// drag gesture across frames.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScrollAreaState {
    /// Current vertical scroll position in logical pixels.
    pub scroll_y: f32,
    /// Active thumb drag state if the user is currently holding the scrollbar.
    pub drag_state: Option<ScrollbarDragState>,
}

impl ScrollAreaState {
    /// Creates a new state with the specified initial vertical scroll offset.
    #[inline]
    pub const fn new(scroll_y: f32) -> Self {
        Self {
            scroll_y,
            drag_state: None,
        }
    }

    /// Returns whether a scrollbar thumb drag gesture is currently in progress.
    #[inline]
    pub const fn is_dragging(&self) -> bool {
        self.drag_state.is_some()
    }

    /// Updates the current scroll offset clamped to `[0.0, max_scroll_y]`.
    #[inline]
    pub fn set_scroll_y(&mut self, scroll_y: f32, max_scroll_y: f32) {
        self.scroll_y = scroll_y.clamp(0.0, max_scroll_y.max(0.0));
    }

    /// Resets any ongoing drag operation without altering the current scroll offset.
    #[inline]
    pub fn reset_drag(&mut self) {
        self.drag_state = None;
    }

    /// Resets both the scroll position to zero and clears any active drag interaction.
    #[inline]
    pub fn reset(&mut self) {
        self.scroll_y = 0.0;
        self.drag_state = None;
    }

    /// Synchronizes this local state with the current scroll offset stored in the [`iris_core::tree::UiTree`].
    #[inline]
    pub fn sync_from_tree(
        &mut self,
        tree: &iris_core::tree::UiTree,
        container_id: iris_core::id::WidgetId,
    ) {
        if let Some(node) = tree.get(container_id) {
            self.scroll_y = node.style.scroll_offset_y;
        }
    }
}

/// Evaluated outcome of a scroll interaction (drag, track click, or wheel gesture).
///
/// Signals event consumption, updated scroll offset, layout invalidation flags,
/// pointer capture management instructions, and hardware cursor overrides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollResponse {
    /// Whether the event targeted the scrollbar/viewport and was consumed.
    pub consumed: bool,
    /// Current vertical scroll offset in logical pixels clamped to valid limits.
    pub scroll_y: f32,
    /// Whether the scroll position changed as a result of processing this event.
    pub layout_changed: bool,
    /// Whether the caller should acquire pointer capture to receive out-of-bounds pointer moves.
    pub capture_pointer: bool,
    /// Whether the caller should release any previously held pointer capture.
    pub release_pointer: bool,
    /// Hardware cursor override requested for scrollbar hover or active dragging.
    pub cursor: Option<WidgetCursor>,
}

impl ScrollResponse {
    /// Constructs an unconsumed response indicating no scroll interaction occurred.
    #[inline]
    pub const fn unconsumed(scroll_y: f32) -> Self {
        Self {
            consumed: false,
            scroll_y,
            layout_changed: false,
            capture_pointer: false,
            release_pointer: false,
            cursor: None,
        }
    }

    /// Constructs a consumed response reflecting an updated scroll position.
    #[inline]
    pub const fn consumed(scroll_y: f32, layout_changed: bool) -> Self {
        Self {
            consumed: true,
            scroll_y,
            layout_changed,
            capture_pointer: false,
            release_pointer: false,
            cursor: Some(WidgetCursor::Default),
        }
    }

    /// Returns a new response with the requested hardware cursor override set.
    #[inline]
    pub const fn with_cursor(mut self, cursor: WidgetCursor) -> Self {
        self.cursor = Some(cursor);
        self
    }
}

/// Evaluates pointer events against scrollbar track and thumb geometry.
///
/// Handles:
/// 1. Mouse down on thumb -> begins continuous drag gesture with pointer capture request.
/// 2. Mouse down on track outside thumb -> jumps directly to clicked offset and begins continuous drag.
/// 3. Mouse move during active drag -> translates vertical delta into proportional scroll position.
/// 4. Mouse up / release -> ends active drag and requests pointer capture release.
/// 5. Mouse move over track/thumb when idle -> reports [`WidgetCursor::Default`] hover cursor.
/// 6. Mouse scroll during active drag -> swallowed to prevent scroll jitter while dragging.
#[must_use]
pub fn handle_scrollbar_event(
    state: &mut ScrollAreaState,
    event: &UiEvent,
    frame: &ScrollAreaFrame,
) -> ScrollResponse {
    let Some(ref geom) = frame.scrollbar else {
        // If there is no active scrollbar (content fits in viewport)
        if state.is_dragging() {
            if matches!(
                event,
                UiEvent::MouseUp {
                    button: MouseButton::Left,
                    ..
                }
            ) {
                state.drag_state = None;
                return ScrollResponse {
                    consumed: true,
                    scroll_y: state.scroll_y,
                    layout_changed: false,
                    capture_pointer: false,
                    release_pointer: true,
                    cursor: Some(WidgetCursor::Default),
                };
            } else if matches!(event, UiEvent::MouseScroll { .. }) {
                // Swallowing wheel scroll events during active drag to eliminate jitter
                return ScrollResponse {
                    consumed: true,
                    scroll_y: state.scroll_y,
                    layout_changed: false,
                    capture_pointer: false,
                    release_pointer: false,
                    cursor: Some(WidgetCursor::Default),
                };
            }
        }
        return ScrollResponse::unconsumed(state.scroll_y);
    };

    // Case 1: Active thumb dragging in progress (Pointer Capture active)
    if let Some(drag) = state.drag_state {
        match event {
            UiEvent::MouseUp {
                button: MouseButton::Left,
                ..
            } => {
                state.drag_state = None;
                return ScrollResponse {
                    consumed: true,
                    scroll_y: state.scroll_y,
                    layout_changed: false,
                    capture_pointer: false,
                    release_pointer: true,
                    cursor: Some(WidgetCursor::Default),
                };
            }
            UiEvent::MouseMove { point } => {
                let delta_y = point.y - drag.start_cursor_y;
                let delta_scroll = ScrollBarGeometry::scroll_from_thumb_drag(
                    delta_y,
                    geom.track_rect.height,
                    geom.thumb_rect.height,
                    frame.max_scroll_y,
                );
                let new_scroll =
                    (drag.initial_scroll_y + delta_scroll).clamp(0.0, frame.max_scroll_y);
                let layout_changed = (new_scroll - state.scroll_y).abs() > 0.001;
                state.scroll_y = new_scroll;

                return ScrollResponse {
                    consumed: true,
                    scroll_y: new_scroll,
                    layout_changed,
                    capture_pointer: false,
                    release_pointer: false,
                    cursor: Some(WidgetCursor::Default),
                };
            }
            UiEvent::MouseScroll { .. } => {
                // Swallowing wheel scroll events during active drag to eliminate jitter
                return ScrollResponse {
                    consumed: true,
                    scroll_y: state.scroll_y,
                    layout_changed: false,
                    capture_pointer: false,
                    release_pointer: false,
                    cursor: Some(WidgetCursor::Default),
                };
            }
            _ => return ScrollResponse::unconsumed(state.scroll_y),
        }
    }

    // Case 2: Idle state - detect mouse down on thumb or track
    if let UiEvent::MouseDown {
        button: MouseButton::Left,
        point,
    } = event
    {
        if geom.thumb_rect.contains_point(*point) {
            // Drag started directly on thumb
            state.drag_state = Some(ScrollbarDragState::new(point.y, state.scroll_y));
            return ScrollResponse {
                consumed: true,
                scroll_y: state.scroll_y,
                layout_changed: false,
                capture_pointer: true,
                release_pointer: false,
                cursor: Some(WidgetCursor::Default),
            };
        } else if geom.track_rect.contains_point(*point) {
            // Click on track outside thumb -> jump to position, then engage continuous drag
            let new_scroll = ScrollBarGeometry::scroll_from_track_click(
                point.y,
                geom.track_rect.y,
                geom.track_rect.height,
                geom.thumb_rect.height,
                frame.max_scroll_y,
            )
            .clamp(0.0, frame.max_scroll_y);

            let layout_changed = (new_scroll - state.scroll_y).abs() > 0.001;
            state.scroll_y = new_scroll;
            state.drag_state = Some(ScrollbarDragState::new(point.y, new_scroll));

            return ScrollResponse {
                consumed: true,
                scroll_y: new_scroll,
                layout_changed,
                capture_pointer: true,
                release_pointer: false,
                cursor: Some(WidgetCursor::Default),
            };
        }
    }

    // Case 3: Idle state hover over scrollbar track or thumb
    if let UiEvent::MouseMove { point } = event
        && geom.track_rect.contains_point(*point)
    {
        return ScrollResponse {
            consumed: false,
            scroll_y: state.scroll_y,
            layout_changed: false,
            capture_pointer: false,
            release_pointer: false,
            cursor: Some(WidgetCursor::Default),
        };
    }

    ScrollResponse::unconsumed(state.scroll_y)
}

/// Unified handler evaluating both mouse wheel gestures and scrollbar thumb/track dragging.
///
/// Dispatches incoming events through [`evaluate_wheel_scroll`] when wheel events target the
/// viewport, and through [`handle_scrollbar_event`] when pointer events target or continue dragging
/// the scrollbar geometry.
#[must_use]
pub fn handle_scroll_area_event(
    state: &mut ScrollAreaState,
    event: &UiEvent,
    frame: &ScrollAreaFrame,
    wheel_config: Option<&WheelScrollConfig>,
) -> ScrollResponse {
    // 1. Evaluate scrollbar interactions first (thumb drag has priority over viewport wheel)
    let bar_resp = handle_scrollbar_event(state, event, frame);
    if bar_resp.consumed || bar_resp.cursor.is_some() {
        return bar_resp;
    }

    // 2. Evaluate mouse wheel gestures if not dragging scrollbar
    if let UiEvent::MouseScroll { .. } = event {
        let default_config = WheelScrollConfig::default();
        let config = wheel_config.unwrap_or(&default_config);
        let wheel_resp = evaluate_wheel_scroll(
            event,
            frame.viewport_rect,
            state.scroll_y,
            frame.max_scroll_y,
            config,
        );

        if wheel_resp.consumed {
            let layout_changed = (wheel_resp.new_scroll_y - state.scroll_y).abs() > 0.001;
            state.scroll_y = wheel_resp.new_scroll_y;
            return ScrollResponse {
                consumed: true,
                scroll_y: wheel_resp.new_scroll_y,
                layout_changed,
                capture_pointer: false,
                release_pointer: false,
                cursor: None,
            };
        }
    }

    ScrollResponse::unconsumed(state.scroll_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scroll_area::style::ScrollAreaStyle;
    use iris_core::geometry::{Point, Rect};
    use iris_core::id::WidgetId;

    fn create_test_frame(scroll_y: f32) -> ScrollAreaFrame {
        let vp = Rect::new(0.0, 0.0, 200.0, 400.0);
        let content_height = 800.0;
        let max_scroll_y = 400.0;
        let style = ScrollAreaStyle::default();
        let scrollbar = ScrollBarGeometry::compute_vertical(vp, content_height, scroll_y, &style);

        ScrollAreaFrame {
            container_id: WidgetId::default(),
            viewport_rect: vp,
            content_height,
            max_scroll_y,
            scrollbar,
        }
    }

    #[test]
    fn test_thumb_drag_capture_and_release() {
        let mut state = ScrollAreaState::new(0.0);
        let frame = create_test_frame(state.scroll_y);
        let geom = frame.scrollbar.as_ref().unwrap();

        // 1. Click on thumb -> begins drag, requests pointer capture and default cursor
        let thumb_center = Point::new(
            geom.thumb_rect.x + geom.thumb_rect.width * 0.5,
            geom.thumb_rect.y + geom.thumb_rect.height * 0.5,
        );
        let down_event = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: thumb_center,
        };

        let resp = handle_scrollbar_event(&mut state, &down_event, &frame);
        assert!(resp.consumed);
        assert!(resp.capture_pointer);
        assert!(!resp.release_pointer);
        assert_eq!(resp.cursor, Some(WidgetCursor::Default));
        assert!(state.is_dragging());

        // 2. Drag thumb downwards by 50px
        let move_point = Point::new(thumb_center.x + 100.0, thumb_center.y + 50.0); // X is outside viewport to verify pointer capture
        let move_event = UiEvent::MouseMove { point: move_point };

        let move_resp = handle_scrollbar_event(&mut state, &move_event, &frame);
        assert!(move_resp.consumed);
        assert!(move_resp.layout_changed);
        assert!(move_resp.scroll_y > 0.0);
        assert_eq!(move_resp.cursor, Some(WidgetCursor::Default));
        assert_eq!(state.scroll_y, move_resp.scroll_y);

        // 3. Release left mouse button -> ends drag, requests pointer release
        let up_event = UiEvent::MouseUp {
            button: MouseButton::Left,
            point: move_point,
        };

        let up_resp = handle_scrollbar_event(&mut state, &up_event, &frame);
        assert!(up_resp.consumed);
        assert!(!up_resp.capture_pointer);
        assert!(up_resp.release_pointer);
        assert_eq!(up_resp.cursor, Some(WidgetCursor::Default));
        assert!(!state.is_dragging());
    }

    #[test]
    fn test_track_jump_click() {
        let mut state = ScrollAreaState::new(0.0);
        let frame = create_test_frame(state.scroll_y);
        let geom = frame.scrollbar.as_ref().unwrap();

        // Click near the bottom of the track (below thumb)
        let track_click = Point::new(
            geom.track_rect.x + geom.track_rect.width * 0.5,
            geom.track_rect.y + geom.track_rect.height - 10.0,
        );
        let down_event = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: track_click,
        };

        let resp = handle_scrollbar_event(&mut state, &down_event, &frame);
        assert!(resp.consumed);
        assert!(resp.layout_changed);
        assert!(resp.capture_pointer);
        assert_eq!(resp.cursor, Some(WidgetCursor::Default));
        assert!(resp.scroll_y > 300.0); // Should jump near max scroll
        assert!(state.is_dragging());
    }

    #[test]
    fn test_active_drag_swallows_mouse_scroll_preventing_jitter() {
        let mut state = ScrollAreaState::new(50.0);
        let frame = create_test_frame(state.scroll_y);
        let geom = frame.scrollbar.as_ref().unwrap();

        // Begin drag on thumb
        let thumb_center = Point::new(
            geom.thumb_rect.x + geom.thumb_rect.width * 0.5,
            geom.thumb_rect.y + geom.thumb_rect.height * 0.5,
        );
        let down_event = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: thumb_center,
        };
        let _ = handle_scrollbar_event(&mut state, &down_event, &frame);
        assert!(state.is_dragging());

        // MouseScroll event occurs during active thumb dragging
        let wheel_event = UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: -5.0, // Wheel spun downwards
            point: thumb_center,
        };

        let resp = handle_scrollbar_event(&mut state, &wheel_event, &frame);
        assert!(resp.consumed);
        assert!(!resp.layout_changed);
        assert_eq!(resp.scroll_y, 50.0); // Preserved without jitter jump
        assert_eq!(resp.cursor, Some(WidgetCursor::Default));
        assert!(state.is_dragging());

        // Also test through unified handle_scroll_area_event
        let unified_resp = handle_scroll_area_event(&mut state, &wheel_event, &frame, None);
        assert!(unified_resp.consumed);
        assert!(!unified_resp.layout_changed);
        assert_eq!(unified_resp.scroll_y, 50.0);
    }

    #[test]
    fn test_scrollbar_hover_cursor() {
        let mut state = ScrollAreaState::new(0.0);
        let frame = create_test_frame(state.scroll_y);
        let geom = frame.scrollbar.as_ref().unwrap();

        // Hover over scrollbar track
        let track_hover = Point::new(
            geom.track_rect.x + geom.track_rect.width * 0.5,
            geom.track_rect.y + geom.track_rect.height * 0.5,
        );
        let move_event = UiEvent::MouseMove { point: track_hover };

        let resp = handle_scrollbar_event(&mut state, &move_event, &frame);
        assert!(!resp.consumed); // Does not block clicks
        assert_eq!(resp.cursor, Some(WidgetCursor::Default));

        let unified_resp = handle_scroll_area_event(&mut state, &move_event, &frame, None);
        assert_eq!(unified_resp.cursor, Some(WidgetCursor::Default));
    }

    #[test]
    fn test_unhandled_event_passes_through() {
        let mut state = ScrollAreaState::new(10.0);
        let frame = create_test_frame(state.scroll_y);

        // Click outside scrollbar
        let outside_point = Point::new(50.0, 50.0);
        let down_event = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: outside_point,
        };

        let resp = handle_scrollbar_event(&mut state, &down_event, &frame);
        assert!(!resp.consumed);
        assert_eq!(resp.scroll_y, 10.0);
        assert_eq!(resp.cursor, None);
        assert!(!state.is_dragging());
    }

    #[test]
    fn test_unified_scroll_area_event_with_wheel() {
        let mut state = ScrollAreaState::new(10.0);
        let frame = create_test_frame(state.scroll_y);

        // Mouse wheel scroll inside viewport
        let wheel_event = UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: -2.0, // Scroll down
            point: Point::new(50.0, 50.0),
        };

        let resp = handle_scroll_area_event(&mut state, &wheel_event, &frame, None);
        assert!(resp.consumed);
        assert!(resp.layout_changed);
        assert!(resp.scroll_y > 10.0);
        assert_eq!(state.scroll_y, resp.scroll_y);
    }
}