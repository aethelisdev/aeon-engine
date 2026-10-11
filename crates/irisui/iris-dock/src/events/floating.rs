// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Native Floating Windows Interaction Engine
//!
//! Manages title bar dragging, 8-directional border/corner edge resizing,
//! Z-order stacking management (bring-to-front), dock-back, and close button actions
//! for independent floating windows.
//!

use crate::drag_drop::DropZone;
use crate::events::drag_drop::PendingTabDrag;
use crate::floating::bounds::FloatingWindowClampBounds;
use crate::floating::cursor::{DEFAULT_RESIZE_MARGIN, FloatingResizeEdge, detect_resize_edge};
use crate::floating::model::{
    FloatingDragState, FloatingWindowClickAction, evaluate_floating_window_click,
};
use crate::state::DockState;
use crate::tab_viewer::TabViewer;
use iris_core::event::{KeyCode, MouseButton, UiEvent};
use iris_core::node::WidgetCursor;

/// Result of evaluating an interaction event against floating windows.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FloatingInteractionResponse<T> {
    /// Suggested cursor icon when hovering or dragging window edges/titles.
    pub cursor: Option<WidgetCursor>,
    /// Floating window ID that was closed during this interaction, if any.
    pub closed_window: Option<u64>,
    /// Floating window ID requested to be docked back into the main tree, if any.
    pub dock_back_window: Option<u64>,
    /// Information needed to initiate a pending tab drag from a floating tab.
    pub pending_tab_drag: Option<PendingTabDrag<T>>,
    /// Whether an active drag (title or resize) was started or active.
    pub drag_active: bool,
    /// Whether the incoming event was intercepted and consumed.
    pub consumed: bool,
    /// Whether window geometry or order changed, requiring layout invalidation.
    pub layout_changed: bool,
}

impl<T> FloatingInteractionResponse<T> {
    /// Returns an empty response with no actions, unconsumed, and unchanged layout.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            cursor: None,
            closed_window: None,
            dock_back_window: None,
            pending_tab_drag: None,
            drag_active: false,
            consumed: false,
            layout_changed: false,
        }
    }
}

/// Evaluates an incoming interaction event against independent floating windows.
///
/// Handles:
/// 1. **Title Bar Dragging**: Moves the window, clamping coordinates against visible bounds.
/// 2. **8-Way Edge Resizing**: Resizes width/height along all edges and corners with minimum dimensions.
/// 3. **Z-Order Management**: Clicks promote the active window to the top of the stack.
/// 4. **Close & Dock-Back Buttons**: Closes windows or rejoins them into the main dock hierarchy.
/// 5. **Tab Header Interactions**: Switches active tab and prepares pending tear-off drags.
/// 6. **Hover Feedback**: Returns directional resize and pointer cursors.
///
/// # Arguments
/// * `dock_state` - Mutable reference to the docking state managing floating windows.
/// * `viewer` - Tab viewer implementing tab titles and metrics.
/// * `event` - Raw interaction event to process.
/// * `clamp_bounds` - Boundary constraints ensuring windows remain accessible and visible.
///
/// # Returns
/// A [`FloatingInteractionResponse`] describing state changes, cursor feedback, and layout invalidation.
pub fn handle_floating_window_event<T: Clone + Copy + PartialEq, V: TabViewer<T>>(
    dock_state: &mut DockState<T>,
    viewer: &V,
    event: &UiEvent,
    clamp_bounds: FloatingWindowClampBounds,
) -> FloatingInteractionResponse<T> {
    match event {
        UiEvent::MouseMove { point } => {
            // 1. Process active floating drag (Title translation or Edge resize)
            if let Some((win_id, mode)) = dock_state.active_floating_drag {
                let Some(win) = dock_state
                    .floating_windows
                    .iter_mut()
                    .find(|w| w.id == win_id)
                else {
                    return FloatingInteractionResponse::empty();
                };

                match mode {
                    FloatingDragState::Title { offset } => {
                        win.rect.x = point.x - offset.x;
                        win.rect.y = point.y - offset.y;
                        win.clamp_to_bounds(&clamp_bounds);

                        return FloatingInteractionResponse {
                            cursor: Some(WidgetCursor::Grabbing),
                            drag_active: true,
                            consumed: true,
                            layout_changed: true,
                            ..FloatingInteractionResponse::empty()
                        };
                    }
                    FloatingDragState::Resize(edge) => {
                        let min_w = clamp_bounds.min_width;
                        let min_h = clamp_bounds.min_height;
                        let max_w = (clamp_bounds.screen_width - win.rect.x).max(min_w);
                        let max_h =
                            (clamp_bounds.screen_height - clamp_bounds.bottom_inset - win.rect.y)
                                .max(min_h);

                        match edge {
                            FloatingResizeEdge::Right => {
                                win.rect.width = (point.x - win.rect.x).clamp(min_w, max_w);
                            }
                            FloatingResizeEdge::Bottom => {
                                win.rect.height = (point.y - win.rect.y).clamp(min_h, max_h);
                            }
                            FloatingResizeEdge::Left => {
                                let old_right = win.rect.x + win.rect.width;
                                let min_x =
                                    (clamp_bounds.min_visible_margin - win.rect.width).min(0.0);
                                win.rect.x = point.x.clamp(min_x, old_right - min_w);
                                win.rect.width = old_right - win.rect.x;
                            }
                            FloatingResizeEdge::Top => {
                                let old_bottom = win.rect.y + win.rect.height;
                                let min_y = clamp_bounds.min_y;
                                win.rect.y = point.y.clamp(min_y, old_bottom - min_h);
                                win.rect.height = old_bottom - win.rect.y;
                            }
                            FloatingResizeEdge::BottomRight => {
                                win.rect.width = (point.x - win.rect.x).clamp(min_w, max_w);
                                win.rect.height = (point.y - win.rect.y).clamp(min_h, max_h);
                            }
                            FloatingResizeEdge::BottomLeft => {
                                let old_right = win.rect.x + win.rect.width;
                                let min_x =
                                    (clamp_bounds.min_visible_margin - win.rect.width).min(0.0);
                                win.rect.x = point.x.clamp(min_x, old_right - min_w);
                                win.rect.width = old_right - win.rect.x;
                                win.rect.height = (point.y - win.rect.y).clamp(min_h, max_h);
                            }
                            FloatingResizeEdge::TopRight => {
                                win.rect.width = (point.x - win.rect.x).clamp(min_w, max_w);
                                let old_bottom = win.rect.y + win.rect.height;
                                let min_y = clamp_bounds.min_y;
                                win.rect.y = point.y.clamp(min_y, old_bottom - min_h);
                                win.rect.height = old_bottom - win.rect.y;
                            }
                            FloatingResizeEdge::TopLeft => {
                                let old_right = win.rect.x + win.rect.width;
                                let min_x =
                                    (clamp_bounds.min_visible_margin - win.rect.width).min(0.0);
                                win.rect.x = point.x.clamp(min_x, old_right - min_w);
                                win.rect.width = old_right - win.rect.x;
                                let old_bottom = win.rect.y + win.rect.height;
                                let min_y = clamp_bounds.min_y;
                                win.rect.y = point.y.clamp(min_y, old_bottom - min_h);
                                win.rect.height = old_bottom - win.rect.y;
                            }
                        }

                        return FloatingInteractionResponse {
                            cursor: Some(edge.cursor().to_widget_cursor()),
                            drag_active: true,
                            consumed: true,
                            layout_changed: true,
                            ..FloatingInteractionResponse::empty()
                        };
                    }
                }
            }

            // 2. Hover feedback when idle (Resize cursors on borders, pointer on buttons)
            for win in dock_state.floating_windows.iter().rev() {
                if let Some(edge) = detect_resize_edge(win.rect, *point, DEFAULT_RESIZE_MARGIN) {
                    return FloatingInteractionResponse {
                        cursor: Some(edge.cursor().to_widget_cursor()),
                        consumed: false,
                        layout_changed: false,
                        ..FloatingInteractionResponse::empty()
                    };
                }

                let bar_rect = win.title_bar_rect(clamp_bounds.title_bar_height);
                if bar_rect.contains_point(*point) {
                    return FloatingInteractionResponse {
                        cursor: Some(WidgetCursor::Pointer),
                        consumed: false,
                        layout_changed: false,
                        ..FloatingInteractionResponse::empty()
                    };
                }
            }

            FloatingInteractionResponse::empty()
        }

        UiEvent::MouseDown {
            button: MouseButton::Left,
            point,
        } => {
            // 1. Evaluate control hits (DockBack, Close, TabSelected, TitleDrag, Resize)
            if let Some(action) = evaluate_floating_window_click(
                &dock_state.floating_windows,
                viewer,
                *point,
                DEFAULT_RESIZE_MARGIN,
                clamp_bounds.title_bar_height,
            ) {
                match action {
                    FloatingWindowClickAction::DockBack { window_id } => {
                        let fallback_leaf = dock_state.tree.find_first_leaf();
                        if let Some(leaf) = fallback_leaf {
                            let _ =
                                dock_state.dock_floating_window(window_id, leaf, DropZone::Center);
                        }
                        return FloatingInteractionResponse {
                            dock_back_window: Some(window_id),
                            consumed: true,
                            layout_changed: true,
                            ..FloatingInteractionResponse::empty()
                        };
                    }
                    FloatingWindowClickAction::Close { window_id } => {
                        let _ = dock_state.close_floating_window(window_id);
                        return FloatingInteractionResponse {
                            closed_window: Some(window_id),
                            consumed: true,
                            layout_changed: true,
                            ..FloatingInteractionResponse::empty()
                        };
                    }
                    FloatingWindowClickAction::TabSelected {
                        window_id,
                        leaf_id,
                        tab_index,
                        tab,
                        window_rect,
                    } => {
                        dock_state.bring_floating_to_front(window_id);
                        if let Some(w) = dock_state
                            .floating_windows
                            .iter_mut()
                            .find(|w| w.id == window_id)
                        {
                            let _ = w.tree.set_active_tab(leaf_id, tab_index);
                        }
                        return FloatingInteractionResponse {
                            pending_tab_drag: Some(PendingTabDrag::with_floating_window(
                                leaf_id,
                                tab_index,
                                tab,
                                *point,
                                window_rect,
                                window_id,
                            )),
                            cursor: Some(WidgetCursor::Pointer),
                            consumed: true,
                            layout_changed: true,
                            ..FloatingInteractionResponse::empty()
                        };
                    }
                    FloatingWindowClickAction::TitleDragStart { window_id, offset } => {
                        dock_state.bring_floating_to_front(window_id);
                        dock_state.active_floating_drag =
                            Some((window_id, FloatingDragState::Title { offset }));
                        return FloatingInteractionResponse {
                            drag_active: true,
                            cursor: Some(WidgetCursor::Grabbing),
                            consumed: true,
                            layout_changed: true,
                            ..FloatingInteractionResponse::empty()
                        };
                    }
                    FloatingWindowClickAction::ResizeStart { window_id, edge } => {
                        dock_state.bring_floating_to_front(window_id);
                        dock_state.active_floating_drag =
                            Some((window_id, FloatingDragState::Resize(edge)));
                        return FloatingInteractionResponse {
                            drag_active: true,
                            cursor: Some(edge.cursor().to_widget_cursor()),
                            consumed: true,
                            layout_changed: true,
                            ..FloatingInteractionResponse::empty()
                        };
                    }
                }
            }

            // 2. Clicking inside content body of a floating window brings it to front and focuses it
            if let Some(win_id) = dock_state
                .floating_windows
                .iter()
                .rev()
                .find(|w| w.rect.contains_point(*point))
                .map(|w| w.id)
            {
                dock_state.bring_floating_to_front(win_id);
                for w in &mut dock_state.floating_windows {
                    w.is_focused = w.id == win_id;
                }

                // Return layout_changed so Z-order is reflected, but consumed: false so child widgets receive the click
                return FloatingInteractionResponse {
                    consumed: false,
                    layout_changed: true,
                    ..FloatingInteractionResponse::empty()
                };
            }

            FloatingInteractionResponse::empty()
        }

        UiEvent::MouseUp {
            button: MouseButton::Left,
            point: _,
        } => {
            if dock_state.active_floating_drag.is_some() {
                dock_state.active_floating_drag = None;
                return FloatingInteractionResponse {
                    drag_active: false,
                    consumed: true,
                    layout_changed: true,
                    ..FloatingInteractionResponse::empty()
                };
            }

            FloatingInteractionResponse::empty()
        }

        UiEvent::KeyboardInput {
            key: KeyCode::Escape,
            is_pressed: true,
            ..
        } => {
            if dock_state.active_floating_drag.is_some() {
                dock_state.active_floating_drag = None;
                return FloatingInteractionResponse {
                    drag_active: false,
                    consumed: true,
                    layout_changed: true,
                    ..FloatingInteractionResponse::empty()
                };
            }

            FloatingInteractionResponse::empty()
        }

        _ => FloatingInteractionResponse::empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::floating::FloatingWindow;
    use crate::tree::DockTree;
    use iris_core::geometry::{Point, Rect};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestPanel {
        Hierarchy,
        Inspector,
    }

    struct TestViewer;
    impl TabViewer<TestPanel> for TestViewer {
        fn title(&self, tab: &TestPanel) -> String {
            match tab {
                TestPanel::Hierarchy => "Hierarchy".to_string(),
                TestPanel::Inspector => "Inspector".to_string(),
            }
        }

        fn ui(&mut self, _scope: &mut iris_widgets::UiScope<'_>, _tab: &TestPanel) {}
    }

    #[test]
    fn test_floating_title_drag_lifecycle_and_clamping() {
        let tree = DockTree::<TestPanel>::new();
        let mut dock_state = DockState::new(tree);

        let win = FloatingWindow::new(
            1,
            "Window 1",
            Rect::new(100.0, 100.0, 300.0, 200.0),
            vec![TestPanel::Hierarchy],
        );
        dock_state.floating_windows.push(win);

        let viewer = TestViewer;
        let bounds = FloatingWindowClampBounds::new(1920.0, 1080.0).with_min_y(30.0);

        // Click on title bar background (x=250.0, y=110.0)
        let click = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(250.0, 110.0),
        };
        let resp_click = handle_floating_window_event(&mut dock_state, &viewer, &click, bounds);
        assert!(resp_click.consumed);
        assert!(resp_click.drag_active);
        assert!(dock_state.active_floating_drag.is_some());

        // Move to top edge (y=10.0, which is above min_y=30.0)
        let move_top = UiEvent::MouseMove {
            point: Point::new(250.0, 10.0),
        };
        let resp_move = handle_floating_window_event(&mut dock_state, &viewer, &move_top, bounds);
        assert!(resp_move.consumed);
        assert_eq!(dock_state.floating_windows[0].rect.y, 30.0); // Clamped to min_y

        // Release mouse
        let release = UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(250.0, 10.0),
        };
        let resp_release = handle_floating_window_event(&mut dock_state, &viewer, &release, bounds);
        assert!(resp_release.consumed);
        assert!(dock_state.active_floating_drag.is_none());
    }

    #[test]
    fn test_floating_8way_resize() {
        let tree = DockTree::<TestPanel>::new();
        let mut dock_state = DockState::new(tree);

        let win = FloatingWindow::new(
            1,
            "Window 1",
            Rect::new(200.0, 200.0, 300.0, 200.0),
            vec![TestPanel::Hierarchy],
        );
        dock_state.floating_windows.push(win);

        let viewer = TestViewer;
        let bounds = FloatingWindowClampBounds::new(1920.0, 1080.0)
            .with_min_size(220.0, 140.0)
            .with_min_y(0.0);

        // Right resize
        let click_right = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(500.0, 250.0), // right edge of 200 + 300
        };
        let resp_right =
            handle_floating_window_event(&mut dock_state, &viewer, &click_right, bounds);
        assert!(resp_right.drag_active);

        let move_right = UiEvent::MouseMove {
            point: Point::new(550.0, 250.0),
        };
        handle_floating_window_event(&mut dock_state, &viewer, &move_right, bounds);
        assert_eq!(dock_state.floating_windows[0].rect.width, 350.0);

        // Release
        let up = UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(550.0, 250.0),
        };
        handle_floating_window_event(&mut dock_state, &viewer, &up, bounds);
        assert!(dock_state.active_floating_drag.is_none());

        // Bottom resize below min_height
        let click_bottom = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(300.0, 400.0), // bottom edge of 200 + 200
        };
        handle_floating_window_event(&mut dock_state, &viewer, &click_bottom, bounds);

        let move_bottom_shrink = UiEvent::MouseMove {
            point: Point::new(300.0, 220.0), // shrink height to 20px
        };
        handle_floating_window_event(&mut dock_state, &viewer, &move_bottom_shrink, bounds);
        assert_eq!(dock_state.floating_windows[0].rect.height, 140.0); // Clamped to min_height
    }

    #[test]
    fn test_floating_z_order_bring_to_front() {
        let tree = DockTree::<TestPanel>::new();
        let mut dock_state = DockState::new(tree);

        let win_1 = FloatingWindow::new(
            1,
            "Window 1",
            Rect::new(100.0, 100.0, 200.0, 200.0),
            vec![TestPanel::Hierarchy],
        );
        let win_2 = FloatingWindow::new(
            2,
            "Window 2",
            Rect::new(200.0, 200.0, 200.0, 200.0),
            vec![TestPanel::Inspector],
        );
        dock_state.floating_windows.push(win_1);
        dock_state.floating_windows.push(win_2);

        assert_eq!(dock_state.floating_windows[0].id, 1);
        assert_eq!(dock_state.floating_windows[1].id, 2);

        let viewer = TestViewer;
        let bounds = FloatingWindowClampBounds::new(1920.0, 1080.0);

        // Click inside Window 1 body (x=150.0, y=150.0)
        let click_win1 = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(150.0, 150.0),
        };
        let resp = handle_floating_window_event(&mut dock_state, &viewer, &click_win1, bounds);

        assert!(!resp.consumed); // Body click allows child widgets to receive event
        assert!(resp.layout_changed);

        // Window 1 should now be at the end (top of stack)
        assert_eq!(dock_state.floating_windows[0].id, 2);
        assert_eq!(dock_state.floating_windows[1].id, 1);
        assert!(dock_state.floating_windows[1].is_focused);
    }

    #[test]
    fn test_floating_close_button() {
        let tree = DockTree::<TestPanel>::new();
        let mut dock_state = DockState::new(tree);

        let win = FloatingWindow::new(
            10,
            "Window 10",
            Rect::new(100.0, 100.0, 300.0, 200.0),
            vec![TestPanel::Hierarchy],
        );
        dock_state.floating_windows.push(win);

        let viewer = TestViewer;
        let bounds = FloatingWindowClampBounds::new(1920.0, 1080.0);

        // Close button `✖` is at right - 32.0 px (400 - 32 = 368.0 to 388.0, y=102.0)
        let close_click = UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(375.0, 110.0),
        };
        let resp = handle_floating_window_event(&mut dock_state, &viewer, &close_click, bounds);

        assert!(resp.consumed);
        assert_eq!(resp.closed_window, Some(10));
        assert!(dock_state.floating_windows.is_empty());
    }

    #[test]
    fn test_floating_hover_resize_cursor() {
        let tree = DockTree::<TestPanel>::new();
        let mut dock_state = DockState::new(tree);

        let win = FloatingWindow::new(
            1,
            "Window 1",
            Rect::new(100.0, 100.0, 300.0, 200.0),
            vec![TestPanel::Hierarchy],
        );
        dock_state.floating_windows.push(win);

        let viewer = TestViewer;
        let bounds = FloatingWindowClampBounds::new(1920.0, 1080.0);

        // Hover over bottom-right corner (x=400.0, y=300.0)
        let hover_corner = UiEvent::MouseMove {
            point: Point::new(398.0, 298.0),
        };
        let resp = handle_floating_window_event(&mut dock_state, &viewer, &hover_corner, bounds);

        assert_eq!(resp.cursor, Some(WidgetCursor::NwseResize));
    }
}