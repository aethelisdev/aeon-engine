// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Allocation-free pointer gesture tracking independent of a windowing backend.

use crate::Point;
use std::time::Duration;

/// Pointer gestures accumulated since the previous frame, in logical UI coordinates.
/// Feed all pointer transitions before routing consumption. Call `end_frame` only after consumers
/// have observed release edges, and `cancel` on focus loss so captures cannot remain stuck.
#[derive(Debug, Default)]
pub struct PointerState {
    /// Most recent valid pointer position; absent after leaving an uncaptured window.
    pub position: Option<Point>,
    /// Accumulated logical movement since the previous frame.
    pub delta: Point,
    /// Whether the primary button is currently held.
    pub primary_down: bool,
    /// Primary press edge, retained until the frame is consumed.
    pub primary_pressed: bool,
    /// Primary release edge, including release after dragging.
    pub primary_released: bool,
    /// Release within the click distance of the press origin.
    pub primary_clicked: bool,
    /// Second nearby click within the double-click interval.
    pub double_clicked: bool,
    /// Secondary click edge, for panel context and detach actions.
    pub secondary_clicked: bool,
    /// Logical primary press origin, retained through its release frame.
    pub press_origin: Option<Point>,
    last_click: Option<(Duration, Point)>,
    press_time: Option<Duration>,
    secondary_origin: Option<Point>,
    moved_since_press: bool,
}

impl PointerState {
    /// Updates position while accumulating movement across multiple platform events.
    pub fn move_to(&mut self, position: Point) {
        if let Some(previous) = self.position {
            self.delta.x += position.x - previous.x;
            self.delta.y += position.y - previous.y;
        }
        self.position = Some(position);
        if self.primary_down && self.press_origin.is_some_and(|p| distance_squared(p, position) > 25.0) {
            self.moved_since_press = true;
        }
    }

    /// Records a primary transition using a monotonic timestamp supplied by the host.
    /// Repeated transitions are ignored. Clicks require a short release near the press origin;
    /// moving away and back still counts as a drag, preventing accidental close after dragging.
    pub fn primary_button(&mut self, down: bool, now: Duration) {
        if self.primary_down == down { return; }
        self.primary_down = down;
        if down {
            self.primary_pressed = true;
            self.press_origin = self.position;
            self.press_time = Some(now);
            self.moved_since_press = false;
        } else {
            self.primary_released = true;
            if !self.moved_since_press
                && self.press_time.is_some_and(|time| now.saturating_sub(time) <= Duration::from_millis(600))
                && let (Some(origin), Some(position)) = (self.press_origin, self.position)
                && distance_squared(origin, position) <= 25.0
            {
                self.primary_clicked = true;
                self.double_clicked = self.last_click.is_some_and(|(time, p)| {
                    now.saturating_sub(time) <= Duration::from_millis(400) && distance_squared(p, position) <= 36.0
                });
                self.last_click = if self.double_clicked { None } else { Some((now, position)) };
            }
        }
    }

    /// Tracks a secondary click without confusing it with a primary drag release.
    pub fn secondary_button(&mut self, down: bool) {
        if down {
            self.secondary_origin = self.position;
        } else if let (Some(origin), Some(position)) = (self.secondary_origin.take(), self.position) {
            self.secondary_clicked = distance_squared(origin, position) <= 25.0;
        }
    }

    /// Reports whether the current primary gesture has crossed the drag threshold.
    pub fn dragging(&self) -> bool {
        self.primary_down && self.moved_since_press
    }

    /// Clears transient edges without losing held buttons or their capture origin.
    pub fn end_frame(&mut self) {
        self.delta = Point::ZERO;
        self.primary_pressed = false;
        self.primary_released = false;
        self.primary_clicked = false;
        self.double_clicked = false;
        self.secondary_clicked = false;
        if !self.primary_down { self.press_origin = None; }
    }

    /// Cancels all gestures after focus loss without synthesizing a click or drop.
    pub fn cancel(&mut self) { *self = Self::default(); }
}

fn distance_squared(a: Point, b: Point) -> f32 {
    (a.x - b.x).powi(2) + (a.y - b.y).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_returning_to_origin_is_not_a_click() {
        let mut input = PointerState::default();
        input.move_to(Point::new(10.0, 10.0));
        input.primary_button(true, Duration::ZERO);
        input.move_to(Point::new(30.0, 10.0));
        input.move_to(Point::new(10.0, 10.0));
        input.primary_button(false, Duration::from_millis(200));
        assert!(input.primary_released);
        assert!(!input.primary_clicked);
    }

    #[test]
    fn release_survives_until_frame_end_and_focus_loss_cancels() {
        let mut input = PointerState::default();
        input.move_to(Point::ZERO);
        input.primary_button(true, Duration::ZERO);
        input.end_frame();
        assert!(input.primary_down);
        input.primary_button(false, Duration::from_millis(100));
        assert!(input.primary_clicked);
        input.end_frame();
        assert!(!input.primary_released);
        input.primary_button(true, Duration::from_millis(150));
        input.cancel();
        assert!(!input.primary_down);
        assert!(!input.primary_clicked);
    }

    #[test]
    fn double_click_requires_time_and_position_proximity() {
        let mut input = PointerState::default();
        input.move_to(Point::ZERO);
        input.primary_button(true, Duration::ZERO);
        input.primary_button(false, Duration::from_millis(50));
        input.end_frame();
        input.primary_button(true, Duration::from_millis(100));
        input.primary_button(false, Duration::from_millis(150));
        assert!(input.double_clicked);
        input.end_frame();
        input.move_to(Point::new(100.0, 0.0));
        input.primary_button(true, Duration::from_millis(200));
        input.primary_button(false, Duration::from_millis(250));
        assert!(!input.double_clicked);
    }
}