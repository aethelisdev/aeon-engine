// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Subsystem for constructing individual panel nodes (Stats, Hierarchy, Inspector, Console, Assets, Material, Timeline, UI Designer).
//!

use super::hierarchy::{self, HierarchyPanelParams};
use super::stats::{self, StatsPanelParams};
use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds the Performance Stats & Telemetry profiler panel if active in either docked or floating mode.
    pub(crate) fn build_stats_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(stats_rect) = params.panel_rects.stats
            && stats_rect.width > 20.0
            && stats_rect.height > 20.0
        {
            let local_stats_rect = Rect::new(0.0, 0.0, stats_rect.width, stats_rect.height);
            let stats_params = StatsPanelParams {
                panel_rect: local_stats_rect,
                cursor_pos: self.cursor_pos(),
                wireframe_enabled: params.viewport.wireframe_enabled,
                grid_enabled: params.viewport.grid_enabled,
                fps: self.stats.displayed_fps,
                frame_pacing: params.telemetry.frame_pacing,
                frame_pacing_stats: params.telemetry.frame_pacing_stats,
                cpu_timings: params.telemetry.cpu_timings,
                gpu_pass_timings: params.telemetry.gpu_pass_timings,
                draw_call_stats: params.telemetry.draw_call_stats,
                vram_stats: params.telemetry.vram_stats,
                render_triangles: params.telemetry.render_triangles as u64,
                render_vertices: params.telemetry.render_vertices as u64,
                gpu_adapter_name: params.telemetry.gpu_adapter_name,
                gpu_backend: params.telemetry.gpu_backend,
                active_entities_count: params.scene.active_entities_count,
                selected_entity: params.scene.selected_entity,
            };

            stats::build_stats_panel(scope, &stats_params, &mut self.stats.actions);
            self.stats.last_rect = Some(stats_rect);
            self.stats.frame_ring_buffer = Some(*params.telemetry.frame_pacing);
            self.chrome.last_zoom_factor = params.context.zoom_factor;
        } else {
            self.stats.last_rect = None;
            self.stats.frame_ring_buffer = None;
        }
    }

    /// Builds the Scene Hierarchy panel if active in either docked or floating mode.
    pub(crate) fn build_hierarchy_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(hierarchy_rect) = params.panel_rects.hierarchy
            && hierarchy_rect.width > 20.0
            && hierarchy_rect.height > 20.0
        {
            let local_hier_rect = Rect::new(0.0, 0.0, hierarchy_rect.width, hierarchy_rect.height);

            // Sync live text edit buffer from IrisUI EventState if active
            if let Some(buf) = scope
                .tree()
                .event_state
                .text_edit_buffer(hierarchy::HIERARCHY_TAG_SEARCH_INPUT)
            {
                self.hierarchy.interactions.search_input.set_text(buf);
                self.hierarchy.search_input.set_text(buf);
                if let Some(cur) = scope
                    .tree()
                    .event_state
                    .text_edit_cursor(hierarchy::HIERARCHY_TAG_SEARCH_INPUT)
                {
                    self.hierarchy.interactions.search_input.cursor_byte_idx = cur;
                    self.hierarchy.search_input.cursor_byte_idx = cur;
                }
            } else if scope.tree().event_state.frame_clicked_tag
                == Some(hierarchy::HIERARCHY_TAG_SEARCH_CLEAR)
            {
                self.hierarchy.interactions.search_input.clear();
                self.hierarchy.search_input.clear();
                self.clear_focus();
                scope.tree_mut().event_state.clear_text_edit();
            }

            let cursor_pos = self.cursor_pos();
            let blink_caret = (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2);
            let ctrl_held = self.chrome.ctrl_held;
            let shift_held = self.chrome.shift_held;
            let focus_state = &self.focus_state;
            let hierarchy = &mut self.hierarchy;
            let hier_params = HierarchyPanelParams {
                panel_rect: local_hier_rect,
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                search_query: hierarchy.interactions.search_input.text(),
                is_editing: params.context.is_editing,
                is_2d: params.context.is_2d_mode,
                active_submenu: hierarchy.active_submenu,
                active_sub_submenu: hierarchy.active_sub_submenu,
                is_add_menu_open: hierarchy.is_add_menu_open,
                active_context_menu: hierarchy.active_context_menu,
                cursor_pos,
                blink_caret,
                focus_state,
                cursor_byte_idx: hierarchy.interactions.search_input.cursor_byte_idx,
                ctrl_held,
                shift_held,
            };

            hierarchy::build_hierarchy_panel(
                scope,
                &hier_params,
                &mut hierarchy.rows_cache,
                &mut hierarchy.interactions.actions,
                &mut hierarchy.tree_state,
            );

            // Resync immediately after building hierarchy in case key events were processed in input_box_tagged
            if let Some(buf) = scope
                .tree()
                .event_state
                .text_edit_buffer(hierarchy::HIERARCHY_TAG_SEARCH_INPUT)
            {
                self.hierarchy.interactions.search_input.set_text(buf);
                self.hierarchy.search_input.set_text(buf);
                if let Some(cur) = scope
                    .tree()
                    .event_state
                    .text_edit_cursor(hierarchy::HIERARCHY_TAG_SEARCH_INPUT)
                {
                    self.hierarchy.interactions.search_input.cursor_byte_idx = cur;
                    self.hierarchy.search_input.cursor_byte_idx = cur;
                }
            }

            let mut should_clear = false;
            let mut set_query = None;
            for action in &self.hierarchy.interactions.actions {
                match action {
                    hierarchy::HierarchyAction::ClearSearchQuery => should_clear = true,
                    hierarchy::HierarchyAction::SetSearchQuery(q) => set_query = Some(q.clone()),
                    hierarchy::HierarchyAction::OpenAddMenu(_) => {
                        self.hierarchy.is_add_menu_open = true;
                        self.hierarchy.active_submenu = None;
                        self.hierarchy.active_sub_submenu = None;
                        self.notifier.tag_all();
                        self.chrome.needs_layout_rebuild = true;
                    }
                    hierarchy::HierarchyAction::CloseAddMenu => {
                        self.hierarchy.is_add_menu_open = false;
                        self.hierarchy.active_submenu = None;
                        self.hierarchy.active_sub_submenu = None;
                        self.notifier.tag_all();
                        self.chrome.needs_layout_rebuild = true;
                    }
                    _ => {}
                }
            }
            if should_clear {
                self.hierarchy.interactions.search_input.clear();
                self.hierarchy.search_input.clear();
                self.clear_focus();
                scope.tree_mut().event_state.clear_text_edit();
            } else if let Some(q) = set_query {
                self.hierarchy.interactions.search_input.set_text(&q);
                self.hierarchy.search_input.set_text(&q);
            }

            self.hierarchy.last_rect = Some(hierarchy_rect);
        } else {
            self.hierarchy.last_rect = None;
        }
    }

    /// Builds the Scene Inspector panel if active in either docked or floating mode.
    pub(crate) fn build_inspector_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(inspector_rect) = params.panel_rects.inspector
            && inspector_rect.width > 20.0
            && inspector_rect.height > 20.0
        {
            let num_input_ref = self
                .inspector
                .active_number_input
                .as_ref()
                .filter(|session| Some(session.entity) == params.scene.selected_entity)
                .map(|session| super::inspector::ActiveNumberInputState {
                    id: session.id,
                    buffer: session.buffer.as_str(),
                    cursor_idx: session.cursor_idx,
                    is_all_selected: session.is_all_selected,
                });
            let text_input_ref = self
                .inspector
                .active_text_input
                .as_ref()
                .filter(|(ent, _, _)| Some(*ent) == params.scene.selected_entity)
                .map(|(_, id, s)| (*id, s.as_str()));
            let rename_buf_ref = self
                .inspector
                .rename_input
                .as_ref()
                .filter(|(ent, _)| Some(*ent) == params.scene.selected_entity)
                .map(|(_, input)| input.text());
            let hex_buf_ref = self
                .inspector
                .hex_buffer
                .as_ref()
                .filter(|(ent, _)| Some(*ent) == params.scene.selected_entity)
                .map(|(_, s)| s.as_str());

            let local_insp_rect = Rect::new(0.0, 0.0, inspector_rect.width, inspector_rect.height);
            let insp_params = super::inspector::InspectorPanelParams {
                panel_rect: local_insp_rect,
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                inspector_euler: params.panel_data.inspector_euler,
                inspector_color_hex: params.panel_data.inspector_color_hex,
                saved_swatches: params.panel_data.saved_swatches,
                cursor_pos: self.cursor_pos(),
                active_dropdown: self.inspector.active_dropdown,
                active_submenu: self.inspector.active_submenu,
                is_add_menu_open: self.inspector.is_add_menu_open,
                is_color_picker_open: self.inspector.is_color_picker_open,
                active_number_input: num_input_ref,
                active_text_input: text_input_ref,
                active_rename_buffer: rename_buf_ref,
                is_rename_all_selected: self.inspector.rename_is_all_selected
                    && rename_buf_ref.is_some(),
                active_hex_buffer: hex_buf_ref,
                inspector_hsv: self.inspector.hsv,
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
            };

            super::inspector::build_inspector_panel(
                scope,
                &insp_params,
                &mut self.inspector.interactions.actions,
            );
            self.inspector.inspected_entity = params.scene.selected_entity;
            self.inspector.inspector_euler = *params.panel_data.inspector_euler;
            if let Some(entity) = params.scene.selected_entity {
                if let Ok(name) = params.scene.world.get::<&ae_core::ecs::Name>(entity) {
                    self.inspector.inspected_entity_name.clear();
                    self.inspector
                        .inspected_entity_name
                        .push_str(name.0.as_str());
                } else {
                    self.inspector.inspected_entity_name.clear();
                }
            } else {
                self.inspector.inspected_entity_name.clear();
            }
        } else {
            self.inspector.inspected_entity = None;
            self.inspector.inspected_entity_name.clear();
        }
    }

    /// Builds the Developer Console panel if active in either docked or floating mode.
    pub(crate) fn build_console_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(console_rect) = params.panel_rects.console
            && console_rect.width > 20.0
            && console_rect.height > 20.0
        {
            self.console.panel_rect = Some(console_rect);
            let local_console_rect = Rect::new(0.0, 0.0, console_rect.width, console_rect.height);

            // Sync live text edit buffer from IrisUI EventState if active
            if let Some(buf) = scope
                .tree()
                .event_state
                .text_edit_buffer(super::console::CONSOLE_TAG_SEARCH_INPUT)
            {
                self.console.search_input.set_text(buf);
                if let Some(cur) = scope
                    .tree()
                    .event_state
                    .text_edit_cursor(super::console::CONSOLE_TAG_SEARCH_INPUT)
                {
                    self.console.search_input.cursor_byte_idx = cur;
                }
            } else if scope.tree().event_state.frame_clicked_tag
                == Some(super::console::CONSOLE_TAG_SEARCH_CLEAR)
            {
                self.console.search_input.clear();
                self.clear_focus();
                scope.tree_mut().event_state.clear_text_edit();
            }

            let console_params = super::console::ConsolePanelParams {
                panel_rect: local_console_rect,
                entries: params.panel_data.console_entries,
                filter: self.console.filter,
                search_query: self.console.search_input.text(),
                auto_scroll: self.console.auto_scroll,
                cursor_pos: self.cursor_pos(),
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
                focus_state: &self.focus_state,
                cursor_byte_idx: self.console.search_input.cursor_byte_idx,
            };

            let mut actions = Vec::new();
            super::console::build_console_panel(scope, &console_params, &mut actions);

            // Resync immediately after building console in case key events were processed in input_box_tagged
            if let Some(buf) = scope
                .tree()
                .event_state
                .text_edit_buffer(super::console::CONSOLE_TAG_SEARCH_INPUT)
            {
                self.console.search_input.set_text(buf);
                if let Some(cur) = scope
                    .tree()
                    .event_state
                    .text_edit_cursor(super::console::CONSOLE_TAG_SEARCH_INPUT)
                {
                    self.console.search_input.cursor_byte_idx = cur;
                }
            }

            let has_console_actions = !actions.is_empty();
            for action in actions {
                match action {
                    super::console::ConsoleAction::ClearLogs => {
                        if let Ok(mut lock) = ae_editor::editor_logger::LOGGER.logs.lock() {
                            lock.clear();
                        }
                        ae_editor::editor_logger::LOGGER
                            .log_count
                            .store(0, std::sync::atomic::Ordering::Relaxed);
                        if let Some(tree) =
                            self.panel_tree_mut(crate::ui::panel_layout::PanelId::Console)
                        {
                            tree.set_scroll_offset(super::console::CONSOLE_TAG_VIEWPORT, 0.0);
                        }
                    }
                    super::console::ConsoleAction::SetFilter(level) => {
                        self.console.filter = level;
                    }
                    super::console::ConsoleAction::ToggleAutoScroll => {
                        self.console.auto_scroll = !self.console.auto_scroll;
                    }
                    super::console::ConsoleAction::FocusSearch => {
                        self.set_focus_tag(super::console::CONSOLE_TAG_SEARCH_INPUT);
                    }
                    super::console::ConsoleAction::ClearSearch => {
                        self.console.search_input.clear();
                        self.clear_focus();
                        scope.tree_mut().event_state.clear_text_edit();
                    }
                    super::console::ConsoleAction::CopyLog(_) => {}
                }
            }
            if has_console_actions {
                self.notifier.tag_all();
                self.chrome.needs_layout_rebuild = true;
            }
        } else {
            self.console.panel_rect = None;
        }
    }

    /// Builds the Content / Asset Browser panel if active in either docked or floating mode.
    pub(crate) fn build_assets_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(assets_rect) = params.panel_rects.assets
            && assets_rect.width > 20.0
            && assets_rect.height > 20.0
        {
            // Sync live text edit buffer from IrisUI EventState if active
            if let Some(buf) = scope
                .tree()
                .event_state
                .text_edit_buffer(super::assets::ASSETS_TAG_SEARCH_INPUT)
            {
                self.assets.search_input.set_text(buf);
                self.assets.interactions.search_input.set_text(buf);
                if let Some(cur) = scope
                    .tree()
                    .event_state
                    .text_edit_cursor(super::assets::ASSETS_TAG_SEARCH_INPUT)
                {
                    self.assets.search_input.cursor_byte_idx = cur;
                    self.assets.interactions.search_input.cursor_byte_idx = cur;
                }
            } else if scope.tree().event_state.frame_clicked_tag
                == Some(super::assets::ASSETS_TAG_SEARCH_CLEAR)
            {
                self.assets.search_input.clear();
                self.assets.interactions.search_input.clear();
                self.clear_focus();
                scope.tree_mut().event_state.clear_text_edit();
            }

            let is_root_folder =
                params.panel_data.asset_browser.current_folder == std::path::Path::new("assets");
            let query_lower = self.assets.search_input.text().trim().to_ascii_lowercase();

            let filtered_items: Vec<_> = params
                .panel_data
                .asset_browser
                .cached_items
                .iter()
                .filter(|item| {
                    if !params.panel_data.asset_browser.show_engine_content
                        && item.source == crate::assets::types::AssetSource::Engine
                    {
                        return false;
                    }
                    if params.context.is_2d_mode && item.is_3d {
                        return false;
                    }
                    if !params.context.is_2d_mode
                        && !item.is_3d
                        && item.category == crate::assets::types::AssetCategory::Scenes
                    {
                        return false;
                    }
                    if !is_root_folder
                        && !item
                            .path
                            .starts_with(&params.panel_data.asset_browser.current_folder)
                    {
                        return false;
                    }
                    if params.panel_data.asset_browser.active_category
                        != crate::assets::types::AssetCategory::All
                        && item.category != params.panel_data.asset_browser.active_category
                    {
                        return false;
                    }
                    if !query_lower.is_empty()
                        && !item.name.to_ascii_lowercase().contains(&query_lower)
                        && !item
                            .relative_path
                            .to_ascii_lowercase()
                            .contains(&query_lower)
                    {
                        return false;
                    }
                    true
                })
                .cloned()
                .collect();

            self.assets.selected_asset = params.panel_data.asset_browser.selected_asset.clone();
            self.assets.filtered_items_cache = filtered_items.clone();
            self.assets.subfolders_cache = params.panel_data.asset_browser.subfolders.clone();

            let cursor_pos = self.cursor_pos();
            let blink_caret = (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2);
            let screen_size = (self.screen_width, self.screen_height);
            let focus_state = &self.focus_state;

            let local_assets_rect = Rect::new(0.0, 0.0, assets_rect.width, assets_rect.height);
            let assets_params = super::assets::AssetsPanelParams {
                panel_rect: local_assets_rect,
                screen_size,
                current_folder: &self.assets.current_folder,
                search_query: self.assets.interactions.search_input.text(),
                active_category: params.panel_data.asset_browser.active_category,
                view_mode: params.panel_data.asset_browser.view_mode,
                selected_asset: params.panel_data.asset_browser.selected_asset.as_deref(),
                cached_items: &params.panel_data.asset_browser.cached_items,
                filtered_items: &filtered_items,
                is_2d_mode: params.context.is_2d_mode,
                show_engine_content: params.panel_data.asset_browser.show_engine_content,
                sidebar_width: params.panel_data.asset_browser.sidebar_width,
                sidebar_collapsed: params.panel_data.asset_browser.sidebar_collapsed,
                cursor_pos,
                blink_caret,
                active_context_menu: self.assets.context_menu.as_ref(),
                active_preview_modal: self.assets.preview_modal.as_ref(),
                subfolders: &params.panel_data.asset_browser.subfolders,
                thumbnail_layers: &self.assets.thumbnail_layers,
                focus_state,
                cursor_byte_idx: self.assets.interactions.search_input.cursor_byte_idx,
            };

            let mut actions = Vec::new();
            let metrics = super::assets::build_assets_panel(
                scope,
                &assets_params,
                &mut self.assets.tree_state,
                &mut actions,
            );

            // Resync immediately after building assets panel
            if let Some(buf) = scope
                .tree()
                .event_state
                .text_edit_buffer(super::assets::ASSETS_TAG_SEARCH_INPUT)
            {
                self.assets.search_input.set_text(buf);
                self.assets.interactions.search_input.set_text(buf);
                if let Some(cur) = scope
                    .tree()
                    .event_state
                    .text_edit_cursor(super::assets::ASSETS_TAG_SEARCH_INPUT)
                {
                    self.assets.search_input.cursor_byte_idx = cur;
                    self.assets.interactions.search_input.cursor_byte_idx = cur;
                }
            }

            for action in actions {
                match action {
                    super::assets::AssetsPanelAction::NavigateFolder(ref path) => {
                        self.assets.tree_state.select_single(path.clone());
                        let mut curr = path.as_path();
                        while let Some(parent) = curr.parent() {
                            self.assets
                                .tree_state
                                .set_expanded(parent.to_path_buf(), true);
                            curr = parent;
                        }
                        self.assets.current_folder = path.clone();
                        self.assets.actions.push(action);
                        self.notifier.tag_all();
                    }
                    super::assets::AssetsPanelAction::SelectAsset(ref opt) => {
                        self.assets.selected_asset = opt.clone();
                        self.assets.actions.push(action);
                        self.notifier.tag_all();
                    }
                    super::assets::AssetsPanelAction::FocusSearch => {
                        self.set_focus_tag(super::assets::ASSETS_TAG_SEARCH_INPUT);
                    }
                    super::assets::AssetsPanelAction::ClearSearch => {
                        self.assets.interactions.search_input.clear();
                        self.assets.search_input.clear();
                        self.clear_focus();
                        scope.tree_mut().event_state.clear_text_edit();
                        self.assets.actions.push(action);
                        self.notifier.tag_all();
                    }

                    other => {
                        self.assets.actions.push(other);
                        self.notifier.tag_all();
                    }
                }
            }

            self.assets.panel_rect = Some(assets_rect);
            self.assets.sidebar_rect = metrics
                .sidebar_rect
                .map(|r| Rect::new(r.x + assets_rect.x, r.y + assets_rect.y, r.width, r.height));
            self.assets.content_viewport_rect = Some(Rect::new(
                metrics.content_viewport_rect.x + assets_rect.x,
                metrics.content_viewport_rect.y + assets_rect.y,
                metrics.content_viewport_rect.width,
                metrics.content_viewport_rect.height,
            ));
            self.assets.context_menu_card_rect = metrics.context_menu_card_rect;
        } else {
            self.assets.panel_rect = None;
            self.assets.sidebar_rect = None;
            self.assets.content_viewport_rect = None;
            self.assets.context_menu_card_rect = None;
        }
    }

    /// Builds the Animation Timeline Studio panel if active in either docked or floating mode.
    pub(crate) fn build_timeline_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(timeline_rect) = params.panel_rects.timeline {
            let anim_player = params.scene.selected_entity.and_then(|ent| {
                params
                    .scene
                    .world
                    .get::<&ae_animation::AnimationPlayer>(ent)
                    .ok()
            });

            let events = std::mem::take(&mut self.timeline.pending_interaction_events);

            let local_timeline_rect =
                Rect::new(0.0, 0.0, timeline_rect.width, timeline_rect.height);
            let timeline_params = super::timeline::TimelinePanelParams {
                panel_rect: local_timeline_rect,
                entity: params.scene.selected_entity,
                animation_player: anim_player.as_deref(),
                cursor_pos: self.cursor_pos(),
                is_dragging_scrubber: self.timeline.is_dragging,
                events: &events,
            };

            let duration = super::timeline::build_timeline_panel(
                scope,
                &timeline_params,
                &mut self.timeline.actions,
            );
            self.timeline.panel_rect = Some(timeline_rect);
            self.timeline.clip_duration = duration;
        } else {
            self.timeline.panel_rect = None;
            self.timeline.is_dragging = false;
            self.timeline.active_scrubber_track = None;
        }
    }

    /// Builds the Material & Surface Studio panel if active in either docked or floating mode.
    pub(crate) fn build_material_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(material_rect) = params.panel_rects.material {
            let events = std::mem::take(&mut self.material.pending_interaction_events);

            let local_material_rect =
                Rect::new(0.0, 0.0, material_rect.width, material_rect.height);
            let material_params = super::material::MaterialPanelParams {
                panel_rect: local_material_rect,
                entity: params.scene.selected_entity,
                world: params.scene.world,
                textures: params.panel_data.textures,
                models: params.panel_data.models,
                cursor_pos: self.cursor_pos(),
                events: &events,
            };

            super::material::build_material_panel(
                scope,
                &material_params,
                &mut self.material.interactions.actions,
            );
            self.material.panel_rect = Some(material_rect);
            self.material.active_model = params
                .scene
                .selected_entity
                .and_then(|e| params.scene.world.get::<&ae_core::ecs::ModelId>(e).ok())
                .map(|m| m.0);
        } else {
            self.material.panel_rect = None;
            self.material.active_model = None;
        }
    }

    /// Builds the 2D Visual UI Designer panel if active in either docked or floating mode.
    pub(crate) fn build_ui_designer_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(designer_rect) = params.panel_rects.ui_designer {
            let local_designer_rect =
                Rect::new(0.0, 0.0, designer_rect.width, designer_rect.height);
            let designer_params = super::ui_designer::UiDesignerPanelParams {
                panel_rect: local_designer_rect,
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                cursor_pos: self.cursor_pos(),
                state: params.panel_data.ui_designer_state,
                is_aspect_dropdown_open: self.ui_designer.is_aspect_open,
                is_add_menu_open: self.ui_designer.is_add_menu_open,
            };

            let _metrics = super::ui_designer::build_ui_designer_panel(
                scope,
                &designer_params,
                &mut self.ui_designer,
            );
            if self.ui_designer.has_pending_actions() {
                self.notifier.tag_all();
                self.chrome.needs_layout_rebuild = true;
            }
        } else {
            self.ui_designer.drag_contexts.clear();
            self.ui_designer.canvas_metrics = Default::default();
        }
    }

    /// Dispatches rendering for a specific panel into the target container.
    ///
    /// Resolves the requested [`PanelId`] and delegates to the corresponding
    /// specialized panel builder module.
    pub(crate) fn render_panel_by_id(
        &mut self,
        panel: crate::ui::panel_layout::PanelId,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        use crate::ui::panel_layout::PanelId;
        match panel {
            PanelId::Viewport => {
                let vp_rect = super::floating_layer::active_panel_content_rect(
                    params.context.layout_state,
                    PanelId::Viewport,
                )
                .unwrap_or(params.viewport.viewport_rect);
                if vp_rect.width > 20.0 && vp_rect.height > 20.0 {
                    self.build_viewport_content(scope, vp_rect, params);
                }
            }
            PanelId::Hierarchy => self.build_hierarchy_panel_if_active(scope, params),
            PanelId::Inspector => self.build_inspector_panel_if_active(scope, params),
            PanelId::Console => self.build_console_panel_if_active(scope, params),
            PanelId::Assets => self.build_assets_panel_if_active(scope, params),
            PanelId::MaterialEditor => self.build_material_panel_if_active(scope, params),
            PanelId::AnimationTimeline => self.build_timeline_panel_if_active(scope, params),
            PanelId::UiDesigner => self.build_ui_designer_panel_if_active(scope, params),
            PanelId::Stats => self.build_stats_panel_if_active(scope, params),
        }
    }

    /// Renders an external or custom panel registered in [`Self::panels`] by its identifier.
    ///
    /// Returns `true` if a registered panel was found and rendered, or `false` otherwise.
    pub fn render_custom_panel(&mut self, panel_id: &str, parent: WidgetId, bounds: Rect) -> bool {
        let (panels, tree) = (&mut self.panels, &mut self.tree);
        if let Some(panel) = panels.get_mut(panel_id) {
            panel.render(tree, parent, bounds);
            true
        } else {
            false
        }
    }

    /// Synchronizes localized dirty state from specialized panel state structures into the dock panel registry.
    ///
    /// For each registered panel, queries its corresponding localized state's
    /// `check_and_sync_dirty` evaluation and delegates the result to the [`EditorDockPanel`].
    pub(crate) fn sync_panel_registry_dirty(&mut self, params: &OverlayUpdateParams<'_>) {
        use crate::ui::panel_layout::PanelId;

        let vp_dirty = self.viewport_hud.check_and_sync_dirty(params);
        if let Some(panel) = self
            .panels
            .get_downcast_mut::<EditorDockPanel>(PanelId::Viewport.id_str())
        {
            panel.set_dirty(vp_dirty);
        }

        let mat_dirty = self.material.check_and_sync_dirty(params);
        if let Some(panel) = self
            .panels
            .get_downcast_mut::<EditorDockPanel>(PanelId::MaterialEditor.id_str())
        {
            panel.set_dirty(mat_dirty);
        }

        let tl_dirty = self.timeline.check_and_sync_dirty(params);
        if let Some(panel) = self
            .panels
            .get_downcast_mut::<EditorDockPanel>(PanelId::AnimationTimeline.id_str())
        {
            panel.set_dirty(tl_dirty);
        }

        let ui_dirty = self.ui_designer.check_and_sync_dirty(params);
        if let Some(panel) = self
            .panels
            .get_downcast_mut::<EditorDockPanel>(PanelId::UiDesigner.id_str())
        {
            panel.set_dirty(ui_dirty);
        }

        let insp_dirty = self.inspector.check_and_sync_dirty(params);
        if let Some(panel) = self
            .panels
            .get_downcast_mut::<EditorDockPanel>(PanelId::Inspector.id_str())
        {
            panel.set_dirty(insp_dirty);
        }

        let hier_dirty = self.hierarchy.check_and_sync_dirty(params);
        if let Some(panel) = self
            .panels
            .get_downcast_mut::<EditorDockPanel>(PanelId::Hierarchy.id_str())
        {
            panel.set_dirty(hier_dirty);
        }

        let con_dirty = self.console.check_and_sync_dirty(params);
        if let Some(panel) = self
            .panels
            .get_downcast_mut::<EditorDockPanel>(PanelId::Console.id_str())
        {
            panel.set_dirty(con_dirty);
        }

        let assets_dirty = self.assets.check_and_sync_dirty(params);
        if let Some(panel) = self
            .panels
            .get_downcast_mut::<EditorDockPanel>(PanelId::Assets.id_str())
        {
            panel.set_dirty(assets_dirty);
        }

        let stats_dirty = self.stats.check_and_sync_dirty(params);
        if let Some(panel) = self
            .panels
            .get_downcast_mut::<EditorDockPanel>(PanelId::Stats.id_str())
        {
            panel.set_dirty(stats_dirty);
        }

        self.sync_panel_registry_bounds(params);
    }

    /// Synchronizes current dock bounds rectangles into registered dock panels and advances their retained lifecycles.
    ///
    /// For every active tool panel, delegates bounds tracking and in-place flexbox relayout
    /// directly to [`EditorDockPanel::update_lifecycle`]. If dimensions change, in-place relayout
    /// is performed without allocating new widget nodes and the panel is tagged for redraw in [`UiNotifier`].
    pub(crate) fn sync_panel_registry_bounds(&mut self, params: &OverlayUpdateParams<'_>) {
        use crate::ui::panel_layout::PanelId;

        let (panels, layout_engine, notifier) = (
            &mut self.panels,
            &mut self.layout_engine,
            &mut self.notifier,
        );

        for &panel_id in PanelId::all_tool_panels() {
            let rect_opt = match panel_id {
                PanelId::Viewport => Some(params.viewport.viewport_rect),
                PanelId::Hierarchy => params.panel_rects.hierarchy,
                PanelId::Inspector => params.panel_rects.inspector,
                PanelId::Console => params.panel_rects.console,
                PanelId::Assets => params.panel_rects.assets,
                PanelId::MaterialEditor => params.panel_rects.material,
                PanelId::AnimationTimeline => params.panel_rects.timeline,
                PanelId::UiDesigner => params.panel_rects.ui_designer,
                PanelId::Stats => params.panel_rects.stats,
            };

            if let Some(panel) = panels.get_downcast_mut::<EditorDockPanel>(panel_id.id_str()) {
                if let Some(rect) = rect_opt {
                    let result = panel.update_lifecycle(layout_engine, rect);
                    if result == PanelLifecycleResult::RelayoutInPlace {
                        notifier.tag_redraw(panel_id);
                    }
                } else {
                    panel.clear_bounds();
                }
            }
        }
    }
}

pub use super::dock_panel::{EditorDockPanel, PanelLifecycleResult, create_default_panel_registry};

#[path = "update_panels_tests.rs"]
#[cfg(test)]
mod tests;