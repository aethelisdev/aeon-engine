// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Mouse and Touchpad Scroll Delta Normalization
//!
//! Provides high-precision normalization for platform mouse wheel and touchpad events
//! ([`winit::event::MouseScrollDelta`]).
//!
//! Converts discrete wheel clicks (`LineDelta`) and continuous touchpad / smooth-scroll
//! physical deltas (`PixelDelta`) into unified, HighDPI-scaled logical pixel vectors ([`glam::Vec2`]).

use glam::Vec2;
use winit::event::MouseScrollDelta;

/// Standard line step in logical pixels per discrete mouse wheel tick.
///
/// In desktop operating systems, one mechanical wheel notch corresponds to approximately
/// 3 lines of text or 32 logical pixels of smooth scroll distance.
pub const DEFAULT_SCROLL_LINE_STEP: f32 = 32.0;

/// Minimum safe scale factor used to guard against division-by-zero or negative scale values.
pub const MIN_SAFE_SCALE_FACTOR: f64 = 1e-4;

/// Normalizes a native [`MouseScrollDelta`] into a logical pixel scroll vector ([`Vec2`]).
///
/// Handles both discrete mouse wheel ticks ([`MouseScrollDelta::LineDelta`]) and continuous
/// high-precision touchpad / trackball offsets ([`MouseScrollDelta::PixelDelta`]).
///
/// ### Mathematical Transformation
/// - **Discrete Wheel Ticks (`LineDelta(x, y)`)**:
///   $$\Delta x = x \times 32.0\text{ px},\quad \Delta y = y \times 32.0\text{ px}$$
/// - **Continuous Physical Pixels (`PixelDelta(PhysicalPosition { x, y })`)**:
///   $$\Delta x = \frac{x}{\text{scale\_factor}},\quad \Delta y = \frac{y}{\text{scale\_factor}}$$
///
/// ### Safety and Invariant Guarantees
/// - **Division by Zero Protection**: If `scale_factor` is non-positive or non-finite,
///   the normalizer safely falls back to a neutral 1.0 scale factor.
/// - **Non-Finite Coordinate Sanitization**: If any input coordinate is `NaN` or infinite (`Inf`),
///   it is safely neutralized to `0.0` to preserve mathematical integrity and avoid downstream
///   layout or rendering panics.
///
/// # Arguments
/// * `delta` - Native scroll delta emitted by `winit`.
/// * `scale_factor` - Window HighDPI scale factor (e.g. 1.0, 1.25, 1.5, 2.0).
///
/// # Returns
/// Logical pixel offset vector `Vec2` where positive `y` represents scrolling upwards
/// and positive `x` represents scrolling to the right.
///
/// # Example
/// ```rust
/// use iris_winit::events::normalize_scroll_delta;
/// use winit::event::MouseScrollDelta;
/// use glam::Vec2;
///
/// let line_delta = MouseScrollDelta::LineDelta(0.0, 2.0);
/// let logical_delta = normalize_scroll_delta(line_delta, 1.5);
/// assert_eq!(logical_delta, Vec2::new(0.0, 64.0));
/// ```
#[inline]
pub fn normalize_scroll_delta(delta: MouseScrollDelta, scale_factor: f64) -> Vec2 {
    normalize_scroll_delta_with_line_step(delta, scale_factor, DEFAULT_SCROLL_LINE_STEP)
}

/// Normalizes a native [`MouseScrollDelta`] using a custom line step height.
///
/// Useful for specialized viewport panels or text consoles that require line scrolling
/// synchronized with dynamic font metrics rather than the default 32px step.
///
/// # Arguments
/// * `delta` - Native scroll delta emitted by `winit`.
/// * `scale_factor` - Window HighDPI scale factor.
/// * `line_step` - Custom line step size in logical pixels.
///
/// # Returns
/// Logical pixel offset vector `Vec2`.
#[inline]
pub fn normalize_scroll_delta_with_line_step(
    delta: MouseScrollDelta,
    scale_factor: f64,
    line_step: f32,
) -> Vec2 {
    let effective_step = if line_step.is_finite() && line_step > 0.0 {
        line_step
    } else {
        DEFAULT_SCROLL_LINE_STEP
    };

    match delta {
        MouseScrollDelta::LineDelta(x, y) => {
            let dx = if x.is_finite() {
                x * effective_step
            } else {
                0.0
            };
            let dy = if y.is_finite() {
                y * effective_step
            } else {
                0.0
            };
            Vec2::new(dx, dy)
        }
        MouseScrollDelta::PixelDelta(pos) => {
            let safe_scale = if scale_factor.is_finite() && scale_factor >= MIN_SAFE_SCALE_FACTOR {
                scale_factor
            } else {
                1.0
            };

            let dx = if pos.x.is_finite() {
                (pos.x / safe_scale) as f32
            } else {
                0.0
            };
            let dy = if pos.y.is_finite() {
                (pos.y / safe_scale) as f32
            } else {
                0.0
            };

            let safe_dx = if dx.is_finite() { dx } else { 0.0 };
            let safe_dy = if dy.is_finite() { dy } else { 0.0 };

            Vec2::new(safe_dx, safe_dy)
        }
    }
}