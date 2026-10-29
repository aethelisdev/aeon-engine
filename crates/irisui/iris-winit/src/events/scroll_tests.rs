// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::scroll::{
    DEFAULT_SCROLL_LINE_STEP, MIN_SAFE_SCALE_FACTOR, normalize_scroll_delta,
    normalize_scroll_delta_with_line_step,
};
use glam::Vec2;
use winit::dpi::PhysicalPosition;
use winit::event::MouseScrollDelta;

#[test]
fn test_line_delta_standard_integer_steps() {
    let delta_up = MouseScrollDelta::LineDelta(0.0, 1.0);
    let norm_up = normalize_scroll_delta(delta_up, 1.0);
    assert_eq!(norm_up, Vec2::new(0.0, DEFAULT_SCROLL_LINE_STEP));

    let delta_down = MouseScrollDelta::LineDelta(0.0, -3.0);
    let norm_down = normalize_scroll_delta(delta_down, 1.5);
    assert_eq!(norm_down, Vec2::new(0.0, -3.0 * DEFAULT_SCROLL_LINE_STEP));

    let delta_horizontal = MouseScrollDelta::LineDelta(2.0, 0.0);
    let norm_h = normalize_scroll_delta(delta_horizontal, 2.0);
    assert_eq!(norm_h, Vec2::new(2.0 * DEFAULT_SCROLL_LINE_STEP, 0.0));

    let delta_diagonal = MouseScrollDelta::LineDelta(-1.0, 2.0);
    let norm_diag = normalize_scroll_delta(delta_diagonal, 1.25);
    assert_eq!(
        norm_diag,
        Vec2::new(-DEFAULT_SCROLL_LINE_STEP, 2.0 * DEFAULT_SCROLL_LINE_STEP)
    );
}

#[test]
fn test_line_delta_fractional_steps() {
    let delta = MouseScrollDelta::LineDelta(0.5, -0.25);
    let norm = normalize_scroll_delta(delta, 1.0);
    assert_eq!(norm.x, 0.5 * DEFAULT_SCROLL_LINE_STEP);
    assert_eq!(norm.y, -0.25 * DEFAULT_SCROLL_LINE_STEP);
}

#[test]
fn test_pixel_delta_high_dpi_scaling() {
    // 1.0x Scale Factor
    let delta = MouseScrollDelta::PixelDelta(PhysicalPosition::new(100.0, -200.0));
    let norm_1x = normalize_scroll_delta(delta, 1.0);
    assert_eq!(norm_1x, Vec2::new(100.0, -200.0));

    // 1.25x Scale Factor (125% DPI)
    let delta = MouseScrollDelta::PixelDelta(PhysicalPosition::new(125.0, -250.0));
    let norm_1_25x = normalize_scroll_delta(delta, 1.25);
    assert!((norm_1_25x.x - 100.0).abs() < 1e-5);
    assert!((norm_1_25x.y - -200.0).abs() < 1e-5);

    // 1.5x Scale Factor (150% DPI)
    let delta = MouseScrollDelta::PixelDelta(PhysicalPosition::new(150.0, -300.0));
    let norm_1_5x = normalize_scroll_delta(delta, 1.5);
    assert!((norm_1_5x.x - 100.0).abs() < 1e-5);
    assert!((norm_1_5x.y - -200.0).abs() < 1e-5);

    // 2.0x Scale Factor (Retina / 4K UHD)
    let delta = MouseScrollDelta::PixelDelta(PhysicalPosition::new(200.0, -400.0));
    let norm_2x = normalize_scroll_delta(delta, 2.0);
    assert_eq!(norm_2x, Vec2::new(100.0, -200.0));

    // 2.5x Scale Factor
    let delta = MouseScrollDelta::PixelDelta(PhysicalPosition::new(250.0, -500.0));
    let norm_2_5x = normalize_scroll_delta(delta, 2.5);
    assert!((norm_2_5x.x - 100.0).abs() < 1e-5);
    assert!((norm_2_5x.y - -200.0).abs() < 1e-5);
}

#[test]
fn test_zero_delta_handling() {
    let zero_line = MouseScrollDelta::LineDelta(0.0, 0.0);
    assert_eq!(normalize_scroll_delta(zero_line, 1.0), Vec2::ZERO);

    let zero_pixel = MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, 0.0));
    assert_eq!(normalize_scroll_delta(zero_pixel, 2.0), Vec2::ZERO);
}

#[test]
fn test_division_by_zero_and_abnormal_scale_protection() {
    let delta = MouseScrollDelta::PixelDelta(PhysicalPosition::new(50.0, 100.0));

    // 0.0 scale factor: should safely fallback to 1.0 without dividing by zero or producing Infinity
    let norm_zero = normalize_scroll_delta(delta, 0.0);
    assert_eq!(norm_zero, Vec2::new(50.0, 100.0));

    // Negative scale factor: should safely fallback to 1.0
    let norm_neg = normalize_scroll_delta(delta, -2.0);
    assert_eq!(norm_neg, Vec2::new(50.0, 100.0));

    // Below minimum safe threshold
    let norm_tiny = normalize_scroll_delta(delta, MIN_SAFE_SCALE_FACTOR / 10.0);
    assert_eq!(norm_tiny, Vec2::new(50.0, 100.0));

    // NaN scale factor
    let norm_nan_scale = normalize_scroll_delta(delta, f64::NAN);
    assert_eq!(norm_nan_scale, Vec2::new(50.0, 100.0));

    // Infinite scale factor
    let norm_inf_scale = normalize_scroll_delta(delta, f64::INFINITY);
    assert_eq!(norm_inf_scale, Vec2::new(50.0, 100.0));
}

#[test]
fn test_non_finite_coordinate_sanitization() {
    // LineDelta with NaN and Inf
    let nan_line = MouseScrollDelta::LineDelta(f32::NAN, 1.0);
    let norm_nan_line = normalize_scroll_delta(nan_line, 1.0);
    assert_eq!(norm_nan_line.x, 0.0);
    assert_eq!(norm_nan_line.y, DEFAULT_SCROLL_LINE_STEP);

    let inf_line = MouseScrollDelta::LineDelta(f32::INFINITY, f32::NEG_INFINITY);
    let norm_inf_line = normalize_scroll_delta(inf_line, 1.0);
    assert_eq!(norm_inf_line, Vec2::ZERO);

    // PixelDelta with NaN and Inf
    let nan_pixel = MouseScrollDelta::PixelDelta(PhysicalPosition::new(f64::NAN, 120.0));
    let norm_nan_pixel = normalize_scroll_delta(nan_pixel, 1.5);
    assert_eq!(norm_nan_pixel.x, 0.0);
    assert_eq!(norm_nan_pixel.y, 80.0);

    let inf_pixel = MouseScrollDelta::PixelDelta(PhysicalPosition::new(f64::INFINITY, -50.0));
    let norm_inf_pixel = normalize_scroll_delta(inf_pixel, 2.0);
    assert_eq!(norm_inf_pixel.x, 0.0);
    assert_eq!(norm_inf_pixel.y, -25.0);
}

#[test]
fn test_custom_line_step() {
    let delta = MouseScrollDelta::LineDelta(1.0, -2.0);

    // Custom 24px line step (e.g. for small text console)
    let norm_24 = normalize_scroll_delta_with_line_step(delta, 1.0, 24.0);
    assert_eq!(norm_24, Vec2::new(24.0, -48.0));

    // Invalid non-positive line step should fallback to DEFAULT_SCROLL_LINE_STEP (32.0)
    let norm_invalid = normalize_scroll_delta_with_line_step(delta, 1.0, -10.0);
    assert_eq!(
        norm_invalid,
        Vec2::new(DEFAULT_SCROLL_LINE_STEP, -2.0 * DEFAULT_SCROLL_LINE_STEP)
    );

    let norm_nan_step = normalize_scroll_delta_with_line_step(delta, 1.0, f32::NAN);
    assert_eq!(
        norm_nan_step,
        Vec2::new(DEFAULT_SCROLL_LINE_STEP, -2.0 * DEFAULT_SCROLL_LINE_STEP)
    );
}