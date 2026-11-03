// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Ergonomic Docking Splitter Divider Hit-Testing & Tolerance Engine
//!
//! Provides the single source of truth (SSOT) constants, hit testing, and cursor
//! determination for docking splitters. Expands the physical 2.0px splitter divider line
//! with an ergonomic interaction tolerance zone of $\pm 4.0\text{px}$ (yielding an
//! authoritative 10.0px total interactive hit zone), preventing pixel-hunting when
//! resizing panes across horizontal and vertical binary partitions.

use crate::tree::{DockNodeId, SplitDirection};
use iris_core::geometry::{Point, Rect};
use iris_core::node::WidgetCursor;

/// Default visual line thickness of docking splitter dividers in logical pixels.
///
/// Keeps divider lines clean, minimal, and unobtrusive while [`DEFAULT_SPLITTER_HIT_MARGIN`]
/// expands interactive grabbing ergonomically.
pub const DEFAULT_SPLITTER_THICKNESS: f32 = 2.0;

/// Default invisible interaction hit margin in logical pixels added symmetrically to each side of a splitter.
///
/// Adding $4.0\text{px}$ to either side of a $2.0\text{px}$ splitter yields a $10.0\text{px}$ total hit zone.
pub const DEFAULT_SPLITTER_HIT_MARGIN: f32 = 4.0;

/// Total interactive hit zone thickness across the active divider axis in logical pixels.
///
/// Equal to `DEFAULT_SPLITTER_THICKNESS + 2.0 * DEFAULT_SPLITTER_HIT_MARGIN` ($2.0 + 2 \times 4.0 = 10.0\text{px}$).
pub const DEFAULT_SPLITTER_TOTAL_HIT_ZONE: f32 =
    DEFAULT_SPLITTER_THICKNESS + (2.0 * DEFAULT_SPLITTER_HIT_MARGIN);

/// Native Iris splitter hit target linked to a stable split node.
///
/// Encapsulates the visual geometry, split orientation, and ergonomic hit testing
/// with interactive margins ($\pm 4.0\text{px}$) for docking dividers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockSplitterTarget {
    /// Split node in the docking tree that owns the divider.
    pub node: DockNodeId,
    /// Partition axis along which the divider moves (`Horizontal` or `Vertical`).
    pub direction: SplitDirection,
    /// Visual bounding rectangle of the divider line.
    pub rect: Rect,
    /// Total width or height of the parent split container.
    pub total_dimension: f32,
}

impl DockSplitterTarget {
    /// Ergonomic hit-test margin in logical pixels applied to thin splitters ($\pm 4.0\text{px}$).
    pub const HIT_MARGIN: f32 = DEFAULT_SPLITTER_HIT_MARGIN;

    /// Creates a new splitter hit target with the given split node, axis, geometry, and parent extent.
    ///
    /// # Arguments
    /// * `node` - ID of the owning split node in the docking tree.
    /// * `direction` - Axis of division (`Horizontal` or `Vertical`).
    /// * `rect` - Bounding rectangle of the visible splitter line.
    /// * `total_dimension` - Total parent container extent along the division axis.
    #[inline]
    #[must_use]
    pub const fn new(
        node: DockNodeId,
        direction: SplitDirection,
        rect: Rect,
        total_dimension: f32,
    ) -> Self {
        Self {
            node,
            direction,
            rect,
            total_dimension,
        }
    }

    /// Checks whether the point is contained within this splitter's interactive hit zone
    /// using the default ergonomic margin ([`DEFAULT_SPLITTER_HIT_MARGIN`] = 4.0px).
    ///
    /// # Arguments
    /// * `point` - Coordinates of the pointer in logical screen space.
    #[inline]
    #[must_use]
    pub fn contains_point(&self, point: Point) -> bool {
        hit_splitter(
            self.rect,
            self.direction,
            point,
            DEFAULT_SPLITTER_HIT_MARGIN,
        )
    }

    /// Checks whether the point is contained within this splitter's interactive hit zone
    /// with an explicitly specified hit margin in logical pixels.
    ///
    /// # Arguments
    /// * `point` - Coordinates of the pointer in logical screen space.
    /// * `margin` - Half-margin in logical pixels added symmetrically to the divider axis.
    #[inline]
    #[must_use]
    pub fn contains_point_with_margin(&self, point: Point, margin: f32) -> bool {
        hit_splitter(self.rect, self.direction, point, margin)
    }

    /// Computes the expanded interactive hit rectangle taking into account the default margin (4.0px).
    ///
    /// For a horizontal splitter (vertical divider bar), expands the width by $\pm 4.0\text{px}$.
    /// For a vertical splitter (horizontal divider bar), expands the height by $\pm 4.0\text{px}$.
    #[inline]
    #[must_use]
    pub fn hit_bounds(&self) -> Rect {
        self.hit_bounds_with_margin(DEFAULT_SPLITTER_HIT_MARGIN)
    }

    /// Computes the expanded interactive hit rectangle taking into account an explicit margin.
    ///
    /// # Arguments
    /// * `margin` - Half-margin in logical pixels to expand along the divider's movement axis.
    #[inline]
    #[must_use]
    pub fn hit_bounds_with_margin(&self, margin: f32) -> Rect {
        match self.direction {
            SplitDirection::Horizontal => Rect::new(
                self.rect.x - margin,
                self.rect.y,
                self.rect.width + 2.0 * margin,
                self.rect.height,
            ),
            SplitDirection::Vertical => Rect::new(
                self.rect.x,
                self.rect.y - margin,
                self.rect.width,
                self.rect.height + 2.0 * margin,
            ),
        }
    }

    /// Returns the appropriate resize cursor for this splitter divider.
    ///
    /// - `SplitDirection::Horizontal`: returns [`WidgetCursor::ColResize`] (resizes left/right columns).
    /// - `SplitDirection::Vertical`: returns [`WidgetCursor::RowResize`] (resizes top/bottom rows).
    #[inline]
    #[must_use]
    pub const fn hover_cursor(&self) -> WidgetCursor {
        match self.direction {
            SplitDirection::Horizontal => WidgetCursor::ColResize,
            SplitDirection::Vertical => WidgetCursor::RowResize,
        }
    }
}

/// Evaluates whether a screen-space point falls within a splitter divider's interactive hit zone.
///
/// Expands the splitter rectangle along its active split axis by `margin` logical pixels
/// on both sides, while clamping the perpendicular bounds to the divider's physical length.
///
/// - `SplitDirection::Horizontal`: divider moves left/right. Hit zone is `[x - margin, x + width + margin]`.
/// - `SplitDirection::Vertical`: divider moves up/down. Hit zone is `[y - margin, y + height + margin]`.
#[inline]
#[must_use]
pub fn hit_splitter(rect: Rect, dir: SplitDirection, p: Point, margin: f32) -> bool {
    match dir {
        SplitDirection::Horizontal => {
            p.x >= rect.x - margin
                && p.x <= rect.x + rect.width + margin
                && p.y >= rect.y
                && p.y <= rect.y + rect.height
        }
        SplitDirection::Vertical => {
            p.x >= rect.x
                && p.x <= rect.x + rect.width
                && p.y >= rect.y - margin
                && p.y <= rect.y + rect.height + margin
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::handle_splitter_event;
    use crate::state::DockState;
    use crate::tree::DockTree;
    use iris_core::event::{MouseButton, UiEvent};

    #[test]
    fn test_splitter_constants_and_geometry() {
        assert_eq!(DEFAULT_SPLITTER_THICKNESS, 2.0);
        assert_eq!(DEFAULT_SPLITTER_HIT_MARGIN, 4.0);
        assert_eq!(
            DEFAULT_SPLITTER_TOTAL_HIT_ZONE,
            DEFAULT_SPLITTER_THICKNESS + 2.0 * DEFAULT_SPLITTER_HIT_MARGIN
        );
        assert_eq!(DEFAULT_SPLITTER_TOTAL_HIT_ZONE, 10.0);
        assert_eq!(DockSplitterTarget::HIT_MARGIN, 4.0);
    }

    #[test]
    fn test_horizontal_splitter_hit_zone_and_boundaries() {
        // Horizontal split divider bar: visual line is 2.0px wide at x in [200.0, 202.0], height 500.0 (y in [50.0, 550.0])
        let target = DockSplitterTarget::new(
            DockNodeId::default(),
            SplitDirection::Horizontal,
            Rect::new(200.0, 50.0, 2.0, 500.0),
            1000.0,
        );

        let bounds = target.hit_bounds();
        assert_eq!(bounds.x, 196.0);
        assert_eq!(bounds.y, 50.0);
        assert_eq!(bounds.width, 10.0);
        assert_eq!(bounds.height, 500.0);

        // 1. Center of the 2px bar
        assert!(target.contains_point(Point::new(201.0, 300.0)));

        // 2. Exact inner edges of the 2px visual bar
        assert!(target.contains_point(Point::new(200.0, 300.0)));
        assert!(target.contains_point(Point::new(202.0, 300.0)));

        // 3. Exact outer tolerance boundaries (+/- 4.0px hit margin => x in [196.0, 206.0])
        assert!(target.contains_point(Point::new(196.0, 300.0)));
        assert!(target.contains_point(Point::new(206.0, 300.0)));

        // 4. Just outside the 4.0px tolerance boundary
        assert!(!target.contains_point(Point::new(195.9, 300.0)));
        assert!(!target.contains_point(Point::new(206.1, 300.0)));

        // 5. Vertical range limits [50.0, 550.0]
        assert!(target.contains_point(Point::new(201.0, 50.0)));
        assert!(target.contains_point(Point::new(201.0, 550.0)));
        assert!(!target.contains_point(Point::new(201.0, 49.9)));
        assert!(!target.contains_point(Point::new(201.0, 550.1)));
    }

    #[test]
    fn test_vertical_splitter_hit_zone_and_boundaries() {
        // Vertical split divider bar: visual line is 2.0px high at y in [150.0, 152.0], width 600.0 (x in [0.0, 600.0])
        let target = DockSplitterTarget::new(
            DockNodeId::default(),
            SplitDirection::Vertical,
            Rect::new(0.0, 150.0, 600.0, 2.0),
            800.0,
        );

        let bounds = target.hit_bounds();
        assert_eq!(bounds.x, 0.0);
        assert_eq!(bounds.y, 146.0);
        assert_eq!(bounds.width, 600.0);
        assert_eq!(bounds.height, 10.0);

        // 1. Center of the 2px bar
        assert!(target.contains_point(Point::new(300.0, 151.0)));

        // 2. Exact inner edges
        assert!(target.contains_point(Point::new(300.0, 150.0)));
        assert!(target.contains_point(Point::new(300.0, 152.0)));

        // 3. Exact outer tolerance boundaries (+/- 4.0px hit margin => y in [146.0, 156.0])
        assert!(target.contains_point(Point::new(300.0, 146.0)));
        assert!(target.contains_point(Point::new(300.0, 156.0)));

        // 4. Just outside the 4.0px tolerance boundary
        assert!(!target.contains_point(Point::new(300.0, 145.9)));
        assert!(!target.contains_point(Point::new(300.0, 156.1)));

        // 5. Horizontal range limits [0.0, 600.0]
        assert!(target.contains_point(Point::new(0.0, 151.0)));
        assert!(target.contains_point(Point::new(600.0, 151.0)));
        assert!(!target.contains_point(Point::new(-0.1, 151.0)));
        assert!(!target.contains_point(Point::new(600.1, 151.0)));
    }

    #[test]
    fn test_splitter_cursor_inference() {
        let h_target = DockSplitterTarget::new(
            DockNodeId::default(),
            SplitDirection::Horizontal,
            Rect::new(100.0, 0.0, 2.0, 400.0),
            500.0,
        );
        assert_eq!(h_target.hover_cursor(), WidgetCursor::ColResize);

        let v_target = DockSplitterTarget::new(
            DockNodeId::default(),
            SplitDirection::Vertical,
            Rect::new(0.0, 100.0, 400.0, 2.0),
            500.0,
        );
        assert_eq!(v_target.hover_cursor(), WidgetCursor::RowResize);
    }

    #[test]
    fn test_mouse_down_drag_initiation_from_margin_boundaries() {
        let mut tree = DockTree::<()>::new();
        let split_container = tree.create_leaf(vec![]);
        tree.set_root(split_container);
        let _ = tree
            .split(split_container, SplitDirection::Horizontal, 0.5, vec![])
            .expect("Split succeeds");

        let target = DockSplitterTarget::new(
            split_container,
            SplitDirection::Horizontal,
            Rect::new(200.0, 0.0, 2.0, 600.0),
            1000.0,
        );
        let splitters = [target];

        // 1. Mouse down at exact left tolerance edge (x = 196.0, which is 4.0px left of 200.0)
        let mut dock_state = DockState::new(tree.clone());
        let press_left = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(196.0, 250.0),
        };
        let resp_left = handle_splitter_event(&mut dock_state, &press_left, &splitters);
        assert!(
            resp_left.consumed,
            "Press at left -4px edge must be consumed"
        );
        assert_eq!(resp_left.cursor, Some(WidgetCursor::ColResize));
        assert!(
            dock_state.active_splitter.is_some(),
            "Drag session must start at left -4px margin"
        );

        // 2. Mouse down at exact right tolerance edge (x = 206.0, which is 4.0px right of 202.0)
        let mut dock_state = DockState::new(tree.clone());
        let press_right = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(206.0, 250.0),
        };
        let resp_right = handle_splitter_event(&mut dock_state, &press_right, &splitters);
        assert!(
            resp_right.consumed,
            "Press at right +4px edge must be consumed"
        );
        assert_eq!(resp_right.cursor, Some(WidgetCursor::ColResize));
        assert!(
            dock_state.active_splitter.is_some(),
            "Drag session must start at right +4px margin"
        );

        // 3. Mouse down 4.1px away (x = 195.9) must NOT initiate drag
        let mut dock_state = DockState::new(tree.clone());
        let press_miss = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(195.9, 250.0),
        };
        let resp_miss = handle_splitter_event(&mut dock_state, &press_miss, &splitters);
        assert!(
            !resp_miss.consumed,
            "Press beyond hit margin must not be consumed"
        );
        assert_eq!(resp_miss.cursor, None);
        assert!(
            dock_state.active_splitter.is_none(),
            "Drag session must not start beyond hit margin"
        );

        // 4. Vertical splitter mouse down at top edge (y = 146.0, 4px above 150.0)
        let v_target = DockSplitterTarget::new(
            split_container,
            SplitDirection::Vertical,
            Rect::new(0.0, 150.0, 800.0, 2.0),
            600.0,
        );
        let v_splitters = [v_target];
        let mut v_dock_state = DockState::new(tree);
        let press_v_top = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(400.0, 146.0),
        };
        let resp_v_top = handle_splitter_event(&mut v_dock_state, &press_v_top, &v_splitters);
        assert!(resp_v_top.consumed);
        assert_eq!(resp_v_top.cursor, Some(WidgetCursor::RowResize));
        assert!(v_dock_state.active_splitter.is_some());
    }
}