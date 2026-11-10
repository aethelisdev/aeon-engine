// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 2D geometry primitives, bounding boxes, corner radii, borders, and shadows.

use crate::color::Color;
use bytemuck::{Pod, Zeroable};

/// A 2D point with floating-point coordinates.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Pod, Zeroable)]
pub struct Point {
    /// Horizontal X coordinate in pixels.
    pub x: f32,
    /// Vertical Y coordinate in pixels.
    pub y: f32,
}

impl Point {
    /// Origin point `(0.0, 0.0)`.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Creates a new `Point` with specified `x` and `y` coordinates.
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Converts this point to a 2-element array `[x, y]`.
    #[inline]
    pub const fn to_array(self) -> [f32; 2] {
        [self.x, self.y]
    }

    /// Clamps this point to lie within the specified rectangular boundary.
    ///
    /// Ensures `x` is constrained to `[bounds.x, bounds.right()]` and `y` is constrained to
    /// `[bounds.y, bounds.bottom()]`. If bounds have inverted dimensions (negative width/height),
    /// the intervals are automatically normalized. Coordinates with non-finite values (NaN)
    /// fall back to the normalized minimum bounds.
    ///
    /// # Arguments
    /// * `bounds` - Bounding rectangle constraining this point.
    ///
    /// # Returns
    /// A new clamped [`Point`].
    #[inline]
    #[must_use]
    pub fn clamp_to_rect(self, bounds: Rect) -> Self {
        let min_x = bounds.x.min(bounds.right());
        let max_x = bounds.x.max(bounds.right());
        let min_y = bounds.y.min(bounds.bottom());
        let max_y = bounds.y.max(bounds.bottom());

        let x = if self.x.is_nan() {
            min_x
        } else {
            self.x.clamp(min_x, max_x)
        };

        let y = if self.y.is_nan() {
            min_y
        } else {
            self.y.clamp(min_y, max_y)
        };

        Self::new(x, y)
    }
}

/// A 2D size with width and height in pixels.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Pod, Zeroable)]
pub struct Size {
    /// Width dimension in pixels.
    pub width: f32,
    /// Height dimension in pixels.
    pub height: f32,
}

impl Size {
    /// Zero size `(0.0, 0.0)`.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Creates a new `Size` with given width and height.
    #[inline]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Converts this size to a 2-element array `[width, height]`.
    #[inline]
    pub const fn to_array(self) -> [f32; 2] {
        [self.width, self.height]
    }
}

/// A 2D axis-aligned rectangle defined by position and dimensions.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Pod, Zeroable)]
pub struct Rect {
    /// Horizontal coordinate of the top-left corner.
    pub x: f32,
    /// Vertical coordinate of the top-left corner.
    pub y: f32,
    /// Width dimension in pixels.
    pub width: f32,
    /// Height dimension in pixels.
    pub height: f32,
}

impl Rect {
    /// Empty zero-sized rectangle at the origin.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Creates a new `Rect` from `x`, `y`, `width`, and `height`.
    #[inline]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a new `Rect` from a top-left origin `Point` and a `Size`.
    #[inline]
    pub const fn from_origin_size(origin: Point, size: Size) -> Self {
        Self::new(origin.x, origin.y, size.width, size.height)
    }

    /// Creates a `Rect` from minimum (top-left) and maximum (bottom-right) coordinates.
    #[inline]
    pub fn from_min_max(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self::new(
            min_x,
            min_y,
            (max_x - min_x).max(0.0),
            (max_y - min_y).max(0.0),
        )
    }

    /// Returns the top-left origin point of the rectangle.
    #[inline]
    pub const fn origin(self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Returns the size dimensions of the rectangle.
    #[inline]
    pub const fn size(self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Returns the right boundary X coordinate (`x + width`).
    #[inline]
    pub fn right(self) -> f32 {
        self.x + self.width
    }

    /// Returns the bottom boundary Y coordinate (`y + height`).
    #[inline]
    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    /// Checks if a 2D point is contained within this rectangle's bounds.
    #[inline]
    pub fn contains_point(self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.right()
            && point.y >= self.y
            && point.y <= self.bottom()
    }

    /// Intersects this rectangle with another rectangle, returning the overlapping area.
    pub fn intersect(self, other: Self) -> Self {
        let min_x = self.x.max(other.x);
        let min_y = self.y.max(other.y);
        let max_x = self.right().min(other.right());
        let max_y = self.bottom().min(other.bottom());

        if max_x >= min_x && max_y >= min_y {
            Self::new(min_x, min_y, max_x - min_x, max_y - min_y)
        } else {
            Self::ZERO
        }
    }

    /// Computes the minimal bounding rectangle enclosing both `self` and `other`.
    ///
    /// If either rectangle is [`Self::ZERO`], returns the other rectangle.
    #[inline]
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        if self == Self::ZERO {
            other
        } else if other == Self::ZERO {
            self
        } else {
            Self::from_min_max(
                self.x.min(other.x),
                self.y.min(other.y),
                self.right().max(other.right()),
                self.bottom().max(other.bottom()),
            )
        }
    }

    /// Converts the rectangle to a 4-element array `[x, y, width, height]`.
    #[inline]
    pub const fn to_array(self) -> [f32; 4] {
        [self.x, self.y, self.width, self.height]
    }

    /// Clamps this rectangle to remain completely within the specified boundary bounds.
    ///
    /// Adjusts the origin `(x, y)` so that the rectangle does not extend outside `bounds`.
    /// If `self.width` or `self.height` exceeds `bounds`, dimensions are constrained to
    /// fit within `bounds.width` and `bounds.height`.
    ///
    /// This is the primary algorithm used for clamping floating windows, context menus,
    /// and popups within visible screen or viewport limits.
    ///
    /// # Arguments
    /// * `bounds` - Bounding rectangle (e.g. desktop screen, viewport, or container).
    ///
    /// # Returns
    /// A new [`Rect`] guaranteed to lie within `bounds`.
    #[inline]
    #[must_use]
    pub fn clamp_to_bounds(self, bounds: Rect) -> Self {
        let w = self.width.min(bounds.width.max(0.0)).max(0.0);
        let h = self.height.min(bounds.height.max(0.0)).max(0.0);

        let max_x = (bounds.right() - w).max(bounds.x);
        let max_y = (bounds.bottom() - h).max(bounds.y);

        let x = if self.x.is_nan() {
            bounds.x
        } else {
            self.x.clamp(bounds.x, max_x)
        };

        let y = if self.y.is_nan() {
            bounds.y
        } else {
            self.y.clamp(bounds.y, max_y)
        };

        Self::new(x, y, w, h)
    }

    /// Clamps this rectangle's position `(x, y)` to remain within `bounds` while preserving its dimensions.
    ///
    /// Unlike [`Self::clamp_to_bounds`], this method never alters `self.width` or `self.height`.
    /// If the rectangle is wider or taller than `bounds`, its origin is pinned to `bounds.x` or `bounds.y`.
    ///
    /// # Arguments
    /// * `bounds` - Bounding rectangle constraining the origin.
    ///
    /// # Returns
    /// A new [`Rect`] with clamped position and preserved dimensions.
    #[inline]
    #[must_use]
    pub fn clamp_position_to_bounds(self, bounds: Rect) -> Self {
        let max_x = (bounds.right() - self.width).max(bounds.x);
        let max_y = (bounds.bottom() - self.height).max(bounds.y);

        let x = if self.x.is_nan() {
            bounds.x
        } else {
            self.x.clamp(bounds.x, max_x)
        };

        let y = if self.y.is_nan() {
            bounds.y
        } else {
            self.y.clamp(bounds.y, max_y)
        };

        Self::new(x, y, self.width, self.height)
    }
}

/// Padding or margin insets for the four edges of a box.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Pod, Zeroable)]
pub struct Insets {
    /// Top inset in pixels.
    pub top: f32,
    /// Right inset in pixels.
    pub right: f32,
    /// Bottom inset in pixels.
    pub bottom: f32,
    /// Left inset in pixels.
    pub left: f32,
}

impl Insets {
    /// Zero insets on all four sides.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Creates uniform insets on all four sides.
    #[inline]
    pub const fn all(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    /// Creates insets with symmetric vertical and horizontal values.
    #[inline]
    pub const fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }

    /// Creates insets with individual values for each side.
    #[inline]
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Returns the total horizontal insets (`left + right`).
    #[inline]
    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    /// Returns the total vertical insets (`top + bottom`).
    #[inline]
    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }

    /// Converts insets to a 4-element array `[top, right, bottom, left]`.
    #[inline]
    pub const fn to_array(self) -> [f32; 4] {
        [self.top, self.right, self.bottom, self.left]
    }
}

/// Corner radii for rounded rectangles used in SDF shaders.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Pod, Zeroable)]
pub struct CornerRadii {
    /// Top-left corner radius in pixels.
    pub top_left: f32,
    /// Top-right corner radius in pixels.
    pub top_right: f32,
    /// Bottom-right corner radius in pixels.
    pub bottom_right: f32,
    /// Bottom-left corner radius in pixels.
    pub bottom_left: f32,
}

impl CornerRadii {
    /// Zero corner radii (sharp rectangular corners).
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Creates uniform corner radii for all four corners.
    #[inline]
    pub const fn all(radius: f32) -> Self {
        Self::new(radius, radius, radius, radius)
    }

    /// Creates individual corner radii for each corner.
    #[inline]
    pub const fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    /// Converts radii to a 4-element array `[top_left, top_right, bottom_right, bottom_left]`.
    #[inline]
    pub const fn to_array(self) -> [f32; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
    }
}

/// Border styling definition with edge insets and color.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Border {
    /// Inset thicknesses for top, right, bottom, left borders.
    pub width: Insets,
    /// Border stroke color.
    pub color: Color,
}

impl Border {
    /// No border.
    pub const NONE: Self = Self {
        width: Insets::ZERO,
        color: Color::TRANSPARENT,
    };

    /// Creates a uniform border with specified width and color.
    #[inline]
    pub fn uniform(width: f32, color: Color) -> Self {
        Self {
            width: Insets::all(width),
            color,
        }
    }
}

/// Box shadow definition for GPU SDF drop shadows.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoxShadow {
    /// Shadow displacement offset in pixels (X horizontal, Y vertical).
    pub offset: Point,
    /// Gaussian blur radius in pixels.
    pub blur: f32,
    /// Spread expansion/shrinkage radius in pixels.
    pub spread: f32,
    /// Shadow color (including alpha opacity).
    pub color: Color,
}

impl BoxShadow {
    /// No box shadow.
    pub const NONE: Self = Self {
        offset: Point::ZERO,
        blur: 0.0,
        spread: 0.0,
        color: Color::TRANSPARENT,
    };

    /// Creates a new box shadow definition.
    #[inline]
    pub const fn new(offset_x: f32, offset_y: f32, blur: f32, spread: f32, color: Color) -> Self {
        Self {
            offset: Point::new(offset_x, offset_y),
            blur,
            spread,
            color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_clamp_to_rect_standard() {
        let bounds = Rect::new(100.0, 50.0, 200.0, 150.0); // x: [100, 300], y: [50, 200]

        // Point already inside
        let pt_inside = Point::new(150.0, 100.0);
        assert_eq!(pt_inside.clamp_to_rect(bounds), pt_inside);

        // Point outside left & top
        let pt_top_left = Point::new(50.0, 10.0);
        assert_eq!(pt_top_left.clamp_to_rect(bounds), Point::new(100.0, 50.0));

        // Point outside right & bottom
        let pt_bottom_right = Point::new(350.0, 250.0);
        assert_eq!(
            pt_bottom_right.clamp_to_rect(bounds),
            Point::new(300.0, 200.0)
        );
    }

    #[test]
    fn test_point_clamp_to_rect_inverted_and_nan() {
        // Inverted bounds (right < x)
        let inverted = Rect::new(300.0, 200.0, -200.0, -150.0);
        let pt = Point::new(50.0, 10.0);
        let clamped = pt.clamp_to_rect(inverted);
        assert_eq!(clamped, Point::new(100.0, 50.0));

        // NaN coordinates
        let pt_nan = Point::new(f32::NAN, f32::NAN);
        let bounds = Rect::new(10.0, 20.0, 100.0, 100.0);
        let clamped_nan = pt_nan.clamp_to_rect(bounds);
        assert_eq!(clamped_nan, Point::new(10.0, 20.0));
    }

    #[test]
    fn test_rect_clamp_to_bounds_contained() {
        let screen = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        let window = Rect::new(200.0, 150.0, 400.0, 300.0);

        let clamped = window.clamp_to_bounds(screen);
        assert_eq!(clamped, window, "Contained rect must remain unchanged");
    }

    #[test]
    fn test_rect_clamp_to_bounds_overflow_edges() {
        let screen = Rect::new(0.0, 0.0, 1920.0, 1080.0);

        // Overflow left
        let win_left = Rect::new(-50.0, 100.0, 300.0, 200.0);
        assert_eq!(
            win_left.clamp_to_bounds(screen),
            Rect::new(0.0, 100.0, 300.0, 200.0)
        );

        // Overflow top
        let win_top = Rect::new(100.0, -40.0, 300.0, 200.0);
        assert_eq!(
            win_top.clamp_to_bounds(screen),
            Rect::new(100.0, 0.0, 300.0, 200.0)
        );

        // Overflow right (1800 + 300 = 2100 > 1920 -> shifted to 1920 - 300 = 1620)
        let win_right = Rect::new(1800.0, 100.0, 300.0, 200.0);
        assert_eq!(
            win_right.clamp_to_bounds(screen),
            Rect::new(1620.0, 100.0, 300.0, 200.0)
        );

        // Overflow bottom (950 + 200 = 1150 > 1080 -> shifted to 1080 - 200 = 880)
        let win_bottom = Rect::new(100.0, 950.0, 300.0, 200.0);
        assert_eq!(
            win_bottom.clamp_to_bounds(screen),
            Rect::new(100.0, 880.0, 300.0, 200.0)
        );
    }

    #[test]
    fn test_rect_clamp_to_bounds_larger_than_bounds() {
        let screen = Rect::new(10.0, 20.0, 500.0, 400.0);
        let oversized = Rect::new(0.0, 0.0, 800.0, 600.0);

        let clamped = oversized.clamp_to_bounds(screen);
        assert_eq!(clamped.x, 10.0);
        assert_eq!(clamped.y, 20.0);
        assert_eq!(clamped.width, 500.0, "Width must clamp to screen width");
        assert_eq!(clamped.height, 400.0, "Height must clamp to screen height");
    }

    #[test]
    fn test_rect_clamp_position_to_bounds_preserves_dimensions() {
        let screen = Rect::new(0.0, 0.0, 500.0, 400.0);
        let oversized = Rect::new(-50.0, -50.0, 800.0, 600.0);

        let clamped = oversized.clamp_position_to_bounds(screen);
        assert_eq!(clamped.x, 0.0);
        assert_eq!(clamped.y, 0.0);
        assert_eq!(clamped.width, 800.0, "Dimensions must be preserved");
        assert_eq!(clamped.height, 600.0, "Dimensions must be preserved");
    }

    #[test]
    fn test_rect_clamp_nan_coordinates() {
        let screen = Rect::new(10.0, 20.0, 500.0, 400.0);
        let nan_rect = Rect::new(f32::NAN, f32::NAN, 100.0, 100.0);

        let clamped = nan_rect.clamp_to_bounds(screen);
        assert_eq!(clamped.x, 10.0);
        assert_eq!(clamped.y, 20.0);
        assert_eq!(clamped.width, 100.0);
        assert_eq!(clamped.height, 100.0);
    }
}