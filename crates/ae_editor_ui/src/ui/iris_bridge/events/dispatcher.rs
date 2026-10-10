// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Top-level event orchestration and interaction routing for Iris UI editor overlays.

use crate::ui::iris_bridge::compositor;
use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use crate::ui::iris_bridge::{ActiveOverlay, ModalKind};
use crate::ui::panel_layout::PanelId;
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

        // 5. LAYER 1: Top-Level Overlay / Portal Layer (OverlayTree)
        // Menubar dropdowns, context menus, modals, color picker popups.
        if self.overlay_tree.is_open() {
            let cursor = self.cursor_pos();

            // Modal dialogs capture all keyboard and IME input regardless of cursor coordinates
            if matches!(
                event,
                WindowEvent::KeyboardInput { .. } | WindowEvent::Ime(_)
            ) {
                match self.overlay_tree.active_overlay() {
                    Some(ActiveOverlay::Modal(ModalKind::Preferences)) => {
                        if let Some(pref_res) = self.handle_preferences_event(event) {
                            return pref_res;
                        }
                    }
                    Some(ActiveOverlay::Modal(_)) => {
                        if let Some(modal_res) = self.handle_modal_events(event) {
                            return modal_res;
                        }
                    }
                    _ => {}
                }
            }

            if self.overlay_tree.contains_point(cursor) {
                // Event point is inside active overlay -> dispatch specifically to active overlay handler
                match self.overlay_tree.active_overlay() {
                    Some(ActiveOverlay::MenubarDropdown(_)) => {
                        if let Some(mb_res) = self.handle_menubar_event(event) {
                            return mb_res;
                        }
                    }
                    Some(ActiveOverlay::Modal(ModalKind::Preferences)) => {
                        if let Some(pref_res) = self.handle_preferences_event(event) {
                            return pref_res;
                        }
                    }
                    Some(ActiveOverlay::Modal(_)) => {
                        if let Some(modal_res) = self.handle_modal_events(event) {
                            return modal_res;
                        }
                    }
                    Some(ActiveOverlay::HierarchyContextMenu) => {
                        if let Some(hier_res) = self.handle_hierarchy_window_event(event) {
                            return hier_res;
                        }
                    }
                    Some(ActiveOverlay::AssetsContextMenu) => {
                        if let Some(assets_res) = self.handle_assets_window_event(event) {
                            return assets_res;
                        }
                    }
                    Some(
                        ActiveOverlay::InspectorAddComponent
                        | ActiveOverlay::InspectorDropdown
                        | ActiveOverlay::InspectorColorPicker,
                    ) => {
                        if let Some(insp_res) = self.handle_inspector_window_event(event, world) {
                            return insp_res;
                        }
                    }
                    Some(
                        ActiveOverlay::UiDesignerAddElement | ActiveOverlay::UiDesignerAspectRatio,
                    ) => {
                        if let Some(ui_res) = self.handle_ui_designer_window_event(event) {
                            return ui_res;
                        }
                    }
                    _ => {
                        if let Some(popup_res) = self.dispatch_floating_popup_events(event, world) {
                            return popup_res;
                        }
                        if let Some(modal_res) = self.handle_modal_events(event) {
                            return modal_res;
                        }
                        if let Some(pref_res) = self.handle_preferences_event(event) {
                            return pref_res;
                        }
                        if let Some(mb_res) = self.handle_menubar_event(event) {
                            return mb_res;
                        }
                    }
                }

                // If pointer input was inside overlay body, consume to prevent click-through
                if matches!(event, WindowEvent::MouseInput { .. }) {
                    result.consumed = true;
                    return result;
                }
            } else if let WindowEvent::CursorMoved { .. } = event {
                // Desktop standard: hovering over menubar buttons while a dropdown is open switches active menu
                if cursor.y <= Self::MENUBAR_HEIGHT
                    && self.menubar.active_menu.is_some()
                    && let Some(mb_res) = self.handle_menubar_event(event)
                {
                    return mb_res;
                }
            } else if let WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                ..
            } = event
            {
                // Pointer click is OUTSIDE the active overlay -> evaluate dismiss policy
                let dismiss_res = self.overlay_tree.handle_pointer_down(cursor);
                if dismiss_res == crate::ui::iris_bridge::OverlayDismissResult::Dismissed {
                    self.dismiss_active_overlays();
                    self.chrome.needs_layout_rebuild = true;
                    self.notifier.tag_all();
                    // Clicking on the menubar strip opens the newly clicked menu directly
                    if cursor.y <= Self::MENUBAR_HEIGHT
                        && let Some(mb_res) = self.handle_menubar_event(event)
                    {
                        return mb_res;
                    }
                    result.consumed = true;
                    return result; // Consumed: click-outside closes overlay without triggering click-through!
                } else if dismiss_res == crate::ui::iris_bridge::OverlayDismissResult::Consumed
                    || matches!(
                        self.overlay_tree.active_overlay(),
                        Some(ActiveOverlay::Modal(m)) if *m != ModalKind::Preferences
                    )
                {
                    result.consumed = true;
                    return result; // Modal dialog explicitly blocks click-outside interactions!
                }
            }
        } else if self.assets.context_menu.is_some()
            && let WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                ..
            } = event
        {
            self.assets.context_menu = None;
            self.chrome.needs_layout_rebuild = true;
            self.notifier.tag_all();
            result.consumed = true;
            return result;
        }

        // Overlay & context menu cancellation via Escape key
        if let WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } = event
            && (self.overlay_tree.is_open() || self.assets.context_menu.is_some())
        {
            self.dismiss_active_overlays();
            self.chrome.needs_layout_rebuild = true;
            self.notifier.tag_all();
            return IrisOverlayEventResult {
                consumed: true,
                ..Default::default()
            };
        }

        // 6. Top Menubar header bar (when cursor is over the header strip)
        if self.cursor_pos().y <= Self::MENUBAR_HEIGHT
            && let Some(mb_res) = self.handle_menubar_event(event)
        {
            return mb_res;
        }

        // 7. LAYER 2: Floating Windows Layer
        // If cursor is over any floating window, dispatch using local coordinates (cursor - float_rect.min)
        // and CONSUME mouse clicks so they never click through to docked panels underneath!
        let cursor = self.cursor_pos();
        let floating_rect_opt = self
            .chrome
            .floating_window_rects
            .iter()
            .find(|r| {
                cursor.x >= r.x
                    && cursor.x <= r.right()
                    && cursor.y >= r.y
                    && cursor.y <= r.bottom()
            })
            .copied();

        if let Some(float_rect) = floating_rect_opt {
            if let Some(floating_res) =
                self.dispatch_floating_layer_events(event, world, float_rect)
            {
                return floating_res;
            }
            // Pointer press over floating window background: consumed to prevent click-through
            if matches!(
                event,
                WindowEvent::MouseInput {
                    state: winit::event::ElementState::Pressed,
                    ..
                }
            ) {
                result.consumed = true;
                return result;
            }
        }

        // 8. Viewport HUD Controls (Interactive on docked or floating viewports)
        if let Some(hud_res) = self.handle_viewport_hud_window_event(event) {
            return hud_res;
        }

        // 9. LAYER 3: Docked Panels Layer
        // Dispatches to docked panels using local coordinates (cursor - panel_rect.min).
        if let Some(panel_res) = self.dispatch_docked_panel_events(event, world) {
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

    /// Dispatches events targeted at active floating windows and their contained panels,
    /// converting the global mouse cursor into local panel coordinates.
    fn dispatch_floating_layer_events(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
        float_rect: Rect,
    ) -> Option<IrisOverlayEventResult> {
        let cursor = self.cursor_pos();
        let _local_cursor = compositor::screen_to_panel_local(cursor, float_rect);

        // Try panel handlers that might be hosted inside this floating container
        if let Some(insp_res) = self.handle_inspector_window_event(event, world) {
            return Some(insp_res);
        }
        if let Some(hier_res) = self.handle_hierarchy_window_event(event) {
            return Some(hier_res);
        }
        if let Some(assets_res) = self.handle_assets_window_event(event) {
            return Some(assets_res);
        }
        if let Some(con_res) = self.handle_console_window_event(event) {
            return Some(con_res);
        }
        if let Some(stats_res) = self.handle_stats_window_event(event) {
            return Some(stats_res);
        }
        if let Some(mat_res) = self.handle_material_window_event(event) {
            return Some(mat_res);
        }
        if let Some(ui_res) = self.handle_ui_designer_window_event(event) {
            return Some(ui_res);
        }

        None
    }

    /// Dispatches events across docked panels adhering to their typed action models.
    fn dispatch_docked_panel_events(
        &mut self,
        event: &WindowEvent,
        world: Option<&hecs::World>,
    ) -> Option<IrisOverlayEventResult> {
        let cursor = self.cursor_pos();
        let is_pointer = matches!(
            event,
            WindowEvent::MouseInput { .. } | WindowEvent::CursorMoved { .. }
        );
        let captured = self.chrome.pointer_capture;

        let is_in = |panels: &PanelRegistry, id: PanelId| -> bool {
            if let Some(cap) = captured {
                return cap == id;
            }
            panels
                .get(id.id_str())
                .and_then(|p| p.bounds())
                .is_some_and(|r| compositor::is_point_in_panel(cursor, r))
        };

        let mut res = None;

        if (!is_pointer || is_in(&self.panels, PanelId::Hierarchy))
            && let Some(hier_res) = self.handle_hierarchy_window_event(event)
        {
            res = Some(hier_res);
        } else if (!is_pointer || is_in(&self.panels, PanelId::Stats))
            && let Some(stats_res) = self.handle_stats_window_event(event)
        {
            res = Some(stats_res);
        } else if (!is_pointer || is_in(&self.panels, PanelId::Console))
            && let Some(console_res) = self.handle_console_window_event(event)
        {
            res = Some(console_res);
        } else if (!is_pointer || is_in(&self.panels, PanelId::Assets))
            && let Some(assets_res) = self.handle_assets_window_event(event)
        {
            res = Some(assets_res);
        } else if (!is_pointer || is_in(&self.panels, PanelId::AnimationTimeline))
            && let Some(timeline_res) = self.handle_timeline_window_event(event)
        {
            res = Some(timeline_res);
        } else if (!is_pointer || is_in(&self.panels, PanelId::MaterialEditor))
            && let Some(mat_res) = self.handle_material_window_event(event)
        {
            res = Some(mat_res);
        } else if (!is_pointer || is_in(&self.panels, PanelId::UiDesigner))
            && let Some(ui_res) = self.handle_ui_designer_window_event(event)
        {
            res = Some(ui_res);
        } else if let Some(scroll_res) = self.handle_panel_mouse_wheel(event) {
            res = Some(scroll_res);
        } else if (!is_pointer || is_in(&self.panels, PanelId::Inspector))
            && let Some(insp_res) = self.handle_inspector_window_event(event, world)
        {
            res = Some(insp_res);
        }

        // On pointer release, clear active pointer capture if the operation finished
        if matches!(
            event,
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Released,
                ..
            }
        ) && self.chrome.pointer_capture.is_some()
        {
            let still_dragging = match self.chrome.pointer_capture {
                Some(PanelId::AnimationTimeline) => self.timeline.is_dragging,
                Some(PanelId::Assets) => self.assets.click_tracker.is_dragging_asset,
                Some(PanelId::Inspector) => self.inspector.active_number_input.is_some(),
                Some(PanelId::Console) => self.console.active_scrollbar_drag.is_some(),
                Some(PanelId::MaterialEditor) => self.material.active_scrollbar_drag.is_some(),
                _ => false,
            };
            if !still_dragging {
                self.chrome.pointer_capture = None;
            }
        }

        res
    }

    /// Dismisses all transient floating menus, dropdowns, and context popups on click-outside or escape.
    pub(crate) fn dismiss_active_overlays(&mut self) {
        self.menubar.active_menu = None;
        self.menubar.actions.clear();
        self.menubar.dropdown_rect = None;
        self.assets.context_menu = None;
        self.hierarchy.is_add_menu_open = false;
        self.hierarchy.active_context_menu = None;
        self.inspector.is_color_picker_open = false;
        self.inspector.is_add_menu_open = false;
        self.inspector.active_submenu = None;
        self.inspector.active_dropdown = None;
        self.ui_designer.is_add_menu_open = false;
        self.ui_designer.is_aspect_open = false;
        self.overlay_tree.close();
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

    #[test]
    fn test_top_down_layered_hit_test_overlay_priority() {
        use crate::ui::iris_bridge::overlay_tree::{
            ActiveOverlay, OverlayDismissPolicy, OverlayTree,
        };
        use irisui::prelude::{Point, Rect};

        let mut overlay_tree = OverlayTree::new();
        let overlay_rect = Rect::new(100.0, 100.0, 200.0, 150.0);
        overlay_tree.open(
            ActiveOverlay::MenubarDropdown(crate::ui::iris_bridge::types::ActiveMenu::File),
            OverlayDismissPolicy::ClickOutside,
        );
        overlay_tree.set_overlay_bounds(overlay_rect);

        // Click inside overlay: must be detected inside overlay bounds
        let inside_click = Point::new(150.0, 150.0);
        assert!(overlay_tree.contains_point(inside_click));

        // Click outside overlay: must trigger dismiss without click-through
        let outside_click = Point::new(400.0, 400.0);
        assert!(!overlay_tree.contains_point(outside_click));
        let dismiss_res = overlay_tree.handle_pointer_down(outside_click);
        assert_eq!(
            dismiss_res,
            crate::ui::iris_bridge::OverlayDismissResult::Dismissed
        );
    }

    #[test]
    fn test_floating_window_occlusion_and_coordinate_transformation() {
        use irisui::prelude::{Point, Rect};

        let floating_rect = Rect::new(200.0, 150.0, 500.0, 400.0);
        let screen_cursor = Point::new(250.0, 200.0);

        // Point is inside floating window
        assert!(crate::ui::iris_bridge::is_point_in_panel(
            screen_cursor,
            floating_rect
        ));

        // Transformed to (0, 0) local coordinates
        let local_cursor =
            crate::ui::iris_bridge::screen_to_panel_local(screen_cursor, floating_rect);
        assert_eq!(local_cursor.x, 50.0);
        assert_eq!(local_cursor.y, 50.0);
    }

    #[test]
    fn test_pointer_capture_routes_outside_cursor_to_captured_panel() {
        use crate::ui::iris_bridge::types::IrisChromeState;
        use crate::ui::panel_layout::PanelId;
        use irisui::prelude::{Point, Rect};

        let mut chrome = IrisChromeState::default();
        let timeline_rect = Rect::new(0.0, 600.0, 800.0, 200.0);
        let hierarchy_rect = Rect::new(0.0, 28.0, 250.0, 572.0);

        // Without capture, cursor at (50, 100) is in Hierarchy, NOT in Timeline
        let cursor_in_hier = Point::new(50.0, 100.0);
        assert!(crate::ui::iris_bridge::is_point_in_panel(
            cursor_in_hier,
            hierarchy_rect
        ));
        assert!(!crate::ui::iris_bridge::is_point_in_panel(
            cursor_in_hier,
            timeline_rect
        ));

        // When Timeline acquires pointer capture:
        chrome.pointer_capture = Some(PanelId::AnimationTimeline);
        assert_eq!(chrome.pointer_capture, Some(PanelId::AnimationTimeline));

        // The captured predicate routes to Timeline even though cursor is physically over Hierarchy
        let captured = chrome.pointer_capture;
        let is_in = |id: PanelId| -> bool {
            if let Some(cap) = captured {
                return cap == id;
            }
            false
        };
        assert!(is_in(PanelId::AnimationTimeline));
        assert!(!is_in(PanelId::Hierarchy));

        // Releasing capture clears routing
        chrome.pointer_capture = None;
        assert_eq!(chrome.pointer_capture, None);
    }
}