// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Window Event Coordinator
//!
//! Forwards winit window events to Iris UI overlays, native dock coordinator,
//! keyboard hotkey handlers, and manages modal/dialog lifecycle state.

pub mod hit_test;
pub mod keyboard;

use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::workbench::state::EngineUi;
use irisui::dock::{DockArea, DockInteractionConfig, FloatingWindowClampBounds};
use irisui::prelude::{Point, Rect, UiEvent, WidgetCursor};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::window::Window;

impl EngineUi {
    /// Forwards winit window events to Iris UI and the native dock coordinator.
    pub fn handle_event(
        &mut self,
        window: &Window,
        event: &WindowEvent,
        world: Option<&hecs::World>,
    ) -> bool {
        // Synchronize active floating window boundaries with IrisEditorOverlay for occlusion testing
        self.iris_overlay.chrome.floating_window_rects = self
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

        let iris_res = self.iris_overlay.handle_event(event_ref, world);
        if let Some(act) = iris_res.ui_action {
            self.pending_actions.push(act);
        }
        if let Some(panel) = iris_res.toggle_panel {
            self.layout_state.activate_or_open(panel);
        }
        if let Some((leaf, tab_idx)) = iris_res.activate_dock_tab {
            let _ = self
                .layout_state
                .dock_state
                .tree
                .set_active_tab(leaf, tab_idx);
            self.iris_overlay.chrome.active_dock_overflow = None;
            self.iris_overlay.chrome.needs_layout_rebuild = true;
            return true;
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
            let _ = crate::assets::file_ops::delete_asset_or_folder(&target);
            if self.asset_browser.selected_asset.as_ref() == Some(&target) {
                self.asset_browser.selected_asset = None;
            }
        }
        if iris_res.cancel_delete {
            self.asset_browser.delete_confirmation = None;
        }

        let modal_closed = iris_res.close_about
            || iris_res.cancel_delete
            || iris_res.confirm_delete
            || iris_res.cancel_new_folder
            || iris_res.create_folder.is_some()
            || iris_res.cancel_rename
            || iris_res.apply_rename.is_some();

        if let Some(folder_name) = iris_res.create_folder
            && let Some(parent) = self.asset_browser.new_folder_parent.take()
        {
            let _ = crate::assets::file_ops::create_subfolder(&parent, &folder_name);
            self.iris_overlay.modals.clear_new_folder();
            self.asset_browser.new_folder_name.clear();
        }
        if iris_res.cancel_new_folder {
            self.asset_browser.new_folder_parent = None;
            self.iris_overlay.modals.clear_new_folder();
            self.asset_browser.new_folder_name.clear();
        }

        if let Some(new_name) = iris_res.apply_rename
            && let Some(ren) = self.asset_browser.rename_state.take()
        {
            let _ = crate::assets::file_ops::rename_asset_or_folder(&ren.target_path, &new_name);
            self.iris_overlay.modals.clear_rename();
        }
        if iris_res.cancel_rename {
            self.asset_browser.rename_state = None;
            self.iris_overlay.modals.clear_rename();
        }

        if modal_closed {
            self.iris_overlay.overlay_tree.close();
            self.iris_overlay.notifier.tag_all();
            self.iris_overlay.chrome.needs_layout_rebuild = true;
        }

        if iris_res.clear_console_entries {
            self.console_entries.clear();
            self.console_last_count = 0;
        }

        // Layer isolation mandate: events consumed by Iris UI overlays or panels must never leak to underlying systems
        if iris_res.consumed {
            return true;
        }

        // Process keyboard shortcuts and search live-typing
        if self.handle_keyboard_shortcut(event) {
            return true;
        }

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

        // Process autonomous dock events (splitters, tabs, drag-and-drop, and floating windows)
        let (dock_consumed, dock_cursor) =
            self.handle_dock_event(event_ref, workspace_rect, screen_w, screen_h);

        // Apply requested cursor
        let requested_cursor = if let Some(cursor) = dock_cursor {
            match cursor {
                irisui::prelude::WidgetCursor::Default => winit::window::CursorIcon::Default,
                irisui::prelude::WidgetCursor::Pointer => winit::window::CursorIcon::Pointer,
                irisui::prelude::WidgetCursor::Grab => winit::window::CursorIcon::Grab,
                irisui::prelude::WidgetCursor::Grabbing => winit::window::CursorIcon::Grabbing,
                irisui::prelude::WidgetCursor::Text => winit::window::CursorIcon::Text,
                irisui::prelude::WidgetCursor::ColResize => winit::window::CursorIcon::ColResize,
                irisui::prelude::WidgetCursor::RowResize => winit::window::CursorIcon::RowResize,
                irisui::prelude::WidgetCursor::EwResize => winit::window::CursorIcon::EwResize,
                irisui::prelude::WidgetCursor::NsResize => winit::window::CursorIcon::NsResize,
                irisui::prelude::WidgetCursor::NwseResize => winit::window::CursorIcon::NwseResize,
                irisui::prelude::WidgetCursor::NeswResize => winit::window::CursorIcon::NeswResize,
                irisui::prelude::WidgetCursor::Crosshair => winit::window::CursorIcon::Crosshair,
                irisui::prelude::WidgetCursor::NotAllowed => winit::window::CursorIcon::NotAllowed,
            }
        } else {
            self.iris_overlay.requested_cursor_icon()
        };
        window.set_cursor(requested_cursor);

        let consumed = iris_res.consumed || dock_consumed;
        if consumed {
            self.iris_overlay.notifier.tag_all();
            self.iris_overlay.chrome.needs_layout_rebuild = true;
        }
        consumed
    }

    /// Dispatches cursor motion and mouse button events into [`irisui::dock::DockArea::handle_event`].
    ///
    /// Manages splitter divider resizes, tab selection, middle-click close, tab drag-and-drop,
    /// and floating window 8-way resize/dragging in a unified autonomous pass.
    ///
    /// Returns `(consumed, cursor_override)`.
    pub(crate) fn handle_dock_event(
        &mut self,
        event: &WindowEvent,
        workspace_rect: Rect,
        screen_w: f32,
        screen_h: f32,
    ) -> (bool, Option<WidgetCursor>) {
        let ui_event = match event {
            WindowEvent::CursorMoved { position, .. } => UiEvent::MouseMove {
                point: Point::new(position.x as f32, position.y as f32),
            },
            WindowEvent::MouseInput { state, button, .. } => {
                let btn = match button {
                    MouseButton::Left => irisui::prelude::MouseButton::Left,
                    MouseButton::Right => irisui::prelude::MouseButton::Right,
                    MouseButton::Middle => irisui::prelude::MouseButton::Middle,
                    MouseButton::Other(b) => irisui::prelude::MouseButton::Other(*b),
                    _ => irisui::prelude::MouseButton::Left,
                };
                let point = self.iris_overlay.chrome.cursor_pos;
                match state {
                    ElementState::Pressed => UiEvent::MouseDown { button: btn, point },
                    ElementState::Released => UiEvent::MouseUp { button: btn, point },
                }
            }
            _ => return (false, None),
        };

        // If modal or dropdown is open and event is mouse press, ignore dock interaction
        if let UiEvent::MouseDown { point, .. } = ui_event
            && self.iris_overlay.is_point_over_modal_or_dropdown(point)
        {
            return (false, None);
        }

        let clamp_bounds = FloatingWindowClampBounds {
            min_width: 220.0,
            min_height: 140.0,
            screen_width: screen_w,
            screen_height: screen_h,
            min_y: IrisEditorOverlay::MENUBAR_HEIGHT,
            bottom_inset: IrisEditorOverlay::STATUS_BAR_HEIGHT,
            min_visible_margin: 60.0,
            title_bar_height: crate::ui::iris_bridge::native_dock::NATIVE_DOCK_TAB_HEIGHT,
        };

        let config = DockInteractionConfig {
            splitter_thickness: crate::ui::iris_bridge::native_dock::SPLITTER_THICKNESS,
            tab_bar_height: crate::ui::iris_bridge::native_dock::NATIVE_DOCK_TAB_HEIGHT,
            clamp_bounds,
        };

        let viewer = crate::ui::panel_layout::PanelTabViewer::default();
        let res = DockArea::handle_event(
            &mut self.layout_state.dock_state,
            &viewer,
            &ui_event,
            workspace_rect,
            &config,
        );

        if res.layout_changed {
            self.iris_overlay.chrome.needs_layout_rebuild = true;
            self.iris_overlay.notifier.tag_all();
        }

        (res.consumed, res.cursor_override)
    }
}