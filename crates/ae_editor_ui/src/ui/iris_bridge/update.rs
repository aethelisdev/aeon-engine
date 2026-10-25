// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Overlay lifecycle, initialization, and tree reconstruction subsystem for Iris UI editor overlays.

use super::menubar;
use super::preferences;
use super::status_bar;
use super::theme::*;
use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use crate::ui::panel_layout::PanelId;
use irisui::prelude::*;
use irisui::text::TextSystem;
use irisui::widgets::controls::FocusState;
use irisui::widgets::modal::ModalDialogAction;

impl IrisEditorOverlay {
    /// Height of the top menubar panel in physical pixels.
    pub const MENUBAR_HEIGHT: f32 = menubar::MENUBAR_HEIGHT;

    /// Height of the bottom status bar in physical pixels.
    pub const STATUS_BAR_HEIGHT: f32 = status_bar::STATUS_BAR_HEIGHT;

    /// Initializes a new Iris UI editor overlay pipeline for the specified surface format.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let mut tree = UiTree::new();
        tree.set_theme(super::theme::EditorThemeTokens::default().to_iris_theme());
        Self {
            tree,
            layout_engine: LayoutEngine::new(),
            renderer: IrisRenderer::new(device, target_format),
            text_system: TextSystem::new(),
            text_renderer: None,
            command_list: DrawCommandList::new(),
            notifier: UiNotifier::new(),
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
            focus_manager: FocusManager::new(),
            focus_state: FocusState::new(),
            pending_actions: Vec::new(),
        }
    }

    /// Intercepts and processes UI interaction events via autonomous Iris UI tree dispatch.
    ///
    /// Interacts directly with the primary retained [`UiTree`], handling elevated modal and popup
    /// layers transparently through the single-tree architecture.
    pub fn handle_event(&mut self, event: &UiEvent) -> super::types::IrisOverlayEventResult {
        // Track logical cursor coordinates for layout, drag-drop, and inspection queries
        if let UiEvent::MouseMove { point }
        | UiEvent::MouseDown { point, .. }
        | UiEvent::MouseUp { point, .. } = event
        {
            self.chrome.cursor_pos = *point;
        }

        if let UiEvent::KeyboardInput {
            key: KeyCode::Escape,
            is_pressed: true,
            ..
        } = event
        {
            self.clear_focus();
            self.tree.event_state.clear_text_edit();
            self.menubar.active_menu = None;
            self.menubar.actions.clear();
            self.menubar.dropdown_rect = None;
            self.close_active_popups();
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
            let mut res = super::types::IrisOverlayEventResult::consumed(true);
            if self.modals.is_about_active {
                res.close_about = true;
            } else if self.preferences.card_rect.is_some() {
                res.close_preferences = true;
                res.preferences_action = Some(super::preferences::PreferencesAction::Close);
            } else if self.modals.is_delete_active {
                res.cancel_delete = true;
            } else if self.modals.is_new_folder_active {
                res.cancel_new_folder = true;
            } else if self.modals.is_rename_active {
                res.cancel_rename = true;
            }
            self.assets.preview_modal = None;
            return res;
        }

        if let UiEvent::MouseDown { point, .. } = event {
            let hit = self.tree.hit_test_layered(*point);
            let is_over_overlay =
                hit.is_some_and(|h| self.tree.effective_layer(h) >= UiLayer::Modal);
            if !is_over_overlay && self.close_active_popups() {
                self.notifier.tag_all();
                self.chrome.needs_layout_rebuild = true;
            }
        }

        let dispatch = self.tree.dispatch_event(event);
        let clicked_tag = self.tree.event_state.frame_clicked_tag;

        if self.tree.event_state.active_text_edit.is_none() && self.focus_state.has_focus() {
            self.clear_focus();
        }

        let should_wake = dispatch.is_consumed()
            || !self.pending_actions.is_empty()
            || self.inspector.has_pending_actions()
            || self.hierarchy.has_pending_actions();

        if should_wake {
            if dispatch.target_tag == super::console::types::CONSOLE_TAG_VIEWPORT
                && dispatch.scroll_delta.1 > 0.0
            {
                self.console.auto_scroll = false;
            }

            // Route dirty notification to the specific dock panel owning the targeted widget
            let mut routed_to_panel = false;
            if let Some(target_id) = dispatch.target
                && let Some(boundary_id) = self.tree.find_invalidation_boundary(target_id)
                && let Some(dock_frame) = &self.chrome.native_dock_frame
                && let Some(panel_id) = dock_frame.tab_for_leaf_container(boundary_id)
            {
                self.notifier.tag_redraw(panel_id);
                routed_to_panel = true;
            }

            if self.hierarchy.has_pending_actions() {
                self.notifier
                    .tag_redraw(crate::ui::panel_layout::PanelId::Hierarchy);
                routed_to_panel = true;
            }
            if self.inspector.has_pending_actions() {
                self.notifier
                    .tag_redraw(crate::ui::panel_layout::PanelId::Inspector);
                routed_to_panel = true;
            }

            // If the event was unrouted (outside any dock leaf, e.g. MenuBar, StatusBar, Modal),
            // escalate to global shell invalidation
            if !routed_to_panel {
                self.notifier.tag_all();
                self.chrome.needs_layout_rebuild = true;
            }
        }

        let mut result = super::types::IrisOverlayEventResult::consumed(dispatch.is_consumed());

        // Evaluate modal actions and button triggers in unified tree (STRICTLY on clicks, never on hover)
        if let Some(clicked_tag) = clicked_tag
            && clicked_tag != 0
        {
            match clicked_tag {
                ModalDialogAction::TAG_CLOSE => {
                    result.consumed = true;
                    if self.modals.is_about_active {
                        result.close_about = true;
                    } else if self.preferences.card_rect.is_some() {
                        result.close_preferences = true;
                        result.preferences_action =
                            Some(super::preferences::PreferencesAction::Close);
                    } else if self.modals.is_delete_active {
                        result.cancel_delete = true;
                    } else if self.modals.is_new_folder_active {
                        result.cancel_new_folder = true;
                    } else if self.modals.is_rename_active {
                        result.cancel_rename = true;
                    }
                    self.assets.preview_modal = None;
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                }
                ModalDialogAction::TAG_CANCEL => {
                    result.consumed = true;
                    if self.modals.is_delete_active {
                        result.cancel_delete = true;
                    } else if self.modals.is_new_folder_active {
                        result.cancel_new_folder = true;
                    } else if self.modals.is_rename_active {
                        result.cancel_rename = true;
                    }
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                }
                ModalDialogAction::TAG_CONFIRM => {
                    result.consumed = true;
                    if self.modals.is_about_active {
                        result.close_about = true;
                    } else if self.modals.is_new_folder_active {
                        result.create_folder =
                            Some(self.modals.new_folder_input.text().to_string());
                    } else if self.modals.is_rename_active {
                        result.apply_rename = Some(self.modals.rename_input.text().to_string());
                    }
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                }
                ModalDialogAction::TAG_DANGER => {
                    result.consumed = true;
                    if self.modals.is_delete_active {
                        result.confirm_delete = true;
                    }
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                }
                super::preferences::PREF_TAG_CLOSE => {
                    result.consumed = true;
                    result.close_preferences = true;
                    result.preferences_action = Some(super::preferences::PreferencesAction::Close);
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                }
                _ => {}
            }
        }

        result
    }

    /// Reconstructs and resolves layout for the top menu bar, active dropdown, modals, and bottom status bar.
    pub fn update_overlays(&mut self, mut params: OverlayUpdateParams<'_>) {
        let (screen_width, screen_height) = params.context.dimensions;
        self.screen_width = screen_width;
        self.screen_height = screen_height;

        let active_tokens = self.chrome.theme_preset.tokens();
        self.tree.set_theme(active_tokens.to_iris_theme());

        self.modals.sync_from_dialogs(&params.dialogs);

        let modal_active = self.modals.is_about_active
            || params.dialogs.show_preferences
            || self.modals.is_delete_active
            || self.modals.is_new_folder_active
            || self.modals.is_rename_active
            || self.modals.is_loading_active;

        let floating_count = params
            .context
            .layout_state
            .dock_state
            .floating_windows
            .len();

        let has_drag_payload = params.panel_data.asset_browser.drag_payload.is_some();
        let cursor = self.cursor_pos();
        let cursor_moved = (self.chrome.last_cursor_pos.x - cursor.x).abs() > 0.001
            || (self.chrome.last_cursor_pos.y - cursor.y).abs() > 0.001;

        // Unified Tree Hit-Testing:
        // 1. First inspect elevated overlay layers (popups, dropdowns, modals) in the Shell tree
        let mut hovered_target = None;
        let mut hovered_panel = None;

        if let Some(hit_id) = self.tree.hit_test_layered(cursor) {
            let hit_layer = self.tree.effective_layer(hit_id);
            if hit_layer >= UiLayer::Modal
                && let Some(target) = self.tree.hit_test_target(cursor)
                && target.tag != 0
            {
                hovered_target = Some(target.tag);
            }
        }

        // 2. Active dock panel under cursor
        if !self.is_point_over_overlay(cursor)
            && let Some(dock_frame) = &self.chrome.native_dock_frame
        {
            for (pid, bounds) in &dock_frame.panel_rects {
                if *pid != PanelId::Viewport && bounds.contains_point(cursor) {
                    hovered_panel = Some(*pid);
                    break;
                }
            }
        }

        // 3. Unified tree hit-testing (MenuBar, StatusBar, Viewport HUD, Dock panels)
        if hovered_target.is_none() {
            hovered_target = self
                .tree
                .hit_test_target(cursor)
                .and_then(|h| if h.tag != 0 { Some(h.tag) } else { None });
        }

        let hover_target_changed = cursor_moved && hovered_target != self.chrome.last_hovered_tag;
        self.chrome.hovered_tag = hovered_target;

        // Hover transitions update reactive paint shaders in Branch 3 without rebuilding panel DOM
        self.chrome.hovered_panel = hovered_panel;

        // 1. Genuine global shell events (only these trigger global tag_all)
        if self.chrome.last_dimensions != params.context.dimensions
            || self.chrome.last_theme_preset != self.chrome.theme_preset
            || (self.chrome.last_zoom_factor - params.context.zoom_factor).abs() > 1e-4
            || self.chrome.last_floating_count != floating_count
            || self.modals.last_modal_active != modal_active
            || self.preferences.last_tab != self.preferences.tab
            || self.menubar.active_menu.is_some()
            || self.chrome.active_dock_overflow.is_some()
            || self.chrome.needs_layout_rebuild
            || has_drag_payload
        {
            self.notifier.tag_all();
        }

        if !self.is_visible {
            self.command_list.clear();
            return;
        }

        let blink_caret = (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2);
        let current_focused_tag = self.tree.event_state.focused_tag;
        let focus_target_changed = current_focused_tag != self.chrome.last_focused_tag;
        let has_text_focus = self.is_any_text_input_focused();
        let blink_changed = has_text_focus && (blink_caret != self.chrome.last_blink_caret);

        let floating_changed = floating_count != self.chrome.last_floating_count
            || params
                .context
                .layout_state
                .dock_state
                .floating_windows
                .iter()
                .zip(&self.chrome.floating_window_rects)
                .any(|(w, r)| w.rect != *r);

        let modal_active_changed = self.modals.last_modal_active != modal_active;
        let menu_active =
            self.menubar.active_menu.is_some() || self.menubar.dropdown_rect.is_some();
        let menu_changed = menu_active != self.chrome.last_menu_active;
        let inspector_popups = self.inspector.is_add_menu_open
            || self.inspector.is_color_picker_open
            || self.inspector.active_dropdown.is_some();
        let inspector_popups_changed = inspector_popups != self.chrome.last_inspector_popups;
        let hierarchy_popups =
            self.hierarchy.is_add_menu_open || self.hierarchy.active_context_menu.is_some();
        let hierarchy_popups_changed = hierarchy_popups != self.chrome.last_hierarchy_popups;
        let preferences_active = params.dialogs.show_preferences;
        let preferences_changed = preferences_active != self.chrome.last_preferences_active
            || (preferences_active && self.preferences.last_tab != self.preferences.tab);

        let dock_revision = params.context.layout_state.dock_state.revision();
        let dock_changed = dock_revision != self.chrome.last_dock_revision;

        let viewport_texture_changed =
            params.viewport.has_viewport_texture != self.chrome.last_has_viewport_texture;

        let is_shell_dirty = self.chrome.always_rebuild
            || self.tree.root().is_none()
            || self.chrome.native_dock_frame.is_none()
            || self.notifier.is_global_dirty()
            || viewport_texture_changed
            || dock_changed
            || self.chrome.is_dirty(
                params.context.dimensions,
                params.context.zoom_factor,
                floating_count,
                has_drag_payload,
            )
            || floating_changed
            || params
                .context
                .layout_state
                .dock_state
                .active_splitter
                .is_some()
            || params.context.layout_state.dock_state.active_drag.is_some()
            || modal_active_changed
            || menu_changed
            || inspector_popups_changed
            || hierarchy_popups_changed
            || preferences_changed
            || self.chrome.needs_layout_rebuild;

        self.chrome.last_dock_revision = dock_revision;

        if self.menubar.active_menu.is_none() {
            self.menubar.actions.clear();
            self.menubar.dropdown_rect = None;
        }
        if !params.dialogs.show_preferences {
            self.preferences.card_rect = None;
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
        let folder_changed =
            self.assets.current_folder != params.panel_data.asset_browser.current_folder;
        if folder_changed {
            self.assets.current_folder = params.panel_data.asset_browser.current_folder.clone();
            self.notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::Assets);
        }

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

            // 1. Top MenuBar
            if let Some(action) = menubar::build_top_menu_bar(
                &mut self.tree,
                root,
                &menubar::MenuBarParams {
                    screen_width,
                    layout_state: params.context.layout_state,
                    can_undo: params.context.can_undo,
                    can_redo: params.context.can_redo,
                    is_editing: params.context.is_editing,
                },
            ) {
                self.menubar.actions.push(action);
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

            // 2b. Baseline dividers under MenuBar and above StatusBar (positioned absolutely with explicit styles)
            let mut scope = UiScope::new(&mut self.tree, root);
            scope.empty_box_passive_named(
                "IrisTopMenuBarDivider",
                Style::new()
                    .position_absolute()
                    .left(0.0)
                    .top(Self::MENUBAR_HEIGHT - 1.0)
                    .width(screen_width)
                    .height(1.0)
                    .background(BORDER_MICRON),
            );
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
                let (left, top) = if let Some(pos) = self.preferences.pos {
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
            let dock_style = super::native_dock::editor_dock_style_with_tokens(&active_tokens);
            let mut tree = std::mem::take(&mut self.tree);
            let root_id = tree.root().unwrap_or(root);
            {
                let mut scope = UiScope::new(&mut tree, root_id);
                let dock_frame =
                    irisui::dock::DockArea::from_ref(&params.context.layout_state.dock_state)
                        .bounds(workspace_rect)
                        .style(dock_style)
                        .cursor_pos(cursor)
                        .cursor_occluded(is_cursor_occluded)
                        .show(
                            &mut scope,
                            &mut crate::ui::panel_layout::PanelTabViewer::with_dispatch(
                                |leaf_scope, panel| {
                                    self.render_panel_by_id(*panel, leaf_scope, &params);
                                },
                            ),
                        );

                self.chrome.floating_window_rects = dock_frame.floating_rects.clone();
                self.chrome.native_dock_frame = Some(dock_frame.into());
            }
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

            // 5. SHELL DIALOGS (Modal windows rendered on elevated layers in the Shell tree):
            self.build_shell_dialogs(
                root,
                &mut params,
                super::shell_dialogs::ShellDialogsContext {
                    screen_width,
                    screen_height,
                    cursor,
                    workspace_rect,
                    is_cursor_occluded,
                },
            );

            self.notifier.clear_all();
            self.command_list.clear();
            self.chrome.needs_text_prepare = true;
            self.populate_draw_commands(root, None, Some(params.telemetry.frame_pacing));
        } else {
            // =========================================================================
            // Branch 2: Granular Subtree Invalidation (Zero tree.clear(), O(M) Layout)
            // =========================================================================
            // When the shell layout is intact, dirty panels are rebuilt in-place within their
            // isolated boundary leaf containers without destroying the tree or other panels.
            let mut dirty_panels: Vec<crate::ui::panel_layout::PanelId> =
                self.notifier.dirty_panels().iter().copied().collect();

            if self.tree.has_dirty_boundaries() {
                if let Some(dock_frame) = &self.chrome.native_dock_frame {
                    for boundary_id in self.tree.dirty_boundaries.clone() {
                        if let Some(panel_id) = dock_frame.tab_for_leaf_container(boundary_id)
                            && !dirty_panels.contains(&panel_id)
                        {
                            dirty_panels.push(panel_id);
                        }
                    }
                }
                self.tree.clear_dirty_boundaries();
            }

            let mut rebuilt_any_panel = false;
            for panel_id in dirty_panels {
                if let Some(dock_frame) = &self.chrome.native_dock_frame {
                    if let Some(leaf_id) = dock_frame.leaf_container(panel_id) {
                        if let Err(err) = self.rebuild_panel_content(panel_id, leaf_id, &params) {
                            log::warn!(
                                "Failed partial subtree rebuild for panel {panel_id:?}: {err:?}"
                            );
                        } else {
                            rebuilt_any_panel = true;
                        }
                    } else {
                        self.notifier.clear_panel(panel_id);
                    }
                }
            }

            // =========================================================================
            // Branch 3: Retained Sleep In-Place Draw Refresh (Zero Tree Mutation)
            // =========================================================================
            let has_stats_canvas = self
                .tree
                .node_tagged(super::stats::types::STATS_TAG_CANVAS)
                .is_some();
            let compass_changed = self.viewport_hud.camera_angles
                != (params.viewport.camera.pitch.0, params.viewport.camera.yaw.0);
            let has_volatile_canvas = has_stats_canvas || compass_changed;

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

            if blink_changed || hover_target_changed || focus_target_changed {
                self.chrome.needs_text_prepare = true;
            }

            if rebuilt_any_panel
                || has_volatile_canvas
                || self.chrome.needs_scroll_sync
                || hover_target_changed
                || focus_target_changed
                || blink_changed
            {
                self.command_list.clear();
                if let Some(root) = self.tree.root() {
                    self.populate_draw_commands(root, None, Some(params.telemetry.frame_pacing));
                }
            }
        }

        self.chrome.last_dimensions = params.context.dimensions;
        self.chrome.last_zoom_factor = params.context.zoom_factor;
        self.chrome.last_floating_count = floating_count;
        self.modals.last_modal_active = modal_active;
        self.preferences.last_tab = self.preferences.tab;
        self.chrome.last_has_viewport_texture = params.viewport.has_viewport_texture;
        self.chrome.last_has_drag_payload = has_drag_payload;
        self.chrome.last_cursor_pos = cursor;
        self.chrome.last_hovered_tag = self.chrome.hovered_tag;
        self.chrome.last_hovered_panel = self.chrome.hovered_panel;
        self.chrome.last_focused_tag = current_focused_tag;
        self.chrome.last_blink_caret = blink_caret;
        self.chrome.last_menu_active = menu_active;
        self.chrome.last_inspector_popups = inspector_popups;
        self.chrome.last_hierarchy_popups = hierarchy_popups;
        self.chrome.last_preferences_active = preferences_active;
        self.chrome.last_theme_preset = self.chrome.theme_preset;
        self.chrome.needs_layout_rebuild = false;
        self.chrome.needs_scroll_sync = false;
        self.notifier.clear_all();
        self.tree.clear_frame_clicks();
    }

    /// Closes all transient active popups and context menus, returning `true` if any were open.
    fn close_active_popups(&mut self) -> bool {
        let mut closed_any = false;
        if self.inspector.is_add_menu_open {
            self.inspector.is_add_menu_open = false;
            self.inspector.active_submenu = None;
            closed_any = true;
        }
        if self.inspector.is_color_picker_open {
            self.inspector.is_color_picker_open = false;
            closed_any = true;
        }
        if self.hierarchy.is_add_menu_open {
            self.hierarchy.is_add_menu_open = false;
            self.hierarchy.active_submenu = None;
            self.hierarchy.active_sub_submenu = None;
            closed_any = true;
        }
        if self.hierarchy.active_context_menu.is_some() {
            self.hierarchy.active_context_menu = None;
            closed_any = true;
        }
        closed_any
    }
}