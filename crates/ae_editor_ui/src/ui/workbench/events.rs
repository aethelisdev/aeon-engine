// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::workbench::state::{EngineUi, FloatingDragState, FloatingResizeEdge};
use irisui::dock::SplitDirection;
use irisui::prelude::{Point, Rect};
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    window::Window,
};

impl EngineUi {
    /// Forwards winit window events to Iris UI and the native dock coordinator.
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        // Synchronize active floating window boundaries with IrisEditorOverlay for occlusion testing
        self.iris_overlay.floating_window_rects = self
            .layout_state
            .dock_state
            .floating_windows
            .iter()
            .map(|w| Rect::new(w.rect.x, w.rect.y, w.rect.width, w.rect.height))
            .collect();

        let zoom = self.scale_factor();
        let scaled_event;
        let event_ref = match event {
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                scaled_event = WindowEvent::CursorMoved {
                    device_id: *device_id,
                    position: winit::dpi::PhysicalPosition::new(
                        position.x / (zoom as f64),
                        position.y / (zoom as f64),
                    ),
                };
                &scaled_event
            }
            other => other,
        };

        let iris_res = self.iris_overlay.handle_event(event_ref);
        if let Some(act) = iris_res.ui_action {
            self.pending_actions.push(act);
        }
        if let Some(panel) = iris_res.toggle_panel {
            self.layout_state.activate_or_open(panel);
        }
        if iris_res.reset_layout {
            self.layout_state.reset_to_default();
        }
        if iris_res.open_preferences {
            self.show_preferences = true;
        }
        if iris_res.close_preferences {
            self.show_preferences = false;
        }
        if let Some(pref_act) = iris_res.preferences_action {
            if let crate::ui::iris_bridge::PreferencesAction::SetUiScale(s) = pref_act {
                self.ui_zoom_factor = s;
            }
            self.pending_preferences_actions.push(pref_act);
        }
        if iris_res.open_about {
            self.show_about = true;
        }
        if iris_res.close_about {
            self.show_about = false;
        }
        if iris_res.confirm_delete
            && let Some(target) = self.asset_browser.delete_confirmation.take()
        {
            let _ = crate::ui::panels::assets::file_ops::delete_asset_or_folder(&target);
            if self.asset_browser.selected_asset.as_ref() == Some(&target) {
                self.asset_browser.selected_asset = None;
            }
            self.asset_browser.revision = self.asset_browser.revision.wrapping_add(1);
        }
        if iris_res.cancel_delete {
            self.asset_browser.delete_confirmation = None;
            self.asset_browser.revision = self.asset_browser.revision.wrapping_add(1);
        }

        if let Some(folder_name) = iris_res.create_folder
            && let Some(parent) = self.asset_browser.new_folder_parent.take()
        {
            let _ = crate::ui::panels::assets::file_ops::create_subfolder(&parent, &folder_name);
            self.iris_overlay.new_folder_buffer.clear();
            self.asset_browser.new_folder_name.clear();
            self.asset_browser.revision = self.asset_browser.revision.wrapping_add(1);
        }
        if iris_res.cancel_new_folder {
            self.asset_browser.new_folder_parent = None;
            self.iris_overlay.new_folder_buffer.clear();
            self.asset_browser.new_folder_name.clear();
            self.asset_browser.revision = self.asset_browser.revision.wrapping_add(1);
        }

        if let Some(new_name) = iris_res.apply_rename
            && let Some(ren) = self.asset_browser.rename_state.take()
        {
            let _ = crate::ui::panels::assets::file_ops::rename_asset_or_folder(
                &ren.target_path,
                &new_name,
            );
            self.iris_overlay.rename_buffer.clear();
            self.asset_browser.revision = self.asset_browser.revision.wrapping_add(1);
        }
        if iris_res.cancel_rename {
            self.asset_browser.rename_state = None;
            self.iris_overlay.rename_buffer.clear();
            self.asset_browser.revision = self.asset_browser.revision.wrapping_add(1);
        }

        if iris_res.clear_console_entries {
            self.console_entries.clear();
            self.console_last_count = 0;
        }

        // Keyboard shortcuts: UI scaling (Ctrl + / Ctrl - / Ctrl 0)
        if let WindowEvent::KeyboardInput {
            event: key_event, ..
        } = event
            && key_event.state == ElementState::Pressed
            && self.iris_overlay.ctrl_held
            && !self.wants_keyboard_input()
        {
            let mut scale_changed = false;
            match key_event.physical_key {
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Equal)
                | winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::NumpadAdd) => {
                    self.step_ui_scale(true);
                    scale_changed = true;
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Minus)
                | winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::NumpadSubtract) => {
                    self.step_ui_scale(false);
                    scale_changed = true;
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit0)
                | winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Numpad0) => {
                    self.reset_ui_scale();
                    scale_changed = true;
                }
                _ => {
                    if let winit::keyboard::Key::Character(ref c) = key_event.logical_key {
                        if c == "+" || c == "=" {
                            self.step_ui_scale(true);
                            scale_changed = true;
                        } else if c == "-" || c == "_" {
                            self.step_ui_scale(false);
                            scale_changed = true;
                        } else if c == "0" {
                            self.reset_ui_scale();
                            scale_changed = true;
                        }
                    }
                }
            }

            if scale_changed {
                self.pending_actions
                    .push(crate::ui::types::EngineUiAction::SetUiScale(
                        self.scale_factor(),
                    ));
                return true;
            }
        }

        // Hierarchy search bar live typing
        if self.iris_overlay.hierarchy_is_search_focused
            && let WindowEvent::KeyboardInput {
                event: key_event, ..
            } = event
            && key_event.state == ElementState::Pressed
        {
            if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) =
                key_event.logical_key
            {
                self.iris_overlay.hierarchy_is_search_focused = false;
                return true;
            }
            if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace) =
                key_event.logical_key
            {
                self.iris_overlay.hierarchy_search_query.pop();
                return true;
            }
            if let Some(text) = &key_event.text {
                for c in text.chars() {
                    if !c.is_control() {
                        self.iris_overlay.hierarchy_search_query.push(c);
                    }
                }
                return true;
            }
        }

        let p = self.iris_overlay.cursor_pos;
        let win_size = window.inner_size();
        let screen_w = win_size.width as f32 / zoom;
        let screen_h = win_size.height as f32 / zoom;
        let workspace_rect = Rect::new(
            0.0,
            IrisEditorOverlay::MENUBAR_HEIGHT,
            screen_w,
            (screen_h - IrisEditorOverlay::MENUBAR_HEIGHT - IrisEditorOverlay::STATUS_BAR_HEIGHT)
                .max(0.0),
        );

        // Enforce workspace boundary clamping on all floating windows
        self.layout_state.clamp_floating_windows(
            screen_w,
            screen_h,
            IrisEditorOverlay::MENUBAR_HEIGHT,
            IrisEditorOverlay::STATUS_BAR_HEIGHT,
        );

        // Process native dock and floating window interactions
        let mut dock_consumed = false;
        match event {
            WindowEvent::CursorMoved { .. } => {
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
                            self.layout_state.bump_revision();
                        } else {
                            let _ = self.layout_state.dock_state.start_tab_drag(
                                pending.leaf,
                                pending.tab_index,
                                p,
                                pending.leaf_rect,
                            );
                            self.layout_state.bump_revision();
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
                                let max_y =
                                    (screen_h - IrisEditorOverlay::STATUS_BAR_HEIGHT - TAB_BAR_H)
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
                                    let max_h = (screen_h
                                        - IrisEditorOverlay::STATUS_BAR_HEIGHT
                                        - win.rect.y)
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
                                    let max_h = (screen_h
                                        - IrisEditorOverlay::STATUS_BAR_HEIGHT
                                        - win.rect.y)
                                        .max(140.0);
                                    win.rect.width = (p.x - win.rect.x).clamp(220.0, max_w);
                                    win.rect.height = (p.y - win.rect.y).clamp(140.0, max_h);
                                }
                                FloatingResizeEdge::BottomLeft => {
                                    let old_right = win.rect.x + win.rect.width;
                                    let min_x = (60.0 - win.rect.width).min(0.0);
                                    win.rect.x = p.x.clamp(min_x, old_right - 220.0);
                                    win.rect.width = old_right - win.rect.x;
                                    let max_h = (screen_h
                                        - IrisEditorOverlay::STATUS_BAR_HEIGHT
                                        - win.rect.y)
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
                    self.layout_state.bump_revision();
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
                    self.layout_state.bump_revision();
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
                    self.layout_state.bump_revision();
                    dock_consumed = true;
                }
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                // 0. If click is over an active floating modal (Preferences, About, Modals, Dropdowns),
                // underlying floating windows and dock splitters/tabs MUST NOT be touched!
                if self.iris_overlay.is_point_over_modal_or_dropdown(p) {
                    return true;
                }

                // 1. Check Floating Window controls (highest priority among panels)
                let mut floating_dock_back = None;
                let mut floating_close = None;
                let mut floating_tab_action = None;

                for win in self.layout_state.dock_state.floating_windows.iter().rev() {
                    const TAB_BAR_H: f32 = 26.0;
                    let bar_rect = Rect::new(win.rect.x, win.rect.y, win.rect.width, TAB_BAR_H);

                    // Dock-back button `⤢`
                    let dock_btn_rect =
                        Rect::new(bar_rect.right() - 56.0, bar_rect.y + 2.0, 20.0, 22.0);
                    if dock_btn_rect.contains_point(p) {
                        floating_dock_back = Some(win.id);
                        break;
                    }

                    // Close button `✖`
                    let close_btn_rect =
                        Rect::new(bar_rect.right() - 32.0, bar_rect.y + 2.0, 20.0, 22.0);
                    if close_btn_rect.contains_point(p) {
                        floating_close = Some(win.id);
                        break;
                    }

                    // Floating window tab pill click check
                    let mut clicked_tab = None;
                    let mut current_tab_x = bar_rect.x + 12.0;
                    for (leaf_id, node) in win.tree.iter() {
                        if let irisui::dock::DockNode::Leaf { tabs, .. } = node {
                            for (tab_idx, panel) in tabs.iter().enumerate() {
                                let char_count = panel.title().chars().count();
                                let tab_w = (16.0 + 6.0 + (char_count as f32) * 6.8 + 14.0)
                                    .clamp(52.0, 160.0);
                                let tab_rect =
                                    Rect::new(current_tab_x, bar_rect.y, tab_w, TAB_BAR_H);
                                if tab_rect.contains_point(p) {
                                    clicked_tab = Some((leaf_id, tab_idx, *panel, win.rect));
                                    break;
                                }
                                current_tab_x += tab_w + 2.0;
                            }
                        }
                    }

                    if let Some((leaf_id, tab_idx, panel, leaf_rect)) = clicked_tab {
                        floating_tab_action = Some((win.id, leaf_id, tab_idx, panel, leaf_rect));
                        break;
                    }

                    // Title bar drag
                    if bar_rect.contains_point(p) {
                        self.active_floating_drag = Some((
                            win.id,
                            FloatingDragState::Title {
                                offset: Point::new(p.x - win.rect.x, p.y - win.rect.y),
                            },
                        ));
                        dock_consumed = true;
                        break;
                    }

                    // Window edge resize check
                    let win_rect =
                        Rect::new(win.rect.x, win.rect.y, win.rect.width, win.rect.height);
                    if win_rect.contains_point(p) {
                        const MARGIN: f32 = 6.0;
                        let on_left = p.x <= win.rect.x + MARGIN;
                        let on_right = p.x >= win_rect.right() - MARGIN;
                        let on_top = p.y <= win.rect.y + MARGIN;
                        let on_bottom = p.y >= win_rect.bottom() - MARGIN;

                        let edge = if on_top && on_left {
                            Some(FloatingResizeEdge::TopLeft)
                        } else if on_top && on_right {
                            Some(FloatingResizeEdge::TopRight)
                        } else if on_bottom && on_left {
                            Some(FloatingResizeEdge::BottomLeft)
                        } else if on_bottom && on_right {
                            Some(FloatingResizeEdge::BottomRight)
                        } else if on_left {
                            Some(FloatingResizeEdge::Left)
                        } else if on_right {
                            Some(FloatingResizeEdge::Right)
                        } else if on_top {
                            Some(FloatingResizeEdge::Top)
                        } else if on_bottom {
                            Some(FloatingResizeEdge::Bottom)
                        } else {
                            None
                        };

                        if let Some(e) = edge {
                            self.active_floating_drag =
                                Some((win.id, FloatingDragState::Resize(e)));
                            dock_consumed = true;
                            break;
                        }
                    }
                }

                if let Some(win_id) = floating_dock_back {
                    self.layout_state.smart_dock_back_panel(win_id);
                    return true;
                }
                if let Some(win_id) = floating_close {
                    let _ = self.layout_state.dock_state.close_floating_window(win_id);
                    self.layout_state.bump_revision();
                    return true;
                }
                if let Some((win_id, leaf_id, tab_idx, panel, leaf_rect)) = floating_tab_action {
                    if let Some(w) = self
                        .layout_state
                        .dock_state
                        .floating_windows
                        .iter_mut()
                        .find(|w| w.id == win_id)
                    {
                        let _ = w.tree.set_active_tab(leaf_id, tab_idx);
                        self.layout_state.bump_revision();
                    }
                    self.pending_tab_drag = Some(crate::ui::workbench::state::PendingTabDrag {
                        leaf: leaf_id,
                        tab_index: tab_idx,
                        panel,
                        press_pos: p,
                        leaf_rect,
                        floating_window_id: Some(win_id),
                    });
                    dock_consumed = true;
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

                if !dock_consumed
                    && !is_over_floating
                    && let Some(ref frame) = self.iris_overlay.native_dock_frame
                {
                    // Close buttons
                    if let Some(target) = frame
                        .close_targets
                        .iter()
                        .find(|t| t.rect.contains_point(p))
                    {
                        self.layout_state.close_tab(target.leaf, target.tab_index);
                        dock_consumed = true;
                    }
                    // Tab pills (switch active tab and prepare drag)
                    else if let Some(target) =
                        frame.tab_targets.iter().find(|t| t.rect.contains_point(p))
                    {
                        let _ = self
                            .layout_state
                            .dock_state
                            .tree
                            .set_active_tab(target.leaf, target.tab_index);
                        self.layout_state.bump_revision();
                        self.pending_tab_drag = Some(crate::ui::workbench::state::PendingTabDrag {
                            leaf: target.leaf,
                            tab_index: target.tab_index,
                            panel: target.panel,
                            press_pos: p,
                            leaf_rect: target.leaf_rect,
                            floating_window_id: None,
                        });
                        dock_consumed = true;
                    }
                    // Splitters (initiate divider resize drag)
                    else if let Some(target) = frame
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
                        self.layout_state.bump_revision();
                        dock_consumed = true;
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
                    self.layout_state.bump_revision();
                    dock_consumed = true;
                }
                if self.layout_state.dock_state.active_splitter.is_some() {
                    self.layout_state.dock_state.end_splitter_drag();
                    self.layout_state.bump_revision();
                    dock_consumed = true;
                }
                if self.layout_state.dock_state.active_drag.is_some() {
                    let _ = self
                        .layout_state
                        .dock_state
                        .drop_tab_or_float(Point::new(400.0, 300.0));
                    self.layout_state.bump_revision();
                    dock_consumed = true;
                }
            }

            _ => {}
        }

        // Apply requested cursor
        let requested_cursor = self.iris_overlay.requested_cursor_icon();
        if requested_cursor != winit::window::CursorIcon::Default {
            window.set_cursor(requested_cursor);
        } else {
            window.set_cursor(winit::window::CursorIcon::Default);
        }

        iris_res.consumed || dock_consumed
    }

    /// Returns true if the point is over any UI panel, floating modal dialog, or outside the 3D viewport.
    pub fn is_point_over_ui_rects(&self, pos: [f32; 2]) -> bool {
        let point = Point::new(pos[0], pos[1]);

        // 1. Top menubar & active modal dialogs / preferences / popups (always highest z-order)
        if pos[1] <= IrisEditorOverlay::MENUBAR_HEIGHT
            || self.iris_overlay.about_targets.is_some()
            || self.iris_overlay.delete_targets.is_some()
            || self.iris_overlay.new_folder_targets.is_some()
            || self.iris_overlay.rename_targets.is_some()
            || self.iris_overlay.loading_targets.is_some()
            || self.iris_overlay.assets_preview_modal.is_some()
            || self.ui_rects.iter().any(|rect| rect.contains_point(point))
        {
            return true;
        }

        if let Some(ref targets) = self.iris_overlay.preferences_targets
            && (targets.card_rect.contains_point(point)
                || targets
                    .active_dropdown_popup_rect
                    .is_some_and(|r| r.contains_point(point)))
        {
            return true;
        }

        if let Some(dd_rect) = self.iris_overlay.dropdown_rect
            && dd_rect.contains_point(point)
        {
            return true;
        }

        // 2. If the point is inside the active 3D viewport canvas (docked or floating)
        if self.last_viewport_rect.contains_point(point) {
            // Check if there are Viewport HUD interactive controls (toolbar buttons, dropdown, compass, billboard icons)
            if let Some(ref hud) = self.iris_overlay.viewport_hud_targets {
                if let Some(dd_rect) = hud.active_dropdown_popup_rect
                    && dd_rect.contains_point(point)
                {
                    return true;
                }
                if hud.buttons.iter().any(|(_, r)| r.contains_point(point))
                    || hud
                        .dropdown_triggers
                        .iter()
                        .any(|(_, r)| r.contains_point(point))
                    || hud
                        .compass_knobs
                        .iter()
                        .any(|(_, r)| r.contains_point(point))
                    || hud
                        .billboard_icons
                        .iter()
                        .any(|(_, r)| r.contains_point(point))
                {
                    return true;
                }
            }

            // Check if another detached floating window occludes this viewport point
            let is_occluded_by_other_floating = self
                .layout_state
                .dock_state
                .floating_windows
                .iter()
                .any(|w| {
                    let contains_viewport = w
                        .tree
                        .all_tabs()
                        .contains(&crate::ui::panel_layout::PanelId::Viewport);
                    if !contains_viewport {
                        let w_rect = Rect::new(w.rect.x, w.rect.y, w.rect.width, w.rect.height);
                        w_rect.contains_point(point)
                    } else {
                        false
                    }
                });

            if is_occluded_by_other_floating {
                return true;
            }

            // Directly over the 3D viewport canvas: mouse input belongs 100% to 3D scene
            return false;
        }

        // 3. Point is outside 3D viewport canvas -> belongs to UI panels
        true
    }
}