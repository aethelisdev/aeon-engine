// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hardware-Accelerated 2D HSV Color Representation & Math (`iris-widgets::color_picker`)
//!
//! Provides bidirectional RGB/HSV/HEX conversions, color state tracking,
//! and interaction drag modes for color pickers and palettes in Iris UI.
//!

use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};

/// Converts standard RGB components (0.0 ..= 1.0) to HSV representation.
///
/// Returns `(hue, saturation, value)` where:
/// - `hue`: `0.0 .. 360.0` degrees
/// - `saturation`: `0.0 ..= 1.0`
/// - `value`: `0.0 ..= 1.0`
///
/// # Arguments
/// * `r` - Red channel in logical range `[0.0, 1.0]`.
/// * `g` - Green channel in logical range `[0.0, 1.0]`.
/// * `b` - Blue channel in logical range `[0.0, 1.0]`.
#[inline]
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let v = max;
    let s = if max > 1e-5 { delta / max } else { 0.0 };

    let h = if delta < 1e-5 {
        0.0
    } else if (max - r).abs() < 1e-5 {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max - g).abs() < 1e-5 {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };
    (h, s, v)
}

/// Converts HSV color representation to standard [`Color`] (RGBA float).
///
/// # Arguments
/// - `h`: Hue in degrees `0.0 .. 360.0`
/// - `s`: Saturation `0.0 ..= 1.0`
/// - `v`: Value / Brightness `0.0 ..= 1.0`
#[inline]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let h_norm = (h % 360.0 + 360.0) % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h_norm / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = match (h_norm / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color::rgba(r1 + m, g1 + m, b1 + m, 1.0)
}

/// Dynamic state tracker for the 2D HSV Color representation.
///
/// Encapsulates hue, saturation, value, and alpha channels with bidirectional
/// conversion methods to and from [`Color`] and hexadecimal strings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HsvColorPickerState {
    /// Active Hue in degrees `0.0 .. 360.0`.
    pub hue: f32,
    /// Active Saturation `0.0 ..= 1.0`.
    pub saturation: f32,
    /// Active Value / Brightness `0.0 ..= 1.0`.
    pub value: f32,
    /// Active Alpha transparency `0.0 ..= 1.0`.
    pub alpha: f32,
}

impl Default for HsvColorPickerState {
    fn default() -> Self {
        Self {
            hue: 180.0,
            saturation: 0.8,
            value: 0.9,
            alpha: 1.0,
        }
    }
}

impl HsvColorPickerState {
    /// Constructs a new [`HsvColorPickerState`] from an existing [`Color`].
    #[must_use]
    pub fn from_color(color: Color) -> Self {
        let (h, s, v) = rgb_to_hsv(color.r, color.g, color.b);
        Self {
            hue: h,
            saturation: s,
            value: v,
            alpha: color.a,
        }
    }

    /// Converts the current HSV state to a concrete [`Color`] instance.
    #[must_use]
    pub fn to_color(&self) -> Color {
        let mut c = hsv_to_rgb(self.hue, self.saturation, self.value);
        c.a = self.alpha;
        c
    }

    /// Returns the uppercase HEX string representation (e.g. `"#38BDF8"`).
    #[must_use]
    pub fn to_hex(&self) -> String {
        let c = self.to_color();
        let r = (c.r.clamp(0.0, 1.0) * 255.0) as u8;
        let g = (c.g.clamp(0.0, 1.0) * 255.0) as u8;
        let b = (c.b.clamp(0.0, 1.0) * 255.0) as u8;
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }

    /// Updates Saturation and Value based on an absolute click/drag point over the 2D SV box.
    pub fn update_from_sv_point(&mut self, point: Point, sv_rect: Rect) {
        if sv_rect.width <= 0.0 || sv_rect.height <= 0.0 {
            return;
        }
        let rel_x = (point.x - sv_rect.x).clamp(0.0, sv_rect.width);
        let rel_y = (point.y - sv_rect.y).clamp(0.0, sv_rect.height);

        self.saturation = (rel_x / sv_rect.width).clamp(0.0, 1.0);
        self.value = (1.0 - (rel_y / sv_rect.height)).clamp(0.0, 1.0);
    }

    /// Updates Hue based on an absolute click/drag point over the vertical Hue bar.
    pub fn update_from_hue_point(&mut self, point: Point, hue_rect: Rect) {
        if hue_rect.height <= 0.0 {
            return;
        }
        let rel_y = (point.y - hue_rect.y).clamp(0.0, hue_rect.height);
        self.hue = (rel_y / hue_rect.height * 360.0).clamp(0.0, 360.0);
    }
}

/// Drag mode for continuous Color Picker mouse interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorPickerDragMode {
    /// Dragging across the 2D Saturation-Value matrix.
    SaturationValue,
    /// Dragging along the vertical Rainbow Hue spectrum bar.
    Hue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_hsv_roundtrip() {
        let colors = [
            Color::RED,
            Color::GREEN,
            Color::BLUE,
            Color::WHITE,
            Color::BLACK,
            Color::rgba(0.2, 0.4, 0.8, 1.0),
        ];

        for c in colors {
            let (h, s, v) = rgb_to_hsv(c.r, c.g, c.b);
            let back = hsv_to_rgb(h, s, v);
            assert!(
                (c.r - back.r).abs() < 1e-4,
                "R mismatch for {:?}: got {}",
                c,
                back.r
            );
            assert!(
                (c.g - back.g).abs() < 1e-4,
                "G mismatch for {:?}: got {}",
                c,
                back.g
            );
            assert!(
                (c.b - back.b).abs() < 1e-4,
                "B mismatch for {:?}: got {}",
                c,
                back.b
            );
        }
    }

    #[test]
    fn test_hsv_state_conversions() {
        let state = HsvColorPickerState {
            hue: 200.0,
            saturation: 0.5,
            value: 0.8,
            alpha: 1.0,
        };
        let color = state.to_color();
        let reconstructed = HsvColorPickerState::from_color(color);
        assert!((state.hue - reconstructed.hue).abs() < 0.5);
        assert!((state.saturation - reconstructed.saturation).abs() < 0.01);
        assert!((state.value - reconstructed.value).abs() < 0.01);

        let hex = state.to_hex();
        assert!(hex.starts_with('#') && hex.len() == 7);
    }

    #[test]
    fn test_update_from_sv_and_hue_point() {
        let mut state = HsvColorPickerState::default();
        let sv_rect = Rect::new(10.0, 20.0, 100.0, 100.0);
        let hue_rect = Rect::new(120.0, 20.0, 20.0, 100.0);

        // Center click on SV box
        state.update_from_sv_point(Point::new(60.0, 70.0), sv_rect);
        assert!((state.saturation - 0.5).abs() < 0.01);
        assert!((state.value - 0.5).abs() < 0.01);

        // Halfway click on Hue bar
        state.update_from_hue_point(Point::new(130.0, 70.0), hue_rect);
        assert!((state.hue - 180.0).abs() < 1.0);
    }
}