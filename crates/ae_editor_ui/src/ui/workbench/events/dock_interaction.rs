// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Native Dock & Floating Window Event Handling
//!
//! Handles splitter dividers, floating window moves/resizes, and tab docking interactions.

use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::workbench::state::EngineUi;
use irisui::dock::SplitDirection;
use irisui::dock::{FloatingDragState, FloatingResizeEdge};
use irisui::prelude::{Point, Rect};
use winit::event::{ElementState, MouseButton, WindowEvent};

impl EngineUi {
    /// Processes cursor motions for docking splitters, tab drags, and floating window drag/resizing.
    pub(crate) fn handle_dock_cursor_moved(
        &mut self,
        p: Point,
        screen_w: f32,
        screen_h: f32,
        workspace_rect: Rect,
    ) -> bool {
        let mut dock_consumed = false;

        // 0. Activate pending tab drag if cursor motion exceeds threshold (4.0 px)
        if let Some(pending) = self.pending_tab_drag {
            let dx = p.x - pending.press_pos.x;
            let dy = p.y - pending.press_pos.y;
            if dx * dx + dy * dy >= 16.0 {
                if let Some(win_id) = pending.floating_window_id {
                    let _ = self.layout_state.dock_state.start_floating_tab_drag(
                        win_id,
                        pending.leaf,
                        pending.tab_index,
                        p,
                    );
                } else {
                    let _ = self.layout_state.dock_state.start_tab_drag(
                        pending.leaf,
                        pending.tab_index,
                        p,
                        pending.leaf_rect,
                    );
                }
                self.pending_tab_drag = None;
                dock_consumed = true;
            }
        }

        // 1. Floating window title bar dragging or edge resizing
        if let Some((win_id, mode)) = self.active_floating_drag {
            if let Some(win) = self
                .layout_state
                .dock_state
                .floating_windows
                .iter_mut()
                .find(|w| w.id == win_id)
            {
                match mode {
                    FloatingDragState::Title { offset } => {
                        let target_x = p.x - offset.x;
                        let target_y = p.y - offset.y;
                        const TAB_BAR_H: f32 = 26.0;
                        let min_y = IrisEditorOverlay::MENUBAR_HEIGHT;
                        let max_y = (screen_h - IrisEditorOverlay::STATUS_BAR_HEIGHT - TAB_BAR_H)
                            .max(min_y);
                        win.rect.y = target_y.clamp(min_y, max_y);

                        let max_x = (screen_w - 60.0).max(0.0);
                        let min_x = (60.0 - win.rect.width).min(0.0);
                        win.rect.x = target_x.clamp(min_x, max_x);
                    }
                    FloatingDragState::Resize(edge) => match edge {
                        FloatingResizeEdge::Right => {
                            let max_w = (screen_w - win.rect.x).max(220.0);
                            win.rect.width = (p.x - win.rect.x).clamp(220.0, max_w);
                        }
                        FloatingResizeEdge::Bottom => {
                            let max_h =
                                (screen_h - IrisEditorOverlay::STATUS_BAR_HEIGHT - win.rect.y)
                                    .max(140.0);
                            win.rect.height = (p.y - win.rect.y).clamp(140.0, max_h);
                        }
                        FloatingResizeEdge::Left => {
                            let old_right = win.rect.x + win.rect.width;
                            let min_x = (60.0 - win.rect.width).min(0.0);
                            win.rect.x = p.x.clamp(min_x, old_right - 220.0);
                            win.rect.width = old_right - win.rect.x;
                        }
                        FloatingResizeEdge::Top => {
                            let old_bottom = win.rect.y + win.rect.height;
                            let min_y = IrisEditorOverlay::MENUBAR_HEIGHT;
                            win.rect.y = p.y.clamp(min_y, old_bottom - 140.0);
                            win.rect.height = old_bottom - win.rect.y;
                        }
                        FloatingResizeEdge::BottomRight => {
                            let max_w = (screen_w - win.rect.x).max(220.0);
                            let max_h =
                                (screen_h - IrisEditorOverlay::STATUS_BAR_HEIGHT - win.rect.y)
                                    .max(140.0);
                            win.rect.width = (p.x - win.rect.x).clamp(220.0, max_w);
                            win.rect.height = (p.y - win.rect.y).clamp(140.0, max_h);
                        }
                        FloatingResizeEdge::BottomLeft => {
                            let old_right = win.rect.x + win.rect.width;
                            let min_x = (60.0 - win.rect.width).min(0.0);
                            win.rect.x = p.x.clamp(min_x, old_right - 220.0);
                            win.rect.width = old_right - win.rect.x;
                            let max_h =
                                (screen_h - IrisEditorOverlay::STATUS_BAR_HEIGHT - win.rect.y)
                                    .max(140.0);
                            win.rect.height = (p.y - win.rect.y).clamp(140.0, max_h);
                        }
                        FloatingResizeEdge::TopRight => {
                            let max_w = (screen_w - win.rect.x).max(220.0);
                            win.rect.width = (p.x - win.rect.x).clamp(220.0, max_w);
                            let old_bottom = win.rect.y + win.rect.height;
                            let min_y = IrisEditorOverlay::MENUBAR_HEIGHT;
                            win.rect.y = p.y.clamp(min_y, old_bottom - 140.0);
                            win.rect.height = old_bottom - win.rect.y;
                        }
                        FloatingResizeEdge::TopLeft => {
                            let old_right = win.rect.x + win.rect.width;
                            let min_x = (60.0 - win.rect.width).min(0.0);
                            win.rect.x = p.x.clamp(min_x, old_right - 220.0);
                            win.rect.width = old_right - win.rect.x;
                            let old_bottom = win.rect.y + win.rect.height;
                            let min_y = IrisEditorOverlay::MENUBAR_HEIGHT;
                            win.rect.y = p.y.clamp(min_y, old_bottom - 140.0);
                            win.rect.height = old_bottom - win.rect.y;
                        }
                    },
                }
            }
            dock_consumed = true;
        }

        // 2. Active splitter divider dragging
        if let Some(drag) = self.layout_state.dock_state.active_splitter {
            let current_cursor = match drag.direction {
                SplitDirection::Horizontal => p.x,
                SplitDirection::Vertical => p.y,
            };
            self.layout_state
                .dock_state
                .update_splitter_drag(current_cursor);
            dock_consumed = true;
        }

        // 3. Active tab drag-and-drop
        if self.layout_state.dock_state.active_drag.is_some() {
            let computed = irisui::dock::compute_dock_layout_with_viewer(
                &self.layout_state.dock_state.tree,
                workspace_rect,
                crate::ui::iris_bridge::native_dock::SPLITTER_THICKNESS,
                crate::ui::iris_bridge::native_dock::NATIVE_DOCK_TAB_HEIGHT,
                &crate::ui::panel_layout::PanelTabViewer,
            );
            self.layout_state.dock_state.update_tab_drag(p, &computed);
            dock_consumed = true;
        }

        dock_consumed
    }

    /// Handles mouse button press and release events on dock elements, splitters, and floating windows.
    pub(crate) fn handle_dock_mouse_input(&mut self, p: Point, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                // 0. If click is over an active floating modal, underlying dock is protected
                if self.iris_overlay.is_point_over_modal_or_dropdown(p) {
                    return true;
                }

                // 1. Check Floating Window controls
                if let Some(action) = irisui::dock::evaluate_floating_window_click(
                    &self.layout_state.dock_state.floating_windows,
                    &crate::ui::panel_layout::PanelTabViewer,
                    p,
                    6.0,
                    26.0,
                ) {
                    match action {
                        irisui::dock::FloatingWindowClickAction::DockBack { window_id } => {
                            self.layout_state.smart_dock_back_panel(window_id);
                            return true;
                        }
                        irisui::dock::FloatingWindowClickAction::Close { window_id } => {
                            let _ = self
                                .layout_state
                                .dock_state
                                .close_floating_window(window_id);
                            return true;
                        }
                        irisui::dock::FloatingWindowClickAction::TabSelected {
                            window_id,
                            leaf_id,
                            tab_index,
                            tab,
                            window_rect,
                        } => {
                            if let Some(w) = self
                                .layout_state
                                .dock_state
                                .floating_windows
                                .iter_mut()
                                .find(|w| w.id == window_id)
                            {
                                let _ = w.tree.set_active_tab(leaf_id, tab_index);
                            }
                            self.pending_tab_drag =
                                Some(crate::ui::workbench::state::PendingTabDrag {
                                    leaf: leaf_id,
                                    tab_index,
                                    panel: tab,
                                    press_pos: p,
                                    leaf_rect: window_rect,
                                    floating_window_id: Some(window_id),
                                });
                            return true;
                        }
                        irisui::dock::FloatingWindowClickAction::TitleDragStart {
                            window_id,
                            offset,
                        } => {
                            self.active_floating_drag =
                                Some((window_id, FloatingDragState::Title { offset }));
                            return true;
                        }
                        irisui::dock::FloatingWindowClickAction::ResizeStart {
                            window_id,
                            edge,
                        } => {
                            self.active_floating_drag =
                                Some((window_id, FloatingDragState::Resize(edge)));
                            return true;
                        }
                    }
                }

                // 2. Check Native Dock Frame targets (Tabs, Close buttons, Splitters)
                let is_over_floating =
                    self.layout_state
                        .dock_state
                        .floating_windows
                        .iter()
                        .any(|w| {
                            Rect::new(w.rect.x, w.rect.y, w.rect.width, w.rect.height)
                                .contains_point(p)
                        });

                if !is_over_floating
                    && let Some(ref frame) = self.iris_overlay.chrome.native_dock_frame
                {
                    // Tab overflow items
                    if let Some(target) = frame
                        .overflow_item_targets
                        .iter()
                        .find(|t| t.rect.contains_point(p))
                    {
                        let _ = self
                            .layout_state
                            .dock_state
                            .tree
                            .set_active_tab(target.leaf, target.tab_index);
                        self.iris_overlay.chrome.active_dock_overflow = None;
                        self.iris_overlay.chrome.needs_layout_rebuild = true;
                        return true;
                    }
                    // Tab overflow chevron buttons
                    if let Some(target) = frame
                        .chevron_targets
                        .iter()
                        .find(|t| t.rect.contains_point(p))
                    {
                        if self
                            .iris_overlay
                            .chrome
                            .active_dock_overflow
                            .as_ref()
                            .is_some_and(|(l, _)| *l == target.leaf)
                        {
                            self.iris_overlay.chrome.active_dock_overflow = None;
                        } else {
                            self.iris_overlay.chrome.active_dock_overflow =
                                Some((target.leaf, target.rect));
                        }
                        self.iris_overlay.chrome.needs_layout_rebuild = true;
                        return true;
                    }
                    // Close buttons
                    if let Some(target) = frame
                        .close_targets
                        .iter()
                        .find(|t| t.rect.contains_point(p))
                    {
                        self.layout_state.close_tab(target.leaf, target.tab_index);
                        self.iris_overlay.chrome.active_dock_overflow = None;
                        return true;
                    }
                    // Tab pills
                    if let Some(target) =
                        frame.tab_targets.iter().find(|t| t.rect.contains_point(p))
                    {
                        let _ = self
                            .layout_state
                            .dock_state
                            .tree
                            .set_active_tab(target.leaf, target.tab_index);
                        self.pending_tab_drag = Some(crate::ui::workbench::state::PendingTabDrag {
                            leaf: target.leaf,
                            tab_index: target.tab_index,
                            panel: target.tab,
                            press_pos: p,
                            leaf_rect: target.leaf_rect,
                            floating_window_id: None,
                        });
                        self.iris_overlay.chrome.active_dock_overflow = None;
                        return true;
                    }
                    // Splitters
                    if let Some(target) = frame
                        .splitter_targets
                        .iter()
                        .find(|t| t.rect.contains_point(p))
                    {
                        let start_coord = match target.direction {
                            SplitDirection::Horizontal => p.x,
                            SplitDirection::Vertical => p.y,
                        };
                        self.layout_state.dock_state.start_splitter_drag(
                            target.node,
                            target.direction,
                            start_coord,
                            target.total_dimension,
                        );
                        self.iris_overlay.chrome.active_dock_overflow = None;
                        return true;
                    }

                    if self.iris_overlay.chrome.active_dock_overflow.is_some() {
                        self.iris_overlay.chrome.active_dock_overflow = None;
                        self.iris_overlay.chrome.needs_layout_rebuild = true;
                    }
                }
            }

            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.pending_tab_drag = None;

                if self.active_floating_drag.is_some() {
                    self.active_floating_drag = None;
                    return true;
                }
                if self.layout_state.dock_state.active_splitter.is_some() {
                    self.layout_state.dock_state.end_splitter_drag();
                    return true;
                }
                if self.layout_state.dock_state.active_drag.is_some() {
                    let _ = self
                        .layout_state
                        .dock_state
                        .drop_tab_or_float(Point::new(400.0, 300.0));
                    return true;
                }
            }

            _ => {}
        }

        false
    }
}