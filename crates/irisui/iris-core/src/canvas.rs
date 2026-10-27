// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Volatile Immediate Canvas Drawing Primitives (`iris-core::canvas`)
//!
//! Provides lightweight, POD-like geometry structures for immediate-mode drawing
//! inside high-frequency UI components (telemetry graphs, timelines, oscilloscope curves).

use crate::color::Color;
use crate::geometry::{Point, Rect};

/// A volatile drawing primitive rendered directly by the GPU compiler.
///
/// Designed for high-frequency (per-frame) visual components like oscilloscope graphs,
/// audio waveforms, profiler timelines, and cursor indicators where allocating persistent
/// retained nodes in the `UiTree` would cause unnecessary layout thrashing and garbage accumulation.
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasPrimitive {
    /// Solid or rounded rectangle filled with a single color.
    Rect {
        /// Destination rectangle in screen coordinates.
        rect: Rect,
        /// Fill color.
        color: Color,
        /// Corner radius in pixels for rounded corners.
        corner_radius: f32,
    },
    /// Hollow rectangle border outline.
    Border {
        /// Destination rectangle in screen coordinates.
        rect: Rect,
        /// Border stroke thickness in pixels.
        border_width: f32,
        /// Border stroke color.
        color: Color,
        /// Corner radius in pixels for rounded corners.
        corner_radius: f32,
    },
    /// Straight line segment between two points with configurable thickness.
    Line {
        /// Starting coordinate in screen pixels.
        start: Point,
        /// Ending coordinate in screen pixels.
        end: Point,
        /// Line stroke thickness in pixels.
        thickness: f32,
        /// Line stroke color.
        color: Color,
    },
    /// Volatile immediate-mode text string rendered at absolute screen coordinates.
    Text {
        /// Starting top-left origin coordinate in screen pixels.
        pos: Point,
        /// String text content.
        text: String,
        /// Font size in pixels.
        font_size: f32,
        /// Text fill color.
        color: Color,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_primitive_variants() {
        let r = CanvasPrimitive::Rect {
            rect: Rect::new(10.0, 20.0, 100.0, 50.0),
            color: Color::WHITE,
            corner_radius: 4.0,
        };
        let b = CanvasPrimitive::Border {
            rect: Rect::new(10.0, 20.0, 100.0, 50.0),
            border_width: 1.0,
            color: Color::BLACK,
            corner_radius: 4.0,
        };
        let l = CanvasPrimitive::Line {
            start: Point::new(0.0, 0.0),
            end: Point::new(100.0, 100.0),
            thickness: 2.0,
            color: Color::RED,
        };
        let t = CanvasPrimitive::Text {
            pos: Point::new(10.0, 10.0),
            text: "Telemetry".to_string(),
            font_size: 12.0,
            color: Color::WHITE,
        };

        assert_ne!(r, b);
        assert_ne!(r, l);
        assert_ne!(r, t);
    }
}