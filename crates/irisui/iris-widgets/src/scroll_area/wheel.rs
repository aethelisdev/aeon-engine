// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Mouse Wheel Scroll Engine (`iris-widgets::scroll_area::wheel`)
//!
//! Autonomous evaluation of mouse wheel and touchpad scroll gestures against
//! hardware-clipped viewport bounds with proportional clamping and delta tracking.

use iris_core::event::UiEvent;
use iris_core::geometry::Rect;

/// Default scroll increment applied per mouse wheel notch in logical pixels.
pub const DEFAULT_SCROLL_STEP: f32 = 28.0;

/// Configuration options customizing mouse wheel scroll step physics and orientation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelScrollConfig {
    /// Logical pixel distance scrolled per standard mouse wheel discrete step.
    pub step_size: f32,
    /// Whether vertical wheel scroll direction should be inverted.
    pub invert_vertical: bool,
}

impl Default for WheelScrollConfig {
    #[inline]
    fn default() -> Self {
        Self {
            step_size: DEFAULT_SCROLL_STEP,
            invert_vertical: false,
        }
    }
}

impl WheelScrollConfig {
    /// Creates a wheel configuration with the specified per-notch step size in logical pixels.
    #[inline]
    pub const fn with_step_size(step_size: f32) -> Self {
        Self {
            step_size,
            invert_vertical: false,
        }
    }
}

/// Evaluated outcome of a mouse wheel interaction against a scrollable viewport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelScrollResponse {
    /// Whether the incoming mouse scroll event targeted this viewport and was consumed.
    pub consumed: bool,
    /// Updated vertical scroll offset clamped to `[0.0, max_scroll_y]`.
    pub new_scroll_y: f32,
    /// Effective vertical scroll distance applied after clamping.
    pub delta_y_applied: f32,
}

impl WheelScrollResponse {
    /// Returns an unconsumed response preserving the existing scroll offset.
    #[inline]
    pub const fn unconsumed(current_scroll_y: f32) -> Self {
        Self {
            consumed: false,
            new_scroll_y: current_scroll_y,
            delta_y_applied: 0.0,
        }
    }
}

/// Evaluates a raw interaction event against viewport boundaries to compute updated vertical scroll.
///
/// If the event is a [`UiEvent::MouseScroll`] and the cursor coordinates fall inside `viewport_rect`,
/// the vertical scroll offset is adjusted by the wheel delta scaled by `config.step_size` and clamped
/// strictly within `[0.0, max_scroll_y]`.
///
/// Returns a [`WheelScrollResponse`] indicating consumption and the new scroll position.
#[inline]
pub fn evaluate_wheel_scroll(
    event: &UiEvent,
    viewport_rect: Rect,
    current_scroll_y: f32,
    max_scroll_y: f32,
    config: &WheelScrollConfig,
) -> WheelScrollResponse {
    let UiEvent::MouseScroll { delta_y, point, .. } = event else {
        return WheelScrollResponse::unconsumed(current_scroll_y);
    };

    // If cursor is outside the target viewport boundaries, ignore the scroll gesture
    if !viewport_rect.contains_point(*point) {
        return WheelScrollResponse::unconsumed(current_scroll_y);
    }

    let max_valid_scroll = max_scroll_y.max(0.0);
    if max_valid_scroll <= 0.0 {
        // Content completely fits inside viewport; consume wheel to prevent fall-through
        return WheelScrollResponse {
            consumed: true,
            new_scroll_y: 0.0,
            delta_y_applied: 0.0,
        };
    }

    let multiplier = if config.invert_vertical { -1.0 } else { 1.0 };
    // Downwards wheel rotation produces negative delta_y in windowing systems,
    // which corresponds to advancing the scroll offset downwards (increasing scroll_y).
    let raw_delta = -delta_y * config.step_size * multiplier;
    let clamped_scroll = (current_scroll_y + raw_delta).clamp(0.0, max_valid_scroll);
    let delta_applied = clamped_scroll - current_scroll_y;

    WheelScrollResponse {
        consumed: true,
        new_scroll_y: clamped_scroll,
        delta_y_applied: delta_applied,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::geometry::Point;

    #[test]
    fn test_wheel_scroll_down_and_up_clamping() {
        let vp = Rect::new(50.0, 50.0, 400.0, 300.0);
        let cursor = Point::new(100.0, 100.0);
        let config = WheelScrollConfig::default();
        let max_scroll = 500.0;

        // Downwards scroll: delta_y = -2.0 -> advance downwards by 2 * 28.0 = +56.0
        let event_down = UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: -2.0,
            point: cursor,
        };
        let res1 = evaluate_wheel_scroll(&event_down, vp, 0.0, max_scroll, &config);
        assert!(res1.consumed);
        assert_eq!(res1.new_scroll_y, 56.0);
        assert_eq!(res1.delta_y_applied, 56.0);

        // Clamping at bottom limit
        let event_huge_down = UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: -30.0,
            point: cursor,
        };
        let res2 = evaluate_wheel_scroll(&event_huge_down, vp, 480.0, max_scroll, &config);
        assert!(res2.consumed);
        assert_eq!(res2.new_scroll_y, 500.0);
        assert_eq!(res2.delta_y_applied, 20.0);

        // Upwards scroll: delta_y = +1.0 -> decrease scroll by 28.0
        let event_up = UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: 1.0,
            point: cursor,
        };
        let res3 = evaluate_wheel_scroll(&event_up, vp, 50.0, max_scroll, &config);
        assert!(res3.consumed);
        assert_eq!(res3.new_scroll_y, 22.0);
        assert_eq!(res3.delta_y_applied, -28.0);

        // Clamping at top limit (0.0)
        let res4 = evaluate_wheel_scroll(&event_up, vp, 10.0, max_scroll, &config);
        assert!(res4.consumed);
        assert_eq!(res4.new_scroll_y, 0.0);
        assert_eq!(res4.delta_y_applied, -10.0);
    }

    #[test]
    fn test_wheel_scroll_outside_viewport_ignored() {
        let vp = Rect::new(100.0, 100.0, 200.0, 200.0);
        let outside_cursor = Point::new(50.0, 50.0);
        let event = UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: -2.0,
            point: outside_cursor,
        };

        let res = evaluate_wheel_scroll(&event, vp, 120.0, 300.0, &WheelScrollConfig::default());
        assert!(!res.consumed);
        assert_eq!(res.new_scroll_y, 120.0);
        assert_eq!(res.delta_y_applied, 0.0);
    }

    #[test]
    fn test_wheel_scroll_content_fits_viewport() {
        let vp = Rect::new(0.0, 0.0, 300.0, 400.0);
        let cursor = Point::new(100.0, 100.0);
        let event = UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: -2.0,
            point: cursor,
        };

        // max_scroll = 0.0 means content fits entirely inside viewport
        let res = evaluate_wheel_scroll(&event, vp, 0.0, 0.0, &WheelScrollConfig::default());
        assert!(res.consumed);
        assert_eq!(res.new_scroll_y, 0.0);
        assert_eq!(res.delta_y_applied, 0.0);
    }
}