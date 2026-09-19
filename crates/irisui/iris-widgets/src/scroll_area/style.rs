// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # ScrollArea and ScrollBar Styling (`iris-widgets::scroll_area::style`)
//!
//! Provides color schemes, dimensional thickness, insets, and corner radii for
//! scroll containers and scrollbar indicators in Iris UI.
//!

use iris_core::color::Color;

/// Visual styling configuration for scrollbars within a [`super::area::ScrollAreaBuilder`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollAreaStyle {
    /// Thickness of the scrollbar track and thumb in physical pixels.
    pub thickness: f32,
    /// Distance between the viewport outer boundary and the scrollbar in physical pixels.
    pub inset: f32,
    /// Background color of the scrollbar track.
    pub track_bg: Color,
    /// Corner border radius of the scrollbar track in physical pixels.
    pub track_border_radius: f32,
    /// Background color of the scrollbar thumb in its idle state.
    pub thumb_bg_idle: Color,
    /// Background color of the scrollbar thumb when hovered by the mouse cursor.
    pub thumb_bg_hover: Color,
    /// Background color of the scrollbar thumb while actively being dragged.
    pub thumb_bg_drag: Color,
    /// Corner border radius of the scrollbar thumb in physical pixels.
    pub thumb_border_radius: f32,
    /// Minimum physical length (height for vertical, width for horizontal) of the thumb.
    pub thumb_min_length: f32,
}

impl Default for ScrollAreaStyle {
    fn default() -> Self {
        Self::dark_default()
    }
}

impl ScrollAreaStyle {
    /// Standard dark studio theme for scrollbars.
    #[must_use]
    pub const fn dark_default() -> Self {
        Self {
            thickness: 4.0,
            inset: 3.0,
            track_bg: Color::rgba(0.12, 0.14, 0.20, 0.40),
            track_border_radius: 2.0,
            thumb_bg_idle: Color::rgba(0.30, 0.35, 0.46, 0.70),
            thumb_bg_hover: Color::rgba(0.0, 0.85, 1.0, 0.85),
            thumb_bg_drag: Color::rgba(0.0, 0.95, 1.0, 1.0),
            thumb_border_radius: 2.0,
            thumb_min_length: 24.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_area_style_defaults() {
        let style = ScrollAreaStyle::default();
        assert_eq!(style, ScrollAreaStyle::dark_default());
        assert_eq!(style.thickness, 4.0);
        assert_eq!(style.inset, 3.0);
        assert_eq!(style.thumb_min_length, 24.0);
    }
}