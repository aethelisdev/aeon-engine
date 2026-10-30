// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Numeric Scrubber Dragging & Pointer Capture
//!
//! Provides autonomous horizontal scrubbing state, speed multipliers, modifier sensitivity
//! adjustments (Shift for fine-tuning, Ctrl for coarse increments), and pointer capture
//! management for numeric input fields.

use iris_core::event::{KeyCode, UiEvent};
use iris_core::node::WidgetCursor;

/// Motion displacement threshold in logical pixels required to enter active dragging mode.
///
/// Displacements below this threshold are treated as stationary clicks, allowing controls
/// to switch into direct keyboard text entry without unintentionally modifying values.
pub const NUMERIC_DRAG_THRESHOLD: f32 = 2.0;

/// Default fine-tuning sensitivity multiplier applied when Shift is held.
pub const NUMERIC_DRAG_FINE_MULTIPLIER: f64 = 0.1;

/// Default coarse-tuning sensitivity multiplier applied when Control / Command is held.
pub const NUMERIC_DRAG_COARSE_MULTIPLIER: f64 = 10.0;

/// Active horizontal mouse drag state for interactive numeric scrubbers and property sliders.
///
/// Encapsulates the starting mouse position, initial field value, sensitivity step,
/// boundary clamping, and active dragging flags.
///
/// # Examples
///
/// ```
/// use iris_widgets::controls::numeric_drag::NumericDragState;
///
/// let mut drag = NumericDragState::new(100.0, 50.0, 1.0, 0.0, 100.0);
/// let res = drag.update(120.0, false, false);
/// assert!(res.has_dragged);
/// assert_eq!(res.value, Some(70.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericDragState {
    /// Starting X coordinate of the cursor when mouse was pressed down.
    pub start_x: f32,
    /// Starting scalar value of the numeric field before dragging began.
    pub start_val: f64,
    /// Value change delta per dragged logical pixel.
    pub speed: f64,
    /// Lower clamp boundary for the numeric value.
    pub min_val: f64,
    /// Upper clamp boundary for the numeric value.
    pub max_val: f64,
    /// Current accumulated value during the active drag session.
    pub current_val: f64,
    /// Optional semantic widget tag or numeric field identifier.
    pub tag: u64,
    /// Whether the mouse has moved beyond [`NUMERIC_DRAG_THRESHOLD`].
    pub has_dragged: bool,
    /// Whether this drag session is currently actively tracking pointer movement.
    pub is_active: bool,
}

impl Default for NumericDragState {
    fn default() -> Self {
        Self {
            start_x: 0.0,
            start_val: 0.0,
            speed: 1.0,
            min_val: f64::NEG_INFINITY,
            max_val: f64::INFINITY,
            current_val: 0.0,
            tag: 0,
            has_dragged: false,
            is_active: false,
        }
    }
}

/// Interaction response returned after evaluating pointer movement or release for numeric drag.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NumericDragResponse {
    /// The updated clamped numeric value, or `None` if movement was below threshold or inactive.
    pub value: Option<f64>,
    /// Whether the pointer displacement has exceeded [`NUMERIC_DRAG_THRESHOLD`].
    pub has_dragged: bool,
    /// Whether this event was consumed and should prevent click-through.
    pub consumed: bool,
    /// Desired mouse cursor icon override (`EwResize` while dragging).
    pub cursor: Option<WidgetCursor>,
    /// Whether the host application should capture pointer events.
    pub capture_pointer: bool,
    /// Whether active pointer capture should be released.
    pub release_pointer: bool,
    /// Whether the user released the mouse without dragging (in-place stationary click).
    pub is_click_in_place: bool,
    /// Whether the drag operation was explicitly cancelled (e.g., via Escape key).
    pub is_cancelled: bool,
}

impl NumericDragState {
    /// Motion displacement threshold in logical pixels required to enter active dragging mode (`2.0 px`).
    pub const DEFAULT_THRESHOLD: f32 = 2.0;

    /// Default fine-tuning sensitivity multiplier applied when Shift is held (`0.1x`).
    pub const DEFAULT_FINE_MULTIPLIER: f64 = 0.1;

    /// Default coarse-tuning sensitivity multiplier applied when Control / Command is held (`10.0x`).
    pub const DEFAULT_COARSE_MULTIPLIER: f64 = 10.0;

    /// Creates a new active numeric drag session.
    ///
    /// # Arguments
    ///
    /// * `start_x` - Screen X coordinate where the pointer button was pressed.
    /// * `start_val` - Original numeric value before scrubbing begins.
    /// * `speed` - Value delta per dragged screen pixel.
    /// * `min_val` - Minimum allowed clamped value.
    /// * `max_val` - Maximum allowed clamped value.
    #[must_use]
    pub fn new(start_x: f32, start_val: f64, speed: f64, min_val: f64, max_val: f64) -> Self {
        Self {
            start_x,
            start_val,
            speed: if speed.abs() > f64::EPSILON {
                speed
            } else {
                1.0
            },
            min_val,
            max_val,
            current_val: start_val.clamp(min_val, max_val),
            tag: 0,
            has_dragged: false,
            is_active: true,
        }
    }

    /// Attaches an optional 64-bit semantic widget identifier to this dragging session.
    #[must_use]
    #[inline]
    pub const fn with_tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Evaluates cursor movement during an active scrubbing session.
    ///
    /// Applies fine-tuning multiplier ([`NUMERIC_DRAG_FINE_MULTIPLIER`]) when `shift` is held,
    /// or coarse multiplier ([`NUMERIC_DRAG_COARSE_MULTIPLIER`]) when `ctrl` is held.
    pub fn update(&mut self, current_x: f32, shift: bool, ctrl: bool) -> NumericDragResponse {
        if !self.is_active {
            return NumericDragResponse::default();
        }

        let delta_x = current_x - self.start_x;
        if delta_x.abs() > NUMERIC_DRAG_THRESHOLD {
            self.has_dragged = true;

            let multiplier = if shift {
                NUMERIC_DRAG_FINE_MULTIPLIER
            } else if ctrl {
                NUMERIC_DRAG_COARSE_MULTIPLIER
            } else {
                1.0
            };

            let delta_val = (delta_x as f64) * self.speed * multiplier;
            let new_val = (self.start_val + delta_val).clamp(self.min_val, self.max_val);
            self.current_val = new_val;

            NumericDragResponse {
                value: Some(new_val),
                has_dragged: true,
                consumed: true,
                cursor: Some(WidgetCursor::EwResize),
                capture_pointer: true,
                release_pointer: false,
                is_click_in_place: false,
                is_cancelled: false,
            }
        } else {
            NumericDragResponse {
                value: None,
                has_dragged: false,
                consumed: true,
                cursor: Some(WidgetCursor::EwResize),
                capture_pointer: true,
                release_pointer: false,
                is_click_in_place: false,
                is_cancelled: false,
            }
        }
    }

    /// Concludes the scrubbing session when the pointer button is released.
    ///
    /// Returns `is_click_in_place: true` if the cursor did not move beyond the movement threshold,
    /// allowing the caller to transition into direct text editing.
    pub fn finish(&mut self) -> NumericDragResponse {
        if !self.is_active {
            return NumericDragResponse::default();
        }

        self.is_active = false;
        let is_click = !self.has_dragged;

        NumericDragResponse {
            value: if self.has_dragged {
                Some(self.current_val)
            } else {
                None
            },
            has_dragged: self.has_dragged,
            consumed: true,
            cursor: None,
            capture_pointer: false,
            release_pointer: true,
            is_click_in_place: is_click,
            is_cancelled: false,
        }
    }

    /// Aborts the scrubbing session and restores the initial value prior to dragging.
    pub fn cancel(&mut self) -> NumericDragResponse {
        if !self.is_active {
            return NumericDragResponse::default();
        }

        self.is_active = false;
        let restored_val = self.start_val;
        self.current_val = restored_val;

        NumericDragResponse {
            value: Some(restored_val),
            has_dragged: self.has_dragged,
            consumed: true,
            cursor: None,
            capture_pointer: false,
            release_pointer: true,
            is_click_in_place: false,
            is_cancelled: true,
        }
    }
}

/// Evaluates a high-level [`UiEvent`] against an active or prospective numeric drag session.
///
/// Automatically routes mouse movement, mouse button release, and Escape key cancellations.
pub fn handle_numeric_drag_event(
    state: &mut Option<NumericDragState>,
    event: &UiEvent,
    shift: bool,
    ctrl: bool,
) -> Option<NumericDragResponse> {
    let drag = state.as_mut()?;
    match event {
        UiEvent::MouseMove { point } => Some(drag.update(point.x, shift, ctrl)),
        UiEvent::MouseUp { .. } => {
            let res = drag.finish();
            *state = None;
            Some(res)
        }
        UiEvent::KeyboardInput {
            key: KeyCode::Escape,
            is_pressed: true,
            ..
        } => {
            let res = drag.cancel();
            *state = None;
            Some(res)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::geometry::Point;

    #[test]
    fn test_numeric_drag_scrubber_delta_and_modifiers() {
        let mut drag = NumericDragState::new(100.0, 10.0, 0.5, 0.0, 100.0);
        assert!(drag.is_active);
        assert!(!drag.has_dragged);

        // 1. Move below threshold (1.5px) -> no value change, stationary
        let res_sub = drag.update(101.5, false, false);
        assert!(!res_sub.has_dragged);
        assert_eq!(res_sub.value, None);
        assert!(res_sub.consumed);
        assert_eq!(res_sub.cursor, Some(WidgetCursor::EwResize));

        // 2. Normal drag (+10px) -> delta_val = 10 * 0.5 = 5.0 -> new_val = 15.0
        let res_norm = drag.update(110.0, false, false);
        assert!(res_norm.has_dragged);
        assert_eq!(res_norm.value, Some(15.0));
        assert_eq!(res_norm.cursor, Some(WidgetCursor::EwResize));

        // 3. Shift held (fine multiplier 0.1x) -> delta = +20px -> delta_val = 20 * 0.5 * 0.1 = 1.0 -> new_val = 11.0
        let res_fine = drag.update(120.0, true, false);
        assert_eq!(res_fine.value, Some(11.0));

        // 4. Ctrl held (coarse multiplier 10.0x) -> delta = +10px -> delta_val = 10 * 0.5 * 10 = 50.0 -> new_val = 60.0
        let res_coarse = drag.update(110.0, false, true);
        assert_eq!(res_coarse.value, Some(60.0));

        // 5. Clamping bounds check (drag to right infinity)
        let res_clamped = drag.update(1000.0, false, true);
        assert_eq!(res_clamped.value, Some(100.0));

        // 6. Finish after dragging -> release pointer, not a click in place
        let res_finish = drag.finish();
        assert!(res_finish.release_pointer);
        assert!(!res_finish.is_click_in_place);
        assert_eq!(res_finish.value, Some(100.0));
    }

    #[test]
    fn test_numeric_drag_click_in_place_without_drag() {
        let mut drag = NumericDragState::new(50.0, 42.0, 1.0, 0.0, 100.0);
        // Small jitter below threshold
        drag.update(51.0, false, false);

        // Released mouse -> detected as stationary click
        let res = drag.finish();
        assert!(res.is_click_in_place);
        assert!(!res.has_dragged);
        assert_eq!(res.value, None);
        assert!(res.release_pointer);
    }

    #[test]
    fn test_numeric_drag_cancel_restores_initial_value() {
        let mut drag = NumericDragState::new(100.0, 25.0, 1.0, 0.0, 100.0);
        drag.update(150.0, false, false);
        assert_eq!(drag.current_val, 75.0);

        let res = drag.cancel();
        assert!(res.is_cancelled);
        assert_eq!(res.value, Some(25.0));
        assert_eq!(drag.current_val, 25.0);
        assert!(res.release_pointer);
    }

    #[test]
    fn test_handle_numeric_drag_event_integration() {
        let mut state = Some(NumericDragState::new(20.0, 5.0, 2.0, 0.0, 50.0));

        // MouseMove
        let move_ev = UiEvent::MouseMove {
            point: Point::new(30.0, 10.0),
        };
        let res_move = handle_numeric_drag_event(&mut state, &move_ev, false, false);
        assert!(res_move.is_some());
        let res = res_move.unwrap();
        assert!(res.has_dragged);
        // delta = 10px * 2.0 = 20 -> 5 + 20 = 25
        assert_eq!(res.value, Some(25.0));

        // Escape cancels and clears state
        let esc_ev = UiEvent::KeyboardInput {
            key: KeyCode::Escape,
            text: None,
            is_pressed: true,
            ctrl_held: false,
        };
        let res_esc = handle_numeric_drag_event(&mut state, &esc_ev, false, false);
        assert!(res_esc.is_some());
        let cancel_res = res_esc.unwrap();
        assert!(cancel_res.is_cancelled);
        assert_eq!(cancel_res.value, Some(5.0));
        assert!(state.is_none());
    }
}