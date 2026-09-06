// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Subsystem for constructing individual panel nodes (Stats, Hierarchy, Inspector, Console, Assets, Material, Timeline, UI Designer).
//!

use super::hierarchy::{self, HierarchyPanelParams, HierarchyPanelTargets};
use super::stats::{self, StatsPanelParams, StatsPanelTargets};
use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds the Performance Stats & Telemetry profiler panel if active in either docked or floating mode.
    pub(crate) fn build_stats_panel_if_active(
        &mut self,
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(stats_rect) = params.stats_panel_rect
            && stats_rect.width > 20.0
            && stats_rect.height > 20.0
        {
            let stats_params = StatsPanelParams {
                panel_rect: stats_rect,
                scroll_y: self.stats_scroll_y,
                cursor_pos: self.cursor_pos,
                wireframe_enabled: params.wireframe_enabled,
                grid_enabled: params.grid_enabled,
                fps: params.fps,
                frame_pacing: params.frame_pacing,
                frame_pacing_stats: params.frame_pacing_stats,
                cpu_timings: params.cpu_timings,
                gpu_pass_timings: params.gpu_pass_timings,
                draw_call_stats: params.draw_call_stats,
                vram_stats: params.vram_stats,
                render_triangles: params.render_triangles,
                render_vertices: params.render_vertices,
                gpu_adapter_name: params.gpu_adapter_name,
                gpu_backend: params.gpu_backend,
                active_entities_count: params.active_entities_count,
                selected_entity: params.selected_entity,
            };

            let mut stats_targets = StatsPanelTargets::default();
            let nodes =
                stats::build_stats_panel(&mut self.tree, root, &stats_params, &mut stats_targets);
            stats::update_stats_panel_values(&mut self.tree, &nodes, &stats_params, &stats_targets);
            self.stats_targets = Some(stats_targets);
            self.stats_nodes = Some(nodes);
            self.last_stats_rect = Some(stats_rect);
            self.last_zoom_factor = params.zoom_factor;
        } else {
            self.stats_nodes = None;
            self.stats_targets = None;
            self.last_stats_rect = None;
        }
    }

    /// Builds the Scene Hierarchy panel if active in either docked or floating mode.
    pub(crate) fn build_hierarchy_panel_if_active(
        &mut self,
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(hierarchy_rect) = params.hierarchy_panel_rect
            && hierarchy_rect.width > 20.0
            && hierarchy_rect.height > 20.0
        {
            let hier_params = HierarchyPanelParams {
                panel_rect: hierarchy_rect,
                world: params.world,
                selected_entity: params.selected_entity,
                search_query: &self.hierarchy_search_query,
                is_editing: params.is_editing,
                scroll_y: self.hierarchy_scroll_y,
                active_submenu: self.hierarchy_active_submenu,
                is_add_menu_open: self.hierarchy_is_add_menu_open,
                active_context_menu: self.hierarchy_active_context_menu,
                cursor_pos: self.cursor_pos,
                is_search_focused: self.hierarchy_is_search_focused,
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
            };

            let mut hier_targets = HierarchyPanelTargets::default();
            let _nodes = hierarchy::build_hierarchy_panel(
                &mut self.tree,
                root,
                &hier_params,
                &mut hier_targets,
                &mut self.hierarchy_rows_cache,
            );
            self.hierarchy_targets = Some(hier_targets);
        } else {
            self.hierarchy_targets = None;
        }
    }

    /// Builds the Scene Inspector panel if active in either docked or floating mode.
    pub(crate) fn build_inspector_panel_if_active(
        &mut self,
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(inspector_rect) = params.inspector_panel_rect
            && inspector_rect.width > 20.0
            && inspector_rect.height > 20.0
        {
            let num_input_ref = self
                .inspector_active_number_input
                .as_ref()
                .map(|(id, s)| (*id, s.as_str()));
            let text_input_ref = self
                .inspector_active_text_input
                .as_ref()
                .map(|(id, s)| (*id, s.as_str()));
            let rename_buf_ref = self.inspector_rename_buffer.as_deref();
            let hex_buf_ref = self.inspector_hex_buffer.as_deref();

            let insp_params = super::inspector::InspectorPanelParams {
                panel_rect: inspector_rect,
                world: params.world,
                selected_entity: params.selected_entity,
                inspector_euler: params.inspector_euler,
                inspector_color_hex: params.inspector_color_hex,
                saved_swatches: params.saved_swatches,
                cursor_pos: self.cursor_pos,
                scroll_y: self.inspector_scroll_y,
                active_dropdown: self.inspector_active_dropdown,
                active_submenu: self.inspector_active_submenu,
                is_add_menu_open: self.inspector_is_add_menu_open,
                is_color_picker_open: self.inspector_is_color_picker_open,
                active_number_input: num_input_ref,
                active_text_input: text_input_ref,
                active_rename_buffer: rename_buf_ref,
                active_hex_buffer: hex_buf_ref,
                inspector_hsv: self.inspector_hsv,
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
            };

            let mut insp_targets = super::inspector::InspectorPanelTargets::default();
            super::inspector::build_inspector_panel(
                &mut self.tree,
                root,
                &insp_params,
                &mut insp_targets,
            );
            self.inspector_targets = Some(insp_targets);
        } else {
            self.inspector_targets = None;
        }
    }

    /// Builds the Developer Console panel if active in either docked or floating mode.
    pub(crate) fn build_console_panel_if_active(
        &mut self,
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(console_rect) = params.console_panel_rect
            && console_rect.width > 20.0
            && console_rect.height > 20.0
        {
            let console_params = super::console::ConsolePanelParams {
                panel_rect: console_rect,
                entries: params.console_entries,
                scroll_y: self.console_scroll_y,
                filter: self.console_filter,
                search_query: &self.console_search_query,
                is_search_focused: self.console_is_search_focused,
                auto_scroll: self.console_auto_scroll,
                cursor_pos: self.cursor_pos,
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
            };

            let mut console_targets = super::console::ConsolePanelTargets::default();
            super::console::build_console_panel(
                &mut self.tree,
                root,
                &console_params,
                &mut console_targets,
            );
            self.console_targets = Some(console_targets);
        } else {
            self.console_targets = None;
        }
    }

    /// Builds the Content / Asset Browser panel if active in either docked or floating mode.
    pub(crate) fn build_assets_panel_if_active(
        &mut self,
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(assets_rect) = params.assets_panel_rect
            && assets_rect.width > 20.0
            && assets_rect.height > 20.0
        {
            let is_root_folder =
                params.asset_browser.current_folder == std::path::Path::new("assets");
            let query_lower = self.assets_search_query.trim().to_ascii_lowercase();

            let filtered_items: Vec<_> = params
                .asset_browser
                .cached_items
                .iter()
                .filter(|item| {
                    if !is_root_folder
                        && !item.path.starts_with(&params.asset_browser.current_folder)
                    {
                        return false;
                    }
                    if params.asset_browser.active_category
                        != crate::ui::panels::assets::types::AssetCategory::All
                        && item.category != params.asset_browser.active_category
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

            self.assets_selected_asset = params.asset_browser.selected_asset.clone();

            let assets_params = super::assets::AssetsPanelParams {
                panel_rect: assets_rect,
                screen_size: (self.screen_width, self.screen_height),
                current_folder: &self.assets_current_folder,
                search_query: &self.assets_search_query,
                is_search_focused: self.assets_is_search_focused,
                active_category: params.asset_browser.active_category,
                view_mode: params.asset_browser.view_mode,
                selected_asset: params.asset_browser.selected_asset.as_deref(),
                cached_items: &params.asset_browser.cached_items,
                filtered_items: &filtered_items,
                sidebar_width: params.asset_browser.sidebar_width,
                sidebar_collapsed: params.asset_browser.sidebar_collapsed,
                scroll_y: self.assets_scroll_y,
                tree_scroll_y: self.assets_tree_scroll_y,
                cursor_pos: self.cursor_pos,
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
                active_context_menu: self.assets_context_menu.as_ref(),
                active_preview_modal: self.assets_preview_modal.as_ref(),
                thumbnail_layers: &self.thumbnail_layers,
            };

            let mut assets_targets = super::assets::AssetsPanelTargets::default();
            super::assets::build_assets_panel(
                &mut self.tree,
                root,
                &assets_params,
                &mut assets_targets,
            );
            self.assets_targets = Some(assets_targets);
        } else {
            self.assets_targets = None;
        }
    }

    /// Builds the Animation Timeline Studio panel if active in either docked or floating mode.
    pub(crate) fn build_timeline_panel_if_active(
        &mut self,
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(timeline_rect) = params.timeline_panel_rect {
            let anim_player = params
                .selected_entity
                .and_then(|ent| params.world.get::<&ae_animation::AnimationPlayer>(ent).ok());

            let timeline_params = super::timeline::TimelinePanelParams {
                panel_rect: timeline_rect,
                entity: params.selected_entity,
                animation_player: anim_player.as_deref(),
                cursor_pos: self.cursor_pos,
                is_dragging_scrubber: self.timeline_is_dragging,
            };

            let mut timeline_targets = super::timeline::TimelinePanelTargets::default();
            super::timeline::build_timeline_panel(
                &mut self.tree,
                root,
                &timeline_params,
                &mut timeline_targets,
            );
            self.timeline_targets = Some(timeline_targets);
        } else {
            self.timeline_targets = None;
            self.timeline_is_dragging = false;
        }
    }

    /// Builds the Material & Surface Studio panel if active in either docked or floating mode.
    pub(crate) fn build_material_panel_if_active(
        &mut self,
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(material_rect) = params.material_panel_rect {
            let material_params = super::material::MaterialPanelParams {
                panel_rect: material_rect,
                entity: params.selected_entity,
                world: params.world,
                textures: params.textures,
                models: params.models,
                cursor_pos: self.cursor_pos,
                scroll_y: self.material_scroll_y,
            };

            let mut material_targets = super::material::MaterialPanelTargets::default();
            super::material::build_material_panel(
                &mut self.tree,
                root,
                &material_params,
                &mut material_targets,
            );
            self.material_targets = Some(material_targets);
        } else {
            self.material_targets = None;
        }
    }

    /// Builds the 2D Visual UI Designer panel if active in either docked or floating mode.
    pub(crate) fn build_ui_designer_panel_if_active(
        &mut self,
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(designer_rect) = params.ui_designer_panel_rect {
            let designer_params = super::ui_designer::UiDesignerPanelParams {
                panel_rect: designer_rect,
                world: params.world,
                selected_entity: params.selected_entity,
                cursor_pos: self.cursor_pos,
                state: params.ui_designer_state,
                is_aspect_dropdown_open: self.ui_designer_is_aspect_open,
                is_add_menu_open: self.ui_designer_is_add_menu_open,
            };

            let targets =
                super::ui_designer::build_ui_designer_panel(&mut self.tree, root, &designer_params);
            self.ui_designer_targets = Some(targets);
        } else {
            self.ui_designer_targets = None;
        }
    }
}