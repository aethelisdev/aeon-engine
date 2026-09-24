// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Overlay lifecycle, initialization, and tree reconstruction subsystem for Iris UI editor overlays.

use super::about::build_about_dialog;
use super::hierarchy;
use super::menubar;
use super::modals::{
    self, build_delete_modal, build_loading_overlay, build_new_folder_modal, build_rename_modal,
};
use super::preferences::{self, build_preferences_dialog};
use super::status_bar;
use super::theme::*;
use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;
use irisui::text::TextSystem;

impl IrisEditorOverlay {
    /// Height of the top menubar panel in physical pixels.
    pub const MENUBAR_HEIGHT: f32 = menubar::MENUBAR_HEIGHT;

    /// Height of the bottom status bar in physical pixels.
    pub const STATUS_BAR_HEIGHT: f32 = status_bar::STATUS_BAR_HEIGHT;

    /// Initializes a new Iris UI editor overlay pipeline for the specified surface format.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        Self {
            tree: UiTree::new(),
            layout_engine: LayoutEngine::new(),
            renderer: IrisRenderer::new(device, target_format),
            text_system: TextSystem::new(),
            text_renderer: None,
            command_list: DrawCommandList::new(),
            notifier: UiNotifier::new(),
            panels: super::update_panels::create_default_panel_registry(),
            screen_width: 1920.0,
            screen_height: 1080.0,
            is_visible: true,
            target_format,
            start_time: std::time::Instant::now(),
            tools_texture: None,
            chrome: super::types::IrisChromeState::default(),
            menubar: super::types::MenubarOverlayState::default(),
            modals: super::modals::types::ModalsOverlayState::default(),
            preferences: super::preferences::types::PreferencesDialogState::default(),
            viewport_hud: super::viewport_hud::types::ViewportHudState::default(),
            stats: super::stats::types::StatsPanelState::default(),
            hierarchy: super::hierarchy::types::HierarchyPanelState::default(),
            console: super::console::types::ConsolePanelState::default(),
            assets: super::assets::types::AssetsPanelState::default(),
            timeline: super::timeline::types::TimelinePanelState::default(),
            material: super::material::types::MaterialPanelState::default(),
            ui_designer: super::ui_designer::types::UiDesignerPanelState::default(),
            inspector: super::inspector::types::InspectorPanelState::default(),
        }
    }

    /// Reconstructs and resolves layout for the top menu bar, active dropdown, modals, and bottom status bar.
    pub fn update_overlays(&mut self, params: OverlayUpdateParams<'_>) {
        let (screen_width, screen_height) = params.context.dimensions;
        self.screen_width = screen_width;
        self.screen_height = screen_height;

        let modal_active = params.dialogs.show_about
            || params.dialogs.show_preferences
            || params.dialogs.delete_target.is_some()
            || params.dialogs.new_folder_parent.is_some()
            || params.dialogs.rename_target.is_some()
            || params.dialogs.is_loading_assets;

        let floating_count = params
            .context
            .layout_state
            .dock_state
            .floating_windows
            .len();

        let has_drag_payload = params.panel_data.asset_browser.drag_payload.is_some();
        let cursor_moved = (self.chrome.last_cursor_pos.x - self.cursor_pos().x).abs() > 0.001
            || (self.chrome.last_cursor_pos.y - self.cursor_pos().y).abs() > 0.001;
        let hovered_target = self
            .tree
            .hit_test_target(self.cursor_pos())
            .and_then(|h| if h.tag != 0 { Some(h.tag) } else { None });
        let hover_target_changed = cursor_moved && hovered_target != self.chrome.last_hovered_tag;
        self.chrome.hovered_tag = hovered_target;

        let mat_entity_changed = self.material.last_selected_entity != params.scene.selected_entity;
        let mat_scroll_changed =
            (self.material.last_scroll_y - self.material.scroll_y).abs() > 0.001;
        let mat_dirty = mat_entity_changed || mat_scroll_changed;
        if mat_dirty {
            self.material.last_selected_entity = params.scene.selected_entity;
            self.material.last_scroll_y = self.material.scroll_y;
        }

        let tl_entity_changed = self.timeline.last_selected_entity != params.scene.selected_entity;
        let tl_drag_changed = self.timeline.last_is_dragging != self.timeline.is_dragging;
        let tl_dirty = tl_entity_changed || tl_drag_changed;
        if tl_dirty {
            self.timeline.last_selected_entity = params.scene.selected_entity;
            self.timeline.last_is_dragging = self.timeline.is_dragging;
        }

        let cam_p = params.viewport.camera.position;
        let cam_pos_changed = (self.viewport_hud.last_camera_pos.0 - cam_p.x).abs() > 0.05
            || (self.viewport_hud.last_camera_pos.1 - cam_p.y).abs() > 0.05
            || (self.viewport_hud.last_camera_pos.2 - cam_p.z).abs() > 0.05;
        let pitch_rad = params.viewport.camera.pitch.0;
        let yaw_rad = params.viewport.camera.yaw.0;
        let cam_rot_changed = (self.viewport_hud.last_camera_rot.0 - pitch_rad).abs() > 0.005
            || (self.viewport_hud.last_camera_rot.1 - yaw_rad).abs() > 0.005;
        let gizmo_changed = self.viewport_hud.last_gizmo_mode != Some(params.viewport.gizmo_mode)
            || self.viewport_hud.last_gizmo_space != Some(params.viewport.gizmo_space);
        let vp_rect = params.viewport.viewport_rect;
        let hud_dirty = cam_pos_changed
            || cam_rot_changed
            || gizmo_changed
            || self.viewport_hud.last_wireframe != params.viewport.wireframe_enabled
            || self.viewport_hud.last_is_editing != params.context.is_editing
            || self.viewport_hud.last_is_2d != params.context.is_2d_mode
            || self.viewport_hud.last_selected_entity != params.scene.selected_entity
            || self.viewport_hud.last_viewport_rect != vp_rect;

        if hud_dirty {
            self.viewport_hud.last_camera_pos = (cam_p.x, cam_p.y, cam_p.z);
            self.viewport_hud.last_camera_rot = (pitch_rad, yaw_rad);
            self.viewport_hud.last_gizmo_mode = Some(params.viewport.gizmo_mode);
            self.viewport_hud.last_gizmo_space = Some(params.viewport.gizmo_space);
            self.viewport_hud.last_wireframe = params.viewport.wireframe_enabled;
            self.viewport_hud.last_is_editing = params.context.is_editing;
            self.viewport_hud.last_is_2d = params.context.is_2d_mode;
            self.viewport_hud.last_selected_entity = params.scene.selected_entity;
            self.viewport_hud.last_viewport_rect = vp_rect;
        }

        if self.chrome.last_dimensions != params.context.dimensions
            || (self.chrome.last_zoom_factor - params.context.zoom_factor).abs() > 1e-4
            || self.chrome.last_floating_count != floating_count
            || self.modals.last_modal_active != modal_active
            || self.preferences.last_tab != self.preferences.tab
            || (self.preferences.last_scroll_y - self.preferences.scroll_y).abs() > 0.001
            || self.chrome.last_has_viewport_texture != params.viewport.has_viewport_texture
            || self.chrome.last_has_drag_payload != has_drag_payload
            || self.menubar.active_menu.is_some()
            || self.viewport_hud.dropdown.is_some()
            || self.chrome.active_dock_overflow.is_some()
            || self.chrome.needs_layout_rebuild
            || has_drag_payload
            || hover_target_changed
            || hud_dirty
            || mat_dirty
            || tl_dirty
            || self.ui_designer.is_aspect_open
            || self.ui_designer.is_add_menu_open
            || self.ui_designer.is_panning
            || self.ui_designer.drag_state.is_some()
        {
            self.notifier.tag_all();
        }

        if self.inspector.last_selected_entity != params.scene.selected_entity {
            self.notifier.tag_redraw("inspector");
        }

        // If the Stats & Telemetry panel is active, redraw it every frame so that the
        // frame pacing oscilloscope, 1% low, 0.1% low, and CPU/GPU pass bars update live (0ms lag).
        // Only the numerical FPS text snapshot is windowed to 250ms via rolling frame-count
        // to ensure rock-solid legibility without slot-machine jitter.
        if params.panel_rects.stats.is_some() {
            if self.stats.frame_counter == 0 {
                self.stats.displayed_fps = params.telemetry.fps;
                self.stats.last_fps_refresh = std::time::Instant::now();
            }
            self.stats.frame_counter += 1;
            let now = std::time::Instant::now();
            let elapsed = now
                .duration_since(self.stats.last_fps_refresh)
                .as_secs_f32();
            if elapsed >= 0.25 {
                self.stats.displayed_fps = self.stats.frame_counter as f32 / elapsed;
                self.stats.frame_counter = 0;
                self.stats.last_fps_refresh = now;
            }
            self.notifier.tag_redraw("stats");
        } else {
            self.stats.frame_counter = 0;
        }

        // Poll registered panels for internal reactive changes
        self.notifier.poll_registry(&self.panels);

        if !self.is_visible {
            self.command_list.clear();
            return;
        }

        if !self.notifier.is_any_dirty() && !self.tree.is_empty() {
            // UI is completely clean and sleeping; zero allocations, zero panel flicker or erasure
            self.chrome.last_cursor_pos = self.cursor_pos();
            self.chrome.last_hovered_tag = self.chrome.hovered_tag;
            return;
        }

        let cursor = self.cursor_pos();

        self.tree.clear();
        self.layout_engine.clear();
        self.command_list.clear();
        self.menubar.actions.clear();
        self.chrome.floating_window_rects.clear();
        self.menubar.dropdown_rect = None;
        self.modals.is_about_active = false;
        self.modals.is_delete_active = false;
        self.modals.is_new_folder_active = false;
        self.modals.is_rename_active = false;
        self.modals.is_loading_active = false;
        self.preferences.targets = None;
        self.viewport_hud.is_active = false;
        self.inspector.targets = None;
        self.console.targets = None;
        self.assets.targets = None;
        self.material.targets = None;
        self.ui_designer.targets = None;

        if !self.assets.is_search_focused {
            self.assets.search_query = params.panel_data.asset_browser.search_query.clone();
        }
        self.assets.current_folder = params.panel_data.asset_browser.current_folder.clone();
        self.timeline.selected_entity = params.scene.selected_entity;
        if self.material.selected_entity != params.scene.selected_entity {
            self.material.selected_entity = params.scene.selected_entity;
            self.material.scroll_y = 0.0;
        }

        // Safely commit pending inspector edits on selection change
        if let Some(session) = self.inspector.active_number_input.take() {
            if Some(session.entity) != params.scene.selected_entity {
                if let Ok(v) =
                    super::inspector::evaluate_inspector_math(&session.buffer, session.initial_val)
                {
                    self.inspector
                        .actions
                        .push(super::inspector::InspectorAction::SetNumberValue(
                            session.entity,
                            session.id,
                            v,
                        ));
                    self.inspector.actions.push(
                        super::inspector::InspectorAction::CommitNumberEdit(
                            session.entity,
                            session.id,
                        ),
                    );
                } else {
                    self.inspector.edit_start_snapshot = None;
                }
            } else {
                self.inspector.active_number_input = Some(session);
            }
        }
        if let Some((ent, id, buf)) = self.inspector.active_text_input.take() {
            if Some(ent) != params.scene.selected_entity {
                self.inspector
                    .actions
                    .push(super::inspector::InspectorAction::SetTextValue(
                        ent, id, buf,
                    ));
            } else {
                self.inspector.active_text_input = Some((ent, id, buf));
            }
        }
        if let Some((ent, buf)) = self.inspector.rename_buffer.take() {
            if Some(ent) != params.scene.selected_entity {
                if !buf.trim().is_empty() {
                    self.inspector
                        .actions
                        .push(super::inspector::InspectorAction::RenameEntity(ent, buf));
                }
            } else {
                self.inspector.rename_buffer = Some((ent, buf));
            }
        }

        let Ok(root) = self.tree.create_root() else {
            return;
        };

        // Root container spans full viewport with column layout
        if let Some(node) = self.tree.get_mut(root) {
            node.set_name("IrisEditorRoot");
            node.interactive = false;
            node.set_style(
                Style::new()
                    .flex_col()
                    .justify_content(JustifyContent::SpaceBetween)
                    .width(screen_width)
                    .height(screen_height),
            );
        }

        // 1. Top MenuBar
        let menu_output = menubar::build_top_menu_bar(
            &mut self.tree,
            root,
            screen_width,
            self.menubar.active_menu,
            params.context.is_editing,
            cursor,
        );
        self.menubar.button_ids = menu_output.menu_button_ids.to_vec();

        // 2. Bottom Diagnostics & Status Bar
        let _status_bar_id = status_bar::build_bottom_status_bar(
            &mut self.tree,
            root,
            &status_bar::StatusBarParams {
                screen_width,
                screen_height,
                status_spans: params.context.status_spans,
            },
        );

        // Pre-measure all text nodes to populate intrinsic content_size
        self.measure_tree_text(root);

        // Compute Taffy layout for top menu bar and status bar (only 2 children for clean SpaceBetween pinning)
        let _ = self
            .layout_engine
            .compute_layout(&mut self.tree, Size::new(screen_width, screen_height));

        // 2b. Baseline dividers under MenuBar and above StatusBar (added after Taffy layout to not alter SpaceBetween)
        let top_bar_divider = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(top_bar_divider) {
            node.set_name("IrisTopMenuBarDivider");
            node.computed_rect = Rect::new(0.0, Self::MENUBAR_HEIGHT - 1.0, screen_width, 1.0);
            node.style = Style::new().background(BORDER_MICRON);
        }
        let _ = self.tree.add_child(root, top_bar_divider);

        let bottom_bar_divider = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(bottom_bar_divider) {
            node.set_name("IrisBottomStatusBarDivider");
            node.computed_rect = Rect::new(
                0.0,
                screen_height - Self::STATUS_BAR_HEIGHT,
                screen_width,
                1.0,
            );
            node.style = Style::new().background(BORDER_MICRON);
        }
        let _ = self.tree.add_child(root, bottom_bar_divider);

        // 3. Build Native Iris UI Docking Frame (Splitters, Tab Strips, Compact Snug Tabs, Active Indicators)
        let workspace_rect = Rect::new(
            0.0,
            Self::MENUBAR_HEIGHT,
            screen_width,
            (screen_height - Self::MENUBAR_HEIGHT - Self::STATUS_BAR_HEIGHT).max(1.0),
        );
        let pref_rect = if params.dialogs.show_preferences {
            let (left, top) = if let Some(pos) = self.preferences.pos {
                (pos.x, pos.y)
            } else {
                (
                    ((screen_width - preferences::PREF_CARD_WIDTH) * 0.5).max(0.0),
                    ((screen_height - preferences::PREF_CARD_HEIGHT) * 0.5).max(28.0),
                )
            };
            Some(Rect::new(
                left,
                top,
                preferences::PREF_CARD_WIDTH,
                preferences::PREF_CARD_HEIGHT,
            ))
        } else {
            None
        };

        let is_cursor_occluded = self.is_point_over_modal_or_dropdown(cursor)
            || pref_rect.is_some_and(|r| r.contains_point(cursor))
            || params
                .context
                .layout_state
                .dock_state
                .floating_windows
                .iter()
                .any(|w| {
                    Rect::new(w.rect.x, w.rect.y, w.rect.width, w.rect.height)
                        .contains_point(cursor)
                });

        // 3. Native Iris Docking Framework: construct full dock tree (splitters, container tabs, content rects)
        let dock_frame = super::native_dock::build_native_dock(
            &mut self.tree,
            root,
            params.context.layout_state,
            workspace_rect,
            cursor,
            is_cursor_occluded,
        );
        self.chrome.native_dock_frame = Some(dock_frame);

        let is_floating = |panel: crate::ui::panel_layout::PanelId| {
            super::floating_layer::is_panel_in_floating_window(params.context.layout_state, panel)
        };

        // 4. If Viewport canvas is valid and docked, build Viewport content (3D scene texture + HUD)
        if !is_floating(crate::ui::panel_layout::PanelId::Viewport)
            && params.viewport.viewport_rect.width > 20.0
            && params.viewport.viewport_rect.height > 20.0
        {
            self.build_viewport_content(root, params.viewport.viewport_rect, &params);
        }

        // 4. Layer 0: Render all DOCKED panels first (Z-Index: Background Workspace Layer)
        for &panel in crate::ui::panel_layout::PanelId::all_tool_panels() {
            if !is_floating(panel) {
                self.render_panel_by_id(panel, root, &params);
            }
        }

        // 5. Layer 1: Render all FLOATING Windows and their Active Panels (Z-Index: Foreground Layer)
        let (floating_rects, floating_containers) = super::floating_layer::build_floating_windows(
            &mut self.tree,
            root,
            params.context.layout_state,
            cursor,
        );
        self.chrome.floating_window_rects = floating_rects;

        for (panel_id, container_id) in floating_containers {
            self.render_panel_by_id(panel_id, container_id, &params);
        }

        // 6. FLOATING OVERLAYS (Rendered on top of docked and floating panels):

        // 6b. If Preferences dialogue is active, build its floating card
        if params.dialogs.show_preferences {
            let blink_caret = (self.start_time.elapsed().as_millis() % 1060) < 530;
            let (pref_id, targets) = build_preferences_dialog(
                &mut self.tree,
                preferences::PreferencesParams {
                    screen_width,
                    screen_height,
                    window_pos: self.preferences.pos,
                    active_tab: self.preferences.tab,
                    scroll_offset_y: self.preferences.scroll_y,
                    is_scrollbar_dragging: self.preferences.active_scrollbar_drag.is_some(),
                    active_dropdown: self.preferences.dropdown,
                    collapsed_sections: &self.preferences.collapsed_sections,
                    active_number_input: self
                        .preferences
                        .active_number_input
                        .as_ref()
                        .map(|(id, s)| (*id, s.as_str())),
                    blink_caret,
                    cursor_pos: cursor,
                    zoom_factor: params.context.zoom_factor,
                    graphics_settings: params.preferences.graphics_settings,
                    snapping_settings: params.preferences.snapping_settings,
                    editor_config: params.preferences.editor_config,
                    enable_live_updates: params.context.enable_live_updates,
                    enabled_modules: params.preferences.enabled_modules,
                },
            );
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, pref_id);
            }
            self.preferences.targets = Some(targets);
        }

        // 6c. If About Aeon Engine modal dialogue is active, build its centered card
        if params.dialogs.show_about {
            let pending_events = std::mem::take(&mut self.modals.pending_interaction_events);
            let hovered_id = self.tree.hit_test_layered(cursor);
            let _about_id = build_about_dialog(
                &mut self.tree,
                screen_width,
                screen_height,
                cursor,
                &pending_events,
                hovered_id,
            );
            self.modals.is_about_active = true;
        }

        // 6d. If Delete Confirmation modal is active, build its card
        if let Some(target_path) = params.dialogs.delete_target {
            let _del_id = build_delete_modal(
                &mut self.tree,
                target_path,
                screen_width,
                screen_height,
                cursor,
            );
            self.modals.is_delete_active = true;
        }

        let elapsed_secs = self.start_time.elapsed().as_secs_f32();
        let cursor_blink_visible = (self.start_time.elapsed().as_millis() / 530).is_multiple_of(2);

        // 6e. If New Folder modal is active, build its card
        if let Some(parent_path) = params.dialogs.new_folder_parent {
            let input_name = self.modals.new_folder_buffer.as_str();
            let text_width = self
                .text_system
                .measure_text(input_name, 12.0, 28.0, None)
                .width;
            let _folder_id = build_new_folder_modal(
                &mut self.tree,
                modals::FolderModalParams {
                    parent_path,
                    input_text: input_name,
                    text_width,
                    cursor_blink_visible,
                    screen_width,
                    screen_height,
                    cursor_pos: cursor,
                },
            );
            self.modals.is_new_folder_active = true;
        }

        // 6f. If Rename modal is active, build its card
        if let Some((target_path, is_folder)) = params.dialogs.rename_target {
            let input_name = self.modals.rename_buffer.as_str();
            let text_width = self
                .text_system
                .measure_text(input_name, 12.0, 28.0, None)
                .width;
            let _rename_id = build_rename_modal(
                &mut self.tree,
                modals::RenameModalParams {
                    target_path,
                    input_text: input_name,
                    text_width,
                    is_folder,
                    cursor_blink_visible,
                    screen_width,
                    screen_height,
                    cursor_pos: cursor,
                },
            );
            self.modals.is_rename_active = true;
        }

        // 6g. If Asset Loading overlay is active, build its splash screen
        if params.dialogs.is_loading_assets {
            let _loading_id = build_loading_overlay(
                &mut self.tree,
                modals::LoadingOverlayParams {
                    screen_width,
                    screen_height,
                    time_secs: elapsed_secs,
                },
            );
            self.modals.is_loading_active = true;
        }

        // 6h. Hierarchy Add Menu and Context Menu (Rendered as topmost floating overlays)
        if let Some(hier_rect) = params.panel_rects.hierarchy {
            let hier_params = hierarchy::HierarchyPanelParams {
                panel_rect: hier_rect,
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                search_query: &self.hierarchy.search_query,
                is_editing: params.context.is_editing,
                is_2d: params.context.is_2d_mode,
                scroll_y: self.hierarchy.scroll_y,
                active_submenu: self.hierarchy.active_submenu,
                active_sub_submenu: self.hierarchy.active_sub_submenu,
                is_add_menu_open: self.hierarchy.is_add_menu_open,
                active_context_menu: self.hierarchy.active_context_menu,
                cursor_pos: cursor,
                is_search_focused: self.hierarchy.is_search_focused,
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
                collapsed_entities: &self.hierarchy.collapsed_entities,
                hovered_tag: self.chrome.hovered_tag,
            };
            hierarchy::build_hierarchy_overlays(&mut self.tree, root, &hier_params);
        }

        // 6i. Native Dock Drag Overlays (5-way compass navigator, drop zone preview, floating tab badge)
        // Rendered as topmost floating overlays so they are drawn above the 3D Viewport texture,
        // docked panels, and floating windows.
        super::native_dock::build_native_dock_drag_overlays(
            &mut self.tree,
            root,
            params.context.layout_state,
            workspace_rect,
        );

        // 6j. Asset Drag & Viewport Drop Overlays (Landing ring on ground plane & cursor tooltip badge)
        if let Some(payload) = &params.panel_data.asset_browser.drag_payload {
            super::assets::build_asset_drag_overlays(
                &mut self.tree,
                root,
                payload,
                cursor,
                params.viewport.viewport_rect,
                params.viewport.camera,
                params.context.is_2d_mode,
            );
        }

        // 6k. Top Menubar Dropdown Popup (Rendered as topmost overlay above all docked panels,
        // floating windows, and modal dialogs so it always has absolute top visual hierarchy)
        if let Some(active) = self.menubar.active_menu {
            let anchor_x = self
                .menubar
                .button_ids
                .iter()
                .find(|(m, _)| *m == active)
                .and_then(|(_, id)| self.tree.get(*id))
                .map(|n| n.computed_rect.x)
                .unwrap_or(6.0);

            let (_dropdown_id, actions, dd_rect) = menubar::build_floating_dropdown(
                &mut self.tree,
                root,
                menubar::DropdownMenuParams {
                    active,
                    anchor_x,
                    layout_state: params.context.layout_state,
                    can_undo: params.context.can_undo,
                    can_redo: params.context.can_redo,
                    cursor_pos: cursor,
                },
            );

            self.menubar.actions = actions;
            self.menubar.dropdown_rect = Some(dd_rect);
        }

        // 6l. Native Dock Tab Overflow Dropdown Menu (Rendered at top overlay layer)
        if let Some((leaf_id, anchor_rect)) = self.chrome.active_dock_overflow
            && let Some(ref mut frame) = self.chrome.native_dock_frame
        {
            super::native_dock::build_native_dock_overflow_menu(
                &mut self.tree,
                root,
                super::native_dock::NativeDockOverflowMenuParams {
                    leaf_id,
                    anchor_rect,
                    layout_state: params.context.layout_state,
                    cursor_pos: cursor,
                    is_cursor_occluded,
                },
                frame,
            );
        }

        // Populate DrawCommandList from resolved layout nodes (with inline oscilloscope curves)
        self.populate_draw_commands(root, None, Some(params.telemetry.frame_pacing));

        self.chrome.last_dimensions = params.context.dimensions;
        self.chrome.last_zoom_factor = params.context.zoom_factor;
        self.inspector.last_selected_entity = params.scene.selected_entity;
        self.chrome.last_floating_count = floating_count;
        self.modals.last_modal_active = modal_active;
        self.preferences.last_tab = self.preferences.tab;
        self.preferences.last_scroll_y = self.preferences.scroll_y;
        self.chrome.last_has_viewport_texture = params.viewport.has_viewport_texture;
        self.chrome.last_has_drag_payload = has_drag_payload;
        self.chrome.last_cursor_pos = self.cursor_pos();
        self.chrome.last_hovered_tag = self.chrome.hovered_tag;
        self.chrome.needs_layout_rebuild = false;
        self.notifier.clear_all();
    }

    /// Measures intrinsic text dimensions for all nodes with text content in the subtree.
    pub(crate) fn measure_tree_text(&mut self, current: WidgetId) {
        let (font_size, line_height, child_count) = {
            let Some(node) = self.tree.get(current) else {
                return;
            };
            (node.font_size, node.line_height, node.children.len())
        };

        if let Some(node) = self.tree.get(current)
            && let Some(ref text) = node.text
        {
            let measured = self
                .text_system
                .measure_text(text, font_size, line_height, None);
            if let Some(node_mut) = self.tree.get_mut(current) {
                node_mut.content_size = measured;
            }
        }

        for i in 0..child_count {
            if let Some(child) = self
                .tree
                .get(current)
                .and_then(|n| n.children.get(i).copied())
            {
                self.measure_tree_text(child);
            }
        }
    }
}