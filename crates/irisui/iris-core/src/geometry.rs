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

    /// Converts the rectangle to a 4-element array `[x, y, width, height]`.
    #[inline]
    pub const fn to_array(self) -> [f32; 4] {
        [self.x, self.y, self.width, self.height]
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