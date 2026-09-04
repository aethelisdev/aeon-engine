// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Overlay lifecycle, initialization, and tree reconstruction subsystem for Iris UI editor overlays.

use super::about::build_about_dialog;
use super::hierarchy::{self, HierarchyPanelParams, HierarchyPanelTargets};
use super::menubar;
use super::modals::{
    self, build_delete_modal, build_loading_overlay, build_new_folder_modal, build_rename_modal,
};
use super::preferences::{self, build_preferences_dialog};
use super::stats::{self, StatsPanelParams, StatsPanelTargets};
use super::status_bar;
use super::types::{ActiveMenu, IrisEditorOverlay, OverlayUpdateParams};
use super::viewport_hud::{self, ViewportHudParams, ViewportHudTargets};
use irisui::prelude::*;
use irisui::text::TextSystem;
use std::collections::HashSet;

impl IrisEditorOverlay {
    /// Height of the top menubar panel in physical pixels (matching egui geometry).
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
            cursor_pos: Point::new(-1000.0, -1000.0),
            active_menu: None,
            dropdown_items: Vec::new(),
            dropdown_rect: None,
            about_targets: None,
            delete_targets: None,
            new_folder_targets: None,
            rename_targets: None,
            loading_targets: None,
            preferences_targets: None,
            viewport_hud_targets: None,
            viewport_hud_dropdown: None,
            viewport_hud_actions: Vec::new(),
            stats_targets: None,
            stats_nodes: None,
            last_stats_rect: None,
            hierarchy_targets: None,
            hierarchy_rows_cache: Vec::new(),
            hierarchy_scroll_y: 0.0,
            hierarchy_search_query: String::new(),
            hierarchy_is_add_menu_open: false,
            hierarchy_active_submenu: None,
            hierarchy_active_context_menu: None,
            hierarchy_is_search_focused: false,
            hierarchy_actions: Vec::new(),
            inspector_targets: None,
            inspector_scroll_y: 0.0,
            inspector_is_add_menu_open: false,
            inspector_active_submenu: None,
            inspector_active_dropdown: None,
            inspector_active_number_input: None,
            inspector_drag_number: None,
            inspector_edit_start_snapshot: None,
            inspector_rename_buffer: None,
            inspector_hex_buffer: None,
            inspector_hsv: [180.0, 0.8, 0.9],
            inspector_color_drag_mode: None,
            inspector_is_color_picker_open: false,
            inspector_actions: Vec::new(),
            last_dimensions: (0.0, 0.0),
            last_zoom_factor: 1.0,
            needs_layout_rebuild: false,
            stats_scroll_y: 0.0,
            stats_actions: Vec::new(),
            preferences_pos: None,
            preferences_drag_offset: None,
            preferences_tab: 0,
            preferences_scroll_y: 0.0,
            preferences_dropdown: None,
            active_slider_drag: None,
            preferences_actions: Vec::new(),
            viewport_search_query: String::new(),
            viewport_is_search_focused: false,
            new_folder_buffer: String::new(),
            rename_buffer: String::new(),
            collapsed_sections: HashSet::new(),
            active_number_input: None,
            screen_width: 1920.0,
            screen_height: 1080.0,
            is_visible: true,
            target_format,
            start_time: std::time::Instant::now(),
            tools_texture: None,
        }
    }

    /// Reconstructs and resolves layout for the top menu bar, active dropdown, modals, and bottom status bar.
    pub fn update_overlays(&mut self, params: OverlayUpdateParams<'_>) {
        let (screen_width, screen_height) = params.dimensions;
        self.screen_width = screen_width;
        self.screen_height = screen_height;

        if !self.is_visible {
            self.command_list.clear();
            return;
        }

        self.tree.clear();
        self.layout_engine.clear();
        self.command_list.clear();
        self.dropdown_items.clear();
        self.dropdown_rect = None;
        self.about_targets = None;
        self.delete_targets = None;
        self.new_folder_targets = None;
        self.rename_targets = None;
        self.loading_targets = None;
        self.preferences_targets = None;
        self.viewport_hud_targets = None;
        self.stats_targets = None;
        self.inspector_targets = None;

        let Ok(root) = self.tree.create_root() else {
            return;
        };

        if let Some(root_node) = self.tree.get_mut(root) {
            root_node.set_name("IrisRoot");
            root_node.set_style(
                Style::new()
                    .flex_col()
                    .justify_content(JustifyContent::SpaceBetween)
                    .width(screen_width)
                    .height(screen_height),
            );
        }

        // 1. Top MenuBar
        let menu_bar_id = menubar::build_top_menu_bar(
            &mut self.tree,
            screen_width,
            self.cursor_pos,
            self.active_menu,
            params.is_editing,
        );
        let _ = self.tree.add_child(root, menu_bar_id);

        // 2. Bottom Diagnostics & Status Bar
        let status_bar_id = status_bar::build_bottom_status_bar(
            &mut self.tree,
            status_bar::StatusBarParams {
                screen_width,
                screen_height,
                status_spans: params.status_spans,
            },
        );
        let _ = self.tree.add_child(root, status_bar_id);

        // Pre-measure all text nodes to populate intrinsic content_size
        self.measure_tree_text(root);

        // Compute Taffy layout for top menu bar and status bar
        let _ = self
            .layout_engine
            .compute_layout(&mut self.tree, Size::new(screen_width, screen_height));

        // 3. If Viewport canvas is valid, build Viewport HUD (docked)
        if params.viewport_rect.width > 20.0 && params.viewport_rect.height > 20.0 {
            let mut hud_targets = ViewportHudTargets::default();
            viewport_hud::build_viewport_hud(
                &mut self.tree,
                root,
                &ViewportHudParams {
                    viewport_rect: params.viewport_rect,
                    camera: params.camera,
                    wireframe_enabled: params.wireframe_enabled,
                    gizmo_mode: params.gizmo_mode,
                    gizmo_space: params.gizmo_space,
                    snapping: params.snapping_settings,
                    cursor_pos: self.cursor_pos,
                    active_dropdown: self.viewport_hud_dropdown,
                    selected_entity: params.selected_entity,
                    world: params.world,
                    is_editing: params.is_editing,
                },
                &mut hud_targets,
            );
            self.viewport_hud_targets = Some(hud_targets);
        }

        // 4. If Stats & Profiler panel is active, build Stats panel (docked)
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

        // 5. If Scene Hierarchy panel is active, build Scene Hierarchy panel (docked)
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

        // 5b. If Scene Inspector panel is active, build Scene Inspector panel (docked)
        if let Some(inspector_rect) = params.inspector_panel_rect
            && inspector_rect.width > 20.0
            && inspector_rect.height > 20.0
        {
            let num_input_ref = self
                .inspector_active_number_input
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

        // 6. FLOATING OVERLAYS (Rendered on top of docked panels):
        // 6a. If a dropdown menu is open, build its floating popup
        if let Some(active) = self.active_menu {
            let anchor_x = match active {
                ActiveMenu::File => 6.0,
                ActiveMenu::Edit => 44.0,
                ActiveMenu::View => 84.0,
                ActiveMenu::Window => 126.0,
                ActiveMenu::Help => 186.0,
            };

            let (dropdown_id, items, dd_rect) = menubar::build_floating_dropdown(
                &mut self.tree,
                active,
                anchor_x,
                self.cursor_pos,
                params.layout_state,
                params.can_undo,
                params.can_redo,
            );

            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, dropdown_id);
            }
            self.dropdown_items = items;
            self.dropdown_rect = Some(dd_rect);
        }

        // 6b. If Preferences dialogue is active, build its floating card
        if params.show_preferences {
            let blink_caret = (self.start_time.elapsed().as_millis() % 1060) < 530;
            let (pref_id, targets) = build_preferences_dialog(
                &mut self.tree,
                preferences::PreferencesParams {
                    screen_width,
                    screen_height,
                    window_pos: self.preferences_pos,
                    active_tab: self.preferences_tab,
                    scroll_offset_y: self.preferences_scroll_y,
                    active_dropdown: self.preferences_dropdown,
                    collapsed_sections: &self.collapsed_sections,
                    active_number_input: self
                        .active_number_input
                        .as_ref()
                        .map(|(id, s)| (*id, s.as_str())),
                    blink_caret,
                    cursor_pos: self.cursor_pos,
                    zoom_factor: params.zoom_factor,
                    graphics_settings: params.graphics_settings,
                    snapping_settings: params.snapping_settings,
                    editor_config: params.editor_config,
                    enable_live_updates: params.enable_live_updates,
                    enabled_modules: params.enabled_modules,
                },
            );
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, pref_id);
            }
            self.preferences_targets = Some(targets);
        }

        // 6c. If About Aeon Engine modal dialogue is active, build its centered card
        if params.show_about {
            let (about_id, targets) =
                build_about_dialog(&mut self.tree, screen_width, screen_height, self.cursor_pos);
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, about_id);
            }
            self.about_targets = Some(targets);
        }

        // 6d. If Delete Confirmation modal is active, build its card
        if let Some(target_path) = params.delete_target {
            let (del_id, targets) = build_delete_modal(
                &mut self.tree,
                target_path,
                screen_width,
                screen_height,
                self.cursor_pos,
            );
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, del_id);
            }
            self.delete_targets = Some(targets);
        }

        let elapsed_secs = self.start_time.elapsed().as_secs_f32();
        let cursor_blink_visible = (self.start_time.elapsed().as_millis() / 530).is_multiple_of(2);

        // 6e. If New Folder modal is active, build its card
        if let Some(parent_path) = params.new_folder_parent {
            let input_name = self.new_folder_buffer.as_str();
            let text_width = self
                .text_system
                .measure_text(input_name, 12.0, 28.0, None)
                .width;
            let (folder_id, targets) = build_new_folder_modal(
                &mut self.tree,
                modals::FolderModalParams {
                    parent_path,
                    input_text: input_name,
                    text_width,
                    cursor_blink_visible,
                    screen_width,
                    screen_height,
                    cursor_pos: self.cursor_pos,
                },
            );
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, folder_id);
            }
            self.new_folder_targets = Some(targets);
        }

        // 6f. If Rename modal is active, build its card
        if let Some((target_path, is_folder)) = params.rename_target {
            let input_name = self.rename_buffer.as_str();
            let text_width = self
                .text_system
                .measure_text(input_name, 12.0, 28.0, None)
                .width;
            let (rename_id, targets) = build_rename_modal(
                &mut self.tree,
                modals::RenameModalParams {
                    target_path,
                    input_text: input_name,
                    text_width,
                    is_folder,
                    cursor_blink_visible,
                    screen_width,
                    screen_height,
                    cursor_pos: self.cursor_pos,
                },
            );
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, rename_id);
            }
            self.rename_targets = Some(targets);
        }

        // 6g. If Asset Loading overlay is active, build its splash screen
        if params.is_loading_assets {
            let (loading_id, targets) = build_loading_overlay(
                &mut self.tree,
                modals::LoadingOverlayParams {
                    screen_width,
                    screen_height,
                    time_secs: elapsed_secs,
                },
            );
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, loading_id);
            }
            self.loading_targets = Some(targets);
        }

        // Populate DrawCommandList from resolved layout nodes
        self.populate_draw_commands(root, None);

        // Zero-Tree GPU Oscilloscope: directly push 60 curve quads into DrawCommandList
        if let Some(ref nodes) = self.stats_nodes {
            stats::append_oscilloscope_quads(
                &mut self.command_list,
                nodes.canvas_rect,
                params.frame_pacing,
            );
        }

        self.needs_layout_rebuild = false;
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