// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Frameless Window Resize Geometry and Tolerance Matrix
//!
//! Provides mathematical boundary detection for frameless and custom-decorated desktop windows.
//! Computes directional resize triggers and maps window perimeter coordinates to type-safe
//! resize directions and hardware cursor representations.

use iris_core::geometry::{Point, Size};
use iris_core::node::WidgetCursor;

/// Default resize border boundary margin thickness in logical points.
///
/// Used for straight edges (left, right, top, bottom) when evaluating frameless window grabbing.
pub const DEFAULT_BORDER_THICKNESS: f32 = 6.0;

/// Default corner interactive grab radius and dimension in logical points.
///
/// Corner interaction zones are given a larger bounding footprint than straight edges
/// to facilitate responsive, ergonomic diagonal resizing.
pub const DEFAULT_CORNER_SIZE: f32 = 12.0;

/// Sub-pixel floating-point tolerance for pointer coordinates at window outer boundaries.
///
/// Accommodates fractional HighDPI scale factor divisions (e.g., scale factor 1.25, 1.5, 1.75)
/// without dropping grab events at exact boundary edges.
const BOUNDARY_EPSILON: f32 = 1e-4;

/// Specifies the interactive resize direction along a frameless window boundary.
///
/// Encapsulates the eight directional vectors (four cardinal edges and four diagonal corners)
/// triggered when a pointer hovers over or drags a frameless window border.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeDirection {
    /// Interactive top horizontal border.
    Top,
    /// Interactive bottom horizontal border.
    Bottom,
    /// Interactive left vertical border.
    Left,
    /// Interactive right vertical border.
    Right,
    /// Interactive top-left diagonal corner.
    TopLeft,
    /// Interactive top-right diagonal corner.
    TopRight,
    /// Interactive bottom-left diagonal corner.
    BottomLeft,
    /// Interactive bottom-right diagonal corner.
    BottomRight,
}

impl ResizeDirection {
    /// Maps this frameless resize direction to the corresponding native [`winit::window::ResizeDirection`].
    ///
    /// # Invariants
    /// - Maps `Top` -> `North`, `Bottom` -> `South`, `Left` -> `West`, `Right` -> `East`.
    /// - Maps diagonal corners to corresponding bidirectional diagonals (`TopLeft` -> `NorthWest`, etc.).
    #[must_use]
    #[inline]
    pub fn to_winit(self) -> winit::window::ResizeDirection {
        match self {
            Self::Top => winit::window::ResizeDirection::North,
            Self::Bottom => winit::window::ResizeDirection::South,
            Self::Left => winit::window::ResizeDirection::West,
            Self::Right => winit::window::ResizeDirection::East,
            Self::TopLeft => winit::window::ResizeDirection::NorthWest,
            Self::TopRight => winit::window::ResizeDirection::NorthEast,
            Self::BottomLeft => winit::window::ResizeDirection::SouthWest,
            Self::BottomRight => winit::window::ResizeDirection::SouthEast,
        }
    }

    /// Maps this frameless resize direction to an Iris UI retained [`WidgetCursor`].
    ///
    /// Cardinal horizontal edges resolve to [`WidgetCursor::EwResize`], vertical edges
    /// to [`WidgetCursor::NsResize`], and diagonals to [`WidgetCursor::NwseResize`] or [`WidgetCursor::NeswResize`].
    #[must_use]
    #[inline]
    pub fn to_cursor(self) -> WidgetCursor {
        match self {
            Self::Top | Self::Bottom => WidgetCursor::NsResize,
            Self::Left | Self::Right => WidgetCursor::EwResize,
            Self::TopLeft | Self::BottomRight => WidgetCursor::NwseResize,
            Self::TopRight | Self::BottomLeft => WidgetCursor::NeswResize,
        }
    }

    /// Maps this frameless resize direction directly to a native Winit [`winit::window::CursorIcon`].
    #[must_use]
    #[inline]
    pub fn to_winit_cursor(self) -> winit::window::CursorIcon {
        match self {
            Self::Top => winit::window::CursorIcon::NResize,
            Self::Bottom => winit::window::CursorIcon::SResize,
            Self::Left => winit::window::CursorIcon::WResize,
            Self::Right => winit::window::CursorIcon::EResize,
            Self::TopLeft => winit::window::CursorIcon::NwResize,
            Self::TopRight => winit::window::CursorIcon::NeResize,
            Self::BottomLeft => winit::window::CursorIcon::SwResize,
            Self::BottomRight => winit::window::CursorIcon::SeResize,
        }
    }
}

impl From<ResizeDirection> for winit::window::ResizeDirection {
    #[inline]
    fn from(direction: ResizeDirection) -> Self {
        direction.to_winit()
    }
}

impl From<ResizeDirection> for WidgetCursor {
    #[inline]
    fn from(direction: ResizeDirection) -> Self {
        direction.to_cursor()
    }
}

impl From<ResizeDirection> for winit::window::CursorIcon {
    #[inline]
    fn from(direction: ResizeDirection) -> Self {
        direction.to_winit_cursor()
    }
}

/// Evaluates whether a logical cursor coordinate falls along the interactive resize margins of a frameless window.
///
/// Returns `Some(ResizeDirection)` if the point is positioned within the corner or edge tolerance zones,
/// or `None` if the point resides inside the safe interactive window interior or outside window bounds.
///
/// # Mathematical Model
/// Given logical coordinates $x \in [0, W]$ and $y \in [0, H]$:
/// - Effective corner grab footprint is computed as `effective_corner = corner.max(border)`. Geometrically,
///   the corner interaction zone can never be smaller than the intersection of two adjacent borders.
/// - Minimum window dimension check `min_dimension = effective_corner * 2.0`. When window dimensions
///   are smaller than `min_dimension`, border/corner margins collide degenerately, returning `None`.
/// - Corner zones are prioritized:
///   - Top-Left: $x \le \text{corner} \land y \le \text{corner}$
///   - Top-Right: $x \ge W - \text{corner} \land y \le \text{corner}$
///   - Bottom-Left: $x \le \text{corner} \land y \ge H - \text{corner}$
///   - Bottom-Right: $x \ge W - \text{corner} \land y \ge H - \text{corner}$
/// - Straight edge zones:
///   - Left: $x \le \text{border}$
///   - Right: $x \ge W - \text{border}$
///   - Top: $y \le \text{border}$
///   - Bottom: $y \ge H - \text{border}$
///
/// # Robustness Invariants and Edge Cases
/// - Corner grab areas take precedence over edge margins to facilitate diagonal grabbing.
/// - If `corner < border`, corner zones naturally inherit at least `border` thickness (intersection of adjacent borders).
/// - If `border <= 0.0 || corner <= 0.0`, returns `None` (resizing completely disabled).
/// - Small windows: when window dimensions are smaller than `min_dimension`, returns `None` to prevent handle collision.
/// - Out-of-bounds rejection with sub-pixel epsilon (`1e-4`) prevents dropping grab events during HighDPI fractional scaling.
/// - If coordinates or dimensions are negative, non-finite (`NaN` or infinity), returns `None`.
/// - Zero heap allocations, pure $O(1)$ arithmetic operations.
///
/// # Arguments
/// * `point` - Cursor position in logical window coordinates.
/// * `size` - Total logical dimensions of the window.
/// * `border` - Interactive margin thickness for top, bottom, left, and right edges.
/// * `corner` - Interactive dimension for corner diagonal resize regions.
#[must_use]
pub fn evaluate_resize_direction(
    point: Point,
    size: Size,
    border: f32,
    corner: f32,
) -> Option<ResizeDirection> {
    if !point.x.is_finite()
        || !point.y.is_finite()
        || !size.width.is_finite()
        || !size.height.is_finite()
        || !border.is_finite()
        || !corner.is_finite()
    {
        return None;
    }

    if border <= 0.0 || corner <= 0.0 {
        return None;
    }

    // Geometrically, corner interaction zone is at least as large as the intersection of two adjacent borders
    let effective_corner = corner.max(border);
    let min_dimension = effective_corner * 2.0;

    if size.width < min_dimension || size.height < min_dimension {
        return None;
    }

    // Sub-pixel HighDPI boundary rejection with epsilon tolerance
    if point.x < -BOUNDARY_EPSILON
        || point.y < -BOUNDARY_EPSILON
        || point.x > size.width + BOUNDARY_EPSILON
        || point.y > size.height + BOUNDARY_EPSILON
    {
        return None;
    }

    let px = point.x.clamp(0.0, size.width);
    let py = point.y.clamp(0.0, size.height);

    let on_left_corner = px <= effective_corner;
    let on_right_corner = px >= size.width - effective_corner;
    let on_top_corner = py <= effective_corner;
    let on_bottom_corner = py >= size.height - effective_corner;

    // 1. Four corners are prioritized over straight edges
    if on_left_corner && on_top_corner {
        return Some(ResizeDirection::TopLeft);
    }
    if on_right_corner && on_top_corner {
        return Some(ResizeDirection::TopRight);
    }
    if on_left_corner && on_bottom_corner {
        return Some(ResizeDirection::BottomLeft);
    }
    if on_right_corner && on_bottom_corner {
        return Some(ResizeDirection::BottomRight);
    }

    // 2. Four cardinal edges
    let on_left_edge = px <= border;
    let on_right_edge = px >= size.width - border;
    let on_top_edge = py <= border;
    let on_bottom_edge = py >= size.height - border;

    if on_left_edge {
        return Some(ResizeDirection::Left);
    }
    if on_right_edge {
        return Some(ResizeDirection::Right);
    }
    if on_top_edge {
        return Some(ResizeDirection::Top);
    }
    if on_bottom_edge {
        return Some(ResizeDirection::Bottom);
    }

    // 3. Inside safe interior area
    None
}