// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Hardware-accelerated color primitives and color-space conversions.

use bytemuck::{Pod, Zeroable};

/// Converts a non-linear sRGB color channel in `[0.0, 1.0]` to linear gamma space.
#[inline]
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts a linear color channel in `[0.0, 1.0]` to non-linear sRGB gamma space.
#[inline]
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// An RGBA floating-point color representation with components in range `[0.0, 1.0]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Color {
    /// Red channel value in range `[0.0, 1.0]`.
    pub r: f32,
    /// Green channel value in range `[0.0, 1.0]`.
    pub g: f32,
    /// Blue channel value in range `[0.0, 1.0]`.
    pub b: f32,
    /// Alpha opacity value in range `[0.0, 1.0]`.
    pub a: f32,
}

impl Color {
    /// Fully transparent black color (`rgba(0, 0, 0, 0)`).
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    /// Opaque black color (`#000000`).
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    /// Opaque white color (`#ffffff`).
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    /// Opaque pure red color (`#ff0000`).
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    /// Opaque pure green color (`#00ff00`).
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    /// Opaque pure blue color (`#0000ff`).
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    /// Opaque pure yellow color (`#ffff00`).
    pub const YELLOW: Self = Self::new(1.0, 1.0, 0.0, 1.0);
    /// Opaque pure cyan color (`#00ffff`).
    pub const CYAN: Self = Self::new(0.0, 1.0, 1.0, 1.0);
    /// Opaque pure magenta color (`#ff00ff`).
    pub const MAGENTA: Self = Self::new(1.0, 0.0, 1.0, 1.0);

    /// Creates a new `Color` from normalized RGBA float components.
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates an opaque `Color` from normalized RGB float components (`alpha = 1.0`).
    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    /// Creates a `Color` from normalized RGBA float components.
    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::new(r, g, b, a)
    }

    /// Creates a `Color` from 8-bit integer RGBA components in range `[0, 255]`.
    #[inline]
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// Parses a hexadecimal color string (e.g. `"#1e1e24"`, `"#ffffff"`, `"#ff000080"`).
    /// Supports 3, 4, 6, and 8 hex digits, with or without a leading `#`.
    pub fn hex(hex_str: &str) -> Self {
        let clean = hex_str.trim().trim_start_matches('#');
        match clean.len() {
            3 => {
                let r = u8::from_str_radix(&clean[0..1].repeat(2), 16).unwrap_or(0);
                let g = u8::from_str_radix(&clean[1..2].repeat(2), 16).unwrap_or(0);
                let b = u8::from_str_radix(&clean[2..3].repeat(2), 16).unwrap_or(0);
                Self::from_u8(r, g, b, 255)
            }
            4 => {
                let r = u8::from_str_radix(&clean[0..1].repeat(2), 16).unwrap_or(0);
                let g = u8::from_str_radix(&clean[1..2].repeat(2), 16).unwrap_or(0);
                let b = u8::from_str_radix(&clean[2..3].repeat(2), 16).unwrap_or(0);
                let a = u8::from_str_radix(&clean[3..4].repeat(2), 16).unwrap_or(255);
                Self::from_u8(r, g, b, a)
            }
            6 => {
                let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0);
                Self::from_u8(r, g, b, 255)
            }
            8 => {
                let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0);
                let a = u8::from_str_radix(&clean[6..8], 16).unwrap_or(255);
                Self::from_u8(r, g, b, a)
            }
            _ => Self::BLACK,
        }
    }

    /// Converts this sRGB color into linear color space for mathematically accurate GPU rendering.
    #[inline]
    pub fn to_linear(self) -> Self {
        Self {
            r: srgb_to_linear(self.r),
            g: srgb_to_linear(self.g),
            b: srgb_to_linear(self.b),
            a: self.a,
        }
    }

    /// Converts this linear color into non-linear sRGB color space.
    #[inline]
    pub fn to_srgb(self) -> Self {
        Self {
            r: linear_to_srgb(self.r),
            g: linear_to_srgb(self.g),
            b: linear_to_srgb(self.b),
            a: self.a,
        }
    }

    /// Performs linear interpolation between this color and another color by a factor `t` in `[0.0, 1.0]`.
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let clamped_t = t.clamp(0.0, 1.0);
        Self {
            r: self.r + (other.r - self.r) * clamped_t,
            g: self.g + (other.g - self.g) * clamped_t,
            b: self.b + (other.b - self.b) * clamped_t,
            a: self.a + (other.a - self.a) * clamped_t,
        }
    }

    /// Returns the color with modified alpha opacity.
    #[inline]
    pub fn with_alpha(self, alpha: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha.clamp(0.0, 1.0),
        }
    }

    /// Converts the color to a 4-element array of `f32` `[r, g, b, a]`.
    #[inline]
    pub const fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Default for Color {
    #[inline]
    fn default() -> Self {
        Self::WHITE
    }
}

impl From<[f32; 4]> for Color {
    #[inline]
    fn from(arr: [f32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_hex_and_lerp() {
        let red = Color::hex("#ff0000");
        assert_eq!(red, Color::new(1.0, 0.0, 0.0, 1.0));

        let dark = Color::hex("#121218");
        assert!(dark.r > 0.05 && dark.r < 0.1);

        let white = Color::WHITE;
        let black = Color::BLACK;
        let gray = black.lerp(white, 0.5);
        assert!((gray.r - 0.5).abs() < 0.001);
        assert!((gray.g - 0.5).abs() < 0.001);
        assert!((gray.b - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_srgb_linear_roundtrip() {
        let c = Color::new(0.5, 0.25, 0.75, 0.9);
        let linear = c.to_linear();
        let srgb = linear.to_srgb();
        assert!((c.r - srgb.r).abs() < 0.01);
        assert!((c.g - srgb.g).abs() < 0.01);
        assert!((c.b - srgb.b).abs() < 0.01);
    }
}