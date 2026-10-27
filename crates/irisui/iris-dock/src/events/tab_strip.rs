// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Dock Tab Strip & Overflow Menu Interaction Engine
//!
//! Manages tab selection, tab close button actions, middle-click tab closure,
//! and tab overflow dropdown popup menu interactions.

use crate::chrome::{DockChevronTarget, DockCloseTarget, DockTabTarget};
use crate::overflow::DockOverflowItemTarget;
use crate::state::DockState;
use crate::tree::DockNodeId;
use iris_core::event::{MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect};
use iris_core::node::WidgetCursor;

/// Result of evaluating an interaction event against dock tab strips and overflow menus.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TabStripInteractionResponse<T> {
    /// Tab that was activated/selected during this interaction, if any.
    pub selected_tab: Option<(DockNodeId, usize, T)>,
    /// Tab that was closed during this interaction, if any.
    pub closed_tab: Option<(DockNodeId, usize)>,
    /// Whether the tab strip overflow menu opened, toggled, or dismissed.
    pub overflow_toggled: bool,
    /// Suggested cursor icon when hovering clickable tab elements.
    pub cursor: Option<WidgetCursor>,
    /// Whether the incoming event was intercepted and consumed.
    pub consumed: bool,
    /// Whether the dock tree or tab state changed, requiring layout recalculation.
    pub layout_changed: bool,
    /// Information needed to initiate a pending drag if mouse moves past threshold.
    /// Format: `(leaf, tab_index, tab_payload, press_position, leaf_rect)`.
    pub pending_drag: Option<(DockNodeId, usize, T, Point, Rect)>,
}

impl<T> TabStripInteractionResponse<T> {
    /// Returns an empty response with no actions, unconsumed, and unchanged layout.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            selected_tab: None,
            closed_tab: None,
            overflow_toggled: false,
            cursor: None,
            consumed: false,
            layout_changed: false,
            pending_drag: None,
        }
    }
}

/// Evaluates an incoming interaction event against docked tab strips, close buttons, and overflow menus.
///
/// Handles:
/// 1. **Tab Selection**: Left-clicking a tab activates it on the host leaf and prepares a pending drag payload.
/// 2. **Tab Closure**: Left-clicking the `✖` close button or middle-clicking the tab closes it and prunes empty leaves.
/// 3. **Overflow Menu**: Clicking the `▾` chevron toggles the overflow popup; clicking an overflow item activates it.
/// 4. **Outside Dismissal**: Clicking outside an open overflow menu dismisses it automatically.
/// 5. **Hover Feedback**: Pointer cursor returned when hovering clickable tab targets or buttons.
///
/// # Arguments
/// * `dock_state` - Mutable reference to the docking state managing the split tree and overflow state.
/// * `event` - Raw interaction event to process.
/// * `tab_targets` - Slice of rendered tab header hit targets.
/// * `close_targets` - Slice of rendered tab close button hit targets.
/// * `chevron_targets` - Slice of rendered tab overflow chevron button hit targets.
/// * `overflow_items` - Slice of active overflow dropdown menu item hit targets (if menu is open).
///
/// # Returns
/// A [`TabStripInteractionResponse`] describing state changes, cursor feedback, and layout invalidation.
pub fn handle_tab_strip_event<T: Clone + Copy + PartialEq>(
    dock_state: &mut DockState<T>,
    event: &UiEvent,
    tab_targets: &[DockTabTarget<T>],
    close_targets: &[DockCloseTarget],
    chevron_targets: &[DockChevronTarget],
    overflow_items: &[DockOverflowItemTarget],
) -> TabStripInteractionResponse<T> {
    match event {
        UiEvent::MouseDown {
            button: MouseButton::Left,
            point,
        } => {
            // 1. Check if clicking an item inside an active overflow dropdown menu
            if let Some(item) = overflow_items.iter().find(|it| it.contains_point(*point)) {
                let _ = dock_state.set_active_tab(item.leaf, item.tab_index);
                dock_state.active_overflow = None;

                let tab_payload = tab_targets
                    .iter()
                    .find(|t| t.leaf == item.leaf && t.tab_index == item.tab_index)
                    .map(|t| t.tab)
                    .or_else(|| {
                        dock_state.tree.get(item.leaf).and_then(|node| {
                            if let crate::tree::DockNode::Leaf { tabs, .. } = node {
                                tabs.get(item.tab_index).copied()
                            } else {
                                None
                            }
                        })
                    });

                return TabStripInteractionResponse {
                    selected_tab: tab_payload.map(|t| (item.leaf, item.tab_index, t)),
                    closed_tab: None,
                    overflow_toggled: true,
                    cursor: Some(WidgetCursor::Pointer),
                    consumed: true,
                    layout_changed: true,
                    pending_drag: None,
                };
            }

            // 2. Check if clicking an overflow chevron button (`▾`)
            if let Some(chevron) = chevron_targets.iter().find(|c| c.contains_point(*point)) {
                let was_open = dock_state
                    .active_overflow
                    .as_ref()
                    .is_some_and(|(l, _)| *l == chevron.leaf);

                if was_open {
                    dock_state.active_overflow = None;
                } else {
                    dock_state.active_overflow = Some((chevron.leaf, chevron.rect));
                }

                return TabStripInteractionResponse {
                    selected_tab: None,
                    closed_tab: None,
                    overflow_toggled: true,
                    cursor: Some(WidgetCursor::Pointer),
                    consumed: true,
                    layout_changed: true,
                    pending_drag: None,
                };
            }

            // 3. Check if clicking a tab close button (`✖`)
            if let Some(close) = close_targets.iter().find(|c| c.contains_point(*point)) {
                let _ = dock_state.close_tab(close.leaf, close.tab_index);
                dock_state.active_overflow = None;

                return TabStripInteractionResponse {
                    selected_tab: None,
                    closed_tab: Some((close.leaf, close.tab_index)),
                    overflow_toggled: false,
                    cursor: Some(WidgetCursor::Pointer),
                    consumed: true,
                    layout_changed: true,
                    pending_drag: None,
                };
            }

            // 4. Check if clicking a tab header pill
            if let Some(tab_target) = tab_targets.iter().find(|t| t.contains_point(*point)) {
                let _ = dock_state.set_active_tab(tab_target.leaf, tab_target.tab_index);
                dock_state.active_overflow = None;

                return TabStripInteractionResponse {
                    selected_tab: Some((tab_target.leaf, tab_target.tab_index, tab_target.tab)),
                    closed_tab: None,
                    overflow_toggled: false,
                    cursor: Some(WidgetCursor::Pointer),
                    consumed: true,
                    layout_changed: true,
                    pending_drag: Some((
                        tab_target.leaf,
                        tab_target.tab_index,
                        tab_target.tab,
                        *point,
                        tab_target.leaf_rect,
                    )),
                };
            }

            // 5. Dismiss open overflow menu if clicking outside
            if dock_state.active_overflow.is_some() {
                dock_state.active_overflow = None;
                return TabStripInteractionResponse {
                    selected_tab: None,
                    closed_tab: None,
                    overflow_toggled: true,
                    cursor: None,
                    consumed: false,
                    layout_changed: true,
                    pending_drag: None,
                };
            }

            TabStripInteractionResponse::empty()
        }

        UiEvent::MouseDown {
            button: MouseButton::Middle,
            point,
        } => {
            // Middle-click shortcut to close a tab directly
            if let Some(tab_target) = tab_targets.iter().find(|t| t.contains_point(*point)) {
                let _ = dock_state.close_tab(tab_target.leaf, tab_target.tab_index);
                dock_state.active_overflow = None;

                return TabStripInteractionResponse {
                    selected_tab: None,
                    closed_tab: Some((tab_target.leaf, tab_target.tab_index)),
                    overflow_toggled: false,
                    cursor: None,
                    consumed: true,
                    layout_changed: true,
                    pending_drag: None,
                };
            }

            TabStripInteractionResponse::empty()
        }

        UiEvent::MouseMove { point } => {
            // Hover cursor feedback across tab strip elements
            let is_hovering = overflow_items.iter().any(|it| it.contains_point(*point))
                || chevron_targets.iter().any(|c| c.contains_point(*point))
                || close_targets.iter().any(|c| c.contains_point(*point))
                || tab_targets.iter().any(|t| t.contains_point(*point));

            if is_hovering {
                TabStripInteractionResponse {
                    cursor: Some(WidgetCursor::Pointer),
                    consumed: false,
                    ..TabStripInteractionResponse::empty()
                }
            } else {
                TabStripInteractionResponse::empty()
            }
        }

        _ => TabStripInteractionResponse::empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::DockTree;
    use iris_core::geometry::{Point, Rect};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestPanel {
        Hierarchy,
        Inspector,
        Console,
    }

    #[test]
    fn test_tab_strip_tab_selection() {
        let mut tree = DockTree::<TestPanel>::new();
        let leaf = tree.create_leaf(vec![TestPanel::Hierarchy, TestPanel::Inspector]);
        tree.set_root(leaf);
        let mut dock_state = DockState::new(tree);

        let tab_target = DockTabTarget {
            leaf,
            tab_index: 1,
            tab: TestPanel::Inspector,
            rect: Rect::new(80.0, 0.0, 70.0, 26.0),
            leaf_rect: Rect::new(0.0, 0.0, 400.0, 500.0),
        };
        let tab_targets = [tab_target];

        let click_event = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(100.0, 10.0),
        };

        let resp =
            handle_tab_strip_event(&mut dock_state, &click_event, &tab_targets, &[], &[], &[]);

        assert!(resp.consumed);
        assert!(resp.layout_changed);
        assert_eq!(resp.selected_tab, Some((leaf, 1, TestPanel::Inspector)));
        assert!(resp.pending_drag.is_some());

        // Verify active tab in tree changed to 1
        let active = dock_state.tree.get(leaf).map(|n| match n {
            crate::tree::DockNode::Leaf { active_tab, .. } => *active_tab,
            _ => 999,
        });
        assert_eq!(active, Some(1));
    }

    #[test]
    fn test_tab_strip_close_button_and_collapse() {
        let mut tree = DockTree::<TestPanel>::new();
        let leaf = tree.create_leaf(vec![TestPanel::Hierarchy]);
        tree.set_root(leaf);
        let mut dock_state = DockState::new(tree);

        let close_target = DockCloseTarget {
            leaf,
            tab_index: 0,
            rect: Rect::new(70.0, 4.0, 14.0, 14.0),
        };
        let close_targets = [close_target];

        let click_event = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(75.0, 10.0),
        };

        let resp =
            handle_tab_strip_event(&mut dock_state, &click_event, &[], &close_targets, &[], &[]);

        assert!(resp.consumed);
        assert!(resp.layout_changed);
        assert_eq!(resp.closed_tab, Some((leaf, 0)));

        // Empty leaf must be pruned
        assert!(dock_state.tree.get(leaf).is_none());
    }

    #[test]
    fn test_tab_strip_middle_click_close() {
        let mut tree = DockTree::<TestPanel>::new();
        let leaf = tree.create_leaf(vec![TestPanel::Hierarchy, TestPanel::Console]);
        tree.set_root(leaf);
        let mut dock_state = DockState::new(tree);

        let tab_target = DockTabTarget {
            leaf,
            tab_index: 0,
            tab: TestPanel::Hierarchy,
            rect: Rect::new(0.0, 0.0, 80.0, 26.0),
            leaf_rect: Rect::new(0.0, 0.0, 400.0, 500.0),
        };
        let tab_targets = [tab_target];

        let mid_click = UiEvent::MouseDown {
            button: MouseButton::Middle,
            point: Point::new(40.0, 10.0),
        };

        let resp = handle_tab_strip_event(&mut dock_state, &mid_click, &tab_targets, &[], &[], &[]);

        assert!(resp.consumed);
        assert_eq!(resp.closed_tab, Some((leaf, 0)));

        // Hierarchy was closed, Console remains
        let remaining_tabs = dock_state.tree.get(leaf).map(|n| match n {
            crate::tree::DockNode::Leaf { tabs, .. } => tabs.clone(),
            _ => vec![],
        });
        assert_eq!(remaining_tabs, Some(vec![TestPanel::Console]));
    }

    #[test]
    fn test_tab_strip_overflow_chevron_toggle_and_select() {
        let mut tree = DockTree::<TestPanel>::new();
        let leaf = tree.create_leaf(vec![
            TestPanel::Hierarchy,
            TestPanel::Inspector,
            TestPanel::Console,
        ]);
        tree.set_root(leaf);
        let mut dock_state = DockState::new(tree);

        let chevron = DockChevronTarget {
            leaf,
            rect: Rect::new(380.0, 0.0, 20.0, 26.0),
        };
        let chevron_targets = [chevron];

        // 1. Click chevron to open overflow menu
        let chevron_click = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(390.0, 12.0),
        };
        let resp_open = handle_tab_strip_event(
            &mut dock_state,
            &chevron_click,
            &[],
            &[],
            &chevron_targets,
            &[],
        );
        assert!(resp_open.consumed);
        assert!(resp_open.overflow_toggled);
        assert_eq!(
            dock_state.active_overflow,
            Some((leaf, Rect::new(380.0, 0.0, 20.0, 26.0)))
        );

        // 2. Click an item in the open overflow dropdown
        let overflow_item = DockOverflowItemTarget {
            leaf,
            tab_index: 2,
            rect: Rect::new(220.0, 30.0, 180.0, 26.0),
        };
        let overflow_items = [overflow_item];

        let item_click = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(250.0, 40.0),
        };
        let resp_select = handle_tab_strip_event(
            &mut dock_state,
            &item_click,
            &[],
            &[],
            &chevron_targets,
            &overflow_items,
        );
        assert!(resp_select.consumed);
        assert_eq!(
            resp_select.selected_tab,
            Some((leaf, 2, TestPanel::Console))
        );
        assert_eq!(dock_state.active_overflow, None); // Closed after selection
    }
}