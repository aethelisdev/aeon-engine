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
use super::stats;
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
            inspector_active_text_input: None,
            shift_held: false,
            alt_held: false,
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
            console_targets: None,
            console_scroll_y: 0.0,
            console_filter: super::console::ConsoleFilterLevel::All,
            console_search_query: String::new(),
            console_is_search_focused: false,
            console_auto_scroll: true,
            console_actions: Vec::new(),
            assets_targets: None,
            assets_scroll_y: 0.0,
            assets_tree_scroll_y: 0.0,
            assets_search_query: String::new(),
            assets_current_folder: std::path::PathBuf::from("assets"),
            assets_is_search_focused: false,
            assets_click_tracker: super::assets::AssetClickTracker::default(),
            assets_actions: Vec::new(),
            assets_context_menu: None,
            assets_preview_modal: None,
            assets_selected_asset: None,
            thumbnail_layers: std::collections::HashMap::new(),
            next_thumbnail_layer: 16,
            timeline_targets: None,
            timeline_is_dragging: false,
            timeline_actions: Vec::new(),
            timeline_selected_entity: None,
            material_targets: None,
            material_scroll_y: 0.0,
            material_actions: Vec::new(),
            material_selected_entity: None,
            ui_designer_targets: None,
            ui_designer_actions: Vec::new(),
            ui_designer_is_aspect_open: false,
            ui_designer_is_add_menu_open: false,
            ui_designer_drag_state: None,
            ui_designer_is_panning: false,
            ui_designer_last_cursor: Point::new(0.0, 0.0),
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
            floating_window_rects: Vec::new(),
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
        self.floating_window_rects.clear();
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
        self.console_targets = None;
        self.assets_targets = None;
        self.material_targets = None;
        self.ui_designer_targets = None;

        if !self.assets_is_search_focused {
            self.assets_search_query = params.asset_browser.search_query.clone();
        }
        self.assets_current_folder = params.asset_browser.current_folder.clone();
        self.timeline_selected_entity = params.selected_entity;
        self.material_selected_entity = params.selected_entity;

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

        // 4. Layer 0: Render all DOCKED panels first (Z-Index: Background Workspace Layer)
        let is_floating = |panel: crate::ui::panel_layout::PanelId| {
            super::floating_layer::is_panel_in_floating_window(params.layout_state, panel)
        };

        if !is_floating(crate::ui::panel_layout::PanelId::Hierarchy) {
            self.build_hierarchy_panel_if_active(root, &params);
        }
        if !is_floating(crate::ui::panel_layout::PanelId::Inspector) {
            self.build_inspector_panel_if_active(root, &params);
        }
        if !is_floating(crate::ui::panel_layout::PanelId::Console) {
            self.build_console_panel_if_active(root, &params);
        }
        if !is_floating(crate::ui::panel_layout::PanelId::Assets) {
            self.build_assets_panel_if_active(root, &params);
        }
        if !is_floating(crate::ui::panel_layout::PanelId::MaterialEditor) {
            self.build_material_panel_if_active(root, &params);
        }
        if !is_floating(crate::ui::panel_layout::PanelId::AnimationTimeline) {
            self.build_timeline_panel_if_active(root, &params);
        }
        if !is_floating(crate::ui::panel_layout::PanelId::UiDesigner) {
            self.build_ui_designer_panel_if_active(root, &params);
        }
        if !is_floating(crate::ui::panel_layout::PanelId::Stats) {
            self.build_stats_panel_if_active(root, &params);
        }

        // 5. Layer 1: Render all FLOATING Windows and their Active Panels (Z-Index: Foreground Layer)
        let (floating_rects, floating_containers) = super::floating_layer::build_floating_windows(
            &mut self.tree,
            root,
            params.layout_state,
            self.cursor_pos,
        );
        self.floating_window_rects = floating_rects;

        for (panel_id, container_id) in floating_containers {
            match panel_id {
                crate::ui::panel_layout::PanelId::Stats => {
                    self.build_stats_panel_if_active(container_id, &params);
                }
                crate::ui::panel_layout::PanelId::Hierarchy => {
                    self.build_hierarchy_panel_if_active(container_id, &params);
                }
                crate::ui::panel_layout::PanelId::Inspector => {
                    self.build_inspector_panel_if_active(container_id, &params);
                }
                crate::ui::panel_layout::PanelId::Console => {
                    self.build_console_panel_if_active(container_id, &params);
                }
                crate::ui::panel_layout::PanelId::Assets => {
                    self.build_assets_panel_if_active(container_id, &params);
                }
                crate::ui::panel_layout::PanelId::MaterialEditor => {
                    self.build_material_panel_if_active(container_id, &params);
                }
                crate::ui::panel_layout::PanelId::AnimationTimeline => {
                    self.build_timeline_panel_if_active(container_id, &params);
                }
                crate::ui::panel_layout::PanelId::UiDesigner => {
                    self.build_ui_designer_panel_if_active(container_id, &params);
                }
                _ => {}
            }
        }

        // 6. FLOATING OVERLAYS (Rendered on top of docked and floating panels):
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

        // 6h. Hierarchy Add Menu and Context Menu (Rendered as topmost floating overlays)
        if let Some(ref mut hier_targets) = self.hierarchy_targets
            && let Some(hier_rect) = params.hierarchy_panel_rect
        {
            let hier_params = hierarchy::HierarchyPanelParams {
                panel_rect: hier_rect,
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
            hierarchy::build_hierarchy_overlays(&mut self.tree, root, &hier_params, hier_targets);
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