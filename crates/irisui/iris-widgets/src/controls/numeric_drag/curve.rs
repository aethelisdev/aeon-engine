// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Non-Linear Pointer Drag Response Curves (`DragCurve`)
//!
//! Provides mathematically verified linear, exponential (polynomial), and logarithmic
//! velocity ramp curves for pointer scrubbing across wide scalar domains (decibels,
//! frequencies, camera near/far clipping planes, 3D transform scales).

/// Natural logarithm of 2.0 used for logarithmic curve normalization.
const LN_2: f64 = std::f64::consts::LN_2;

/// Default reference pixel displacement required to advance one standard step unit (`10.0 px`).
pub const DEFAULT_PIXELS_PER_STEP: f64 = 10.0;

/// Default fine-tuning sensitivity multiplier applied when Shift is held (`0.1`).
pub const NUMERIC_DRAG_FINE_MULTIPLIER: f64 = 0.1;

/// Default coarse-tuning sensitivity multiplier applied when Control / Command is held (`10.0`).
pub const NUMERIC_DRAG_COARSE_MULTIPLIER: f64 = 10.0;

/// Default exponential polynomial power exponent for non-linear scrubbers (`2.0`).
pub const DEFAULT_EXPONENTIAL_POWER: f32 = 2.0;

/// Non-linear scrubbing curve governing the mapping between mouse displacement ($\Delta x$)
/// and scalar value changes ($\Delta v$).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragCurve {
    /// Standard linear response: value delta is directly proportional to pointer displacement:
    ///
    /// $$\Delta v = \text{sign}(\Delta x) \cdot \left|\frac{\Delta x}{\text{pixels\_per\_step}}\right| \cdot (\text{speed} \cdot \text{pixels\_per\_step}) \cdot \text{multiplier}$$
    Linear,

    /// Exponential polynomial ramp with custom power exponent ($p$):
    ///
    /// $$\Delta v = \text{sign}(\Delta x) \cdot \left|\frac{\Delta x}{\text{pixels\_per\_step}}\right|^p \cdot (\text{speed} \cdot \text{pixels\_per\_step}) \cdot \text{multiplier}$$
    ///
    /// Exponents $p > 1.0$ (typically $p = 2.0$ or $p = 3.0$) grant micro-precision for subtle
    /// cursor movements while permitting rapid leaps across wide value ranges during wider gestures.
    Exponential {
        /// Power exponent defining the acceleration curve ($p > 0.0$, typically 2.0 or 3.0).
        power: f32,
    },

    /// Logarithmic response curve providing smooth sub-linear compression over wide ranges:
    ///
    /// $$\Delta v = \text{sign}(\Delta x) \cdot \frac{\ln\left(1.0 + \left|\frac{\Delta x}{\text{pixels\_per\_step}}\right|\right)}{\ln(2.0)} \cdot (\text{speed} \cdot \text{pixels\_per\_step}) \cdot \text{multiplier}$$
    ///
    /// Normalizes such that at $|\Delta x| = \text{pixels\_per\_step}$, $\Delta v \equiv \text{speed} \cdot \text{pixels\_per\_step} \cdot \text{multiplier}$,
    /// smoothly dampening growth over large displacements.
    Logarithmic,
}

impl Default for DragCurve {
    #[inline]
    fn default() -> Self {
        Self::Linear
    }
}

impl DragCurve {
    /// Constructs a standard linear drag curve.
    #[inline]
    #[must_use]
    pub const fn linear() -> Self {
        Self::Linear
    }

    /// Constructs a quadratic exponential drag curve ($p = 2.0$).
    #[inline]
    #[must_use]
    pub const fn quadratic() -> Self {
        Self::Exponential { power: 2.0 }
    }

    /// Constructs a cubic exponential drag curve ($p = 3.0$).
    #[inline]
    #[must_use]
    pub const fn cubic() -> Self {
        Self::Exponential { power: 3.0 }
    }

    /// Constructs an exponential drag curve with an explicit power exponent.
    #[inline]
    #[must_use]
    pub const fn exponential(power: f32) -> Self {
        Self::Exponential { power }
    }

    /// Constructs a logarithmic sub-linear drag curve.
    #[inline]
    #[must_use]
    pub const fn logarithmic() -> Self {
        Self::Logarithmic
    }

    /// Evaluates the scalar value delta ($\Delta v$) for a given pointer movement.
    ///
    /// # Arguments
    ///
    /// * `delta_x` - Horizontal cursor displacement in logical pixels ($\Delta x$).
    /// * `speed` - Base scalar value change per logical pixel in linear mode.
    /// * `pixels_per_step` - Reference pixel displacement where linear and non-linear curves intersect.
    /// * `multiplier` - Sensitivity modifier (e.g. `0.1` for Shift fine-tuning, `10.0` for Ctrl coarse steps).
    ///
    /// # Invariants
    ///
    /// - **Symmetric**: $f(-\Delta x) = -f(\Delta x)$.
    /// - **Zero at Origin**: $f(0) = 0.0$.
    /// - **Normalization Point**: At $|\Delta x| = \text{pixels\_per\_step}$, linear, exponential, and logarithmic
    ///   curves yield the exact same value delta: $\text{pixels\_per\_step} \cdot \text{speed} \cdot \text{multiplier}$.
    /// - **Robust**: Returns `0.0` on non-finite, `NaN`, or degenerate inputs without panicking.
    #[must_use]
    pub fn compute_delta(
        &self,
        delta_x: f64,
        speed: f64,
        pixels_per_step: f64,
        multiplier: f64,
    ) -> f64 {
        if !delta_x.is_finite()
            || delta_x == 0.0
            || !speed.is_finite()
            || speed == 0.0
            || !multiplier.is_finite()
            || multiplier == 0.0
        {
            return 0.0;
        }

        let pps = if pixels_per_step.is_finite() && pixels_per_step > 0.0 {
            pixels_per_step
        } else {
            DEFAULT_PIXELS_PER_STEP
        };

        let abs_dx = delta_x.abs();
        let sign = delta_x.signum();
        let ref_delta = speed * pps;
        let ratio = abs_dx / pps;

        let scaled = match *self {
            Self::Linear => ratio * ref_delta,
            Self::Exponential { power } => {
                let p = if power.is_finite() && power > 0.0 {
                    power as f64
                } else {
                    1.0
                };
                ratio.powf(p) * ref_delta
            }
            Self::Logarithmic => {
                let log_factor = (1.0 + ratio).ln() / LN_2;
                log_factor * ref_delta
            }
        };

        sign * scaled * multiplier
    }

    /// Evaluates the scalar value delta directly in single-precision [`f32`], resolving
    /// Shift ($0.1\times$) and Ctrl ($10.0\times$) multipliers from explicit modifier booleans.
    #[must_use]
    #[inline]
    pub fn compute_delta_from_modifiers(
        &self,
        delta_x: f32,
        speed: f32,
        pixels_per_step: f32,
        shift: bool,
        ctrl: bool,
    ) -> f32 {
        let mult = if shift {
            NUMERIC_DRAG_FINE_MULTIPLIER
        } else if ctrl {
            NUMERIC_DRAG_COARSE_MULTIPLIER
        } else {
            1.0
        };
        self.compute_delta(delta_x as f64, speed as f64, pixels_per_step as f64, mult) as f32
    }

    /// Evaluates the scalar value delta directly in single-precision [`f32`], resolving
    /// modifier sensitivity multipliers automatically from the active [`iris_core::EventState`].
    #[must_use]
    #[inline]
    pub fn compute_delta_f32(
        &self,
        delta_x: f32,
        speed: f32,
        pixels_per_step: f32,
        event_state: &iris_core::EventState,
    ) -> f32 {
        self.compute_delta_from_modifiers(
            delta_x,
            speed,
            pixels_per_step,
            event_state.frame_shift_held,
            event_state.frame_ctrl_held,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_curve_proportionality_and_sign() {
        let curve = DragCurve::linear();
        let speed = 0.5;
        let pps = 10.0;
        let mult = 1.0;

        // 10px drag with speed 0.5 -> 5.0
        let delta_pos = curve.compute_delta(10.0, speed, pps, mult);
        assert!((delta_pos - 5.0).abs() < 1e-6);

        // Negative drag preserves symmetry
        let delta_neg = curve.compute_delta(-10.0, speed, pps, mult);
        assert!((delta_neg - (-5.0)).abs() < 1e-6);

        // 20px drag with speed 0.5 -> 10.0
        let delta_20 = curve.compute_delta(20.0, speed, pps, mult);
        assert!((delta_20 - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_curve_intersection_at_reference_pixels() {
        let speed = 2.0;
        let pps = 15.0;
        let mult = 1.0;
        let expected = speed * pps; // 30.0

        let linear = DragCurve::Linear;
        let quad = DragCurve::quadratic();
        let cubic = DragCurve::cubic();
        let log = DragCurve::logarithmic();

        let d_lin = linear.compute_delta(pps, speed, pps, mult);
        let d_quad = quad.compute_delta(pps, speed, pps, mult);
        let d_cub = cubic.compute_delta(pps, speed, pps, mult);
        let d_log = log.compute_delta(pps, speed, pps, mult);

        assert!((d_lin - expected).abs() < 1e-6);
        assert!((d_quad - expected).abs() < 1e-6);
        assert!((d_cub - expected).abs() < 1e-6);
        assert!((d_log - expected).abs() < 1e-6);
    }

    #[test]
    fn test_exponential_micro_precision_and_acceleration() {
        let quad = DragCurve::quadratic(); // p = 2.0
        let speed = 1.0;
        let pps = 10.0;
        let mult = 1.0;
        let ref_delta = speed * pps; // 10.0

        // Micro-movement: 1px (ratio 0.1) -> 0.1^2 * 10 = 0.1
        // Compared to linear which would yield 1.0 -> 10x finer precision!
        let d_micro = quad.compute_delta(1.0, speed, pps, mult);
        assert!((d_micro - 0.1).abs() < 1e-6);

        // Small movement: 2px (ratio 0.2) -> 0.04 * 10 = 0.4
        // Compared to linear 2.0 -> 5x finer precision!
        let d_small = quad.compute_delta(2.0, speed, pps, mult);
        assert!((d_small - 0.4).abs() < 1e-6);

        // Reference: 10px -> 10.0
        let d_ref = quad.compute_delta(10.0, speed, pps, mult);
        assert!((d_ref - ref_delta).abs() < 1e-6);

        // Acceleration: 30px (ratio 3.0) -> 3^2 * 10 = 90.0
        // Compared to linear 30.0 -> 3x faster leap!
        let d_acc = quad.compute_delta(30.0, speed, pps, mult);
        assert!((d_acc - 90.0).abs() < 1e-6);

        // Wide acceleration: 100px (ratio 10.0) -> 10^2 * 10 = 1000.0
        // Compared to linear 100.0 -> 10x faster leap!
        let d_wide = quad.compute_delta(100.0, speed, pps, mult);
        assert!((d_wide - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn test_logarithmic_compression() {
        let log = DragCurve::logarithmic();
        let speed = 1.0;
        let pps = 10.0;
        let mult = 1.0;

        // At pps (10px): ratio = 1 -> ln(2)/ln(2) * 10 = 10.0
        let d_ref = log.compute_delta(10.0, speed, pps, mult);
        assert!((d_ref - 10.0).abs() < 1e-6);

        // At 100px (10x pps): ratio = 10 -> ln(11)/ln(2) * 10 = ~34.59
        // Linear would be 100.0, logarithmic compresses by ~3x!
        let d_100 = log.compute_delta(100.0, speed, pps, mult);
        assert!(d_100 < 35.0 && d_100 > 34.0);

        // At 1000px (100x pps): ratio = 100 -> ln(101)/ln(2) * 10 = ~66.58
        // Linear would be 1000.0, logarithmic compresses by ~15x!
        let d_1000 = log.compute_delta(1000.0, speed, pps, mult);
        assert!(d_1000 < 67.0 && d_1000 > 66.0);
    }

    #[test]
    fn test_curve_modifiers_fine_and_coarse() {
        let quad = DragCurve::quadratic();
        let speed = 1.0;
        let pps = 10.0;

        // Shift fine multiplier: 0.1x
        let d_fine = quad.compute_delta(10.0, speed, pps, 0.1);
        assert!((d_fine - 1.0).abs() < 1e-6);

        // Ctrl coarse multiplier: 10.0x
        let d_coarse = quad.compute_delta(10.0, speed, pps, 10.0);
        assert!((d_coarse - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_degenerate_and_nan_inputs() {
        let quad = DragCurve::quadratic();

        assert_eq!(quad.compute_delta(0.0, 1.0, 10.0, 1.0), 0.0);
        assert_eq!(quad.compute_delta(f64::NAN, 1.0, 10.0, 1.0), 0.0);
        assert_eq!(quad.compute_delta(10.0, f64::NAN, 10.0, 1.0), 0.0);
        assert_eq!(quad.compute_delta(10.0, 1.0, f64::NAN, 1.0), 10.0); // pps falls back to default 10.0
        assert_eq!(quad.compute_delta(10.0, 1.0, 10.0, f64::NAN), 0.0);
        assert_eq!(quad.compute_delta(f64::INFINITY, 1.0, 10.0, 1.0), 0.0);
        assert_eq!(quad.compute_delta(10.0, f64::INFINITY, 10.0, 1.0), 0.0);
    }

    #[test]
    fn test_compute_delta_f32_and_event_state_modifiers() {
        let quad = DragCurve::quadratic();
        let speed = 1.0;
        let pps = 10.0;

        // Normal modifier: 1.0x
        let d_normal = quad.compute_delta_from_modifiers(10.0, speed, pps, false, false);
        assert!((d_normal - 10.0).abs() < 1e-5);

        // Shift modifier: 0.1x
        let d_shift = quad.compute_delta_from_modifiers(10.0, speed, pps, true, false);
        assert!((d_shift - 1.0).abs() < 1e-5);

        // Ctrl modifier: 10.0x
        let d_ctrl = quad.compute_delta_from_modifiers(10.0, speed, pps, false, true);
        assert!((d_ctrl - 100.0).abs() < 1e-5);

        // EventState integration
        let mut event_state = iris_core::EventState::default();
        assert!((quad.compute_delta_f32(10.0, speed, pps, &event_state) - 10.0).abs() < 1e-5);

        event_state.frame_shift_held = true;
        assert!((quad.compute_delta_f32(10.0, speed, pps, &event_state) - 1.0).abs() < 1e-5);

        event_state.frame_shift_held = false;
        event_state.frame_ctrl_held = true;
        assert!((quad.compute_delta_f32(10.0, speed, pps, &event_state) - 100.0).abs() < 1e-5);
    }
}