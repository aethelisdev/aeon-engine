// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Window Event Coordinator
//!
//! Forwards winit window events to Iris UI overlays, native dock coordinator,
//! keyboard hotkey handlers, and manages modal/dialog lifecycle state.

pub mod hit_test;
pub mod keyboard;

#[cfg(test)]
mod tests;

use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::workbench::state::EngineUi;
use iris_winit::frameless::WindowInterface;
use irisui::dock::{DockArea, DockInteractionConfig, FloatingWindowClampBounds};
use irisui::prelude::{Rect, UiEvent, WidgetCursor};

impl EngineUi {
    /// Dispatches declarative UI interaction events to Iris UI and the native dock coordinator.
    ///
    /// Receives pure, hardware-abstracted [`UiEvent`] representations translated at window root.
    /// Uses [`WindowInterface`] to interact with the host platform window without concrete bindings.
    pub fn handle_event<W: WindowInterface>(
        &mut self,
        mut window: W,
        event: &UiEvent,
        _world: Option<&hecs::World>,
    ) -> bool {
        let logical_size = window.logical_size();
        let screen_w = logical_size.width;
        let screen_h = logical_size.height;

        // 1. Frameless Window 8-way Native Edge Resizing (Top, Bottom, Left, Right and 4 Corners)
        if self
            .frameless_host
            .handle_ui_event(&mut window, event, logical_size)
        {
            return true;
        }

        // Synchronize active floating window boundaries with IrisEditorOverlay for occlusion testing
        self.iris_overlay.chrome.floating_window_rects = self
            .layout_state
            .dock_state
            .floating_windows
            .iter()
            .map(|w| Rect::new(w.rect.x, w.rect.y, w.rect.width, w.rect.height))
            .collect();

        let iris_res = self.iris_overlay.handle_event(event);
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
            self.iris_overlay.preferences.floating.reset();
        }
        if let Some(pref_act) = iris_res.preferences_action {
            if let crate::ui::iris_bridge::PreferencesAction::SetUiScale(s) = pref_act {
                self.ui_zoom_factor = s;
            }
            if let crate::ui::iris_bridge::PreferencesAction::SelectTab(t) = pref_act {
                self.preferences_tab = t;
                self.iris_overlay.preferences.tab = t;
                self.iris_overlay.preferences.reset_scroll();
            }
            if let crate::ui::iris_bridge::PreferencesAction::SetTheme(t) = pref_act {
                self.iris_overlay.chrome.theme_preset = t;
                self.iris_overlay.notifier.tag_all();
                self.iris_overlay.chrome.needs_layout_rebuild = true;
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

        let modal_closed = iris_res.close_preferences
            || iris_res.close_about
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
            self.iris_overlay.notifier.tag_all();
            self.iris_overlay.chrome.needs_layout_rebuild = true;
        }

        if iris_res.clear_console_entries {
            self.console_entries.clear();
            self.console_last_count = 0;
        }

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
            self.handle_dock_event(event, workspace_rect, screen_w, screen_h);

        if dock_consumed {
            let active_widget_cursor =
                dock_cursor.unwrap_or_else(|| self.iris_overlay.active_cursor());
            window.set_widget_cursor(active_widget_cursor);
            return true;
        }

        // Layer isolation mandate: events consumed by Iris UI overlays or panels must never leak to underlying systems
        if iris_res.consumed {
            let active_widget_cursor = self.iris_overlay.active_cursor();
            window.set_widget_cursor(active_widget_cursor);
            return true;
        }

        // Native Window Title Bar Dragging & Maximize Double-Click on unconsumed MenuBar clicks
        let menubar_rect = Rect::new(0.0, 0.0, screen_w, IrisEditorOverlay::MENUBAR_HEIGHT);
        if self
            .drag_host
            .handle_ui_event(&mut window, event, menubar_rect)
        {
            return true;
        }

        // Process keyboard shortcuts and search live-typing
        if self.handle_keyboard_shortcut(event) {
            return true;
        }

        // Apply requested cursor
        let active_widget_cursor = dock_cursor.unwrap_or_else(|| self.iris_overlay.active_cursor());
        window.set_widget_cursor(active_widget_cursor);

        iris_res.consumed || dock_consumed
    }

    /// Dispatches cursor motion and mouse button events into [`irisui::dock::DockArea::handle_event`].
    ///
    /// Manages splitter divider resizes, tab selection, middle-click close, tab drag-and-drop,
    /// and floating window 8-way resize/dragging in a unified autonomous pass.
    ///
    /// Returns `(consumed, cursor_override)`.
    pub(crate) fn handle_dock_event(
        &mut self,
        ui_event: &UiEvent,
        workspace_rect: Rect,
        screen_w: f32,
        screen_h: f32,
    ) -> (bool, Option<WidgetCursor>) {
        // If modal or dropdown is open and event is mouse press, ignore dock interaction
        if let UiEvent::MouseDown { point, .. } = ui_event
            && self.iris_overlay.is_point_over_overlay(*point)
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
            ui_event,
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