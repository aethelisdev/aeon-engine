// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Top-level event orchestration and interaction routing for Iris UI editor overlays.

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::*;
use winit::event::WindowEvent;

impl IrisEditorOverlay {
    /// Intercepts and processes window mouse input and cursor movement events across all active Iris UI subsystems.
    ///
    /// Verifies both returned result flags and pending panel actions enqueued in [`PanelInteractionState`]
    /// to trigger reactive UI tree invalidation with zero lost frames.
    pub fn handle_event(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
    ) -> IrisOverlayEventResult {
        let result = self.dispatch_window_event_internal(event, world);
        // Reactive Event Invalidation: If any UI element consumed this event, produced
        // an action result, or queued typed actions in its PanelInteractionState, immediately
        // flag the UI tree as dirty so the next frame reflects the change instantly, even
        // if the mouse cursor does not move a single pixel.
        if result.consumed
            || self.has_pending_panel_actions()
            || result.preferences_action.is_some()
            || result.ui_action.is_some()
            || result.toggle_panel.is_some()
            || result.open_preferences
            || result.close_preferences
            || result.open_about
            || result.close_about
            || result.confirm_delete
            || result.cancel_delete
            || result.create_folder.is_some()
            || result.apply_rename.is_some()
            || result.reset_layout
        {
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
        }
        result
    }

    /// Returns `true` if any docked or modal panel has pending queued typed actions in its [`PanelInteractionState`].
    ///
    /// Checks all standardized panel action queues in O(1) without heap allocation or collection draining.
    #[inline]
    #[must_use]
    pub fn has_pending_panel_actions(&self) -> bool {
        self.hierarchy.interactions.has_pending_actions()
            || self.stats.interactions.has_pending_actions()
            || self.console.interactions.has_pending_actions()
            || self.material.interactions.has_pending_actions()
            || self.assets.interactions.has_pending_actions()
            || self.preferences.interactions.has_pending_actions()
            || self.inspector.interactions.has_pending_actions()
            || self.timeline.interactions.has_pending_actions()
            || self.ui_designer.interactions.has_pending_actions()
            || !self.viewport_hud.actions.is_empty()
    }

    /// Internal routing pipeline that tests UI layers in strict Z-order.
    fn dispatch_window_event_internal(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
    ) -> IrisOverlayEventResult {
        let mut result = IrisOverlayEventResult::default();

        // 1. Track modifier keys for accelerated / fine-tune dragging
        if let WindowEvent::ModifiersChanged(modifiers) = event {
            self.chrome.shift_held = modifiers.state().shift_key();
            self.chrome.alt_held = modifiers.state().alt_key();
            self.chrome.ctrl_held = modifiers.state().control_key();
        }

        // 2. Real-time cursor position tracking
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.chrome.cursor_pos = Point::new(position.x as f32, position.y as f32);
        }

        // 3. Continuous global dragging interactions (Inspector scrubbers, Preferences dialog/sliders, Asset drags)
        if let Some(drag_res) = self.dispatch_continuous_drag_events(event) {
            return drag_res;
        }

        // 4. Loading Splash Screen (blocks all underlying interactions)
        if self.modals.is_loading_active {
            result.consumed = true;
            return result;
        }

        // 5. Top Menubar and Dropdowns (Prioritized above modal dialogs whenever a dropdown
        // is open or the cursor is positioned over the menubar header)
        if (self.menubar.active_menu.is_some() || self.cursor_pos().y <= Self::MENUBAR_HEIGHT)
            && let Some(mb_res) = self.handle_menubar_event(event)
        {
            return mb_res;
        }

        // 5b. Active Floating Popups (UiLayer::Popup; must be evaluated before modal dialogs)
        if self.is_point_over_popup(self.cursor_pos()) {
            if let Some(popup_res) = self.dispatch_floating_popup_events(event, world) {
                return popup_res;
            }
        } else if let WindowEvent::MouseInput {
            state: winit::event::ElementState::Pressed,
            ..
        } = event
            && self.assets.context_menu.is_some()
        {
            self.assets.context_menu = None;
            self.chrome.needs_layout_rebuild = true;
            self.notifier.tag_all();
        }

        // 5c. Context Menu Cancellation via Escape key
        if let WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } = event
            && self.assets.context_menu.is_some()
        {
            self.assets.context_menu = None;
            self.chrome.needs_layout_rebuild = true;
            self.notifier.tag_all();
            return IrisOverlayEventResult {
                consumed: true,
                ..Default::default()
            };
        }

        // 6. Generic Modal Dialogs (About, Delete, New Folder, Rename, Asset Preview)
        if let Some(modal_result) = self.handle_modal_events(event) {
            return modal_result;
        }

        // 7. Preferences Floating Dialog
        if let Some(pref_result) = self.handle_preferences_event(event) {
            return pref_result;
        }

        // 8. Top Menubar and Dropdowns (Fallback interaction route)
        if let Some(mb_res) = self.handle_menubar_event(event) {
            return mb_res;
        }

        // 8b. Viewport HUD Controls (Interactive on docked or floating viewports)
        if let Some(hud_res) = self.handle_viewport_hud_window_event(event) {
            return hud_res;
        }

        // 9. Floating Window Occlusion Check:
        // If cursor is over an active floating window, docked panels must NOT claim the event!
        let cursor = self.cursor_pos();
        let is_cursor_over_floating = self.chrome.floating_window_rects.iter().any(|r| {
            cursor.x >= r.x && cursor.x <= r.right() && cursor.y >= r.y && cursor.y <= r.bottom()
        });

        if !is_cursor_over_floating
            && let Some(panel_res) = self.dispatch_docked_panel_events(event, world)
        {
            return panel_res;
        }

        result
    }

    /// Dispatches continuous window-wide dragging and release events (Inspector scrubbers, Preferences, Assets).
    fn dispatch_continuous_drag_events(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        // Inspector active dragging / scrubber motion & release (runs globally)
        if let Some(insp_drag_res) = self.handle_inspector_drag_events(event) {
            return Some(insp_drag_res);
        }

        // Active Preferences Drag Interaction (Window dragging or slider dragging)
        if let Some(pref_drag_res) = self.handle_preferences_drag_events(event) {
            return Some(pref_drag_res);
        }

        // Active Asset Drag Interaction (Global mouse release and Escape cancellation)
        if let Some(asset_drag_res) = self.handle_asset_drag_events(event) {
            return Some(asset_drag_res);
        }

        None
    }

    /// Dispatches events to active floating popups in [`UiLayer::Popup`].
    fn dispatch_floating_popup_events(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
    ) -> Option<IrisOverlayEventResult> {
        if let Some(hud_res) = self.handle_viewport_hud_window_event(event) {
            return Some(hud_res);
        }
        if let Some(assets_res) = self.handle_assets_window_event(event) {
            return Some(assets_res);
        }
        if let Some(hier_res) = self.handle_hierarchy_window_event(event) {
            return Some(hier_res);
        }
        if let Some(insp_res) = self.handle_inspector_window_event(event, world) {
            return Some(insp_res);
        }
        if let Some(ui_res) = self.handle_ui_designer_window_event(event) {
            return Some(ui_res);
        }
        if let Some(dock_res) = self.handle_dock_overflow_event(event) {
            return Some(dock_res);
        }
        None
    }

    /// Dispatches events across docked panels adhering to their typed action models.
    fn dispatch_docked_panel_events(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
    ) -> Option<IrisOverlayEventResult> {
        if let Some(hier_res) = self.handle_hierarchy_window_event(event) {
            return Some(hier_res);
        }

        if let Some(stats_res) = self.handle_stats_window_event(event) {
            return Some(stats_res);
        }

        if let Some(console_res) = self.handle_console_window_event(event) {
            return Some(console_res);
        }

        if let Some(assets_res) = self.handle_assets_window_event(event) {
            return Some(assets_res);
        }

        if let Some(timeline_res) = self.handle_timeline_window_event(event) {
            return Some(timeline_res);
        }

        if let Some(mat_res) = self.handle_material_window_event(event) {
            return Some(mat_res);
        }

        if let Some(ui_res) = self.handle_ui_designer_window_event(event) {
            return Some(ui_res);
        }

        if let Some(scroll_res) = self.handle_panel_mouse_wheel(event) {
            return Some(scroll_res);
        }

        if let Some(insp_res) = self.handle_inspector_window_event(event, world) {
            return Some(insp_res);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::iris_bridge::assets::types::{AssetsPanelAction, AssetsPanelState};
    use crate::ui::iris_bridge::console::{ConsoleAction, ConsolePanelState};
    use crate::ui::iris_bridge::hierarchy::{HierarchyAction, HierarchyPanelState};
    use crate::ui::iris_bridge::inspector::{InspectorAction, InspectorPanelState};
    use crate::ui::iris_bridge::material::{MaterialAction, MaterialPanelState};
    use crate::ui::iris_bridge::preferences::{PreferencesAction, PreferencesDialogState};
    use crate::ui::iris_bridge::stats::{StatsPanelAction, StatsPanelState};
    use crate::ui::iris_bridge::types::PanelInteractionState;

    #[test]
    fn test_panel_interaction_state_pending_actions_invariants() {
        let mut state: PanelInteractionState<(), ConsoleAction> = PanelInteractionState::default();
        assert!(
            !state.has_pending_actions(),
            "Default state must report zero pending actions"
        );

        state.actions.push(ConsoleAction::ClearLogs);
        assert!(
            state.has_pending_actions(),
            "Must report pending action when queue is non-empty"
        );

        let drained = state.take_actions();
        assert_eq!(drained.len(), 1);
        assert!(
            !state.has_pending_actions(),
            "State must report zero pending actions after take_actions"
        );
    }

    #[test]
    fn test_all_panels_pending_actions_invariants() {
        // 1. Hierarchy panel state
        let mut hier = HierarchyPanelState::default();
        assert!(!hier.interactions.has_pending_actions());
        hier.interactions
            .actions
            .push(HierarchyAction::DeleteSelected);
        assert!(hier.interactions.has_pending_actions());
        let _ = hier.interactions.take_actions();
        assert!(!hier.interactions.has_pending_actions());

        // 2. Stats panel state
        let mut stats = StatsPanelState::default();
        assert!(!stats.interactions.has_pending_actions());
        stats
            .interactions
            .actions
            .push(StatsPanelAction::ToggleWireframe);
        assert!(stats.interactions.has_pending_actions());
        let _ = stats.interactions.take_actions();
        assert!(!stats.interactions.has_pending_actions());

        // 3. Console panel state
        let mut console = ConsolePanelState::default();
        assert!(!console.interactions.has_pending_actions());
        console.interactions.actions.push(ConsoleAction::ClearLogs);
        assert!(console.interactions.has_pending_actions());
        let _ = console.interactions.take_actions();
        assert!(!console.interactions.has_pending_actions());

        // 4. Material panel state
        let mut mat = MaterialPanelState::default();
        assert!(!mat.interactions.has_pending_actions());
        mat.interactions
            .actions
            .push(MaterialAction::RemoveTextureFromEntity(
                hecs::Entity::DANGLING,
            ));
        assert!(mat.interactions.has_pending_actions());
        let _ = mat.interactions.take_actions();
        assert!(!mat.interactions.has_pending_actions());

        // 5. Assets panel state
        let mut assets = AssetsPanelState::default();
        assert!(!assets.interactions.has_pending_actions());
        assets
            .interactions
            .actions
            .push(AssetsPanelAction::CleanVram);
        assert!(assets.interactions.has_pending_actions());
        let _ = assets.interactions.take_actions();
        assert!(!assets.interactions.has_pending_actions());

        // 6. Preferences dialog state
        let mut pref = PreferencesDialogState::default();
        assert!(!pref.interactions.has_pending_actions());
        pref.interactions.actions.push(PreferencesAction::Close);
        assert!(pref.interactions.has_pending_actions());
        let _ = pref.interactions.take_actions();
        assert!(!pref.interactions.has_pending_actions());

        // 7. Inspector panel state
        let mut insp = InspectorPanelState::default();
        assert!(!insp.interactions.has_pending_actions());
        insp.interactions
            .actions
            .push(InspectorAction::ToggleColorPicker);
        assert!(insp.interactions.has_pending_actions());
        let _ = insp.interactions.take_actions();
        assert!(!insp.interactions.has_pending_actions());

        // 8. Timeline panel state
        let mut tl = crate::ui::iris_bridge::timeline::TimelinePanelState::default();
        assert!(!tl.interactions.has_pending_actions());
        tl.interactions
            .actions
            .push(crate::ui::iris_bridge::timeline::TimelineAction::TogglePlayPause);
        assert!(tl.interactions.has_pending_actions());
        let _ = tl.interactions.take_actions();
        assert!(!tl.interactions.has_pending_actions());

        // 9. UI Designer panel state
        let mut ui_des = crate::ui::iris_bridge::ui_designer::UiDesignerPanelState::default();
        assert!(!ui_des.interactions.has_pending_actions());
        ui_des
            .interactions
            .actions
            .push(crate::ui::iris_bridge::ui_designer::UiDesignerAction::ToggleGrid);
        assert!(ui_des.interactions.has_pending_actions());
        let _ = ui_des.interactions.take_actions();
        assert!(!ui_des.interactions.has_pending_actions());

        // 10. Viewport HUD state
        let mut vp_hud = crate::ui::iris_bridge::viewport_hud::ViewportHudState::default();
        assert!(vp_hud.actions.is_empty());
        vp_hud
            .actions
            .push(crate::ui::iris_bridge::viewport_hud::ViewportHudAction::ToggleWireframe);
        assert!(!vp_hud.actions.is_empty());
        let _ = vp_hud.take_actions();
        assert!(vp_hud.actions.is_empty());
    }
}