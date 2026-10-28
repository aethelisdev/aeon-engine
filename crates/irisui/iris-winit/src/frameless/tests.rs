// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Frameless Resize Boundary Geometry
//!
//! Verifies four cardinal edges, four diagonal corners, safe interior clearance,
//! HighDPI scaling, disproportionate margins, degenerate small windows,
//! and boundary edge cases (zero/negative sizes, NaN, out of bounds).

use super::geometry::{
    DEFAULT_BORDER_THICKNESS, DEFAULT_CORNER_SIZE, ResizeDirection, evaluate_resize_direction,
};
use iris_core::geometry::{Point, Size};
use iris_core::node::WidgetCursor;
use winit::window::{CursorIcon, ResizeDirection as WinitResizeDirection};

#[test]
fn test_corner_detection_priority() {
    let size = Size::new(800.0, 600.0);
    let border = DEFAULT_BORDER_THICKNESS; // 6.0
    let corner = DEFAULT_CORNER_SIZE; // 12.0

    // TopLeft corner (origin and exact corner boundary)
    assert_eq!(
        evaluate_resize_direction(Point::new(0.0, 0.0), size, border, corner),
        Some(ResizeDirection::TopLeft)
    );
    assert_eq!(
        evaluate_resize_direction(Point::new(corner, corner), size, border, corner),
        Some(ResizeDirection::TopLeft)
    );
    assert_eq!(
        evaluate_resize_direction(Point::new(4.0, 4.0), size, border, corner),
        Some(ResizeDirection::TopLeft)
    );

    // TopRight corner
    assert_eq!(
        evaluate_resize_direction(Point::new(size.width, 0.0), size, border, corner),
        Some(ResizeDirection::TopRight)
    );
    assert_eq!(
        evaluate_resize_direction(
            Point::new(size.width - corner, corner),
            size,
            border,
            corner
        ),
        Some(ResizeDirection::TopRight)
    );
    assert_eq!(
        evaluate_resize_direction(Point::new(size.width - 5.0, 5.0), size, border, corner),
        Some(ResizeDirection::TopRight)
    );

    // BottomLeft corner
    assert_eq!(
        evaluate_resize_direction(Point::new(0.0, size.height), size, border, corner),
        Some(ResizeDirection::BottomLeft)
    );
    assert_eq!(
        evaluate_resize_direction(
            Point::new(corner, size.height - corner),
            size,
            border,
            corner
        ),
        Some(ResizeDirection::BottomLeft)
    );

    // BottomRight corner
    assert_eq!(
        evaluate_resize_direction(Point::new(size.width, size.height), size, border, corner),
        Some(ResizeDirection::BottomRight)
    );
    assert_eq!(
        evaluate_resize_direction(
            Point::new(size.width - corner, size.height - corner),
            size,
            border,
            corner
        ),
        Some(ResizeDirection::BottomRight)
    );
}

#[test]
fn test_cardinal_edge_detection() {
    let size = Size::new(1000.0, 700.0);
    let border = DEFAULT_BORDER_THICKNESS; // 6.0
    let corner = DEFAULT_CORNER_SIZE; // 12.0

    // Left edge (outside corner zone)
    assert_eq!(
        evaluate_resize_direction(Point::new(0.0, 300.0), size, border, corner),
        Some(ResizeDirection::Left)
    );
    assert_eq!(
        evaluate_resize_direction(Point::new(border, 350.0), size, border, corner),
        Some(ResizeDirection::Left)
    );

    // Right edge (outside corner zone)
    assert_eq!(
        evaluate_resize_direction(Point::new(size.width, 300.0), size, border, corner),
        Some(ResizeDirection::Right)
    );
    assert_eq!(
        evaluate_resize_direction(Point::new(size.width - border, 350.0), size, border, corner),
        Some(ResizeDirection::Right)
    );

    // Top edge (outside corner zone)
    assert_eq!(
        evaluate_resize_direction(Point::new(400.0, 0.0), size, border, corner),
        Some(ResizeDirection::Top)
    );
    assert_eq!(
        evaluate_resize_direction(Point::new(500.0, border), size, border, corner),
        Some(ResizeDirection::Top)
    );

    // Bottom edge (outside corner zone)
    assert_eq!(
        evaluate_resize_direction(Point::new(400.0, size.height), size, border, corner),
        Some(ResizeDirection::Bottom)
    );
    assert_eq!(
        evaluate_resize_direction(
            Point::new(500.0, size.height - border),
            size,
            border,
            corner
        ),
        Some(ResizeDirection::Bottom)
    );
}

#[test]
fn test_safe_interior_clearance() {
    let size = Size::new(800.0, 600.0);
    let border = DEFAULT_BORDER_THICKNESS;
    let corner = DEFAULT_CORNER_SIZE;

    // Center of window
    assert_eq!(
        evaluate_resize_direction(Point::new(400.0, 300.0), size, border, corner),
        None
    );

    // Just inside corner margin
    assert_eq!(
        evaluate_resize_direction(Point::new(corner + 0.1, corner + 0.1), size, border, corner),
        None
    );

    // Just inside edge margin
    assert_eq!(
        evaluate_resize_direction(Point::new(border + 0.1, 300.0), size, border, corner),
        None
    );
    assert_eq!(
        evaluate_resize_direction(
            Point::new(size.width - border - 0.1, 300.0),
            size,
            border,
            corner
        ),
        None
    );
    assert_eq!(
        evaluate_resize_direction(Point::new(400.0, border + 0.1), size, border, corner),
        None
    );
    assert_eq!(
        evaluate_resize_direction(
            Point::new(400.0, size.height - border - 0.1),
            size,
            border,
            corner
        ),
        None
    );
}

#[test]
fn test_disproportionate_margins() {
    let size = Size::new(800.0, 600.0);

    // Disproportionate: border (10.0) > corner (4.0)
    // Corner zone must naturally inherit at least border thickness at edge intersections
    let large_border = 10.0;
    let small_corner = 4.0;

    // (6.0, 2.0) is within left border (x <= 10.0) and top border (y <= 10.0) -> TopLeft
    assert_eq!(
        evaluate_resize_direction(Point::new(6.0, 2.0), size, large_border, small_corner),
        Some(ResizeDirection::TopLeft)
    );

    // Edge outside corner zone
    assert_eq!(
        evaluate_resize_direction(Point::new(5.0, 300.0), size, large_border, small_corner),
        Some(ResizeDirection::Left)
    );
}

#[test]
fn test_boundary_edge_cases_and_robustness() {
    let size = Size::new(800.0, 600.0);
    let border = DEFAULT_BORDER_THICKNESS;
    let corner = DEFAULT_CORNER_SIZE;

    // Sub-pixel HighDPI epsilon tolerance: slightly outside boundary by <= 1e-4 remains valid
    assert_eq!(
        evaluate_resize_direction(Point::new(-0.00005, 300.0), size, border, corner),
        Some(ResizeDirection::Left)
    );
    assert_eq!(
        evaluate_resize_direction(
            Point::new(size.width + 0.00005, 300.0),
            size,
            border,
            corner
        ),
        Some(ResizeDirection::Right)
    );

    // Beyond epsilon out of bounds: negative coordinates
    assert_eq!(
        evaluate_resize_direction(Point::new(-1.0, 100.0), size, border, corner),
        None
    );
    assert_eq!(
        evaluate_resize_direction(Point::new(100.0, -0.5), size, border, corner),
        None
    );

    // Beyond epsilon out of bounds: beyond width / height
    assert_eq!(
        evaluate_resize_direction(Point::new(size.width + 0.5, 100.0), size, border, corner),
        None
    );
    assert_eq!(
        evaluate_resize_direction(Point::new(100.0, size.height + 1.0), size, border, corner),
        None
    );

    // Non-finite values (NaN / Inf)
    assert_eq!(
        evaluate_resize_direction(Point::new(f32::NAN, 100.0), size, border, corner),
        None
    );
    assert_eq!(
        evaluate_resize_direction(Point::new(100.0, f32::INFINITY), size, border, corner),
        None
    );
    assert_eq!(
        evaluate_resize_direction(
            Point::new(0.0, 0.0),
            Size::new(f32::NAN, 600.0),
            border,
            corner
        ),
        None
    );

    // Degenerate small windows: smaller than twice the corner margin (min_dimension = 24.0)
    let tiny_size = Size::new(10.0, 10.0);
    assert_eq!(
        evaluate_resize_direction(Point::new(2.0, 2.0), tiny_size, border, corner),
        None
    );

    let degenerate_size = Size::new(20.0, 20.0);
    assert_eq!(
        evaluate_resize_direction(Point::new(2.0, 2.0), degenerate_size, border, corner),
        None
    );

    // Zero or negative window dimensions
    assert_eq!(
        evaluate_resize_direction(Point::new(0.0, 0.0), Size::new(0.0, 600.0), border, corner),
        None
    );
    assert_eq!(
        evaluate_resize_direction(
            Point::new(0.0, 0.0),
            Size::new(800.0, -10.0),
            border,
            corner
        ),
        None
    );

    // Zero or negative border and corner (resizing fully disabled)
    assert_eq!(
        evaluate_resize_direction(Point::new(0.0, 0.0), size, 0.0, corner),
        None
    );
    assert_eq!(
        evaluate_resize_direction(Point::new(0.0, 0.0), size, border, 0.0),
        None
    );
    assert_eq!(
        evaluate_resize_direction(Point::new(0.0, 0.0), size, -5.0, -2.0),
        None
    );
}

#[test]
fn test_resize_direction_conversions() {
    // Top
    assert_eq!(ResizeDirection::Top.to_winit(), WinitResizeDirection::North);
    assert_eq!(ResizeDirection::Top.to_cursor(), WidgetCursor::NsResize);
    assert_eq!(ResizeDirection::Top.to_winit_cursor(), CursorIcon::NResize);
    assert_eq!(
        WinitResizeDirection::from(ResizeDirection::Top),
        WinitResizeDirection::North
    );
    assert_eq!(
        WidgetCursor::from(ResizeDirection::Top),
        WidgetCursor::NsResize
    );
    assert_eq!(CursorIcon::from(ResizeDirection::Top), CursorIcon::NResize);

    // Bottom
    assert_eq!(
        ResizeDirection::Bottom.to_winit(),
        WinitResizeDirection::South
    );
    assert_eq!(ResizeDirection::Bottom.to_cursor(), WidgetCursor::NsResize);
    assert_eq!(
        ResizeDirection::Bottom.to_winit_cursor(),
        CursorIcon::SResize
    );

    // Left
    assert_eq!(ResizeDirection::Left.to_winit(), WinitResizeDirection::West);
    assert_eq!(ResizeDirection::Left.to_cursor(), WidgetCursor::EwResize);
    assert_eq!(ResizeDirection::Left.to_winit_cursor(), CursorIcon::WResize);

    // Right
    assert_eq!(
        ResizeDirection::Right.to_winit(),
        WinitResizeDirection::East
    );
    assert_eq!(ResizeDirection::Right.to_cursor(), WidgetCursor::EwResize);
    assert_eq!(
        ResizeDirection::Right.to_winit_cursor(),
        CursorIcon::EResize
    );

    // TopLeft
    assert_eq!(
        ResizeDirection::TopLeft.to_winit(),
        WinitResizeDirection::NorthWest
    );
    assert_eq!(
        ResizeDirection::TopLeft.to_cursor(),
        WidgetCursor::NwseResize
    );
    assert_eq!(
        ResizeDirection::TopLeft.to_winit_cursor(),
        CursorIcon::NwResize
    );

    // TopRight
    assert_eq!(
        ResizeDirection::TopRight.to_winit(),
        WinitResizeDirection::NorthEast
    );
    assert_eq!(
        ResizeDirection::TopRight.to_cursor(),
        WidgetCursor::NeswResize
    );
    assert_eq!(
        ResizeDirection::TopRight.to_winit_cursor(),
        CursorIcon::NeResize
    );

    // BottomLeft
    assert_eq!(
        ResizeDirection::BottomLeft.to_winit(),
        WinitResizeDirection::SouthWest
    );
    assert_eq!(
        ResizeDirection::BottomLeft.to_cursor(),
        WidgetCursor::NeswResize
    );
    assert_eq!(
        ResizeDirection::BottomLeft.to_winit_cursor(),
        CursorIcon::SwResize
    );

    // BottomRight
    assert_eq!(
        ResizeDirection::BottomRight.to_winit(),
        WinitResizeDirection::SouthEast
    );
    assert_eq!(
        ResizeDirection::BottomRight.to_cursor(),
        WidgetCursor::NwseResize
    );
    assert_eq!(
        ResizeDirection::BottomRight.to_winit_cursor(),
        CursorIcon::SeResize
    );
}