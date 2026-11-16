// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Overlay lifecycle, initialization, and tree reconstruction subsystem for Iris UI editor overlays.

use super::menubar;
use super::preferences;
use super::status_bar;
use super::theme::*;
use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;
use irisui::text::TextSystem;
use irisui::widgets::controls::FocusState;

impl IrisEditorOverlay {
    /// Height of the top menubar panel in physical pixels.
    pub const MENUBAR_HEIGHT: f32 = menubar::MENUBAR_HEIGHT;

    /// Height of the bottom status bar in physical pixels.
    pub const STATUS_BAR_HEIGHT: f32 = status_bar::STATUS_BAR_HEIGHT;

    /// Initializes a new Iris UI editor overlay pipeline for the specified surface format.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let mut tree = UiTree::new();
        tree.set_theme(super::theme::EditorThemeTokens::default().to_iris_theme());
        let mut notifier = UiNotifier::new();
        notifier.register_domain_panel(crate::ui::panel_layout::PanelId::Inspector);
        notifier.register_domain_panel(crate::ui::panel_layout::PanelId::Hierarchy);
        notifier.register_domain_panel(crate::ui::panel_layout::PanelId::MaterialEditor);
        notifier.register_domain_panel(crate::ui::panel_layout::PanelId::AnimationTimeline);

        Self {
            tree,
            layout_engine: LayoutEngine::new(),
            renderer: IrisRenderer::new(device, target_format),
            text_system: TextSystem::new(),
            text_renderer: None,
            command_list: DrawCommandList::new(),
            notifier,
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
            focus_manager: FocusManager::new(),
            focus_state: FocusState::new(),
            pending_actions: Vec::new(),
        }
    }

    /// Reconstructs and resolves layout for the top menu bar, active dropdown, modals, and bottom status bar.
    pub fn update_overlays(&mut self, mut params: OverlayUpdateParams<'_>) {
        let (screen_width, screen_height) = params.context.dimensions;
        let active_tokens = self.chrome.theme_preset.tokens();

        let cursor = self.cursor_pos();

        if !self.is_visible {
            self.command_list.clear();
            return;
        }

        let current_focused_tag = self.tree.event_state.focused_tag;
        let focus_target_changed = current_focused_tag != self.chrome.last_focused_tag;

        let is_shell_dirty = self.tree.root().is_none()
            || self.chrome.dock_frame.is_none()
            || self.notifier.is_global_dirty();

        if self.menubar.active_menu.is_none() {
            self.menubar.dropdown_rect = None;
        }
        if !params.dialogs.show_preferences {
            self.preferences.floating.reset();
            self.preferences.content_rect = None;
        }

        let is_assets_editing = self
            .focus_manager
            .is_tag_focused(super::assets::ASSETS_TAG_SEARCH_INPUT)
            || self
                .focus_state
                .is_tag_focused(super::assets::ASSETS_TAG_SEARCH_INPUT)
            || self
                .tree
                .event_state
                .text_edit_buffer(super::assets::ASSETS_TAG_SEARCH_INPUT)
                .is_some();
        if !is_assets_editing {
            self.assets
                .search_input
                .set_text(&params.panel_data.asset_browser.search_query);
        }
        let assets_dirty = self.assets.current_folder
            != params.panel_data.asset_browser.current_folder
            || self.assets.last_view_mode != params.panel_data.asset_browser.view_mode
            || self.assets.last_active_category != params.panel_data.asset_browser.active_category
            || self.assets.selected_asset != params.panel_data.asset_browser.selected_asset
            || self.assets.last_sidebar_collapsed
                != params.panel_data.asset_browser.sidebar_collapsed
            || self.assets.last_show_engine_content
                != params.panel_data.asset_browser.show_engine_content
            || self.assets.last_cached_items_count
                != params.panel_data.asset_browser.cached_items.len()
            || self.assets.last_is_2d_mode != params.context.is_2d_mode
            || !self.assets.actions.is_empty();

        if assets_dirty {
            self.notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::Assets);
        }

        // Synchronize external domain/history revision (Undo/Redo commands, external entity transactions)
        self.notifier
            .sync_domain_revision(params.context.history_revision);

        let selection_changed = self.chrome.last_selected_entity != params.scene.selected_entity;
        if selection_changed {
            self.chrome.last_selected_entity = params.scene.selected_entity;
            self.notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::Hierarchy);
            self.notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::Inspector);
            self.notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::MaterialEditor);
            self.notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::AnimationTimeline);
        }

        self.timeline.selected_entity = params.scene.selected_entity;
        if self.material.selected_entity != params.scene.selected_entity {
            self.material.selected_entity = params.scene.selected_entity;
            self.tree
                .set_scroll_offset(super::material::types::MATERIAL_TAG_VIEWPORT, 0.0);
        }

        // Safely commit pending inspector edits on selection change
        self.inspector
            .commit_pending_edits_on_selection_change(params.scene.selected_entity);

        if is_shell_dirty {
            self.chrome.needs_full_quad_upload = true;
            // =========================================================================
            // Branch 1: Shell & Topology Reconstruction (tree.clear() + DockArea::show)
            // =========================================================================
            // Only invoked when structural topology changes (initial boot, window resize,
            // dock splitter drag, tab docking/undocking, floating window move, modal display).
            self.tree.clear();
            self.layout_engine.clear();

            let Ok(root) = self.tree.create_root() else {
                return;
            };

            // Declarative configuration of root container spanning full viewport
            let mut root_scope = UiScope::new(&mut self.tree, root);
            root_scope.configure_container(
                "IrisEditorRoot",
                Style::new()
                    .flex_col()
                    .justify_content(JustifyContent::SpaceBetween)
                    .width(screen_width)
                    .height(screen_height),
                false,
            );

            // 1. Top MenuBar rendered via MenuBarDescriptor directly into ActionBus
            if let Some(action_bus) = params.action_bus.as_deref_mut() {
                menubar::build_top_menu_bar(
                    &mut root_scope,
                    &menubar::MenuBarParams {
                        screen_width,
                        layout_state: params.context.layout_state,
                        can_undo: params.context.can_undo,
                        can_redo: params.context.can_redo,
                        is_editing: params.context.is_editing,
                    },
                    action_bus,
                );
            }

            // 1b. Window-Level Modal Dialogs (rendered directly on Root Scope, matching About pattern)
            if self.assets.dialog_host.is_active() {
                super::assets::panel::render_assets_dialog_host(
                    &mut root_scope,
                    &mut self.assets.dialog_host,
                    &mut self.assets.action_bus,
                    (screen_width, screen_height),
                );
            }

            // 2. Bottom Diagnostics & Status Bar
            status_bar::build_bottom_status_bar(
                &mut self.tree,
                root,
                &status_bar::StatusBarParams {
                    screen_width,
                    screen_height,
                    status_spans: params.context.status_spans,
                },
            );

            // 2b. Baseline divider above StatusBar (MenuBar divider is handled by MenuBarDescriptor)
            let mut scope = UiScope::new(&mut self.tree, root);
            scope.empty_box_passive_named(
                "IrisBottomStatusBarDivider",
                Style::new()
                    .position_absolute()
                    .left(0.0)
                    .top(screen_height - Self::STATUS_BAR_HEIGHT)
                    .width(screen_width)
                    .height(1.0)
                    .background(BORDER_MICRON),
            );

            // Pre-measure all text nodes to populate intrinsic content_size
            self.text_system.measure_subtree_text(&mut self.tree, root);

            // Compute Taffy layout for top menu bar, status bar, and dividers
            let _ = self
                .layout_engine
                .compute_layout(&mut self.tree, Size::new(screen_width, screen_height));

            // 3. Build Native Iris UI Docking Frame (Splitters, Tab Strips, Compact Snug Tabs, Active Indicators)
            let workspace_rect = Rect::new(
                0.0,
                Self::MENUBAR_HEIGHT,
                screen_width,
                (screen_height - Self::MENUBAR_HEIGHT - Self::STATUS_BAR_HEIGHT).max(1.0),
            );
            let pref_rect = if params.dialogs.show_preferences {
                let (left, top) = if let Some(pos) = self.preferences.floating.pos {
                    (pos.x, pos.y)
                } else {
                    (
                        ((screen_width - preferences::PREF_CARD_WIDTH) * 0.5).max(0.0),
                        ((screen_height - preferences::PREF_CARD_HEIGHT) * 0.5)
                            .max(Self::MENUBAR_HEIGHT),
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

            let is_cursor_occluded = self.is_point_over_overlay(cursor)
                || pref_rect.is_some_and(|r| {
                    cursor.x >= r.x
                        && cursor.x <= r.right()
                        && cursor.y >= r.y
                        && cursor.y <= r.bottom()
                })
                || params
                    .context
                    .layout_state
                    .dock_state
                    .is_point_over_floating_window(cursor);

            // 3. Declarative Docking Framework: orchestrate layout, splitters, tab bars, and leaf panel UIs via DockArea
            let dock_style = super::theme::editor_dock_style_with_tokens(&active_tokens);
            let mut tree = std::mem::take(&mut self.tree);
            let root_id = tree.root().unwrap_or(root);
            let dock_frame = {
                let mut scope = UiScope::new(&mut tree, root_id);
                let mut registry = crate::ui::workbench::create_editor_panel_registry();
                let mut panel_ctx = crate::ui::workbench::EditorPanelContext {
                    overlay: self,
                    params: &params,
                };
                irisui::dock::DockArea::from_ref(&params.context.layout_state.dock_state)
                    .bounds(workspace_rect)
                    .style(dock_style)
                    .cursor_pos(cursor)
                    .cursor_occluded(is_cursor_occluded)
                    .show(&mut scope, &mut registry.active_tab_viewer(&mut panel_ctx))
            };

            for &(_, leaf_id) in &dock_frame.leaf_containers {
                self.text_system.measure_subtree_text(&mut tree, leaf_id);
                let _ = self
                    .layout_engine
                    .compute_boundary_subtree(&mut tree, leaf_id);
            }

            self.chrome.floating_window_rects = dock_frame.floating_rects.clone();
            self.chrome.dock_frame = Some(dock_frame);
            self.tree = tree;

            let is_floating = |panel: crate::ui::panel_layout::PanelId| {
                super::floating_layer::is_panel_in_floating_window(
                    params.context.layout_state,
                    panel,
                )
            };

            let is_viewport_active = !is_floating(crate::ui::panel_layout::PanelId::Viewport)
                && params.viewport.viewport_rect.width > 20.0
                && params.viewport.viewport_rect.height > 20.0;
            self.viewport_hud.is_active = is_viewport_active;

            self.notifier.clear_all();
            self.command_list.clear();
            self.chrome.needs_text_prepare = true;
            self.populate_draw_commands(root, None, Some(params.telemetry.frame_pacing));
        } else {
            // =========================================================================
            // Branch 3: Retained Sleep In-Place Draw Refresh (Zero Tree Mutation)
            // =========================================================================
            let has_stats_canvas = self
                .tree
                .node_tagged(super::stats::types::STATS_TAG_CANVAS)
                .is_some();
            let has_camera_hud_canvas = self
                .tree
                .node_tagged(super::viewport_hud::TAG_CAMERA_HUD)
                .is_some();
            let has_compass_canvas = self
                .tree
                .node_tagged(super::viewport_hud::TAG_COMPASS_CANVAS)
                .is_some();
            let camera_pos_changed =
                (self.viewport_hud.camera_pos.0 - params.viewport.camera.position.x).abs() > 0.001
                    || (self.viewport_hud.camera_pos.1 - params.viewport.camera.position.y).abs()
                        > 0.001
                    || (self.viewport_hud.camera_pos.2 - params.viewport.camera.position.z).abs()
                        > 0.001;
            let compass_changed = self.viewport_hud.camera_angles
                != (params.viewport.camera.pitch.0, params.viewport.camera.yaw.0);
            let camera_changed = camera_pos_changed || compass_changed;
            let has_volatile_canvas = has_stats_canvas || compass_changed || camera_changed;

            if has_camera_hud_canvas && camera_changed {
                super::viewport_hud::camera_hud::update_camera_hud(
                    &mut self.tree,
                    params.viewport.camera,
                    params.viewport.viewport_rect,
                );
                self.viewport_hud.camera_pos = (
                    params.viewport.camera.position.x,
                    params.viewport.camera.position.y,
                    params.viewport.camera.position.z,
                );
                self.chrome.needs_text_prepare = true;
            }

            if has_compass_canvas && compass_changed {
                super::viewport_hud::compass::update_compass_canvas(
                    &mut self.tree,
                    params.viewport.camera,
                    params.viewport.viewport_rect,
                );
                self.viewport_hud.camera_angles =
                    (params.viewport.camera.pitch.0, params.viewport.camera.yaw.0);
                self.chrome.needs_text_prepare = true;
            }

            if has_stats_canvas {
                irisui::widgets::canvas::update_canvas_by_tag(
                    &mut self.tree,
                    super::stats::types::STATS_TAG_CANVAS,
                    |canvas| {
                        super::stats::graph::draw_oscilloscope_canvas(
                            canvas,
                            Some(params.telemetry.frame_pacing),
                        );
                    },
                );

                let pacing_changed = super::stats::graph::update_oscilloscope_texts(
                    &mut self.tree,
                    params.telemetry.frame_pacing_stats,
                );
                let cpu_changed = super::stats::cpu_breakdown::update_cpu_breakdown_texts(
                    &mut self.tree,
                    params.telemetry.cpu_timings,
                );
                if pacing_changed || cpu_changed {
                    self.chrome.needs_text_prepare = true;
                }
            }

            if focus_target_changed {
                self.chrome.needs_text_prepare = true;
            }

            let non_boundary_change = has_volatile_canvas || focus_target_changed;

            let should_repopulate = non_boundary_change || self.tree.has_dirty_boundaries();

            if should_repopulate {
                self.command_list.clear();
                if let Some(root) = self.tree.root() {
                    self.populate_draw_commands(root, None, Some(params.telemetry.frame_pacing));
                }
            }

            self.chrome.needs_full_quad_upload = non_boundary_change;
        }

        self.chrome.last_dimensions = params.context.dimensions;
        self.chrome.last_zoom_factor = params.context.zoom_factor;
        self.preferences.last_tab = self.preferences.tab;
        self.chrome.last_has_viewport_texture = params.viewport.has_viewport_texture;
        self.chrome.last_cursor_pos = cursor;
        self.chrome.last_hovered_tag = self.chrome.hovered_tag;
        self.chrome.last_hovered_panel = self.chrome.hovered_panel;
        self.chrome.last_focused_tag = current_focused_tag;
        self.chrome.last_theme_preset = self.chrome.theme_preset;
        self.chrome.needs_layout_rebuild = false;
        self.notifier.clear_all();
        self.tree.clear_frame_clicks();
    }
}