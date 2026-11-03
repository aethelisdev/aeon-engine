// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Dock Splitter Drag & Double-Click Reset Controller
//!
//! Manages continuous splitter drag calculations, division ratio clamping within
//! strictly enforced industry-standard boundaries (`[0.10, 0.90]`), automatic
//! structural revision incrementation on the underlying [`DockTree`], and ergonomic
//! double-click reset to an even 50/50 partition balance.

use crate::area::splitter::DockSplitterTarget;
use crate::state::DockState;
use crate::tree::{DockNodeId, SplitDirection};
use iris_core::geometry::Point;
use std::time::{Duration, Instant};

/// Minimum allowable division ratio for a split container node ($10\%$).
///
/// Prevents pane collapse and visual clipping of tabs or toolbar buttons.
pub const MIN_SPLIT_RATIO: f32 = 0.10;

/// Maximum allowable division ratio for a split container node ($90\%$).
///
/// Prevents child panes from starving the complementary child container.
pub const MAX_SPLIT_RATIO: f32 = 0.90;

/// Default division ratio resetting partitioned panes to an even 50/50 balance.
pub const DEFAULT_SPLIT_RATIO: f32 = 0.50;

/// Maximum duration elapsed between two successive clicks to register as a double click.
pub const DOUBLE_CLICK_TIME_THRESHOLD: Duration = Duration::from_millis(300);

/// Maximum spatial distance in logical pixels between two successive clicks to register as a double click.
pub const DOUBLE_CLICK_DISTANCE_THRESHOLD: f32 = 4.0;

/// Outcome of processing a pointer press on a splitter divider target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitterClickResult {
    /// Single click initiated an active splitter drag operation.
    DragStarted,
    /// Second click within temporal and spatial threshold triggered an autonomous reset to 50/50 balance.
    ResetToHalf,
}

/// Clamps a split division ratio strictly within the engine boundaries [`MIN_SPLIT_RATIO`] and [`MAX_SPLIT_RATIO`].
#[inline]
#[must_use]
pub fn clamp_split_ratio(ratio: f32) -> f32 {
    ratio.clamp(MIN_SPLIT_RATIO, MAX_SPLIT_RATIO)
}

/// Calculates the new division ratio for an active drag operation while respecting both
/// engine ratio limits (`[0.10, 0.90]`) and the minimum pane pixel size.
///
/// # Arguments
/// * `start_ratio` - Initial division ratio when the drag session started.
/// * `delta` - Pointer displacement in logical pixels along the partition axis.
/// * `total_dimension` - Total parent container extent in logical pixels.
/// * `min_pane_size` - Minimum allowable pane dimension in logical pixels.
#[must_use]
pub fn compute_drag_split_ratio(
    start_ratio: f32,
    delta: f32,
    total_dimension: f32,
    min_pane_size: f32,
) -> f32 {
    let extent = total_dimension.max(1.0);
    let ratio_delta = delta / extent;

    let min_ratio_from_size = (min_pane_size / extent).clamp(MIN_SPLIT_RATIO, 0.45);
    let max_ratio_from_size = (1.0 - min_ratio_from_size)
        .max(min_ratio_from_size)
        .min(MAX_SPLIT_RATIO);

    (start_ratio + ratio_delta).clamp(min_ratio_from_size, max_ratio_from_size)
}

/// Checks whether a click satisfies the temporal and spatial constraints of a double click
/// on the same docking splitter node.
///
/// # Arguments
/// * `last_time` - Timestamp of the previous click.
/// * `last_pos` - Screen coordinates of the previous click.
/// * `last_node` - Split node key targeted in the previous click.
/// * `now` - Timestamp of the current incoming click.
/// * `pos` - Screen coordinates of the current click.
/// * `node` - Split node key targeted in the current click.
#[inline]
#[must_use]
pub fn is_splitter_double_click(
    last_time: Instant,
    last_pos: Point,
    last_node: DockNodeId,
    now: Instant,
    pos: Point,
    node: DockNodeId,
) -> bool {
    if node != last_node || now < last_time {
        return false;
    }

    let elapsed = now.duration_since(last_time);
    if elapsed > DOUBLE_CLICK_TIME_THRESHOLD {
        return false;
    }

    let dx = pos.x - last_pos.x;
    let dy = pos.y - last_pos.y;
    dx.hypot(dy) < DOUBLE_CLICK_DISTANCE_THRESHOLD
}

/// Processes a mouse press event against a target docking splitter, evaluating double-click reset
/// or initiating continuous pointer-capture dragging.
///
/// If a double click is detected, the partition ratio is reset back to [`DEFAULT_SPLIT_RATIO`] (`0.50`),
/// the tree revision counter is incremented, and active dragging is terminated.
/// Otherwise, a new drag session is started and the click timestamp is recorded.
///
/// # Arguments
/// * `dock_state` - Mutable reference to the docking state controller.
/// * `target` - Clicked splitter divider target.
/// * `point` - Coordinates of the pointer in logical screen pixels.
/// * `now` - Timestamp of the incoming click event.
pub fn handle_splitter_click_with_time<T>(
    dock_state: &mut DockState<T>,
    target: &DockSplitterTarget,
    point: Point,
    now: Instant,
) -> SplitterClickResult {
    let is_double = if let Some((last_time, last_pos, last_node)) = dock_state.last_splitter_click {
        is_splitter_double_click(last_time, last_pos, last_node, now, point, target.node)
    } else {
        false
    };

    if is_double {
        dock_state.last_splitter_click = None;
        dock_state.end_splitter_drag();
        let _ = dock_state.reset_splitter(target.node);
        SplitterClickResult::ResetToHalf
    } else {
        dock_state.last_splitter_click = Some((now, point, target.node));

        let start_coord = match target.direction {
            SplitDirection::Horizontal => point.x,
            SplitDirection::Vertical => point.y,
        };

        dock_state.start_splitter_drag(
            target.node,
            target.direction,
            start_coord,
            target.total_dimension,
        );

        SplitterClickResult::DragStarted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::handle_splitter_event;
    use crate::tree::{DockNode, DockTree};
    use iris_core::event::{MouseButton, UiEvent};
    use iris_core::geometry::Rect;

    #[test]
    fn test_ratio_constants_and_clamping() {
        assert_eq!(MIN_SPLIT_RATIO, 0.10);
        assert_eq!(MAX_SPLIT_RATIO, 0.90);
        assert_eq!(DEFAULT_SPLIT_RATIO, 0.50);

        assert_eq!(clamp_split_ratio(0.01), 0.10);
        assert_eq!(clamp_split_ratio(0.50), 0.50);
        assert_eq!(clamp_split_ratio(0.99), 0.90);
    }

    #[test]
    fn test_compute_drag_split_ratio_bounds() {
        // start_ratio = 0.50, dimension = 1000.0, min_pane_size = 100.0
        // min_ratio should be 0.10, max_ratio should be 0.90
        let r_mid = compute_drag_split_ratio(0.50, 0.0, 1000.0, 100.0);
        assert_eq!(r_mid, 0.50);

        // Huge negative displacement -> clamped to 0.10
        let r_min = compute_drag_split_ratio(0.50, -800.0, 1000.0, 100.0);
        assert_eq!(r_min, 0.10);

        // Huge positive displacement -> clamped to 0.90
        let r_max = compute_drag_split_ratio(0.50, 800.0, 1000.0, 100.0);
        assert_eq!(r_max, 0.90);

        // Moderate movement: +100px on 1000px container => +0.10 => 0.60
        let r_mod = compute_drag_split_ratio(0.50, 100.0, 1000.0, 100.0);
        assert!((r_mod - 0.60).abs() < f32::EPSILON);
    }

    #[test]
    fn test_splitter_drag_updates_ratio_and_bumps_revision() {
        let mut tree = DockTree::<()>::new();
        let center = tree.create_leaf(vec![]);
        tree.set_root(center);
        let _ = tree
            .split_ordered(center, SplitDirection::Horizontal, 0.5, vec![], true)
            .expect("Split succeeds");

        let rev_initial = tree.revision();
        let mut dock_state = DockState::new(tree);
        dock_state.min_pane_size = 50.0;

        // Start drag on split node center
        dock_state.start_splitter_drag(center, SplitDirection::Horizontal, 500.0, 1000.0);
        assert!(dock_state.active_splitter.is_some());

        // Update drag to cursor 650.0 (delta = +150.0 -> ratio delta = +0.15 -> new ratio = 0.65)
        dock_state.update_splitter_drag(650.0);

        if let Some(DockNode::Split { ratio, .. }) = dock_state.tree.get(center) {
            assert!((*ratio - 0.65).abs() < 1e-4);
            assert!(
                dock_state.tree.revision() > rev_initial,
                "Tree revision must monotonically increment on splitter drag ratio update"
            );
        } else {
            panic!("Expected split node");
        }

        let rev_after_move1 = dock_state.tree.revision();

        // Drag all the way past 1000px -> clamped to MAX_SPLIT_RATIO (0.90)
        dock_state.update_splitter_drag(1200.0);
        if let Some(DockNode::Split { ratio, .. }) = dock_state.tree.get(center) {
            assert_eq!(*ratio, 0.90);
            assert!(
                dock_state.tree.revision() > rev_after_move1,
                "Tree revision must increment on boundary clamp ratio update"
            );
        }

        // Drag all the way to negative -> clamped to MIN_SPLIT_RATIO (0.10)
        let rev_before_negative = dock_state.tree.revision();
        dock_state.update_splitter_drag(-200.0);
        if let Some(DockNode::Split { ratio, .. }) = dock_state.tree.get(center) {
            assert_eq!(*ratio, 0.10);
            assert!(
                dock_state.tree.revision() > rev_before_negative,
                "Tree revision must increment when clamped to MIN_SPLIT_RATIO"
            );
        }
    }

    #[test]
    fn test_splitter_double_click_resets_to_half_and_bumps_revision() {
        let mut tree = DockTree::<()>::new();
        let center = tree.create_leaf(vec![]);
        tree.set_root(center);
        let _ = tree
            .split_ordered(center, SplitDirection::Horizontal, 0.5, vec![], true)
            .expect("Split succeeds");

        let mut dock_state = DockState::new(tree);

        // Manually move splitter to an uneven partition ratio (0.80)
        let _ = dock_state.tree.set_split_ratio(center, 0.80);
        if let Some(DockNode::Split { ratio, .. }) = dock_state.tree.get(center) {
            assert_eq!(*ratio, 0.80);
        }

        let target = DockSplitterTarget::new(
            center,
            SplitDirection::Horizontal,
            Rect::new(800.0, 0.0, 2.0, 600.0),
            1000.0,
        );

        let t0 = Instant::now();
        let click_point = Point::new(801.0, 300.0);

        // 1. First click: initiates drag
        let res1 = handle_splitter_click_with_time(&mut dock_state, &target, click_point, t0);
        assert_eq!(res1, SplitterClickResult::DragStarted);
        assert!(dock_state.active_splitter.is_some());

        let rev_before_reset = dock_state.tree.revision();

        // 2. Second click: 120ms later at same point -> double click triggers reset to 0.50
        let t1 = t0 + Duration::from_millis(120);
        let res2 = handle_splitter_click_with_time(&mut dock_state, &target, click_point, t1);
        assert_eq!(res2, SplitterClickResult::ResetToHalf);
        assert!(
            dock_state.active_splitter.is_none(),
            "Drag session must terminate upon double-click reset"
        );

        if let Some(DockNode::Split { ratio, .. }) = dock_state.tree.get(center) {
            assert_eq!(
                *ratio, 0.50,
                "Double-click on splitter must reset partition ratio back to exactly 0.50"
            );
            assert!(
                dock_state.tree.revision() > rev_before_reset,
                "Tree revision must increment on 0.50 reset"
            );
        } else {
            panic!("Expected split node");
        }
    }

    #[test]
    fn test_double_click_threshold_expiration() {
        let mut tree = DockTree::<()>::new();
        let center = tree.create_leaf(vec![]);
        tree.set_root(center);
        let _ = tree
            .split_ordered(center, SplitDirection::Horizontal, 0.7, vec![], true)
            .expect("Split succeeds");

        let mut dock_state = DockState::new(tree);
        let target = DockSplitterTarget::new(
            center,
            SplitDirection::Horizontal,
            Rect::new(700.0, 0.0, 2.0, 600.0),
            1000.0,
        );

        let t0 = Instant::now();
        let click_point = Point::new(701.0, 200.0);

        let res1 = handle_splitter_click_with_time(&mut dock_state, &target, click_point, t0);
        assert_eq!(res1, SplitterClickResult::DragStarted);

        // Click again after 400ms (> 300ms threshold)
        let t1 = t0 + Duration::from_millis(400);
        let res2 = handle_splitter_click_with_time(&mut dock_state, &target, click_point, t1);
        assert_eq!(
            res2,
            SplitterClickResult::DragStarted,
            "Expired click interval must NOT trigger double-click reset"
        );

        if let Some(DockNode::Split { ratio, .. }) = dock_state.tree.get(center) {
            assert_eq!(*ratio, 0.70, "Ratio must remain unchanged");
        }
    }

    #[test]
    fn test_double_click_distance_threshold() {
        let mut tree = DockTree::<()>::new();
        let center = tree.create_leaf(vec![]);
        tree.set_root(center);
        let _ = tree
            .split_ordered(center, SplitDirection::Horizontal, 0.75, vec![], true)
            .expect("Split succeeds");

        let mut dock_state = DockState::new(tree);
        let target = DockSplitterTarget::new(
            center,
            SplitDirection::Horizontal,
            Rect::new(750.0, 0.0, 2.0, 600.0),
            1000.0,
        );

        let t0 = Instant::now();
        let res1 =
            handle_splitter_click_with_time(&mut dock_state, &target, Point::new(751.0, 100.0), t0);
        assert_eq!(res1, SplitterClickResult::DragStarted);

        // Click again within 100ms, but 10px away (> 4.0px distance threshold)
        let t1 = t0 + Duration::from_millis(100);
        let res2 =
            handle_splitter_click_with_time(&mut dock_state, &target, Point::new(751.0, 110.0), t1);
        assert_eq!(
            res2,
            SplitterClickResult::DragStarted,
            "Clicks exceeding distance threshold must NOT trigger double-click reset"
        );
    }

    #[test]
    fn test_end_to_end_event_dispatcher_double_click_reset() {
        let mut tree = DockTree::<()>::new();
        let center = tree.create_leaf(vec![]);
        tree.set_root(center);
        let _ = tree
            .split_ordered(center, SplitDirection::Vertical, 0.30, vec![], true)
            .expect("Split succeeds");

        let mut dock_state = DockState::new(tree);
        let target = DockSplitterTarget::new(
            center,
            SplitDirection::Vertical,
            Rect::new(0.0, 300.0, 800.0, 2.0),
            1000.0,
        );
        let splitters = [target];

        // First click
        let click_event = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(400.0, 301.0),
        };
        let resp1 = handle_splitter_event(&mut dock_state, &click_event, &splitters);
        assert!(resp1.consumed);
        assert!(!resp1.layout_changed);
        assert!(dock_state.active_splitter.is_some());

        // Mouse release
        let up_event = UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(400.0, 301.0),
        };
        let resp_up = handle_splitter_event(&mut dock_state, &up_event, &splitters);
        assert!(resp_up.consumed);
        assert!(dock_state.active_splitter.is_none());

        // Immediate second click (within 300ms)
        let resp2 = handle_splitter_event(&mut dock_state, &click_event, &splitters);
        assert!(resp2.consumed);
        assert!(resp2.layout_changed, "Layout changed must be true on reset");

        if let Some(DockNode::Split { ratio, .. }) = dock_state.tree.get(center) {
            assert_eq!(*ratio, 0.50, "Vertical splitter must reset to 0.50");
        }
    }
}