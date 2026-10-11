// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Dock Splitter Divider Interaction Engine
//!
//! Manages pointer hit-testing, cursor feedback, and continuous dragging
//! (with automatic pointer capture) for docking splitters across horizontal
//! and vertical partitions.
//!

use crate::chrome::DockSplitterTarget;
use crate::state::DockState;
use crate::tree::SplitDirection;
use iris_core::event::{MouseButton, UiEvent};
use iris_core::node::WidgetCursor;

/// Result of processing an interaction event against docking splitter dividers.
///
/// Returned by [`handle_splitter_event`] to communicate cursor overrides,
/// whether the incoming event was consumed, and whether the tree layout was modified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SplitterInteractionResponse {
    /// Suggested cursor icon when hovering over or actively dragging a splitter divider.
    pub cursor: Option<WidgetCursor>,
    /// Whether the event was intercepted and consumed by this splitter divider interaction.
    pub consumed: bool,
    /// Whether the division ratio changed, requiring layout recalculation.
    pub layout_changed: bool,
}

impl SplitterInteractionResponse {
    /// Returns a new response with no cursor override, unconsumed, and unchanged layout.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            cursor: None,
            consumed: false,
            layout_changed: false,
        }
    }
}

/// Processes incoming interaction events against active and target splitters in a docking layout.
///
/// Manages the complete lifecycle of divider interactions:
/// 1. **Continuous Dragging (Pointer Capture)**: If [`DockState::active_splitter`] is set,
///    cursor movements dynamically update the partition ratio even if the cursor leaves the initial geometry.
/// 2. **Drag Release**: Left button release terminates the active dragging session.
/// 3. **Drag Initiation**: Left button click within a splitter's target bounds initiates a new dragging session.
/// 4. **Hover Tracking**: Cursor motion over a splitter without pressing yields the appropriate resize cursor.
///
/// # Arguments
/// * `dock_state` - Mutable reference to the docking state holding the split tree and drag status.
/// * `event` - Incoming raw interaction event to process.
/// * `splitters` - Slice of computed splitter targets produced during the most recent layout/render pass.
///
/// # Returns
/// A [`SplitterInteractionResponse`] indicating event consumption, cursor overrides, and layout dirty status.
pub fn handle_splitter_event<T>(
    dock_state: &mut DockState<T>,
    event: &UiEvent,
    splitters: &[DockSplitterTarget],
) -> SplitterInteractionResponse {
    match event {
        UiEvent::MouseMove { point } => {
            // 1. If actively dragging a splitter, continue with pointer capture
            if let Some(drag) = dock_state.active_splitter {
                let current_coord = match drag.direction {
                    SplitDirection::Horizontal => point.x,
                    SplitDirection::Vertical => point.y,
                };
                dock_state.update_splitter_drag(current_coord);

                let cursor = match drag.direction {
                    SplitDirection::Horizontal => WidgetCursor::ColResize,
                    SplitDirection::Vertical => WidgetCursor::RowResize,
                };

                return SplitterInteractionResponse {
                    cursor: Some(cursor),
                    consumed: true,
                    layout_changed: true,
                };
            }

            // 2. Otherwise, perform hit-testing to provide hover cursor feedback
            if let Some(target) = splitters.iter().find(|t| t.contains_point(*point)) {
                let cursor = match target.direction {
                    SplitDirection::Horizontal => WidgetCursor::ColResize,
                    SplitDirection::Vertical => WidgetCursor::RowResize,
                };
                return SplitterInteractionResponse {
                    cursor: Some(cursor),
                    consumed: false,
                    layout_changed: false,
                };
            }

            SplitterInteractionResponse::empty()
        }

        UiEvent::MouseDown {
            button: MouseButton::Left,
            point,
        } => {
            if let Some(target) = splitters.iter().find(|t| t.contains_point(*point)) {
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

                let cursor = match target.direction {
                    SplitDirection::Horizontal => WidgetCursor::ColResize,
                    SplitDirection::Vertical => WidgetCursor::RowResize,
                };

                return SplitterInteractionResponse {
                    cursor: Some(cursor),
                    consumed: true,
                    layout_changed: false,
                };
            }

            SplitterInteractionResponse::empty()
        }

        UiEvent::MouseUp {
            button: MouseButton::Left,
            ..
        } => {
            if dock_state.active_splitter.is_some() {
                dock_state.end_splitter_drag();
                return SplitterInteractionResponse {
                    cursor: None,
                    consumed: true,
                    layout_changed: false,
                };
            }

            SplitterInteractionResponse::empty()
        }

        _ => SplitterInteractionResponse::empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{DockNode, DockTree};
    use iris_core::{Point, Rect};

    #[test]
    fn test_splitter_hover_feedback() {
        let mut tree = DockTree::<()>::new();
        let split_container = tree.create_leaf(vec![]);
        tree.set_root(split_container);
        let _ = tree
            .split(split_container, SplitDirection::Horizontal, 0.5, vec![])
            .expect("Split succeeds");

        let mut dock_state = DockState::new(tree);
        let target = DockSplitterTarget {
            node: split_container,
            direction: SplitDirection::Horizontal,
            rect: Rect::new(100.0, 0.0, 4.0, 500.0),
            total_dimension: 800.0,
        };

        let splitters = [target];

        // Hit inside splitter area (+ HIT_MARGIN)
        let hit_event = UiEvent::MouseMove {
            point: Point::new(102.0, 250.0),
        };
        let response = handle_splitter_event(&mut dock_state, &hit_event, &splitters);
        assert_eq!(response.cursor, Some(WidgetCursor::ColResize));
        assert!(!response.consumed);

        // Miss outside splitter
        let miss_event = UiEvent::MouseMove {
            point: Point::new(200.0, 250.0),
        };
        let response = handle_splitter_event(&mut dock_state, &miss_event, &splitters);
        assert_eq!(response.cursor, None);
        assert!(!response.consumed);
    }

    #[test]
    fn test_splitter_drag_lifecycle() {
        let mut tree = DockTree::<()>::new();
        let split_container = tree.create_leaf(vec![]);
        tree.set_root(split_container);
        let _ = tree
            .split(split_container, SplitDirection::Horizontal, 0.5, vec![])
            .expect("Split must succeed");

        let mut dock_state = DockState::new(tree);
        let target = DockSplitterTarget {
            node: split_container,
            direction: SplitDirection::Horizontal,
            rect: Rect::new(400.0, 0.0, 4.0, 600.0),
            total_dimension: 800.0,
        };
        let splitters = [target];

        // 1. MouseDown on divider initiates drag
        let down_event = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(401.0, 100.0),
        };
        let resp_down = handle_splitter_event(&mut dock_state, &down_event, &splitters);
        assert!(resp_down.consumed);
        assert_eq!(resp_down.cursor, Some(WidgetCursor::ColResize));
        assert!(dock_state.active_splitter.is_some());

        // 2. MouseMove updates divider with pointer capture (even far from target rect)
        let move_event = UiEvent::MouseMove {
            point: Point::new(500.0, 800.0),
        };
        let resp_move = handle_splitter_event(&mut dock_state, &move_event, &splitters);
        assert!(resp_move.consumed);
        assert!(resp_move.layout_changed);
        assert_eq!(resp_move.cursor, Some(WidgetCursor::ColResize));

        // Ratio should be updated: start was 401.0 (ratio 0.5), moved by +99.0 / 800.0 = +0.12375
        if let Some(DockNode::Split { ratio, .. }) = dock_state.tree.get(split_container) {
            assert!(*ratio > 0.6);
        } else {
            panic!("Expected split node");
        }

        // 3. MouseUp releases the drag
        let up_event = UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(500.0, 800.0),
        };
        let resp_up = handle_splitter_event(&mut dock_state, &up_event, &splitters);
        assert!(resp_up.consumed);
        assert!(dock_state.active_splitter.is_none());
    }

    #[test]
    fn test_vertical_splitter_drag() {
        let mut tree = DockTree::<()>::new();
        let split_container = tree.create_leaf(vec![]);
        tree.set_root(split_container);
        let _ = tree
            .split(split_container, SplitDirection::Vertical, 0.4, vec![])
            .expect("Split must succeed");

        let mut dock_state = DockState::new(tree);
        let target = DockSplitterTarget {
            node: split_container,
            direction: SplitDirection::Vertical,
            rect: Rect::new(0.0, 200.0, 600.0, 4.0),
            total_dimension: 500.0,
        };
        let splitters = [target];

        // Hover test
        let hover_event = UiEvent::MouseMove {
            point: Point::new(300.0, 201.0),
        };
        let resp_hover = handle_splitter_event(&mut dock_state, &hover_event, &splitters);
        assert_eq!(resp_hover.cursor, Some(WidgetCursor::RowResize));

        // Down test
        let down_event = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(300.0, 201.0),
        };
        let resp_down = handle_splitter_event(&mut dock_state, &down_event, &splitters);
        assert!(resp_down.consumed);
        assert_eq!(resp_down.cursor, Some(WidgetCursor::RowResize));
    }
}