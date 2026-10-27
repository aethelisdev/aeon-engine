// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Dock Tab Drag-and-Drop Interaction Engine
//!
//! Manages tab tear-off lifecycle, mouse motion threshold detection, 5-way compass
//! drop zone calculations, preview highlights, tab reordering, and drop commits
//! into docked split leaves or detached floating windows.

use crate::drag_drop::{DropZone, calculate_drop_preview_rect, calculate_screen_drop_zone};
use crate::layout::ComputedDockLayout;
use crate::state::DockState;
use crate::tree::DockNodeId;
use iris_core::event::{KeyCode, MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect};
use iris_core::node::WidgetCursor;

/// Motion distance threshold squared in logical pixels (4.0 px motion, 16.0 px² distance).
pub const TAB_DRAG_MOTION_THRESHOLD_SQ: f32 = 16.0;

/// Default dimension assigned to detached floating windows if source dimensions cannot be inferred.
pub const DEFAULT_FLOATING_PANE_SIZE: Point = Point::new(400.0, 300.0);

/// Describes a pending tab drag-and-drop operation awaiting mouse motion past threshold.
///
/// Initialized when a user presses down on a tab header, and consumed into an active
/// drag operation once cursor movement exceeds [`TAB_DRAG_MOTION_THRESHOLD_SQ`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PendingTabDrag<T> {
    /// Source dock leaf node hosting the tab.
    pub leaf: DockNodeId,
    /// Index of the tab within the source leaf.
    pub tab_index: usize,
    /// Detached tab payload data.
    pub tab: T,
    /// Screen-space cursor coordinates when the mouse button was initially pressed down.
    pub press_pos: Point,
    /// Bounding rectangle of the source pane when pressed.
    pub leaf_rect: Rect,
    /// Optional floating window identifier if dragging a tab originating from a detached window.
    pub floating_window_id: Option<u64>,
}

impl<T> PendingTabDrag<T> {
    /// Creates a new pending tab drag originating from a docked leaf.
    #[must_use]
    pub const fn new(
        leaf: DockNodeId,
        tab_index: usize,
        tab: T,
        press_pos: Point,
        leaf_rect: Rect,
    ) -> Self {
        Self {
            leaf,
            tab_index,
            tab,
            press_pos,
            leaf_rect,
            floating_window_id: None,
        }
    }

    /// Creates a new pending tab drag originating from an existing floating window.
    #[must_use]
    pub const fn with_floating_window(
        leaf: DockNodeId,
        tab_index: usize,
        tab: T,
        press_pos: Point,
        leaf_rect: Rect,
        floating_window_id: u64,
    ) -> Self {
        Self {
            leaf,
            tab_index,
            tab,
            press_pos,
            leaf_rect,
            floating_window_id: Some(floating_window_id),
        }
    }
}

/// Result of evaluating an interaction event against pending and active tab drag operations.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TabDragInteractionResponse {
    /// Whether an active drag operation was started during this interaction.
    pub drag_started: bool,
    /// Active drop target zone and highlight preview bounding box, if hovering over a valid target.
    pub drop_preview: Option<(DropZone, Rect)>,
    /// Whether a tab was successfully released and committed (docked, reordered, or floated).
    pub drop_committed: bool,
    /// Resulting floating window identifier if the tab was detached as an independent floating window.
    pub detached_floating_window: Option<u64>,
    /// Suggested cursor icon when dragging or hovering drag elements.
    pub cursor: Option<WidgetCursor>,
    /// Whether the incoming event was intercepted and consumed.
    pub consumed: bool,
    /// Whether the dock tree or layout hierarchy changed, requiring recalculation.
    pub layout_changed: bool,
}

impl TabDragInteractionResponse {
    /// Returns an empty response with no actions, unconsumed, and unchanged layout.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            drag_started: false,
            drop_preview: None,
            drop_committed: false,
            detached_floating_window: None,
            cursor: None,
            consumed: false,
            layout_changed: false,
        }
    }
}

/// Evaluates an incoming interaction event against pending and active tab drag-and-drop operations.
///
/// Handles:
/// 1. **Motion Threshold Evaluation**: Promotes `pending_drag` to `dock_state.start_tab_drag` (or
///    `start_floating_tab_drag`) once cursor motion exceeds 4.0 logical pixels (`TAB_DRAG_MOTION_THRESHOLD_SQ`).
/// 2. **Continuous Drag Updates**: Updates cursor position, computes tab strip reorder targets,
///    evaluates 5-way navigator anchor zones on hovered leaves, and falls back to leaf/screen edge splits.
/// 3. **Drop Commit**: On mouse release (`MouseUp`), finalizes the drop into target leaf tabs,
///    partitions leaves/screen roots, or detaches the tab into a new floating window.
/// 4. **Cancellation**: Resets pending drag if mouse is released before reaching threshold or if `Escape` is pressed.
///
/// # Arguments
/// * `dock_state` - Mutable reference to the docking state managing split trees, floating windows, and active drag.
/// * `event` - Raw interaction event to process.
/// * `pending_drag` - Mutable reference to the pending drag payload (initiated on tab click).
/// * `workspace_rect` - Total bounding box of the main docked workspace.
/// * `layout` - Computed layout of the docked workspace containing leaf bounds and tab strips.
///
/// # Returns
/// A [`TabDragInteractionResponse`] describing state changes, drop previews, cursor feedback, and layout invalidation.
pub fn handle_tab_drag_event<T: Clone + Copy + PartialEq>(
    dock_state: &mut DockState<T>,
    event: &UiEvent,
    pending_drag: &mut Option<PendingTabDrag<T>>,
    workspace_rect: Rect,
    layout: &ComputedDockLayout<T>,
) -> TabDragInteractionResponse {
    match event {
        UiEvent::MouseMove { point } => {
            let mut drag_started = false;

            // 1. Activate pending tab drag if cursor motion exceeds threshold (4.0 px)
            if let Some(pending) = *pending_drag {
                let dx = point.x - pending.press_pos.x;
                let dy = point.y - pending.press_pos.y;
                if dx * dx + dy * dy >= TAB_DRAG_MOTION_THRESHOLD_SQ {
                    let start_res = if let Some(win_id) = pending.floating_window_id {
                        dock_state.start_floating_tab_drag(
                            win_id,
                            pending.leaf,
                            pending.tab_index,
                            *point,
                        )
                    } else {
                        dock_state.start_tab_drag(
                            pending.leaf,
                            pending.tab_index,
                            *point,
                            pending.leaf_rect,
                        )
                    };

                    if start_res.is_ok() {
                        *pending_drag = None;
                        drag_started = true;
                    }
                }
            }

            // 2. Active tab drag continuous updates
            if dock_state.active_drag.is_some() {
                dock_state.update_tab_drag(*point, layout);

                // Fallback to outer screen drop zone (32px border) if compass was not hit
                if let Some(drag) = dock_state
                    .active_drag
                    .as_mut()
                    .filter(|d| d.hover_target.is_none() && d.tab_reorder_target.is_none())
                    && let Some(screen_zone) =
                        calculate_screen_drop_zone(workspace_rect, *point, 32.0)
                {
                    let preview = calculate_drop_preview_rect(workspace_rect, screen_zone);
                    let root_id = dock_state.tree.root().unwrap_or(drag.source_leaf);
                    drag.hover_target = Some((root_id, screen_zone, preview));
                }

                let preview = dock_state
                    .active_drag
                    .as_ref()
                    .and_then(|d| d.hover_target.map(|(_, z, r)| (z, r)));

                return TabDragInteractionResponse {
                    drag_started,
                    drop_preview: preview,
                    drop_committed: false,
                    detached_floating_window: None,
                    cursor: Some(WidgetCursor::Grabbing),
                    consumed: true,
                    layout_changed: drag_started,
                };
            }

            if drag_started {
                return TabDragInteractionResponse {
                    drag_started: true,
                    drop_preview: None,
                    drop_committed: false,
                    detached_floating_window: None,
                    cursor: Some(WidgetCursor::Grabbing),
                    consumed: true,
                    layout_changed: true,
                };
            }

            TabDragInteractionResponse::empty()
        }

        UiEvent::MouseUp {
            button: MouseButton::Left,
            point: _,
        } => {
            // Discard pending drag if released before threshold
            if pending_drag.is_some() {
                *pending_drag = None;
            }

            // Commit active drag if released
            if dock_state.active_drag.is_some() {
                let detached = dock_state
                    .drop_tab_or_float(DEFAULT_FLOATING_PANE_SIZE)
                    .ok()
                    .flatten();

                return TabDragInteractionResponse {
                    drag_started: false,
                    drop_preview: None,
                    drop_committed: true,
                    detached_floating_window: detached,
                    cursor: None,
                    consumed: true,
                    layout_changed: true,
                };
            }

            TabDragInteractionResponse::empty()
        }

        UiEvent::KeyboardInput {
            key: KeyCode::Escape,
            is_pressed: true,
            ..
        } => {
            // Cancel pending drag
            if pending_drag.is_some() {
                *pending_drag = None;
                return TabDragInteractionResponse {
                    consumed: true,
                    ..TabDragInteractionResponse::empty()
                };
            }

            // Cancel active drag by restoring to source leaf
            if dock_state.active_drag.is_some() {
                if let Some(ref mut drag) = dock_state.active_drag {
                    drag.hover_target = None;
                    drag.tab_reorder_target = None;
                }
                let _ = dock_state.drop_tab();

                return TabDragInteractionResponse {
                    drop_committed: true,
                    cursor: None,
                    consumed: true,
                    layout_changed: true,
                    ..TabDragInteractionResponse::empty()
                };
            }

            TabDragInteractionResponse::empty()
        }

        _ => TabDragInteractionResponse::empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::compute_dock_layout;
    use crate::tree::DockTree;
    use iris_core::geometry::{Point, Rect};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestPanel {
        Hierarchy,
        Inspector,
        Console,
    }

    #[test]
    fn test_pending_tab_drag_below_threshold() {
        let mut tree = DockTree::<TestPanel>::new();
        let leaf = tree.create_leaf(vec![TestPanel::Hierarchy, TestPanel::Inspector]);
        tree.set_root(leaf);
        let mut dock_state = DockState::new(tree);

        let mut pending = Some(PendingTabDrag::new(
            leaf,
            0,
            TestPanel::Hierarchy,
            Point::new(100.0, 50.0),
            Rect::new(0.0, 0.0, 800.0, 600.0),
        ));

        let workspace = Rect::new(0.0, 0.0, 800.0, 600.0);
        let layout = compute_dock_layout(&dock_state.tree, workspace, 4.0, 26.0);

        // Move 2px (dx=2, dy=1 -> 4 + 1 = 5 < 16)
        let small_move = UiEvent::MouseMove {
            point: Point::new(102.0, 51.0),
        };

        let resp = handle_tab_drag_event(
            &mut dock_state,
            &small_move,
            &mut pending,
            workspace,
            &layout,
        );

        assert!(!resp.drag_started);
        assert!(!resp.consumed);
        assert!(pending.is_some());
        assert!(dock_state.active_drag.is_none());
    }

    #[test]
    fn test_pending_tab_drag_above_threshold_activation() {
        let mut tree = DockTree::<TestPanel>::new();
        let leaf = tree.create_leaf(vec![TestPanel::Hierarchy, TestPanel::Inspector]);
        tree.set_root(leaf);
        let mut dock_state = DockState::new(tree);

        let mut pending = Some(PendingTabDrag::new(
            leaf,
            0,
            TestPanel::Hierarchy,
            Point::new(100.0, 50.0),
            Rect::new(0.0, 0.0, 800.0, 600.0),
        ));

        let workspace = Rect::new(0.0, 0.0, 800.0, 600.0);
        let layout = compute_dock_layout(&dock_state.tree, workspace, 4.0, 26.0);

        // Move 6px (dx=6, dy=0 -> 36 >= 16)
        let large_move = UiEvent::MouseMove {
            point: Point::new(106.0, 50.0),
        };

        let resp = handle_tab_drag_event(
            &mut dock_state,
            &large_move,
            &mut pending,
            workspace,
            &layout,
        );

        assert!(resp.drag_started);
        assert!(resp.consumed);
        assert!(resp.layout_changed);
        assert_eq!(resp.cursor, Some(WidgetCursor::Grabbing));
        assert!(pending.is_none());
        assert!(dock_state.active_drag.is_some());
    }

    #[test]
    fn test_tab_drag_mouse_up_before_threshold_clears_pending() {
        let mut tree = DockTree::<TestPanel>::new();
        let leaf = tree.create_leaf(vec![TestPanel::Hierarchy]);
        tree.set_root(leaf);
        let mut dock_state = DockState::new(tree);

        let mut pending = Some(PendingTabDrag::new(
            leaf,
            0,
            TestPanel::Hierarchy,
            Point::new(100.0, 50.0),
            Rect::new(0.0, 0.0, 800.0, 600.0),
        ));

        let workspace = Rect::new(0.0, 0.0, 800.0, 600.0);
        let layout = compute_dock_layout(&dock_state.tree, workspace, 4.0, 26.0);

        let mouse_up = UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(101.0, 50.0),
        };

        let resp =
            handle_tab_drag_event(&mut dock_state, &mouse_up, &mut pending, workspace, &layout);

        assert!(!resp.drag_started);
        assert!(!resp.drop_committed);
        assert!(pending.is_none());
        assert!(dock_state.active_drag.is_none());
    }

    #[test]
    fn test_tab_drag_drop_into_split_zone() {
        let mut tree = DockTree::<TestPanel>::new();
        let leaf_a = tree.create_leaf(vec![TestPanel::Hierarchy]);
        tree.set_root(leaf_a);
        let (first_leaf, _second_leaf) = tree
            .split(
                leaf_a,
                crate::tree::SplitDirection::Horizontal,
                0.5,
                vec![TestPanel::Inspector],
            )
            .unwrap();
        let mut dock_state = DockState::new(tree);

        let workspace = Rect::new(0.0, 0.0, 800.0, 600.0);
        let layout = compute_dock_layout(&dock_state.tree, workspace, 4.0, 26.0);

        // Start drag on first_leaf
        let _ = dock_state.start_tab_drag(
            first_leaf,
            0,
            Point::new(100.0, 50.0),
            Rect::new(0.0, 0.0, 400.0, 600.0),
        );
        let mut pending = None;

        // Move to right split zone (outer border 32px or compass right, e.g. x=785.0 in an 800-wide workspace)
        let move_to_right = UiEvent::MouseMove {
            point: Point::new(785.0, 300.0),
        };
        let resp_move = handle_tab_drag_event(
            &mut dock_state,
            &move_to_right,
            &mut pending,
            workspace,
            &layout,
        );

        assert!(resp_move.consumed);
        assert!(resp_move.drop_preview.is_some());

        // Release mouse to commit split
        let mouse_up = UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(785.0, 300.0),
        };
        let resp_drop =
            handle_tab_drag_event(&mut dock_state, &mouse_up, &mut pending, workspace, &layout);

        assert!(resp_drop.drop_committed);
        assert!(resp_drop.layout_changed);
        assert!(dock_state.active_drag.is_none());

        // Hierarchy should now be in the tree
        assert!(dock_state.tree.find_tab(&TestPanel::Hierarchy).is_some());
    }

    #[test]
    fn test_tab_drag_drop_outside_creates_floating_window() {
        let mut tree = DockTree::<TestPanel>::new();
        let leaf = tree.create_leaf(vec![TestPanel::Hierarchy, TestPanel::Console]);
        tree.set_root(leaf);
        let mut dock_state = DockState::new(tree);

        let workspace = Rect::new(0.0, 0.0, 800.0, 600.0);
        let layout = compute_dock_layout(&dock_state.tree, workspace, 4.0, 26.0);

        // Start drag on Hierarchy
        let _ = dock_state.start_tab_drag(
            leaf,
            0,
            Point::new(100.0, 50.0),
            Rect::new(0.0, 0.0, 400.0, 300.0),
        );
        let mut pending = None;

        // Ensure hover target is None by simulating drop outside workspace
        if let Some(ref mut d) = dock_state.active_drag {
            d.hover_target = None;
            d.tab_reorder_target = None;
        }

        let mouse_up = UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(950.0, 750.0),
        };
        let resp =
            handle_tab_drag_event(&mut dock_state, &mouse_up, &mut pending, workspace, &layout);

        assert!(resp.drop_committed);
        assert!(resp.detached_floating_window.is_some());
        assert_eq!(dock_state.floating_windows.len(), 1);
        assert_eq!(
            dock_state.floating_windows[0]
                .tree
                .find_tab(&TestPanel::Hierarchy)
                .map(|(_, i)| i),
            Some(0)
        );
    }

    #[test]
    fn test_tab_drag_escape_restores_tab() {
        let mut tree = DockTree::<TestPanel>::new();
        let leaf = tree.create_leaf(vec![TestPanel::Hierarchy, TestPanel::Console]);
        tree.set_root(leaf);
        let mut dock_state = DockState::new(tree);

        let workspace = Rect::new(0.0, 0.0, 800.0, 600.0);
        let layout = compute_dock_layout(&dock_state.tree, workspace, 4.0, 26.0);

        let _ = dock_state.start_tab_drag(
            leaf,
            0,
            Point::new(100.0, 50.0),
            Rect::new(0.0, 0.0, 400.0, 300.0),
        );
        let mut pending = None;

        let escape_evt = UiEvent::KeyboardInput {
            key: KeyCode::Escape,
            text: None,
            is_pressed: true,
        };

        let resp = handle_tab_drag_event(
            &mut dock_state,
            &escape_evt,
            &mut pending,
            workspace,
            &layout,
        );

        assert!(resp.drop_committed);
        assert!(resp.layout_changed);
        assert!(dock_state.active_drag.is_none());

        // Tab is restored to the source leaf
        let tabs = dock_state.tree.get(leaf).map(|n| match n {
            crate::tree::DockNode::Leaf { tabs, .. } => tabs.clone(),
            _ => vec![],
        });
        assert_eq!(tabs, Some(vec![TestPanel::Console, TestPanel::Hierarchy]));
    }
}