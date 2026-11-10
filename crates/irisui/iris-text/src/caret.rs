// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Text Editing Caret and Blink Animation Subsystem (`iris-text::caret`)
//!
//! Provides millisecond-accurate cursor blink timing, visibility state machines, and
//! subpixel caret bounding box calculation for interactive text input controls.
//!
//! Eliminates the need for client applications to allocate intermediate formatted strings
//! (such as appending temporary pipe characters `"|"` to active input buffers) by decoupling
//! the physical caret cursor geometry from the underlying glyph stream.

use iris_core::{Color, Point, Rect};

/// Default half-cycle blink duration in milliseconds (530ms visible, 530ms hidden, 1060ms period).
pub const DEFAULT_CARET_BLINK_INTERVAL_MS: u64 = 530;

/// Default caret cursor line width in physical screen pixels.
pub const DEFAULT_CARET_WIDTH: f32 = 1.5;

/// Millisecond-accurate cursor blink animation timer and visibility state coordinator.
///
/// Ensures deterministic on/off square wave toggling for text cursor rendering. Resetting the timer
/// immediately restores cursor visibility, ensuring responsive visual feedback during rapid typing
/// and navigation gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaretBlinkTimer {
    /// Half-period duration in milliseconds during which the caret maintains a single visibility state.
    interval_ms: u64,
    /// Accumulated time in milliseconds within the current full cycle.
    elapsed_ms: u64,
    /// Current rendered visibility state of the caret cursor.
    is_visible: bool,
}

impl Default for CaretBlinkTimer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl CaretBlinkTimer {
    /// Creates a new caret blink timer initialized with the standard OS half-period (530ms).
    ///
    /// The caret begins in an immediately visible state.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self::with_interval(DEFAULT_CARET_BLINK_INTERVAL_MS)
    }

    /// Creates a new caret blink timer with a custom half-cycle duration in milliseconds.
    ///
    /// # Arguments
    /// * `interval_ms` - Half-cycle interval in milliseconds. Clamped to a minimum of 1ms.
    #[must_use]
    #[inline]
    pub const fn with_interval(interval_ms: u64) -> Self {
        let safe_interval = if interval_ms == 0 { 1 } else { interval_ms };
        Self {
            interval_ms: safe_interval,
            elapsed_ms: 0,
            is_visible: true,
        }
    }

    /// Advances the timer by the given delta milliseconds and returns whether the visibility state changed.
    ///
    /// # Arguments
    /// * `delta_ms` - Elapsed duration since the previous tick in milliseconds.
    ///
    /// # Returns
    /// `true` if the visibility transitioned between visible and hidden; otherwise `false`.
    pub fn tick(&mut self, delta_ms: u64) -> bool {
        self.elapsed_ms = (self.elapsed_ms + delta_ms) % (self.interval_ms * 2);
        let new_visibility = self.elapsed_ms < self.interval_ms;
        let changed = new_visibility != self.is_visible;
        self.is_visible = new_visibility;
        changed
    }

    /// Returns `true` if the caret should currently be drawn.
    #[must_use]
    #[inline]
    pub const fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Resets the elapsed animation cycle and immediately forces the caret to be visible.
    ///
    /// Recommended to invoke upon receiving keystrokes, character inputs, or caret navigation events
    /// so the user receives instantaneous visual confirmation of caret position.
    #[inline]
    pub fn reset(&mut self) {
        self.elapsed_ms = 0;
        self.is_visible = true;
    }

    /// Evaluates caret visibility deterministically from an absolute millisecond timestamp and interval.
    ///
    /// Useful for stateless view passes where the widget does not retain an explicit timer struct instance.
    ///
    /// # Arguments
    /// * `elapsed_ms` - Monotonic application uptime in milliseconds.
    /// * `interval_ms` - Half-period blink interval in milliseconds (defaults to 530ms if zero).
    #[must_use]
    #[inline]
    pub const fn is_visible_at(elapsed_ms: u64, interval_ms: u64) -> bool {
        let safe_interval = if interval_ms == 0 {
            DEFAULT_CARET_BLINK_INTERVAL_MS
        } else {
            interval_ms
        };
        (elapsed_ms % (safe_interval * 2)) < safe_interval
    }
}

/// Computes the exact subpixel bounding rectangle for a vertical text cursor line.
///
/// Centers the caret vertically within the line height box and positions it horizontally
/// at the glyph advance offset relative to the text origin.
///
/// # Arguments
/// * `text_origin` - Top-left origin coordinate of the text element.
/// * `cursor_advance_x` - Horizontal advance offset of the target character boundary in pixels.
/// * `font_size` - Nominal font size in pixels.
/// * `line_height` - Total allocated line height in pixels.
/// * `caret_width` - Horizontal stroke width of the caret cursor line in pixels.
#[must_use]
#[inline]
pub fn calculate_caret_rect(
    text_origin: Point,
    cursor_advance_x: f32,
    font_size: f32,
    line_height: f32,
    caret_width: f32,
) -> Rect {
    let top_y = text_origin.y + (line_height - font_size).max(0.0) * 0.5;
    Rect::new(
        text_origin.x + cursor_advance_x,
        top_y,
        caret_width.max(1.0),
        font_size.max(1.0),
    )
}

/// Visual properties and geometric parameters describing an interactive text caret cursor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaretDescriptor {
    /// Bounding box rectangle of the rendered caret line in global or local canvas coordinates.
    pub rect: Rect,
    /// Solid fill color of the cursor.
    pub color: Color,
    /// Whether the cursor is currently in the visible phase of its blink animation cycle.
    pub is_visible: bool,
}

impl CaretDescriptor {
    /// Constructs a new caret descriptor.
    #[must_use]
    #[inline]
    pub const fn new(rect: Rect, color: Color, is_visible: bool) -> Self {
        Self {
            rect,
            color,
            is_visible,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caret_blink_timer_lifecycle() {
        let mut timer = CaretBlinkTimer::with_interval(500);
        assert!(timer.is_visible());

        // Advance 250ms -> still visible
        let changed = timer.tick(250);
        assert!(!changed);
        assert!(timer.is_visible());

        // Advance another 250ms (total 500ms) -> transitions to hidden
        let changed = timer.tick(250);
        assert!(changed);
        assert!(!timer.is_visible());

        // Advance 500ms (total 1000ms) -> transitions back to visible
        let changed = timer.tick(500);
        assert!(changed);
        assert!(timer.is_visible());

        // Hide, then reset -> immediately becomes visible
        timer.tick(500);
        assert!(!timer.is_visible());
        timer.reset();
        assert!(timer.is_visible());
    }

    #[test]
    fn test_caret_is_visible_at_stateless() {
        assert!(CaretBlinkTimer::is_visible_at(0, 500));
        assert!(CaretBlinkTimer::is_visible_at(499, 500));
        assert!(!CaretBlinkTimer::is_visible_at(500, 500));
        assert!(!CaretBlinkTimer::is_visible_at(999, 500));
        assert!(CaretBlinkTimer::is_visible_at(1000, 500));
    }

    #[test]
    fn test_calculate_caret_rect() {
        let origin = Point::new(10.0, 20.0);
        let rect = calculate_caret_rect(origin, 45.0, 14.0, 20.0, 1.5);

        assert_eq!(rect.x, 55.0);
        assert_eq!(rect.y, 23.0); // 20.0 + (20.0 - 14.0) * 0.5
        assert_eq!(rect.width, 1.5);
        assert_eq!(rect.height, 14.0);
    }
}