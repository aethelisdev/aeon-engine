// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Autonomous floating window manager and interaction state machine (`FloatingWindowManager`).
//!
//! Provides coordinate boundary clamping, Z-order elevation on interaction, title bar
//! dragging, and 8-way border resizing without manual host-side coordination.

use crate::events::floating::FloatingInteractionResponse;
use crate::floating::bounds::FloatingWindowClampBounds;
use crate::floating::cursor::{DEFAULT_RESIZE_MARGIN, FloatingResizeEdge, detect_resize_edge};
use crate::floating::model::{
    FloatingDragState, FloatingWindow, FloatingWindowClickAction, evaluate_floating_window_click,
};
use crate::tab_viewer::TabViewer;
use iris_core::event::{KeyCode, MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect};
use iris_core::node::WidgetCursor;

/// Minimum height of the title bar kept visible on screen in logical pixels.
pub const MIN_VISIBLE_TITLE_BAR: f32 = 32.0;

/// Autonomous manager and interaction state machine governing independent floating windows.
///
/// Encapsulates window collection storage, Z-order elevation on click,
/// boundary clamping constraints, and interactive mouse drag/resize state machines.
#[derive(Debug, Clone)]
pub struct FloatingWindowManager<T> {
    /// Active collection of floating windows, ordered by visual Z-stacking (last element is topmost).
    windows: Vec<FloatingWindow<T>>,
    /// Active drag or resize state `(window_id, mode)`.
    active_drag: Option<(u64, FloatingDragState)>,
    /// Screen and workspace constraint bounds.
    clamp_bounds: FloatingWindowClampBounds,
    /// Next window ID generator counter.
    next_id: u64,
}

impl<T> Default for FloatingWindowManager<T> {
    fn default() -> Self {
        let mut bounds = FloatingWindowClampBounds::default();
        bounds.title_bar_height = bounds.title_bar_height.max(MIN_VISIBLE_TITLE_BAR);
        Self::new(bounds)
    }
}

impl<T> FloatingWindowManager<T> {
    /// Creates a new empty `FloatingWindowManager` with the specified clamp bounds.
    ///
    /// Guarantees that `clamp_bounds.title_bar_height` is at least [`MIN_VISIBLE_TITLE_BAR`] (32px).
    pub fn new(mut clamp_bounds: FloatingWindowClampBounds) -> Self {
        clamp_bounds.title_bar_height = clamp_bounds.title_bar_height.max(MIN_VISIBLE_TITLE_BAR);
        Self {
            windows: Vec::new(),
            active_drag: None,
            clamp_bounds,
            next_id: 1,
        }
    }

    /// Attaches an existing collection of floating windows, clamping each to bounds.
    #[must_use]
    pub fn with_windows(mut self, windows: Vec<FloatingWindow<T>>) -> Self {
        for win in windows {
            self.add_window(win);
        }
        self
    }

    /// Returns the active clamp bounds.
    #[inline]
    #[must_use]
    pub fn clamp_bounds(&self) -> &FloatingWindowClampBounds {
        &self.clamp_bounds
    }

    /// Sets new clamp bounds and immediately clamps all managed windows.
    pub fn set_clamp_bounds(&mut self, mut bounds: FloatingWindowClampBounds) {
        bounds.title_bar_height = bounds.title_bar_height.max(MIN_VISIBLE_TITLE_BAR);
        self.clamp_bounds = bounds;
        self.clamp_all();
    }

    /// Clamps all managed floating windows to remain within the active clamp bounds.
    ///
    /// Guarantees that at least [`MIN_VISIBLE_TITLE_BAR`] (32px) remains visible.
    pub fn clamp_all(&mut self) {
        for win in &mut self.windows {
            win.clamp_to_bounds(&self.clamp_bounds);
        }
    }

    /// Returns an immutable slice of all managed floating windows in visual Z-order.
    ///
    /// The last element in the slice represents the topmost window.
    #[inline]
    #[must_use]
    pub fn windows(&self) -> &[FloatingWindow<T>] {
        &self.windows
    }

    /// Returns a mutable slice of all managed floating windows.
    #[inline]
    pub fn windows_mut(&mut self) -> &mut [FloatingWindow<T>] {
        &mut self.windows
    }

    /// Consumes this manager and returns the owned vector of floating windows.
    #[inline]
    #[must_use]
    pub fn into_windows(self) -> Vec<FloatingWindow<T>> {
        self.windows
    }

    /// Returns the number of managed floating windows.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.windows.len()
    }

    /// Returns `true` if there are no managed floating windows.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Looks up a floating window by its unique identifier.
    #[inline]
    #[must_use]
    pub fn get(&self, window_id: u64) -> Option<&FloatingWindow<T>> {
        self.windows.iter().find(|w| w.id == window_id)
    }

    /// Looks up a floating window mutably by its unique identifier.
    #[inline]
    pub fn get_mut(&mut self, window_id: u64) -> Option<&mut FloatingWindow<T>> {
        self.windows.iter_mut().find(|w| w.id == window_id)
    }

    /// Adds a floating window to the manager, clamping it to bounds and elevating it to the top.
    ///
    /// If the window's ID is 0, assigns a unique auto-incremented ID.
    pub fn add_window(&mut self, mut window: FloatingWindow<T>) -> u64 {
        if window.id == 0 {
            window.id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1);
        } else {
            self.next_id = self.next_id.max(window.id.wrapping_add(1));
        }

        window.clamp_to_bounds(&self.clamp_bounds);
        let id = window.id;
        self.windows.push(window);
        self.bring_to_front(id);
        id
    }

    /// Constructs and registers a new floating window hosting the specified tabs.
    pub fn open_window(&mut self, title: impl Into<String>, rect: Rect, tabs: Vec<T>) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let window = FloatingWindow::new(id, title, rect, tabs);
        self.add_window(window)
    }

    /// Removes and closes a floating window by ID.
    ///
    /// If the closed window was currently being dragged, the drag state is cleared.
    /// If other windows remain, the new topmost window receives focus.
    pub fn close_window(&mut self, window_id: u64) -> Option<FloatingWindow<T>> {
        if self.active_drag.is_some_and(|(id, _)| id == window_id) {
            self.active_drag = None;
        }

        let pos = self.windows.iter().position(|w| w.id == window_id)?;
        let win = self.windows.remove(pos);

        if let Some(topmost) = self.windows.last_mut() {
            topmost.is_focused = true;
        }

        Some(win)
    }

    /// Brings the specified floating window to the front of the visual stacking order (Z-order).
    ///
    /// Moves the window to the end of the collection and sets its `is_focused` flag to `true`,
    /// while clearing `is_focused` on all other windows.
    ///
    /// Returns `true` if the window was found and brought to front.
    pub fn bring_to_front(&mut self, window_id: u64) -> bool {
        let Some(pos) = self.windows.iter().position(|w| w.id == window_id) else {
            return false;
        };

        let win = self.windows.remove(pos);
        self.windows.push(win);

        for w in &mut self.windows {
            w.is_focused = w.id == window_id;
        }

        true
    }

    /// Returns the currently active drag or resize state `(window_id, mode)`, if any.
    #[inline]
    #[must_use]
    pub fn active_drag(&self) -> Option<(u64, FloatingDragState)> {
        self.active_drag
    }

    /// Returns `true` if a floating window is currently being dragged or resized.
    #[inline]
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.active_drag.is_some()
    }

    /// Initiates a title bar translation drag for the specified window.
    ///
    /// Automatically brings the window to the front.
    pub fn start_title_drag(&mut self, window_id: u64, offset: Point) {
        self.bring_to_front(window_id);
        self.active_drag = Some((window_id, FloatingDragState::Title { offset }));
    }

    /// Initiates a border resize drag for the specified window along the given edge.
    ///
    /// Automatically brings the window to the front.
    pub fn start_resize(&mut self, window_id: u64, edge: FloatingResizeEdge) {
        self.bring_to_front(window_id);
        self.active_drag = Some((window_id, FloatingDragState::Resize(edge)));
    }

    /// Finalizes the active drag or resize operation, clearing active state.
    ///
    /// Returns the completed drag state if one was active.
    pub fn end_drag(&mut self) -> Option<(u64, FloatingDragState)> {
        self.active_drag.take()
    }

    /// Cancels the active drag or resize operation without applying further updates.
    pub fn cancel_drag(&mut self) -> Option<(u64, FloatingDragState)> {
        self.active_drag.take()
    }

    /// Updates window position or dimensions based on cursor position during an active drag.
    ///
    /// Clamps coordinates to ensure the window remains accessible within the workspace.
    /// Returns `true` if a window was updated and layout changed.
    pub fn update_drag(&mut self, cursor: Point) -> bool {
        let Some((win_id, mode)) = self.active_drag else {
            return false;
        };

        let Some(win) = self.windows.iter_mut().find(|w| w.id == win_id) else {
            return false;
        };

        match mode {
            FloatingDragState::Title { offset } => {
                win.rect.x = cursor.x - offset.x;
                win.rect.y = cursor.y - offset.y;
                win.clamp_to_bounds(&self.clamp_bounds);
                true
            }
            FloatingDragState::Resize(edge) => {
                Self::apply_resize(win, edge, cursor, &self.clamp_bounds);
                true
            }
        }
    }

    /// Internal helper calculating edge resize delta and clamping constraints.
    fn apply_resize(
        win: &mut FloatingWindow<T>,
        edge: FloatingResizeEdge,
        point: Point,
        bounds: &FloatingWindowClampBounds,
    ) {
        let min_w = bounds.min_width;
        let min_h = bounds.min_height;
        let max_w = (bounds.screen_width - win.rect.x).max(min_w);
        let max_h = (bounds.screen_height - bounds.bottom_inset - win.rect.y).max(min_h);

        match edge {
            FloatingResizeEdge::Right => {
                win.rect.width = (point.x - win.rect.x).clamp(min_w, max_w);
            }
            FloatingResizeEdge::Bottom => {
                win.rect.height = (point.y - win.rect.y).clamp(min_h, max_h);
            }
            FloatingResizeEdge::Left => {
                let old_right = win.rect.x + win.rect.width;
                let min_x = (bounds.min_visible_margin - win.rect.width).min(0.0);
                win.rect.x = point.x.clamp(min_x, old_right - min_w);
                win.rect.width = old_right - win.rect.x;
            }
            FloatingResizeEdge::Top => {
                let old_bottom = win.rect.y + win.rect.height;
                let min_y = bounds.min_y;
                win.rect.y = point.y.clamp(min_y, old_bottom - min_h);
                win.rect.height = old_bottom - win.rect.y;
            }
            FloatingResizeEdge::BottomRight => {
                win.rect.width = (point.x - win.rect.x).clamp(min_w, max_w);
                win.rect.height = (point.y - win.rect.y).clamp(min_h, max_h);
            }
            FloatingResizeEdge::BottomLeft => {
                let old_right = win.rect.x + win.rect.width;
                let min_x = (bounds.min_visible_margin - win.rect.width).min(0.0);
                win.rect.x = point.x.clamp(min_x, old_right - min_w);
                win.rect.width = old_right - win.rect.x;
                win.rect.height = (point.y - win.rect.y).clamp(min_h, max_h);
            }
            FloatingResizeEdge::TopRight => {
                win.rect.width = (point.x - win.rect.x).clamp(min_w, max_w);
                let old_bottom = win.rect.y + win.rect.height;
                let min_y = bounds.min_y;
                win.rect.y = point.y.clamp(min_y, old_bottom - min_h);
                win.rect.height = old_bottom - win.rect.y;
            }
            FloatingResizeEdge::TopLeft => {
                let old_right = win.rect.x + win.rect.width;
                let min_x = (bounds.min_visible_margin - win.rect.width).min(0.0);
                win.rect.x = point.x.clamp(min_x, old_right - min_w);
                win.rect.width = old_right - win.rect.x;
                let old_bottom = win.rect.y + win.rect.height;
                let min_y = bounds.min_y;
                win.rect.y = point.y.clamp(min_y, old_bottom - min_h);
                win.rect.height = old_bottom - win.rect.y;
            }
        }
    }

    /// Returns the topmost floating window ID containing the specified screen coordinates, if any.
    ///
    /// Evaluates windows in reverse Z-order (topmost window first).
    #[inline]
    #[must_use]
    pub fn hit_test(&self, point: Point) -> Option<u64> {
        self.windows
            .iter()
            .rev()
            .find(|w| w.rect.contains_point(point))
            .map(|w| w.id)
    }

    /// Checks whether the specified screen coordinates fall inside any managed floating window.
    #[inline]
    #[must_use]
    pub fn is_point_over_floating(&self, point: Point) -> bool {
        self.hit_test(point).is_some()
    }

    /// Collects bounding rectangles of all managed floating windows in visual Z-order.
    #[inline]
    #[must_use]
    pub fn window_rects(&self) -> Vec<Rect> {
        self.windows.iter().map(|w| w.rect).collect()
    }

    /// Evaluates a mouse click against floating window controls, tabs, and headers.
    ///
    /// Automatically brings the clicked window to the front.
    pub fn evaluate_click<V: TabViewer<T>>(
        &mut self,
        viewer: &V,
        point: Point,
        resize_margin: f32,
    ) -> Option<FloatingWindowClickAction<T>>
    where
        T: Clone,
    {
        let action = evaluate_floating_window_click(
            &self.windows,
            viewer,
            point,
            resize_margin,
            self.clamp_bounds.title_bar_height,
        )?;

        match &action {
            FloatingWindowClickAction::DockBack { window_id }
            | FloatingWindowClickAction::Close { window_id }
            | FloatingWindowClickAction::TabSelected { window_id, .. }
            | FloatingWindowClickAction::TitleDragStart { window_id, .. }
            | FloatingWindowClickAction::ResizeStart { window_id, .. } => {
                self.bring_to_front(*window_id);
            }
        }

        Some(action)
    }

    /// Processes an interaction event autonomously across all managed floating windows.
    ///
    /// Manages hover cursors, title bar dragging, boundary margin resizing,
    /// tab switching, and Z-order elevation on click.
    pub fn handle_event<V: TabViewer<T>>(
        &mut self,
        viewer: &V,
        event: &UiEvent,
    ) -> FloatingInteractionResponse<T>
    where
        T: Clone + Copy + PartialEq,
    {
        match event {
            UiEvent::MouseMove { point } => {
                // 1. Process active floating drag (Title translation or Edge resize)
                if self.is_dragging() {
                    let changed = self.update_drag(*point);
                    let cursor = match self.active_drag {
                        Some((_, FloatingDragState::Title { .. })) => Some(WidgetCursor::Grabbing),
                        Some((_, FloatingDragState::Resize(edge))) => {
                            Some(edge.cursor().to_widget_cursor())
                        }
                        None => None,
                    };
                    return FloatingInteractionResponse {
                        cursor,
                        drag_active: true,
                        consumed: true,
                        layout_changed: changed,
                        ..FloatingInteractionResponse::empty()
                    };
                }

                // 2. Hover feedback when idle (Resize cursors on borders, pointer on buttons)
                for win in self.windows.iter().rev() {
                    if let Some(edge) = detect_resize_edge(win.rect, *point, DEFAULT_RESIZE_MARGIN)
                    {
                        return FloatingInteractionResponse {
                            cursor: Some(edge.cursor().to_widget_cursor()),
                            consumed: false,
                            layout_changed: false,
                            ..FloatingInteractionResponse::empty()
                        };
                    }

                    let bar_rect = win.title_bar_rect(self.clamp_bounds.title_bar_height);
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
                if let Some(action) = self.evaluate_click(viewer, *point, DEFAULT_RESIZE_MARGIN) {
                    match action {
                        FloatingWindowClickAction::DockBack { window_id } => {
                            return FloatingInteractionResponse {
                                dock_back_window: Some(window_id),
                                consumed: true,
                                layout_changed: true,
                                ..FloatingInteractionResponse::empty()
                            };
                        }
                        FloatingWindowClickAction::Close { window_id } => {
                            self.close_window(window_id);
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
                            if let Some(w) = self.get_mut(window_id) {
                                let _ = w.tree.set_active_tab(leaf_id, tab_index);
                            }
                            return FloatingInteractionResponse {
                                pending_tab_drag: Some(
                                    crate::events::PendingTabDrag::with_floating_window(
                                        leaf_id,
                                        tab_index,
                                        tab,
                                        *point,
                                        window_rect,
                                        window_id,
                                    ),
                                ),
                                cursor: Some(WidgetCursor::Pointer),
                                consumed: true,
                                layout_changed: true,
                                ..FloatingInteractionResponse::empty()
                            };
                        }
                        FloatingWindowClickAction::TitleDragStart { window_id, offset } => {
                            self.start_title_drag(window_id, offset);
                            return FloatingInteractionResponse {
                                drag_active: true,
                                cursor: Some(WidgetCursor::Grabbing),
                                consumed: true,
                                layout_changed: true,
                                ..FloatingInteractionResponse::empty()
                            };
                        }
                        FloatingWindowClickAction::ResizeStart { window_id, edge } => {
                            self.start_resize(window_id, edge);
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
                if let Some(win_id) = self.hit_test(*point) {
                    self.bring_to_front(win_id);
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
                ..
            } => {
                if self.is_dragging() {
                    self.end_drag();
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
                if self.is_dragging() {
                    self.cancel_drag();
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
}

#[cfg(test)]
mod tests;