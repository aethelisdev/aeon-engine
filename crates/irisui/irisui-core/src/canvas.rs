// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Volatile Immediate Canvas Drawing Primitives (`irisui-core::canvas`)
//!
//! Provides lightweight, POD-like geometry structures for immediate-mode drawing
//! inside high-frequency UI components (telemetry graphs, timelines, oscilloscope curves).

use crate::color::Color;
use crate::geometry::{Point, Rect};

/// Coordinate origin mode governing how a node's [`CanvasPrimitive`] coordinates map to screen space.
///
/// Allows declarative widgets to emit layout-independent procedural geometry centered around or
/// relative to a container whose resolved [`WidgetNode::computed_rect`](crate::WidgetNode::computed_rect)
/// is solved later by the layout engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasOrigin {
    /// Primitive coordinates are already specified in absolute screen pixels.
    #[default]
    Screen,
    /// Primitive coordinates are offsets relative to `node.computed_rect` top-left `(x, y)`.
    NodeTopLeft,
    /// Primitive coordinates are offsets relative to `node.computed_rect` geometric center.
    NodeCenter,
}

impl CanvasOrigin {
    /// Resolves the screen-space translation offset `(dx, dy)` for a node bounded by `node_rect`.
    #[inline]
    #[must_use]
    pub fn resolve_offset(self, node_rect: Rect) -> Point {
        match self {
            Self::Screen => Point::new(0.0, 0.0),
            Self::NodeTopLeft => Point::new(node_rect.x, node_rect.y),
            Self::NodeCenter => Point::new(
                node_rect.x + node_rect.width * 0.5,
                node_rect.y + node_rect.height * 0.5,
            ),
        }
    }
}

/// A volatile drawing primitive rendered directly by the GPU compiler.
///
/// Designed for high-frequency (per-frame) visual components like oscilloscope graphs,
/// audio waveforms, profiler timelines, and cursor indicators where allocating persistent
/// retained nodes in the `UiTree` would cause unnecessary layout thrashing and garbage accumulation.
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasPrimitive {
    /// Solid or rounded rectangle filled with a single color.
    Rect {
        /// Destination rectangle in screen coordinates (or offset per [`CanvasOrigin`]).
        rect: Rect,
        /// Fill color.
        color: Color,
        /// Corner radius in pixels for rounded corners.
        corner_radius: f32,
    },
    /// Rounded rectangle combining fill, uniform border stroke, and corner radius in one quad.
    StyledRect {
        /// Destination rectangle in screen coordinates (or offset per [`CanvasOrigin`]).
        rect: Rect,
        /// Fill color.
        color: Color,
        /// Border stroke color.
        border_color: Color,
        /// Uniform border thickness in pixels.
        border_width: f32,
        /// Corner radius in pixels for rounded corners.
        corner_radius: f32,
    },
    /// Hollow rectangle border outline.
    Border {
        /// Destination rectangle in screen coordinates (or offset per [`CanvasOrigin`]).
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
        /// Starting coordinate in screen pixels (or offset per [`CanvasOrigin`]).
        start: Point,
        /// Ending coordinate in screen pixels (or offset per [`CanvasOrigin`]).
        end: Point,
        /// Line stroke thickness in pixels.
        thickness: f32,
        /// Line stroke color.
        color: Color,
    },
    /// Volatile immediate-mode text string rendered at absolute screen coordinates (or offset per [`CanvasOrigin`]).
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

        let box_rect = Rect::new(100.0, 50.0, 40.0, 20.0);
        assert_eq!(CanvasOrigin::Screen.resolve_offset(box_rect), Point::ZERO);
        assert_eq!(
            CanvasOrigin::NodeTopLeft.resolve_offset(box_rect),
            Point::new(100.0, 50.0)
        );
        assert_eq!(
            CanvasOrigin::NodeCenter.resolve_offset(box_rect),
            Point::new(120.0, 60.0)
        );
    }
}