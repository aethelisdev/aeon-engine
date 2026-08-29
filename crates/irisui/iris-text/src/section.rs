// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Text section and measurement descriptors for typography rendering.

use iris_core::{Color, Rect, TextAlign};
use std::borrow::Cow;

/// A descriptor containing text content, styling, color, and bounding constraints.
#[derive(Debug, Clone, PartialEq)]
pub struct TextSection<'a> {
    /// The string content to render.
    pub text: Cow<'a, str>,
    /// Font size in pixels.
    pub font_size: f32,
    /// Line height in pixels.
    pub line_height: f32,
    /// Foreground text color.
    pub color: Color,
    /// Horizontal text alignment.
    pub align: TextAlign,
    /// Bounding rectangle in screen-space coordinates where text should be placed.
    pub bounds: Rect,
}

impl<'a> TextSection<'a> {
    /// Creates a new `TextSection` with default typography metrics.
    #[inline]
    pub fn new(text: impl Into<Cow<'a, str>>, bounds: Rect) -> Self {
        Self {
            text: text.into(),
            font_size: 14.0,
            line_height: 18.0,
            color: Color::WHITE,
            align: TextAlign::Left,
            bounds,
        }
    }

    /// Sets the font size and line height.
    #[inline]
    pub fn with_font_size(mut self, size: f32, line_height: f32) -> Self {
        self.font_size = size;
        self.line_height = line_height;
        self
    }

    /// Sets the text color.
    #[inline]
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the text alignment.
    #[inline]
    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
}