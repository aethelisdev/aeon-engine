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
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(stats_rect) = params.panel_rects.stats
            && stats_rect.width > 20.0
            && stats_rect.height > 20.0
        {
            let stats_params = StatsPanelParams {
                panel_rect: stats_rect,
                scroll_y: self.stats.scroll_y,
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

            let max_scroll = stats::build_stats_panel(&mut self.tree, root, &stats_params);
            self.stats.max_scroll = max_scroll;
            self.stats.last_rect = Some(stats_rect);
            self.chrome.last_zoom_factor = params.context.zoom_factor;
        } else {
            self.stats.last_rect = None;
        }
    }

    /// Builds the Scene Hierarchy panel if active in either docked or floating mode.
    pub(crate) fn build_hierarchy_panel_if_active(
        &mut self,
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(hierarchy_rect) = params.panel_rects.hierarchy
            && hierarchy_rect.width > 20.0
            && hierarchy_rect.height > 20.0
        {
            let hier_params = HierarchyPanelParams {
                panel_rect: hierarchy_rect,
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
                cursor_pos: self.cursor_pos(),
                is_search_focused: self.hierarchy.is_search_focused,
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
                collapsed_entities: &self.hierarchy.collapsed_entities,
                hovered_tag: self.chrome.hovered_tag,
            };

            let max_scroll = hierarchy::build_hierarchy_panel(
                &mut self.tree,
                root,
                &hier_params,
                &mut self.hierarchy.rows_cache,
            );
            self.hierarchy.max_scroll = max_scroll;
            self.hierarchy.last_rect = Some(hierarchy_rect);
        } else {
            self.hierarchy.last_rect = None;
        }
    }

    /// Builds the Scene Inspector panel if active in either docked or floating mode.
    pub(crate) fn build_inspector_panel_if_active(
        &mut self,
        root: WidgetId,
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
                .rename_buffer
                .as_ref()
                .filter(|(ent, _)| Some(*ent) == params.scene.selected_entity)
                .map(|(_, s)| s.as_str());
            let hex_buf_ref = self
                .inspector
                .hex_buffer
                .as_ref()
                .filter(|(ent, _)| Some(*ent) == params.scene.selected_entity)
                .map(|(_, s)| s.as_str());

            let insp_params = super::inspector::InspectorPanelParams {
                panel_rect: inspector_rect,
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                inspector_euler: params.panel_data.inspector_euler,
                inspector_color_hex: params.panel_data.inspector_color_hex,
                saved_swatches: params.panel_data.saved_swatches,
                cursor_pos: self.cursor_pos(),
                scroll_y: self.inspector.scroll_y,
                active_dropdown: self.inspector.active_dropdown,
                active_submenu: self.inspector.active_submenu,
                is_add_menu_open: self.inspector.is_add_menu_open,
                is_color_picker_open: self.inspector.is_color_picker_open,
                active_number_input: num_input_ref,
                active_text_input: text_input_ref,
                active_rename_buffer: rename_buf_ref,
                active_hex_buffer: hex_buf_ref,
                inspector_hsv: self.inspector.hsv,
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
            };

            let mut insp_targets = super::inspector::InspectorPanelTargets::default();
            super::inspector::build_inspector_panel(
                &mut self.tree,
                root,
                &insp_params,
                &mut insp_targets,
            );
            self.inspector.targets = Some(insp_targets);
        } else {
            self.inspector.targets = None;
        }
    }

    /// Builds the Developer Console panel if active in either docked or floating mode.
    pub(crate) fn build_console_panel_if_active(
        &mut self,
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(console_rect) = params.panel_rects.console
            && console_rect.width > 20.0
            && console_rect.height > 20.0
        {
            self.console.panel_rect = Some(console_rect);
            let console_params = super::console::ConsolePanelParams {
                panel_rect: console_rect,
                entries: params.panel_data.console_entries,
                scroll_y: self.console.scroll_y,
                filter: self.console.filter,
                search_query: &self.console.search_query,
                is_search_focused: self.console.is_search_focused,
                auto_scroll: self.console.auto_scroll,
                cursor_pos: self.cursor_pos(),
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
                is_scrollbar_dragging: self.console.active_scrollbar_drag.is_some(),
                hovered_tag: self.chrome.hovered_tag,
            };

            let max_scroll =
                super::console::build_console_panel(&mut self.tree, root, &console_params);
            self.console.max_scroll_y = max_scroll;
        } else {
            self.console.panel_rect = None;
        }
    }

    /// Builds the Content / Asset Browser panel if active in either docked or floating mode.
    pub(crate) fn build_assets_panel_if_active(
        &mut self,
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(assets_rect) = params.panel_rects.assets
            && assets_rect.width > 20.0
            && assets_rect.height > 20.0
        {
            let is_root_folder =
                params.panel_data.asset_browser.current_folder == std::path::Path::new("assets");
            let query_lower = self.assets.search_query.trim().to_ascii_lowercase();

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

            let assets_params = super::assets::AssetsPanelParams {
                panel_rect: assets_rect,
                screen_size: (self.screen_width, self.screen_height),
                current_folder: &self.assets.current_folder,
                search_query: &self.assets.search_query,
                is_search_focused: self.assets.is_search_focused,
                active_category: params.panel_data.asset_browser.active_category,
                view_mode: params.panel_data.asset_browser.view_mode,
                selected_asset: params.panel_data.asset_browser.selected_asset.as_deref(),
                cached_items: &params.panel_data.asset_browser.cached_items,
                filtered_items: &filtered_items,
                is_2d_mode: params.context.is_2d_mode,
                show_engine_content: params.panel_data.asset_browser.show_engine_content,
                sidebar_width: params.panel_data.asset_browser.sidebar_width,
                sidebar_collapsed: params.panel_data.asset_browser.sidebar_collapsed,
                scroll_y: self.assets.scroll_y,
                tree_scroll_y: self.assets.tree_scroll_y,
                cursor_pos: self.cursor_pos(),
                blink_caret: (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2),
                active_context_menu: self.assets.context_menu.as_ref(),
                active_preview_modal: self.assets.preview_modal.as_ref(),
                thumbnail_layers: &self.assets.thumbnail_layers,
            };

            let mut assets_targets = super::assets::AssetsPanelTargets::default();
            super::assets::build_assets_panel(
                &mut self.tree,
                root,
                &assets_params,
                &mut assets_targets,
            );
            self.assets.targets = Some(assets_targets);
        } else {
            self.assets.targets = None;
        }
    }

    /// Builds the Animation Timeline Studio panel if active in either docked or floating mode.
    pub(crate) fn build_timeline_panel_if_active(
        &mut self,
        root: WidgetId,
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

            let hovered_tag = self.chrome.hovered_tag;
            let events = std::mem::take(&mut self.timeline.pending_interaction_events);

            let timeline_params = super::timeline::TimelinePanelParams {
                panel_rect: timeline_rect,
                entity: params.scene.selected_entity,
                animation_player: anim_player.as_deref(),
                cursor_pos: self.cursor_pos(),
                is_dragging_scrubber: self.timeline.is_dragging,
                events: &events,
                hovered_tag,
            };

            let duration = super::timeline::build_timeline_panel(
                &mut self.tree,
                root,
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
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(material_rect) = params.panel_rects.material {
            let hovered_tag = self.chrome.hovered_tag;
            let events = std::mem::take(&mut self.material.pending_interaction_events);
            let is_scrollbar_dragging = self.material.active_scrollbar_drag.is_some();

            let material_params = super::material::MaterialPanelParams {
                panel_rect: material_rect,
                entity: params.scene.selected_entity,
                world: params.scene.world,
                textures: params.panel_data.textures,
                models: params.panel_data.models,
                cursor_pos: self.cursor_pos(),
                scroll_y: self.material.scroll_y,
                hovered_tag,
                events: &events,
                is_scrollbar_dragging,
            };

            let max_scroll =
                super::material::build_material_panel(&mut self.tree, root, &material_params);
            self.material.max_scroll_y = max_scroll;
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
        root: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) {
        if let Some(designer_rect) = params.panel_rects.ui_designer {
            let designer_params = super::ui_designer::UiDesignerPanelParams {
                panel_rect: designer_rect,
                world: params.scene.world,
                selected_entity: params.scene.selected_entity,
                cursor_pos: self.cursor_pos(),
                state: params.panel_data.ui_designer_state,
                is_aspect_dropdown_open: self.ui_designer.is_aspect_open,
                is_add_menu_open: self.ui_designer.is_add_menu_open,
                hovered_tag: self.chrome.hovered_tag,
            };

            let metrics = super::ui_designer::build_ui_designer_panel(
                &mut self.tree,
                root,
                &designer_params,
                &mut self.ui_designer.drag_contexts,
            );
            self.ui_designer.canvas_metrics = metrics;
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
        parent: WidgetId,
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
                    self.build_viewport_content(parent, vp_rect, params);
                }
            }
            PanelId::Hierarchy => self.build_hierarchy_panel_if_active(parent, params),
            PanelId::Inspector => self.build_inspector_panel_if_active(parent, params),
            PanelId::Console => self.build_console_panel_if_active(parent, params),
            PanelId::Assets => self.build_assets_panel_if_active(parent, params),
            PanelId::MaterialEditor => self.build_material_panel_if_active(parent, params),
            PanelId::AnimationTimeline => self.build_timeline_panel_if_active(parent, params),
            PanelId::UiDesigner => self.build_ui_designer_panel_if_active(parent, params),
            PanelId::Stats => self.build_stats_panel_if_active(parent, params),
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
}

/// Standard dockable panel implementor for built-in editor panels registered in [`PanelRegistry`].
///
/// Wraps a strongly typed [`crate::ui::panel_layout::PanelId`] and provides metadata and lifecycle
/// integration with the Iris UI docking framework.
#[derive(Debug)]
pub struct EditorDockPanel {
    id: String,
    title: String,
    panel_id: crate::ui::panel_layout::PanelId,
}

impl EditorDockPanel {
    /// Creates a new editor panel descriptor for the given [`crate::ui::panel_layout::PanelId`].
    pub fn new(panel_id: crate::ui::panel_layout::PanelId) -> Self {
        Self {
            id: panel_id.id_str().to_string(),
            title: panel_id.title().to_string(),
            panel_id,
        }
    }

    /// Returns the associated [`crate::ui::panel_layout::PanelId`].
    pub fn panel_id(&self) -> crate::ui::panel_layout::PanelId {
        self.panel_id
    }
}

impl DockPanel for EditorDockPanel {
    fn id(&self) -> &str {
        &self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn render(&mut self, _tree: &mut UiTree, _parent: WidgetId, _bounds: Rect) {
        // Built-in editor panels require engine state (OverlayUpdateParams),
        // which are dispatched centrally via IrisEditorOverlay::render_panel_by_id.
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Constructs the standard [`PanelRegistry`] populated with all built-in editor panels.
pub fn create_default_panel_registry() -> PanelRegistry {
    let mut registry = PanelRegistry::default();
    for &panel_id in crate::ui::panel_layout::PanelId::all() {
        registry.register(EditorDockPanel::new(panel_id));
    }
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::panel_layout::PanelId;

    #[test]
    fn test_default_panel_registry_contains_all_panels() {
        let registry = create_default_panel_registry();
        assert_eq!(registry.len(), PanelId::all().len());

        for &panel_id in PanelId::all() {
            let id = panel_id.id_str();
            assert!(registry.contains(id));

            let panel = registry.get(id).expect("panel should exist");
            assert_eq!(panel.id(), id);
            assert_eq!(panel.title(), panel_id.title());

            let downcast = registry
                .get_downcast::<EditorDockPanel>(id)
                .expect("should downcast to EditorDockPanel");
            assert_eq!(downcast.panel_id(), panel_id);
        }
    }

    #[test]
    fn test_editor_dock_panel_render_and_events() {
        let mut panel = EditorDockPanel::new(PanelId::Inspector);
        let mut tree = UiTree::new();
        let parent = tree.create_root().expect("root widget");
        panel.render(&mut tree, parent, Rect::new(0.0, 0.0, 100.0, 100.0));
        assert!(!panel.is_dirty());
    }

    #[test]
    fn test_render_custom_panel_dispatch() {
        struct DynamicPluginPanel {
            rendered: bool,
        }

        impl DockPanel for DynamicPluginPanel {
            fn id(&self) -> &str {
                "dynamic_plugin"
            }
            fn title(&self) -> &str {
                "Plugin"
            }
            fn render(&mut self, _tree: &mut UiTree, _parent: WidgetId, _bounds: Rect) {
                self.rendered = true;
            }
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }

        let mut registry = PanelRegistry::default();
        registry.register(DynamicPluginPanel { rendered: false });

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root widget");

        let panel = registry.get_downcast_mut::<DynamicPluginPanel>("dynamic_plugin");
        assert!(panel.is_some());
        let p = panel.unwrap();
        p.render(&mut tree, root, Rect::new(0.0, 0.0, 200.0, 200.0));
        assert!(p.rendered);
    }

    #[test]
    fn test_stats_visual_sampling_cadence() {
        let mut last_fps_refresh = std::time::Instant::now();
        let mut stats_displayed_fps = 60.0;
        let mut stats_frame_counter: u32 = 0;

        // Simulate 30 frames within an interval before 250ms
        for _ in 0..30 {
            stats_frame_counter += 1;
        }

        // Before 250ms elapses, visual snapshot remains stable
        assert!(last_fps_refresh.elapsed().as_secs_f32() < 0.25);
        assert_eq!(stats_displayed_fps, 60.0);

        // Advance visual sampling manually past 250ms (e.g. 300ms) to verify threshold invariant
        let fake_past = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_millis(300))
            .unwrap_or(last_fps_refresh);
        last_fps_refresh = fake_past;

        let elapsed = std::time::Instant::now()
            .duration_since(last_fps_refresh)
            .as_secs_f32();
        assert!(elapsed >= 0.25);

        if elapsed >= 0.25 {
            stats_displayed_fps = stats_frame_counter as f32 / elapsed;
            stats_frame_counter = 0;
        }
        // 30 frames over ~0.30s = ~100 FPS
        assert!(stats_displayed_fps > 90.0 && stats_displayed_fps < 110.0);
        assert_eq!(stats_frame_counter, 0);
    }
}